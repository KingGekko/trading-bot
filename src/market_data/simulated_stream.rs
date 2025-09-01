use crate::market_data::types::{MarketData, OptionsData, NewsData};
use crate::market_data::unified_websocket::{UnifiedAlpacaWebSocket, StreamType as AlpacaStreamType};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use std::collections::HashMap;
use rand::Rng;
use serde_json;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// Stream types for paper trading
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StreamType {
    Crypto,
    Stocks,
    Options,
    News,
}

/// Stream data structure - one file per stream type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamData {
    pub timestamp: String,
    pub stream_type: String,
    pub symbols: Vec<String>,
    pub data: Vec<MarketData>,
    pub last_update: String,
}

/// Real-time market data stream for TEST mode (Paper Trading with Alpaca data)
pub struct SimulatedMarketStream {
    /// Current stream data for each stream type
    stream_data: Arc<RwLock<HashMap<StreamType, StreamData>>>,
    /// Stream configuration
    update_interval: Duration,
    /// Running state
    is_running: Arc<RwLock<bool>>,
    /// Default symbols for each stream type
    default_symbols: HashMap<StreamType, Vec<String>>,
    /// Alpaca WebSocket for real data (when available)
    alpaca_websocket: Option<UnifiedAlpacaWebSocket>,
    /// Data directory
    data_dir: PathBuf,
}

impl SimulatedMarketStream {
    /// Create a new simulated market stream for paper trading
    pub fn new() -> Self {
        let mut default_symbols = HashMap::new();
        default_symbols.insert(StreamType::Crypto, vec!["BTC/USD".to_string(), "ETH/USD".to_string()]);
        default_symbols.insert(StreamType::Stocks, vec!["AAPL".to_string(), "SPY".to_string(), "TSLA".to_string()]);
        default_symbols.insert(StreamType::Options, vec!["SPY240920C00500000".to_string(), "SPY240920P00500000".to_string()]);
        default_symbols.insert(StreamType::News, vec!["MARKET_NEWS".to_string(), "ECONOMIC_CALENDAR".to_string()]);
        
        // Determine data directory based on operation mode
        let data_dir = if let Ok(mode) = std::env::var("OPERATION_MODE") {
            match mode.as_str() {
                "paper" => PathBuf::from("trading_portfolio"),
                "live" => PathBuf::from("trading_portfolio"),
                _ => PathBuf::from("trading_portfolio"),
            }
        } else {
            PathBuf::from("trading_portfolio") // Default to trading portfolio
        };
        
        Self {
            stream_data: Arc::new(RwLock::new(HashMap::new())),
            update_interval: Duration::from_millis(2000), // Update every 2 seconds
            is_running: Arc::new(RwLock::new(false)),
            default_symbols,
            alpaca_websocket: None,
            data_dir,
        }
    }

    /// Start the market data stream (Alpaca when available, simulated when not)
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize Alpaca WebSocket if possible
        self.initialize_alpaca().await?;
        
        let mut is_running = self.is_running.write().await;
        *is_running = true;
        drop(is_running);

        let stream_data = Arc::clone(&self.stream_data);
        let is_running = Arc::clone(&self.is_running);
        let update_interval = self.update_interval;
        let default_symbols = self.default_symbols.clone();
        let alpaca_ws = self.alpaca_websocket.clone();
        let data_dir = self.data_dir.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(update_interval);
            
