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
pub struct UnifiedAlpacaWebSocket {
    market_data_config: MarketDataWebSocketConfig,
    trading_config: TradingWebSocketConfig,
    data_dir: PathBuf,
    market_data: Arc<RwLock<HashMap<Symbol, MarketData>>>,
    running: Arc<RwLock<bool>>,
    stream_types: Vec<StreamType>,
    websocket_urls: HashMap<StreamType, String>,
    account_verification: Option<AccountVerification>,
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
    ) -> Result<Self> {
        // Determine WebSocket URLs based on stream types
        let mut websocket_urls = HashMap::new();
        
        // Market Data URLs
        if stream_types.contains(&StreamType::MarketData) {
            let market_data_url = match market_data_config.feed.as_str() {
                "test" => "wss://stream.data.alpaca.markets/v2/test".to_string(),
                "iex" => "wss://stream.data.alpaca.markets/v2/iex".to_string(),
                "sip" => "wss://stream.data.alpaca.markets/v2/sip".to_string(),
                "opra" => "wss://stream.data.alpaca.markets/v1beta1/opra".to_string(),
                "indicative" => "wss://stream.data.alpaca.markets/v1beta1/indicative".to_string(),
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
        })
    }

    /// Start unified WebSocket streaming
    pub async fn start_streaming(&self) -> Result<()> {
        info!("🚀 Starting Unified Alpaca WebSocket streaming...");
        
        // Verify account before starting streaming
        let _account_verification = self.verify_account_before_streaming().await?;
        
        // Create data directory
        fs::create_dir_all(&self.data_dir).await?;
        
        // Set running flag
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        // Start all streams concurrently using ultra-threading
        let mut stream_futures = Vec::new();
        
        for stream_type in &self.stream_types {
            if let Some(url) = self.websocket_urls.get(stream_type) {
                let stream_type = stream_type.clone();
                let url = url.clone();
                let future: std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> = match stream_type {
                    StreamType::MarketData => {
                        let config = self.market_data_config.clone();
                        Box::pin(self.run_market_data_stream(stream_type, url, config))
                    }
                    _ => {
                        let config = self.trading_config.clone();
                        Box::pin(self.run_trading_stream(stream_type, url, config))
                    }
                };
                stream_futures.push(future);
            }
        }
        
        // Execute all streams concurrently
        let results = futures::future::join_all(stream_futures).await;
        
        // Check results
        for (i, result) in results.into_iter().enumerate() {
            if let Err(e) = result {
                error!("❌ Stream {} failed: {}", i, e);
            }
        }
        
        Ok(())
    }

    /// Verify account before starting streaming
    async fn verify_account_before_streaming(&self) -> Result<AccountVerification> {
        info!("🔍 Verifying Alpaca account before starting streaming...");
        
        // Create account verifier
        let verifier = AccountVerifier::new(
            self.market_data_config.api_key.clone(),
            self.market_data_config.secret_key.clone(),
            self.is_paper_trading(),
        );
        
        // Verify account
        let verification = verifier.verify_account().await?;
        
        // Validate requested stream types against account permissions
        let valid_streams = verifier.validate_stream_types(&verification, &self.get_stream_type_names())?;
        
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
        self.trading_config.base_url.contains("paper-api.alpaca.markets")
    }

    /// Get stream type names for validation
    fn get_stream_type_names(&self) -> Vec<String> {
        self.stream_types.iter().map(|st| match st {
            StreamType::MarketData => "stocks,crypto,options,news".to_string(),
            StreamType::TradeUpdates => "trade_updates".to_string(),
            StreamType::AccountUpdates => "account_updates".to_string(),
            StreamType::OrderUpdates => "order_updates".to_string(),
        }).collect()
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
        
        let (ws_stream, _) = connect_async(url).await?;
        info!("✅ Market data WebSocket connected successfully");
        
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

    /// Handle market data WebSocket stream
    async fn handle_market_data_stream(
        &self,
        mut ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        config: &MarketDataWebSocketConfig,
    ) -> Result<()> {
        // Wait for welcome message
        if let Some(msg) = ws_stream.next().await {
            match msg? {
                tokio_tungstenite::tungstenite::Message::Text(text) => {
                    let messages: Vec<Value> = serde_json::from_str(&text)?;
                    for msg in messages {
                        if let Some(msg_type) = msg["T"].as_str() {
                            if msg_type == "success" && msg["msg"] == "connected" {
                                info!("✅ Received welcome message from Alpaca market data");
                                break;
                            }
                        }
                    }
                }
                _ => return Err(anyhow!("Expected text message for welcome")),
            }
        }

        // Authenticate
        let auth_msg = json!({
            "action": "auth",
            "key": config.api_key,
            "secret": config.secret_key
        });
        
        ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(auth_msg.to_string().into())).await?;
        info!("🔐 Market data authentication sent");
        
        // Wait for authentication response
        if let Some(msg) = ws_stream.next().await {
            match msg? {
                tokio_tungstenite::tungstenite::Message::Text(text) => {
                    let messages: Vec<Value> = serde_json::from_str(&text)?;
                    for msg in messages {
                        if let Some(msg_type) = msg["T"].as_str() {
                            if msg_type == "success" && msg["msg"] == "authenticated" {
                                info!("✅ Market data authentication successful");
                                break;
                            } else if msg_type == "error" {
                                let error_msg = msg["msg"].as_str().unwrap_or("Unknown error");
                                return Err(anyhow!("Market data authentication failed: {}", error_msg));
                            }
                        }
                    }
                }
                _ => return Err(anyhow!("Expected text message for authentication")),
            }
        }

        // Subscribe to market data streams
        let mut subscriptions = Vec::new();
        
        for stream_type in &self.stream_types {
            match stream_type {
                StreamType::MarketData => {
                    subscriptions.extend(vec!["AAPL", "SPY", "BTC/USD", "ETH/USD"]);
                }
                _ => {}
            }
        }
        
        let subscribe_msg = json!({
            "action": "subscribe",
            "trades": subscriptions.clone(),
            "quotes": subscriptions.clone(),
            "bars": subscriptions
        });
        
        ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(subscribe_msg.to_string().into())).await?;
        info!("📡 Market data subscription sent");
        
        // Process incoming messages
        while {
            let running = self.running.read().await;
            *running
        } {
            match ws_stream.next().await {
                Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                    self.process_market_data_message(&text).await?;
                }
                Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => {
                    info!("🔌 Market data WebSocket connection closed by server");
                    break;
                }
                Some(Err(e)) => {
                    error!("❌ Market data WebSocket error: {}", e);
                    break;
                }
                None => {
                    info!("🔌 Market data WebSocket stream ended");
                    break;
                }
                _ => {}
            }
        }
        
        Ok(())
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
        let messages: Vec<Value> = serde_json::from_str(text)?;
        
        for msg in messages {
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
                    UnifiedAlpacaMessage::Subscription { trades, quotes, bars } => {
                        info!("📡 Market data subscription confirmed - Trades: {:?}, Quotes: {:?}, Bars: {:?}", trades, quotes, bars);
                    }
                    UnifiedAlpacaMessage::Success { msg } => {
                        debug!("✅ Market data success message: {}", msg);
                    }
                    UnifiedAlpacaMessage::Error { code, msg } => {
                        warn!("⚠️ Market data error message ({}): {}", code, msg);
                    }
                    _ => {}
                }
            }
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

    /// Write market data to JSON file (same as before)
    async fn write_market_data_file(&self, symbol: &str, data: &MarketData) -> Result<()> {
        let filename = match symbol {
            s if s.contains("BTC") => "crypto_data_btc.json",
            s if s.contains("ETH") => "crypto_data_eth.json",
            s if s.contains("AAPL") => "stock_data_aapl.json",
            s if s.contains("SPY") => {
                if s.contains("C") || s.contains("P") {
                    "options_data_spy.json"
                } else {
                    "stock_data_spy.json"
                }
            }
            _ => &format!("market_data_{}.json", symbol.to_lowercase().replace('/', "_")),
        };

        let file_path = self.data_dir.join(filename);
        let json_content = serde_json::to_string_pretty(data)?;
        
        fs::write(file_path, json_content).await?;
        debug!("📝 Updated market data file: {}", filename);
        
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
pub fn load_unified_websocket_config() -> Result<(MarketDataWebSocketConfig, TradingWebSocketConfig)> {
    let market_data_config = MarketDataWebSocketConfig {
        api_key: std::env::var("ALPACA_API_KEY")
            .map_err(|_| anyhow!("ALPACA_API_KEY not set"))?,
        secret_key: std::env::var("ALPACA_SECRET_KEY")
            .map_err(|_| anyhow!("ALPACA_SECRET_KEY not set"))?,
        feed: std::env::var("ALPACA_FEED")
            .unwrap_or_else(|_| "test".to_string()),
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
        api_key: std::env::var("ALPACA_API_KEY")
            .map_err(|_| anyhow!("ALPACA_API_KEY not set"))?,
        secret_key: std::env::var("ALPACA_SECRET_KEY")
            .map_err(|_| anyhow!("ALPACA_SECRET_KEY not set"))?,
        base_url: std::env::var("ALPACA_TRADING_WEBSOCKET_URL")
            .unwrap_or_else(|_| "wss://paper-api.alpaca.markets/stream".to_string()),
        reconnect_interval_ms: std::env::var("RECONNECT_INTERVAL_MS")
            .unwrap_or_else(|_| "5000".to_string())
            .parse()
            .unwrap_or(5000),
        max_reconnect_attempts: std::env::var("MAX_RECONNECT_ATTEMPTS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10),
    };
    
    Ok((market_data_config, trading_config))
}
