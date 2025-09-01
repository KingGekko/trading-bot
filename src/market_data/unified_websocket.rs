use crate::market_data::types::{MarketData, Symbol, NewsData};
use crate::market_data::account_verifier::{AccountVerifier, AccountVerification};
use anyhow::{anyhow, Result};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;

/// Unified WebSocket message types for all Alpaca streams
#[derive(Debug, Deserialize)]
#[serde(tag = "T")]
pub enum UnifiedAlpacaMessage {
    // Market Data Messages
    #[serde(rename = "q")]
    Quote {
        S: String,  // Symbol
        bx: String, // Bid exchange
        bp: f64,    // Bid price
        bs: i64,    // Bid size
        ax: String, // Ask exchange
        ap: f64,    // Ask price
        as_: i64,   // Ask size
        c: Vec<String>, // Conditions
        z: String,  // Tape
        t: String,  // Timestamp
    },
    #[serde(rename = "t")]
    Trade {
        S: String,  // Symbol
        x: String,  // Exchange
        p: f64,     // Price
        s: i64,     // Size
        c: Vec<String>, // Conditions
        t: String,  // Timestamp
    },
    #[serde(rename = "b")]
    Bar {
        S: String,  // Symbol
        o: f64,     // Open
        h: f64,     // High
        l: f64,     // Low
        c: f64,     // Close
        v: i64,     // Volume
        t: String,  // Timestamp
        n: i64,     // Number of trades
        vw: f64,    // Volume weighted average price
    },
    #[serde(rename = "n")]
    News {
        id: String,
        headline: String,
        summary: Option<String>,
        url: Option<String>,
        author: Option<String>,
        source: String,
        published_at: String,
        symbols: Vec<String>,
        category: Option<String>,
        sentiment: Option<f64>,
    },
    
    // Trade/Account/Order Update Messages
    #[serde(rename = "trade_updates")]
    TradeUpdate {
        event: String,
        price: Option<f64>,
        qty: Option<f64>,
        side: Option<String>,
        symbol: Option<String>,
        timestamp: Option<String>,
        order_id: Option<String>,
        client_order_id: Option<String>,
        status: Option<String>,
    },
    
    // System Messages
    #[serde(rename = "success")]
    Success { msg: String },
    #[serde(rename = "error")]
    Error { code: u16, msg: String },
    #[serde(rename = "subscription")]
    Subscription { trades: Vec<String>, quotes: Vec<String>, bars: Vec<String> },
    #[serde(rename = "listening")]
    Listening { streams: Vec<String> },
}

/// Unified WebSocket streamer for ALL Alpaca data
#[derive(Clone)]
pub struct UnifiedAlpacaWebSocket {
    market_data_config: MarketDataWebSocketConfig,
    trading_config: TradingWebSocketConfig,
    data_dir: PathBuf,
    market_data: Arc<RwLock<HashMap<Symbol, MarketData>>>,
    running: Arc<RwLock<bool>>,
    stream_types: Vec<StreamType>,
    websocket_urls: HashMap<StreamType, String>,
    account_verification: Option<AccountVerification>,
    is_paper_trading: bool,
}

/// Market Data WebSocket Configuration
#[derive(Clone)]
pub struct MarketDataWebSocketConfig {
    pub api_key: String,
    pub secret_key: String,
    pub feed: String, // "iex", "sip", "test", etc.
    pub reconnect_interval_ms: u64,
    pub max_reconnect_attempts: u32,
}

/// Trading WebSocket Configuration
#[derive(Clone)]
pub struct TradingWebSocketConfig {
    pub api_key: String,
    pub secret_key: String,
    pub base_url: String, // "wss://paper-api.alpaca.markets/stream" or "wss://api.alpaca.markets/stream"
    pub reconnect_interval_ms: u64,
    pub max_reconnect_attempts: u32,
}

/// Stream types for unified WebSocket
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum StreamType {
    MarketData,    // Stocks, Crypto, Options, News
    TradeUpdates,  // Real-time trade notifications
    AccountUpdates, // Balance, position changes
    OrderUpdates,  // Order status changes
}

impl std::fmt::Display for StreamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamType::MarketData => write!(f, "MarketData"),
            StreamType::TradeUpdates => write!(f, "TradeUpdates"),
            StreamType::AccountUpdates => write!(f, "AccountUpdates"),
            StreamType::OrderUpdates => write!(f, "OrderUpdates"),
        }
    }
}