            while *is_running.read().await {
                interval.tick().await;
                
                if let Some(alpaca_ws) = &alpaca_ws {
                    // Use real Alpaca data - start streaming in background
                    let alpaca_ws_clone = alpaca_ws.clone();
                    tokio::spawn(async move {
                        eprintln!("🔍 Attempting to start Alpaca streaming...");
                        match alpaca_ws_clone.start_streaming().await {
                            Ok(_) => {
                                eprintln!("✅ Alpaca streaming started successfully!");
                            }
                            Err(e) => {
                                eprintln!("❌ Alpaca streaming failed: {}", e);
                                eprintln!("🔍 Error details: {:?}", e);
                            }
                        }
                    });
                    
                    // Wait for Alpaca data to be written (check if files exist and are recent)
                    let mut has_alpaca_data = false;
                    let data_dir = if let Ok(mode) = std::env::var("OPERATION_MODE") {
                        match mode.as_str() {
                                                    "paper" => "trading_portfolio",
                        "live" => "trading_portfolio",
                        _ => "trading_portfolio",
                        }
                    } else {
                        "trading_portfolio"
                    };
                    
                    for stream_type in &[StreamType::Crypto, StreamType::Stocks, StreamType::Options, StreamType::News] {
                        let filename = match stream_type {
                            StreamType::Crypto => format!("{}/crypto_stream.json", data_dir),
                            StreamType::Stocks => format!("{}/stocks_stream.json", data_dir),
                            StreamType::Options => format!("{}/options_stream.json", data_dir),
                            StreamType::News => format!("{}/news_stream.json", data_dir),
                        };
                        
                        if let Ok(metadata) = std::fs::metadata(filename) {
                            if let Ok(modified) = metadata.modified() {
                                if let Ok(duration) = std::time::SystemTime::now().duration_since(modified) {
                                    if duration.as_secs() < 10 { // File updated in last 10 seconds
                                        has_alpaca_data = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    
                    if has_alpaca_data {
                        // Alpaca is providing data, skip simulation
                        continue;
                    }
                }
                
                // Fallback to simulated data
                let mut data = stream_data.write().await;
                
                for (stream_type, symbols) in &default_symbols {
                    let stream_data = Self::generate_stream_data(stream_type, symbols);
                    data.insert(stream_type.clone(), stream_data);
                }
                
                drop(data);
                
                // Save to files for persistence
                Self::save_stream_data(&stream_data, &data_dir).await;
            }
        });
        
        Ok(())
    }

    /// Stop the simulated data stream
    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
    }

    /// Get current stream data for a specific stream type
    pub async fn get_stream_data(&self, stream_type: &StreamType) -> Option<StreamData> {
        let data = self.stream_data.read().await;
        data.get(stream_type).cloned()
    }

    /// Get all current stream data
    pub async fn get_all_stream_data(&self) -> HashMap<StreamType, StreamData> {
        let data = self.stream_data.read().await;
        data.clone()
    }

    /// Get available stream types
    pub fn get_available_streams(&self) -> Vec<StreamType> {
        vec![StreamType::Crypto, StreamType::Stocks, StreamType::Options, StreamType::News]
    }

    /// Initialize Alpaca WebSocket if API keys are available
    pub async fn initialize_alpaca(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Try to load Alpaca configuration
        match crate::market_data::load_unified_websocket_config() {
            Ok((market_data_config, trading_config, _)) => {
                // Create data directory
                std::fs::create_dir_all(&self.data_dir)?;
                
                // Create Alpaca WebSocket
                let is_paper_trading = std::env::var("ALPACA_PAPER_TRADING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse::<bool>()
                    .unwrap_or(true);
                
                let alpaca_ws = UnifiedAlpacaWebSocket::new(
                    market_data_config,
                    trading_config,
                    self.data_dir.clone(),
                    vec![AlpacaStreamType::MarketData],
                    is_paper_trading
                )?;
                
                self.alpaca_websocket = Some(alpaca_ws);
                Ok(())
            }
            Err(_) => {
                // No API keys available, will use simulated data
                Ok(())
            }
        }
    }

    /// Generate stream data for a specific stream type
    fn generate_stream_data(stream_type: &StreamType, symbols: &[String]) -> StreamData {
        let mut market_data_vec = Vec::new();
        
        for symbol in symbols {
            let market_data = match stream_type {
                StreamType::Crypto => Self::generate_crypto_data(symbol),
                StreamType::Stocks => Self::generate_stock_data(symbol),
                StreamType::Options => Self::generate_option_data(symbol),
                StreamType::News => Self::generate_news_data(symbol),
            };
            market_data_vec.push(market_data);
        }

        StreamData {
            timestamp: Utc::now().to_rfc3339(),
            stream_type: format!("{:?}", stream_type).to_lowercase(),
            symbols: symbols.to_vec(),
            data: market_data_vec,
            last_update: Utc::now().to_rfc3339(),
        }
    }

    /// Generate crypto market data
    fn generate_crypto_data(symbol: &str) -> MarketData {
        let mut rng = rand::rng();
        let base_price = match symbol {
            "BTC/USD" => 45000.0,
            "ETH/USD" => 2800.0,
            _ => 1000.0,
        };
        
        // Generate realistic price movements (±2% range)
        let price_change = base_price * rng.random_range(-0.02..0.02);
        let current_price = base_price + price_change;
        
        MarketData {
            timestamp: Utc::now(),
            symbol: symbol.to_string(),
            price: current_price,
            volume: rng.random_range(800.0..2000.0),
            high: Some(current_price * rng.random_range(1.0..1.01)),
            low: Some(current_price * rng.random_range(0.99..1.0)),
            open: Some(base_price),
            source: "simulated_crypto_feed".to_string(),
            exchange: "simulated".to_string(),
            change_24h: Some(price_change),
            change_percent: Some((price_change / base_price) * 100.0),
            market_cap: Some(if symbol == "BTC/USD" { 850_000_000_000.0 } else { 350_000_000_000.0 }),
            circulating_supply: Some(if symbol == "BTC/USD" { 18_890_000.0 } else { 120_000_000.0 }),
            options_data: None,
            news_data: None,
        }
    }

    /// Generate stock market data
    fn generate_stock_data(symbol: &str) -> MarketData {
        let mut rng = rand::rng();
        let base_price = match symbol {
            "AAPL" => 175.0,
            "SPY" => 450.0,
            "TSLA" => 250.0,
            _ => 100.0,
        };
        
        let price_change = base_price * rng.random_range(-0.015..0.015);
        let current_price = base_price + price_change;
        
        MarketData {
            timestamp: Utc::now(),
            symbol: symbol.to_string(),
            price: current_price,
            volume: rng.random_range(100000.0..500000.0),
            high: Some(current_price * rng.random_range(1.0..1.008)),
            low: Some(current_price * rng.random_range(0.992..1.0)),
            open: Some(base_price),
            source: "simulated_stock_feed".to_string(),
            exchange: "simulated".to_string(),
            change_24h: Some(price_change),
            change_percent: Some((price_change / base_price) * 100.0),
            market_cap: None,
            circulating_supply: None,
            options_data: None,
            news_data: None,
        }
    }

    /// Generate option market data
    fn generate_option_data(symbol: &str) -> MarketData {
        let mut rng = rand::rng();
        let underlying_price = 450.0; // SPY price
        let strike = underlying_price * rng.random_range(0.95..1.05);
        let option_price = rng.random_range(5.0..25.0);
        
        let options_data = OptionsData {
            strike,
            expiration: Utc::now() + chrono::Duration::days(rng.random_range(30..90)),
            option_type: if symbol.contains("C") { "call".to_string() } else { "put".to_string() },
            underlying: "SPY".to_string(),
            implied_volatility: Some(rng.random_range(0.15..0.45)),
            delta: Some(rng.random_range(-1.0..1.0)),
            gamma: Some(rng.random_range(0.0..0.1)),
            theta: Some(rng.random_range(-0.05..0.0)),
            vega: Some(rng.random_range(0.0..0.2)),
            open_interest: Some(rng.random_range(100..10000)),
            bid: Some(option_price * 0.98),
            ask: Some(option_price * 1.02),
        };
        
        MarketData {
            timestamp: Utc::now(),
            symbol: symbol.to_string(),
            price: option_price,
            volume: rng.random_range(50.0..500.0),
            high: Some(option_price * rng.random_range(1.0..1.05)),
            low: Some(option_price * rng.random_range(0.95..1.0)),
            open: Some(option_price),
            source: "simulated_options_feed".to_string(),
            exchange: "simulated".to_string(),
            change_24h: Some(rng.random_range(-2.0..2.0)),
            change_percent: Some(rng.random_range(-5.0..5.0)),
            market_cap: None,
            circulating_supply: None,
            options_data: Some(options_data),
            news_data: None,
        }
    }

    /// Generate news data
    fn generate_news_data(symbol: &str) -> MarketData {
        let mut rng = rand::rng();
        let headlines = vec![
            "Market volatility increases as Fed meeting approaches",
            "Tech stocks show strong momentum in pre-market trading",
            "Cryptocurrency adoption continues to grow globally",
            "Options trading volume spikes ahead of earnings season",
            "Economic data shows strong consumer spending",
            "Global markets react to central bank policy changes",
        ];
        
        let headline = headlines[rng.random_range(0..headlines.len())].to_string();
        
        let news_data = NewsData {
            headline,
            summary: Some("Simulated news data for paper trading purposes".to_string()),
            url: Some("https://example.com/simulated-news".to_string()),
            author: Some("Test System".to_string()),
            source: "simulated_news".to_string(),
            sentiment: Some(rng.random_range(-0.5..0.5)),
            symbols: vec!["SPY".to_string(), "AAPL".to_string(), "BTC/USD".to_string()],
            category: Some("market_analysis".to_string()),
        };
        
        MarketData {
            timestamp: Utc::now(),
            symbol: symbol.to_string(),
            price: 0.0, // News doesn't have a price
            volume: 0.0, // News doesn't have volume
            high: None,
            low: None,
            open: None,
            source: "simulated_news_feed".to_string(),
            exchange: "simulated".to_string(),
            change_24h: None,
            change_percent: None,
            market_cap: None,
            circulating_supply: None,
            options_data: None,
            news_data: Some(news_data),
        }
    }

    /// Save stream data to files - one file per stream type
    async fn save_stream_data(stream_data: &Arc<RwLock<HashMap<StreamType, StreamData>>>, data_dir: &PathBuf) {
        let data = stream_data.read().await;
        
        // Ensure data directory exists
        let _ = std::fs::create_dir_all(data_dir);
        
        // Save each stream type to its respective file
        for (stream_type, stream_data_item) in data.iter() {
            let filename = match stream_type {
                StreamType::Crypto => format!("{}/crypto_stream.json", data_dir.display()),
                StreamType::Stocks => format!("{}/stocks_stream.json", data_dir.display()),
                StreamType::Options => format!("{}/options_stream.json", data_dir.display()),
                StreamType::News => format!("{}/news_stream.json", data_dir.display()),
            };
            
            if let Ok(json_data) = serde_json::to_string_pretty(stream_data_item) {
                let _ = std::fs::write(filename, json_data);
            }
        }
    }
}

impl Default for SimulatedMarketStream {
    fn default() -> Self {
        Self::new()
    }
}