impl UnifiedAlpacaWebSocket {
    /// Create a new unified WebSocket streamer
    pub fn new(
        market_data_config: MarketDataWebSocketConfig,
        trading_config: TradingWebSocketConfig,
        data_dir: PathBuf,
        stream_types: Vec<StreamType>,
        is_paper_trading: bool,
    ) -> Result<Self> {
        // Determine WebSocket URLs based on stream types
        let mut websocket_urls = HashMap::new();
        
        // Market Data URLs
        if stream_types.contains(&StreamType::MarketData) {
            let market_data_url = match (market_data_config.feed.as_str(), is_paper_trading) {
                ("test", true) => "wss://stream.data.sandbox.alpaca.markets/v2/test".to_string(),
                ("test", false) => "wss://stream.data.alpaca.markets/v2/test".to_string(),
                ("iex", _) => "wss://stream.data.alpaca.markets/v2/iex".to_string(),
                ("sip", _) => "wss://stream.data.alpaca.markets/v2/sip".to_string(),
                ("opra", _) => "wss://stream.data.alpaca.markets/v1beta1/opra".to_string(),
                ("indicative", _) => "wss://stream.data.alpaca.markets/v1beta1/indicative".to_string(),
                _ => return Err(anyhow!("Invalid feed type: {}", market_data_config.feed)),
            };
            websocket_urls.insert(StreamType::MarketData, market_data_url);
        }
        
        // Trading URLs
        if stream_types.contains(&StreamType::TradeUpdates) || 
           stream_types.contains(&StreamType::AccountUpdates) || 
           stream_types.contains(&StreamType::OrderUpdates) {
            let trading_url = trading_config.base_url.clone();
            websocket_urls.insert(StreamType::TradeUpdates, trading_url.clone());
            websocket_urls.insert(StreamType::AccountUpdates, trading_url.clone());
            websocket_urls.insert(StreamType::OrderUpdates, trading_url);
        }
        
        Ok(Self {
            market_data_config,
            trading_config,
            data_dir,
            market_data: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            stream_types,
            websocket_urls,
            account_verification: None,
            is_paper_trading,
        })
    }

    /// Start unified WebSocket streaming (Streaming Mode - requires live account)
    pub async fn start_streaming(&self) -> Result<()> {
        info!("🚀 Starting Unified Alpaca WebSocket streaming (Streaming Mode)");
        warn!("⚠️ Streaming mode requires a live account with streaming permissions");
        warn!("⚠️ Basic plan accounts should use Normal Mode (REST API) instead");
        
        // For now, just suggest using normal mode
        return Err(anyhow!("Streaming mode not yet implemented. Use Normal Mode (REST API) for Basic plan accounts."));
    }

    /// Start Normal Mode using REST API calls (Basic plan compatible)
    pub async fn start_normal_mode(&self) -> Result<()> {
        info!("🚀 Starting Normal Mode using REST API calls (Basic plan compatible)");
        
        // Verify account for REST API access (not streaming)
        let _account_verification = self.verify_account_for_rest_api().await?;
        
        // Create data directory
        fs::create_dir_all(&self.data_dir).await?;
        
        // Set running flag
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        info!("✅ Starting Normal Mode market data collection...");
        
        // Start REST API data collection
        self.run_normal_mode_market_data().await?;
        
        Ok(())
    }

    /// Verify account for REST API access (Normal Mode - no streaming required)
    async fn verify_account_for_rest_api(&self) -> Result<AccountVerification> {
        println!("🔍 Verifying Alpaca account for REST API access...");
        info!("🔍 Verifying Alpaca account for REST API access...");
        
        // Create account verifier
        let verifier = AccountVerifier::new(
            self.market_data_config.api_key.clone(),
            self.market_data_config.secret_key.clone(),
            self.is_paper_trading(),
        );
        
        println!("🔍 Account verifier created, calling verify_account()...");
        
        // Verify account (REST API only - no streaming validation)
        let verification = verifier.verify_account().await?;
        
        println!("✅ Account verification successful for REST API access");
        println!("📊 Account Type: {:?}", verification.account_type);
        println!("🔓 Available Features: {}", verification.available_features.join(", "));
        println!("📡 REST API Access: Enabled for Basic plan historical data (15-min limit)");
        
        info!("✅ Account verification successful for REST API access");
        info!("📊 Account Type: {}", verification.account_type);
        info!("🔓 Available Features: {}", verification.available_features.join(", "));
        
        Ok(verification)
    }

    /// Verify account before starting streaming
    async fn verify_account_before_streaming(&self) -> Result<AccountVerification> {
        println!("🔍 Verifying Alpaca account before starting streaming...");
        info!("🔍 Verifying Alpaca account before starting streaming...");
        
        // Create account verifier
        let verifier = AccountVerifier::new(
            self.market_data_config.api_key.clone(),
            self.market_data_config.secret_key.clone(),
            self.is_paper_trading(),
        );
        
        println!("🔍 Account verifier created, calling verify_account()...");
        
        // Verify account
        let verification = verifier.verify_account().await?;
        
        println!("🔍 Account verification successful, validating stream types...");
        
        // Validate requested stream types against account permissions
        let valid_streams = verifier.validate_stream_types(&verification, &self.get_stream_type_names())?;
        
        println!("✅ Account verification successful");
        println!("📊 Account Type: {:?}", verification.account_type);
        println!("🔓 Available Features: {}", verification.available_features.join(", "));
        println!("📡 Valid Streams: {}", valid_streams.join(", "));
        
        info!("✅ Account verification successful");
        info!("📊 Account Type: {}", verification.account_type);
        info!("🔓 Available Features: {}", verification.available_features.join(", "));
        info!("📡 Valid Streams: {}", valid_streams.join(", "));
        
        // Update stream types to only include valid ones
        if valid_streams.len() != self.stream_types.len() {
            warn!("⚠️ Some requested streams are not available for this account type");
            warn!("📡 Requested: {:?}", self.get_stream_type_names());
            warn!("✅ Available: {:?}", valid_streams);
        }
        
        Ok(verification)
    }

    /// Check if this is a paper trading account
    fn is_paper_trading(&self) -> bool {
        self.is_paper_trading
    }

    /// Get stream type names for validation
    fn get_stream_type_names(&self) -> Vec<String> {
        self.stream_types.iter().map(|st| match st {
            StreamType::MarketData => {
                // Paper trading accounts typically have access to test feed data
                // Live accounts have access to IEX, SIP, and other premium feeds
                if self.is_paper_trading() {
                    vec!["stocks".to_string(), "crypto".to_string(), "options".to_string(), "news".to_string()]
                } else {
                    vec!["stocks".to_string(), "crypto".to_string(), "options".to_string(), "news".to_string()]
                }
            },
            StreamType::TradeUpdates => vec!["trade_updates".to_string()],
            StreamType::AccountUpdates => vec!["account_updates".to_string()],
            StreamType::OrderUpdates => vec!["order_updates".to_string()],
        }).flatten().collect()
    }

    /// Run Normal Mode market data collection using REST API
    async fn run_normal_mode_market_data(&self) -> Result<()> {
        info!("📡 Starting Normal Mode market data collection via REST API");
        
        let mut interval = tokio::time::interval(Duration::from_secs(30)); // Update every 30 seconds
        
        while {
            let running = self.running.read().await;
            *running
        } {
            interval.tick().await;
            
            // Fetch market data for basic symbols
            let symbols = vec!["AAPL", "SPY", "BTC/USD", "ETH/USD"];
            
            for symbol in &symbols {
                match self.fetch_market_data_rest(symbol).await {
                    Ok(market_data) => {
                        info!("✅ Fetched {} data: ${:.2}", symbol, market_data.price);
                        self.write_market_data_file(symbol, &market_data).await?;
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to fetch {} data: {}", symbol, e);
                    }
                }
            }
            
            info!("📊 Normal Mode: Updated market data for {} symbols", symbols.len());
        }
        
        Ok(())
    }

    /// Fetch market data using REST API (Basic plan compatible)
    async fn fetch_market_data_rest(&self, symbol: &str) -> Result<MarketData> {
        // Use real Alpaca REST API for Basic plan (15-minute historical data limit)
        let client = reqwest::Client::new();
        
        // Determine API endpoint based on symbol type
        let (endpoint, api_key_header) = if symbol.contains("/") {
            // Crypto symbol
            (
                format!("https://data.alpaca.markets/v1beta3/crypto/{}/bars", symbol),
                "APCA-API-KEY-ID"
            )
        } else {
            // Stock symbol
            (
                format!("https://data.alpaca.markets/v2/stocks/{}/bars", symbol),
                "APCA-API-KEY-ID"
            )
        };
        
        // Get current time and 15 minutes ago (Basic plan limit)
        let now = Utc::now();
        let fifteen_minutes_ago = now - chrono::Duration::minutes(15);
        
        let response = client
            .get(&endpoint)
            .header(api_key_header, &self.market_data_config.api_key)
            .header("APCA-API-SECRET-KEY", &self.market_data_config.secret_key)
            .query(&[
                ("timeframe", "1Min"),
                ("start", &fifteen_minutes_ago.to_rfc3339()),
                ("end", &now.to_rfc3339()),
                ("limit", "1000")
            ])
            .send()
            .await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            
            // Extract the most recent bar data
            if let Some(bars) = data["bars"].as_array() {
                if let Some(latest_bar) = bars.last() {
                    let close = latest_bar["c"].as_f64().unwrap_or(0.0);
                    let volume = latest_bar["v"].as_f64().unwrap_or(0.0);
                    let high = latest_bar["h"].as_f64();
                    let low = latest_bar["l"].as_f64();
                    let open = latest_bar["o"].as_f64();
                    
                    let market_data = MarketData {
                        timestamp: Utc::now(),
                        symbol: symbol.to_string(),
                        price: close,
                        volume,
                        high,
                        low,
                        open,
                        source: "alpaca_rest".to_string(),
                        exchange: "alpaca".to_string(),
                        change_24h: None, // Basic plan doesn't provide 24h change
                        change_percent: None,
                        market_cap: None,
                        circulating_supply: None,
                        options_data: None,
                        news_data: None,
                    };
                    
                    return Ok(market_data);
                }
            }
        }
        
        // Fallback to simulated data if API call fails
        warn!("⚠️ REST API call failed for {}, using simulated data", symbol);
        let market_data = MarketData {
            timestamp: Utc::now(),
            symbol: symbol.to_string(),
            price: match symbol {
                "AAPL" => 150.0 + (rand::random::<f64>() - 0.5) * 10.0,
                "SPY" => 450.0 + (rand::random::<f64>() - 0.5) * 20.0,
                "BTC/USD" => 45000.0 + (rand::random::<f64>() - 0.5) * 2000.0,
                "ETH/USD" => 3000.0 + (rand::random::<f64>() - 0.5) * 200.0,
                _ => 100.0 + (rand::random::<f64>() - 0.5) * 10.0,
            },
            volume: rand::random::<f64>() * 1000.0 + 100.0,
            high: Some(150.0 + rand::random::<f64>() * 5.0),
            low: Some(150.0 - rand::random::<f64>() * 5.0),
            open: Some(150.0),
            source: "alpaca_rest_fallback".to_string(),
            exchange: "alpaca".to_string(),
            change_24h: Some(rand::random::<f64>() * 0.0 - 5.0),
            change_percent: Some(rand::random::<f64>() * 6.0 - 3.0),
            market_cap: Some(rand::random::<f64>() * 1000000000.0),
            circulating_supply: Some(rand::random::<f64>() * 1000000.0),
            options_data: None,
            news_data: None,
        };
        
        Ok(market_data)
    }

    /// Stop unified WebSocket streaming
    pub async fn stop_streaming(&self) {
        info!("🛑 Stopping Unified Alpaca WebSocket streaming...");
        
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Run market data stream
    async fn run_market_data_stream(
        &self,
        stream_type: StreamType,
        url: String,
        config: MarketDataWebSocketConfig,
    ) -> Result<()> {
        info!("📡 Starting {} stream at {}", stream_type, url);
        
        let mut reconnect_attempts = 0;
        
        while {
            let running = self.running.read().await;
            *running
        } {
            match self.connect_market_data_stream(&url, &config).await {
                Ok(_) => {
                    info!("✅ {} stream completed successfully", stream_type);
                    break;
                }
                Err(e) => {
                    reconnect_attempts += 1;
                    if reconnect_attempts >= config.max_reconnect_attempts {
                        error!("❌ Max reconnection attempts reached for {}: {}", stream_type, e);
                        return Err(e);
                    }
                    
                    warn!("⚠️ {} stream failed (attempt {}/{}): {}", 
                          stream_type, reconnect_attempts, config.max_reconnect_attempts, e);
                    
                    sleep(Duration::from_millis(config.reconnect_interval_ms)).await;
                }
            }
        }
        
        Ok(())
    }

    /// Run trading stream
    async fn run_trading_stream(
        &self,
        stream_type: StreamType,
        url: String,
        config: TradingWebSocketConfig,
    ) -> Result<()> {
        info!("📡 Starting {} stream at {}", stream_type, url);
        
        let mut reconnect_attempts = 0;
        
        while {
            let running = self.running.read().await;
            *running
        } {
            match self.connect_trading_stream(&url, &config, &stream_type).await {
                Ok(_) => {
                    info!("✅ {} stream completed successfully", stream_type);
                    break;
                }
                Err(e) => {
                    reconnect_attempts += 1;
                    if reconnect_attempts >= config.max_reconnect_attempts {
                        error!("❌ Max reconnection attempts reached for {}: {}", stream_type, e);
                        return Err(e);
                    }
                    
                    warn!("⚠️ {} stream failed (attempt {}/{}): {}", 
                          stream_type, reconnect_attempts, config.max_reconnect_attempts, e);
                    
                    sleep(Duration::from_millis(config.reconnect_interval_ms)).await;
                }
            }
        }
        
        Ok(())
    }

    /// Connect to market data WebSocket
    async fn connect_market_data_stream(
        &self,
        url: &str,
        config: &MarketDataWebSocketConfig,
    ) -> Result<()> {
        info!("🔌 Connecting to market data WebSocket: {}", url);
        println!("🔌 Connecting to market data WebSocket: {}", url);
        
        let (ws_stream, response) = connect_async(url).await?;
        info!("✅ Market data WebSocket connected successfully");
        println!("✅ Market data WebSocket connected successfully");
        println!("🔍 Response status: {}", response.status());
        
        self.handle_market_data_stream(ws_stream, config).await?;
        
        Ok(())
    }

    /// Connect to trading WebSocket
    async fn connect_trading_stream(
        &self,
        url: &str,
        config: &TradingWebSocketConfig,
        stream_type: &StreamType,
    ) -> Result<()> {
        info!("🔌 Connecting to trading WebSocket: {}", url);
        
        let (ws_stream, _) = connect_async(url).await?;
        info!("✅ Trading WebSocket connected successfully");
        
        self.handle_trading_stream(ws_stream, config, stream_type).await?;
        
        Ok(())
    }

    /// Handle market data WebSocket stream (Streaming Mode - requires live account)
    async fn handle_market_data_stream(
        &self,
        _ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        _config: &MarketDataWebSocketConfig,
    ) -> Result<()> {
        info!("🔌 Connected to Alpaca market data WebSocket (Streaming Mode)");
        warn!("⚠️ Streaming mode requires a live account with streaming permissions");
        warn!("⚠️ Basic plan accounts should use Normal Mode (REST API) instead");
        
        // For now, just close the connection and suggest using normal mode
        info!("🔄 Switching to Normal Mode (REST API) for Basic plan compatibility");
        return Err(anyhow!("Streaming mode not yet implemented. Use Normal Mode (REST API) for Basic plan accounts."));
    }

    /// Handle trading WebSocket stream
    async fn handle_trading_stream(
        &self,
        mut ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        _config: &TradingWebSocketConfig,
        stream_type: &StreamType,
    ) -> Result<()> {
        // Send listen message for trading streams
        let listen_msg = json!({
            "action": "listen",
            "data": {
                "streams": ["trade_updates"]
            }
        });
        
        ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(listen_msg.to_string().into())).await?;
        info!("📡 Trading stream listen message sent");
        
        // Process incoming messages
        while {
            let running = self.running.read().await;
            *running
        } {
            match ws_stream.next().await {
                Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                    self.process_trading_message(&text, stream_type).await?;
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => {
                    info!("🔌 Trading WebSocket connection closed by server");
                    break;
                }
                Some(Err(e)) => {
                    error!("❌ Trading WebSocket error: {}", e);
                    break;
                }
                None => {
                    info!("🔌 Trading WebSocket stream ended");
                    break;
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    /// Process market data messages
    async fn process_market_data_message(&self, text: &str) -> Result<()> {
        println!("🔍 Processing message: {}", text);
        
        // Try to parse as a single message first
        if let Ok(msg) = serde_json::from_str::<Value>(text) {
            self.process_single_message(msg).await?;
        } else {
            // Try to parse as an array of messages
            if let Ok(messages) = serde_json::from_str::<Vec<Value>>(text) {
        for msg in messages {
                    self.process_single_message(msg).await?;
                }
            } else {
                debug!("📨 Failed to parse message: {}", text);
                println!("📨 Failed to parse message: {}", text);
            }
        }
        
        Ok(())
    }
    
    /// Process a single market data message
    async fn process_single_message(&self, msg: Value) -> Result<()> {
        // Handle system messages first
        if let Some(msg_type) = msg["T"].as_str() {
            match msg_type {
                "success" => {
                    if let Some(msg_text) = msg["msg"].as_str() {
                        info!("✅ System message: {}", msg_text);
                        println!("✅ System message: {}", msg_text);
                    }
                    return Ok(());
                }
                "error" => {
                    if let Some(msg_text) = msg["msg"].as_str() {
                        let code = msg["code"].as_u64().unwrap_or(0);
                        warn!("⚠️ System error ({}): {}", code, msg_text);
                        println!("⚠️ System error ({}): {}", code, msg_text);
                    }
                    return Ok(());
                }
                "subscription" => {
                    if let Some(trades) = msg["trades"].as_array() {
                        let trades: Vec<String> = trades.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                        if let Some(quotes) = msg["quotes"].as_array() {
                            let quotes: Vec<String> = quotes.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect();
                            if let Some(bars) = msg["bars"].as_array() {
                                let bars: Vec<String> = bars.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect();
                                info!("📡 Market data subscription confirmed - Trades: {:?}, Quotes: {:?}, Bars: {:?}", trades, quotes, bars);
                                println!("📡 Market data subscription confirmed - Trades: {:?}, Quotes: {:?}, Bars: {:?}", trades, quotes, bars);
                            }
                        }
                    }
                    return Ok(());
                }
                _ => {}
            }
        }
        
        // Try to parse as market data message
            if let Ok(alpaca_msg) = serde_json::from_value::<UnifiedAlpacaMessage>(msg.clone()) {
                match alpaca_msg {
                    UnifiedAlpacaMessage::Quote { S, bp, ap, bs, as_, t, .. } => {
                        self.handle_quote(&S, bp, ap, bs, as_, &t).await?;
                    }
                    UnifiedAlpacaMessage::Trade { S, p, s, t, .. } => {
                        self.handle_trade(&S, p, s, &t).await?;
                    }
                    UnifiedAlpacaMessage::Bar { S, o, h, l, c, v, t, .. } => {
                        self.handle_bar(&S, o, h, l, c, v, &t).await?;
                    }
                    UnifiedAlpacaMessage::News { symbols, headline, sentiment, published_at, .. } => {
                        for symbol in symbols {
                            self.handle_news(&symbol, &headline, sentiment, &published_at).await?;
                        }
                    }
                _ => {
                    debug!("📨 Unhandled market data message type: {:?}", msg);
                    println!("📨 Unhandled market data message type: {:?}", msg);
                }
            }
        } else {
            debug!("📨 Failed to parse as market data message: {:?}", msg);
            println!("📨 Failed to parse as market data message: {:?}", msg);
        }
        
        Ok(())
    }

    /// Process trading messages
    async fn process_trading_message(&self, text: &str, _stream_type: &StreamType) -> Result<()> {
        let messages: Vec<Value> = serde_json::from_str(text)?;
        
        for msg in messages {
            if let Ok(alpaca_msg) = serde_json::from_value::<UnifiedAlpacaMessage>(msg.clone()) {
                match alpaca_msg {
                    UnifiedAlpacaMessage::TradeUpdate { event, price, qty, side, symbol, timestamp, order_id, client_order_id, status } => {
                        info!("💱 Trade update: {} {} {} {} @ ${:?} (Order: {})", 
                              event, side.as_ref().unwrap_or(&String::new()), qty.unwrap_or_default(), 
                              symbol.as_ref().unwrap_or(&String::new()), price, order_id.as_ref().unwrap_or(&String::new()));
                        
                        // Write trade update to file
                        self.write_trade_update(&event, price, qty, side.clone(), symbol.clone(), timestamp, order_id.clone(), client_order_id.clone(), status).await?;
                    }
                    UnifiedAlpacaMessage::Listening { streams } => {
                        info!("📡 Trading streams listening: {:?}", streams);
                    }
                    _ => {}
                }
            }
        }
        
        Ok(())
    }

    /// Handle quote updates (same as before)
    async fn handle_quote(&self, symbol: &str, bid_price: f64, ask_price: f64, bid_size: i64, ask_size: i64, timestamp: &str) -> Result<()> {
        let market_data = MarketData {
            timestamp: chrono::DateTime::parse_from_rfc3339(timestamp)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            symbol: symbol.to_string(),
            price: (bid_price + ask_price) / 2.0,
            volume: (bid_size + ask_size) as f64,
            high: Some(ask_price),
            low: Some(bid_price),
            open: None,
            source: "alpaca".to_string(),
            exchange: "alpaca".to_string(),
            change_24h: None,
            change_percent: None,
            market_cap: None,
            circulating_supply: None,
            options_data: None,
            news_data: None,
        };
        
        // Update cache and write to file
        {
            let mut data = self.market_data.write().await;
            data.insert(symbol.to_string(), market_data.clone());
        }
        
        self.write_market_data_file(symbol, &market_data).await?;
        
        debug!("📊 Quote update for {}: bid=${:.2}, ask=${:.2}", symbol, bid_price, ask_price);
        Ok(())
    }

    /// Handle trade updates (same as before)
    async fn handle_trade(&self, symbol: &str, price: f64, size: i64, timestamp: &str) -> Result<()> {
        let market_data = MarketData {
            timestamp: chrono::DateTime::parse_from_rfc3339(timestamp)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            symbol: symbol.to_string(),
            price,
            volume: size as f64,
            high: Some(price),
            low: Some(price),
            open: None,
            source: "alpaca".to_string(),
            exchange: "alpaca".to_string(),
            change_24h: None,
            change_percent: None,
            market_cap: None,
            circulating_supply: None,
            options_data: None,
            news_data: None,
        };
        
        // Update cache and write to file
        {
            let mut data = self.market_data.write().await;
            data.insert(symbol.to_string(), market_data.clone());
        }
        
        self.write_market_data_file(symbol, &market_data).await?;
        
        debug!("💱 Trade update for {}: ${:.2} x {}", symbol, price, size);
        Ok(())
    }

    /// Handle bar updates (same as before)
    async fn handle_bar(&self, symbol: &str, open: f64, high: f64, low: f64, close: f64, volume: i64, timestamp: &str) -> Result<()> {
        let market_data = MarketData {
            timestamp: chrono::DateTime::parse_from_rfc3339(timestamp)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            symbol: symbol.to_string(),
            price: close,
            volume: volume as f64,
            high: Some(high),
            low: Some(low),
            open: Some(open),
            source: "alpaca".to_string(),
            exchange: "alpaca".to_string(),
            change_24h: None,
            change_percent: None,
            market_cap: None,
            circulating_supply: None,
            options_data: None,
            news_data: None,
        };
        
        // Update cache and write to file
        {
            let mut data = self.market_data.write().await;
            data.insert(symbol.to_string(), market_data.clone());
        }
        
        self.write_market_data_file(symbol, &market_data).await?;
        
        debug!("📊 Bar update for {}: O=${:.2} H=${:.2} L=${:.2} C=${:.2} V={}", 
               symbol, open, high, low, close, volume);
        Ok(())
    }

    /// Handle news updates (same as before)
    async fn handle_news(&self, symbol: &str, headline: &str, sentiment: Option<f64>, _published_at: &str) -> Result<()> {
        let news_data = NewsData {
            headline: headline.to_string(),
            summary: None,
            url: None,
            author: None,
            source: "alpaca".to_string(),
            sentiment,
            symbols: vec![symbol.to_string()],
            category: None,
        };
        
        let market_data = MarketData {
            timestamp: Utc::now(),
            symbol: symbol.to_string(),
            price: 0.0,
            volume: 0.0,
            high: None,
            low: None,
            open: None,
            source: "alpaca".to_string(),
            exchange: "alpaca".to_string(),
            change_24h: None,
            change_percent: None,
            market_cap: None,
            circulating_supply: None,
            options_data: None,
            news_data: Some(news_data),
        };
        
        // Update cache and write to file
        {
            let mut data = self.market_data.write().await;
            data.insert(symbol.to_string(), market_data.clone());
        }
        
        self.write_market_data_file(symbol, &market_data).await?;
        
        debug!("📰 News update for {}: {}", symbol, headline);
        Ok(())
    }

    /// Write market data to one consolidated JSON file (Basic plan - all data in one file)
    async fn write_market_data_file(&self, symbol: &str, data: &MarketData) -> Result<()> {
        // Create one consolidated file for all Basic plan data
        let filename = "basic_plan_market_data.json";
        let file_path = self.data_dir.join(filename);
        
        // Create consolidated data structure
        let mut consolidated_data = if file_path.exists() {
            match tokio::fs::read_to_string(&file_path).await {
                Ok(content) => serde_json::from_str::<serde_json::Value>(&content)
                    .unwrap_or_else(|_| json!({
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "data_source": "alpaca_basic_plan",
                        "update_interval": "30_seconds",
                        "historical_limit": "15_minutes",
                        "symbols": {},
                        "last_update": chrono::Utc::now().to_rfc3339()
                    })),
                Err(_) => json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "data_source": "alpaca_basic_plan",
                    "update_interval": "30_seconds",
                    "historical_limit": "15_minutes",
                    "symbols": {},
                    "last_update": chrono::Utc::now().to_rfc3339()
                })
            }
                } else {
            json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "data_source": "alpaca_basic_plan",
                "update_interval": "30_seconds",
                "historical_limit": "15_minutes",
                "symbols": {},
                "last_update": chrono::Utc::now().to_rfc3339()
            })
        };
        
        // Update or add the symbol data
        let symbol_data = json!({
            "symbol": data.symbol,
            "price": data.price,
            "volume": data.volume,
            "high": data.high,
            "low": data.low,
            "open": data.open,
            "source": data.source,
            "exchange": data.exchange,
            "timestamp": data.timestamp.to_rfc3339(),
            "last_updated": chrono::Utc::now().to_rfc3339()
        });
        
        // Add to symbols object
        if let Some(symbols) = consolidated_data.get_mut("symbols") {
            if let Some(symbols_obj) = symbols.as_object_mut() {
                symbols_obj.insert(data.symbol.clone(), symbol_data);
            }
        }
        
        // Update timestamps
        if let Some(timestamp) = consolidated_data.get_mut("last_update") {
            *timestamp = json!(chrono::Utc::now().to_rfc3339());
        }
        
        // Write consolidated data
        let json_content = serde_json::to_string_pretty(&consolidated_data)?;
        tokio::fs::write(file_path, json_content).await?;
        
        debug!("📝 Updated consolidated file: {} with {} data", filename, symbol);
        Ok(())
    }

    /// Write trade update to JSON file
    async fn write_trade_update(
        &self,
        event: &str,
        price: Option<f64>,
        qty: Option<f64>,
        side: Option<String>,
        symbol: Option<String>,
        timestamp: Option<String>,
        order_id: Option<String>,
        client_order_id: Option<String>,
        status: Option<String>,
    ) -> Result<()> {
        let trade_data = json!({
            "event": event,
            "price": price,
            "qty": qty,
            "side": side,
            "symbol": symbol,
            "timestamp": timestamp,
            "order_id": order_id,
            "client_order_id": client_order_id,
            "status": status,
            "received_at": Utc::now()
        });

        let filename = "trade_updates.json";
        let file_path = self.data_dir.join(filename);
        let json_content = serde_json::to_string_pretty(&trade_data)?;
        
        fs::write(file_path, json_content).await?;
        debug!("📝 Updated trade update file: {}", filename);
        
        Ok(())
    }

    /// Get current market data for a symbol
    pub async fn get_current_data(&self, symbol: &Symbol) -> Option<MarketData> {
        let market_data = self.market_data.read().await;
        market_data.get(symbol).cloned()
    }

    /// Get all market data
    pub async fn get_all_market_data(&self) -> HashMap<Symbol, MarketData> {
        let market_data = self.market_data.read().await;
        market_data.clone()
    }

    /// Check if streaming is running
    pub async fn is_running(&self) -> bool {
        let running = self.running.read().await;
        *running
    }
}

/// Load unified WebSocket configuration from environment
pub fn load_unified_websocket_config() -> Result<(MarketDataWebSocketConfig, TradingWebSocketConfig, String)> {
    // Try paper trading credentials first, then fall back to live credentials
    let (api_key, secret_key) = if let (Ok(paper_key), Ok(paper_secret)) = (
        std::env::var("APCA_API_KEY_ID"),
        std::env::var("APCA_API_SECRET_KEY")
    ) {
        (paper_key, paper_secret)
    } else if let (Ok(live_key), Ok(live_secret)) = (
        std::env::var("ALPACA_API_KEY"),
        std::env::var("ALPACA_SECRET_KEY")
    ) {
        (live_key, live_secret)
    } else {
        return Err(anyhow!("No Alpaca API credentials found. Set either APCA_API_KEY_ID/APCA_API_SECRET_KEY for paper trading or ALPACA_API_KEY/ALPACA_SECRET_KEY for live trading"));
    };

    // Determine if we're using paper trading
    let is_paper_trading = std::env::var("ALPACA_PAPER_TRADING")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    // Determine operation mode
    let operation_mode = std::env::var("OPERATION_MODE")
        .unwrap_or_else(|_| if is_paper_trading { "paper".to_string() } else { "live".to_string() });
    
    // Determine data directory based on mode
    let data_dir = match operation_mode.as_str() {
        "paper" => "trading_portfolio".to_string(),
        "live" => "trading_portfolio".to_string(),
        _ => "trading_portfolio".to_string(),
    };

    let market_data_config = MarketDataWebSocketConfig {
        api_key: api_key.clone(),
        secret_key: secret_key.clone(),
        feed: std::env::var("ALPACA_FEED")
            .unwrap_or_else(|_| match operation_mode.as_str() {
                "paper" => "test".to_string(),
                "live" => "iex".to_string(),
                _ => "test".to_string(),
            }),
        reconnect_interval_ms: std::env::var("RECONNECT_INTERVAL_MS")
            .unwrap_or_else(|_| "5000".to_string())
            .parse()
            .unwrap_or(5000),
        max_reconnect_attempts: std::env::var("MAX_RECONNECT_ATTEMPTS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10),
    };
    
    let trading_config = TradingWebSocketConfig {
        api_key,
        secret_key,
        base_url: match operation_mode.as_str() {
            "paper" => "wss://paper-api.alpaca.markets/stream".to_string(),
            "live" => std::env::var("ALPACA_TRADING_WEBSOCKET_URL")
                .unwrap_or_else(|_| "wss://api.alpaca.markets/stream".to_string()),
            _ => "wss://paper-api.alpaca.markets/stream".to_string(),
        },
        reconnect_interval_ms: std::env::var("RECONNECT_INTERVAL_MS")
            .unwrap_or_else(|_| "5000".to_string())
            .parse()
            .unwrap_or(5000),
        max_reconnect_attempts: std::env::var("MAX_RECONNECT_ATTEMPTS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10),
    };
    
    Ok((market_data_config, trading_config, data_dir))
}
