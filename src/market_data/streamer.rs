use crate::market_data::types::{MarketData, Symbol, OptionsData, NewsData};
use anyhow::Result;
use chrono::Utc;
use log::{debug, error, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

/// Alpaca API client for streaming market data
#[derive(Clone)]
pub struct AlpacaStreamer {
    /// HTTP client for API requests
    client: Client,
    /// API configuration
    config: AlpacaConfig,
    /// Current market data cache
    market_data: Arc<RwLock<HashMap<Symbol, MarketData>>>,
    /// Data directory for JSON files
    data_dir: PathBuf,
    /// Running state
    running: Arc<RwLock<bool>>,
    /// Stream types to monitor
    stream_types: Vec<StreamType>,
}

/// Configuration for Alpaca API
#[derive(Debug, Clone)]
pub struct AlpacaConfig {
    pub api_key: String,
    pub secret_key: String,
    pub base_url: String,
    pub websocket_url: String,
    pub symbols: Vec<Symbol>,
    pub update_interval_ms: u64,
    pub data_retention_hours: u32,
    pub stream_types: Vec<StreamType>,
}

/// Types of data streams available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamType {
    Options,
    Crypto,
    News,
    Stocks,
}

impl std::fmt::Display for StreamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamType::Options => write!(f, "Options"),
            StreamType::Crypto => write!(f, "Crypto"),
            StreamType::News => write!(f, "News"),
            StreamType::Stocks => write!(f, "Stocks"),
        }
    }
}

/// Alpaca API response types
#[derive(Debug, Deserialize)]
struct AlpacaCryptoBars {
    bars: HashMap<String, AlpacaCryptoBar>,
}

#[derive(Debug, Deserialize)]
struct AlpacaCryptoBar {
    c: f64, // Close price
    h: f64, // High price
    l: f64, // Low price
    o: f64, // Open price
    v: f64, // Volume
    t: String, // Timestamp
}

#[derive(Debug, Deserialize)]
struct AlpacaStockTrade {
    trade: AlpacaTrade,
}

#[derive(Debug, Deserialize)]
struct AlpacaTrade {
    p: f64, // Price
    s: i64, // Size
    x: String, // Exchange
    t: String, // Timestamp
}

#[derive(Debug, Deserialize)]
struct AlpacaOptionsQuote {
    quote: AlpacaOptionsData,
}

#[derive(Debug, Deserialize)]
struct AlpacaOptionsData {
    p: f64, // Price
    s: i64, // Size
    x: String, // Exchange
    t: String, // Timestamp
    strike: f64,
    expiration: String,
    option_type: String,
    underlying: String,
    implied_volatility: Option<f64>,
    delta: Option<f64>,
    gamma: Option<f64>,
    theta: Option<f64>,
    vega: Option<f64>,
    open_interest: Option<i64>,
    bid: Option<f64>,
    ask: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct AlpacaNewsArticle {
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
}

#[derive(Debug, Deserialize)]
struct AlpacaAccount {
    id: String,
    account_number: String,
    status: String,
}

impl AlpacaStreamer {
    /// Create a new Alpaca streamer
    pub fn new(config: AlpacaConfig, data_dir: PathBuf) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            config: config.clone(),
            market_data: Arc::new(RwLock::new(HashMap::new())),
            data_dir,
            running: Arc::new(RwLock::new(false)),
            stream_types: config.stream_types.clone(),
        })
    }

    /// Start the market data streaming
    pub async fn start_streaming(&self) -> Result<()> {
        info!("Starting Alpaca market data streaming...");
        
        // Ensure data directory exists
        fs::create_dir_all(&self.data_dir).await?;
        
        // Test connection first
        self.test_connection().await?;
        
        // Set running state
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        // Start streaming loop
        self.stream_market_data().await?;
        
        Ok(())
    }

    /// Stop the market data streaming
    pub async fn stop_streaming(&self) {
        info!("Stopping Alpaca market data streaming...");
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Test connection to Alpaca API
    async fn test_connection(&self) -> Result<()> {
        info!("Testing Alpaca API connection...");
        
        let response = self
            .client
            .get(&format!("{}/v2/account", self.config.base_url))
            .header("APCA-API-KEY-ID", &self.config.api_key)
            .header("APCA-API-SECRET-KEY", &self.config.secret_key)
            .send()
            .await?;

        if response.status().is_success() {
            let account: AlpacaAccount = response.json().await?;
            info!("Connected to Alpaca account: {}", account.id);
            Ok(())
        } else {
            let error_text = response.text().await?;
            error!("Failed to connect to Alpaca API: {}", error_text);
            Err(anyhow::anyhow!("Failed to connect to Alpaca API"))
        }
    }

    /// Main streaming loop with ULTRA-THREADING
    async fn stream_market_data(&self) -> Result<()> {
        let update_interval = Duration::from_millis(self.config.update_interval_ms);
        
        loop {
            // Check if we should stop
            {
                let running = self.running.read().await;
                if !*running {
                    break;
                }
            }
            
            // ULTRA-THREADING: Process all stream types in parallel
            let stream_futures: Vec<_> = self.stream_types
                .iter()
                .map(|stream_type| {
                    let stream_type = stream_type.clone();
                    tokio::spawn(async move {
                        // This will be handled by the calling context
                        stream_type
                    })
                })
                .collect();
            
            // Wait for all stream type futures to be ready
            let _: Vec<_> = futures::future::join_all(stream_futures).await
                .into_iter()
                .filter_map(|r| r.ok())
                .collect();
            
            // ULTRA-THREADING: Execute all stream types concurrently
            let stream_execution_futures: Vec<_> = self.stream_types
                .iter()
                .map(|stream_type| {
                    let stream_type = stream_type.clone();
                    let streamer = self.clone();
                    tokio::spawn(async move {
                        streamer.stream_data_by_type(&stream_type).await
                    })
                })
                .collect();
            
            // Execute all streams concurrently and collect results
            let results = futures::future::join_all(stream_execution_futures).await;
            
            // Process results
            for (i, result) in results.into_iter().enumerate() {
                match result {
                    Ok(Ok(_)) => {
                        if let Some(stream_type) = self.stream_types.get(i) {
                            debug!("Successfully streamed {} data", stream_type);
                        }
                    }
                    Ok(Err(e)) => {
                        if let Some(stream_type) = self.stream_types.get(i) {
                            warn!("Failed to stream {} data: {}", stream_type, e);
                        }
                    }
                    Err(e) => {
                        if let Some(stream_type) = self.stream_types.get(i) {
                            error!("Stream {} task failed: {}", stream_type, e);
                        }
                    }
                }
            }
            
            // Wait for next update
            sleep(update_interval).await;
        }
        
        Ok(())
    }

    /// Stream data based on stream type
    async fn stream_data_by_type(&self, stream_type: &StreamType) -> Result<()> {
        match stream_type {
            StreamType::Options => self.stream_options_data().await,
            StreamType::Crypto => self.stream_crypto_data().await,
            StreamType::News => self.stream_news_data().await,
            StreamType::Stocks => self.stream_stocks_data().await,
        }
    }

    /// Get market data for a specific symbol
    async fn get_market_data(&self, symbol: &Symbol) -> Result<MarketData> {
        let endpoint = if symbol.contains('/') {
            // Crypto endpoint
            format!("{}/v1beta3/crypto/latest/bars", self.config.base_url)
        } else {
            // Stock endpoint
            format!("{}/v2/stocks/{}/trades/latest", self.config.base_url, symbol)
        };

        let mut request = self
            .client
            .get(&endpoint)
            .header("APCA-API-KEY-ID", &self.config.api_key)
            .header("APCA-API-SECRET-KEY", &self.config.secret_key);

        // Add crypto symbols parameter
        if symbol.contains('/') {
            let base = symbol.split('/').next().unwrap_or("BTC");
            request = request.query(&[("symbols", base)]);
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let data = if symbol.contains('/') {
                self.parse_crypto_data(symbol, response.json::<AlpacaCryptoBars>().await?).await?
            } else {
                self.parse_stock_data(symbol, response.json::<AlpacaStockTrade>().await?).await?
            };
            Ok(data)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("API request failed: {}", error_text))
        }
    }

    /// Parse crypto market data
    async fn parse_crypto_data(&self, symbol: &str, data: AlpacaCryptoBars) -> Result<MarketData> {
        let base = symbol.split('/').next().unwrap_or("BTC");
        let bar = data.bars.get(base).ok_or_else(|| {
            anyhow::anyhow!("No data found for symbol {}", symbol)
        })?;

        Ok(MarketData {
            timestamp: Utc::now(),
            symbol: symbol.to_string(),
            price: bar.c,
            volume: bar.v,
            high: Some(bar.h),
            low: Some(bar.l),
            open: Some(bar.o),
            source: "alpaca_crypto".to_string(),
            exchange: "alpaca".to_string(),
            change_24h: None,
            change_percent: None,
            market_cap: None,
            circulating_supply: None,
            options_data: None,
            news_data: None,
        })
    }

    /// Parse stock market data
    async fn parse_stock_data(&self, symbol: &str, data: AlpacaStockTrade) -> Result<MarketData> {
        let trade = &data.trade;

        Ok(MarketData {
            timestamp: Utc::now(),
            symbol: symbol.to_string(),
            price: trade.p,
            volume: trade.s as f64,
            high: None,
            low: None,
            open: None,
            source: "alpaca_stocks".to_string(),
            exchange: trade.x.clone(),
            change_24h: None,
            change_percent: None,
            market_cap: None,
            circulating_supply: None,
            options_data: None,
            news_data: None,
        })
    }

    /// Parse options data
    async fn parse_options_data(&self, symbol: &str, data: AlpacaOptionsQuote) -> Result<MarketData> {
        let quote = &data.quote;
        
        let options_data = OptionsData {
            strike: quote.strike,
            expiration: chrono::DateTime::parse_from_rfc3339(&quote.expiration)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc),
            option_type: quote.option_type.clone(),
            underlying: quote.underlying.clone(),
            implied_volatility: quote.implied_volatility,
            delta: quote.delta,
            gamma: quote.gamma,
            theta: quote.theta,
            vega: quote.vega,
            open_interest: quote.open_interest,
            bid: quote.bid,
            ask: quote.ask,
        };

        Ok(MarketData {
            timestamp: Utc::now(),
            symbol: symbol.to_string(),
            price: quote.p,
            volume: quote.s as f64,
            high: None,
            low: None,
            open: None,
            source: "alpaca_options".to_string(),
            exchange: quote.x.clone(),
            change_24h: None,
            change_percent: None,
            market_cap: None,
            circulating_supply: None,
            options_data: Some(options_data),
            news_data: None,
        })
    }

    /// Parse news data
    async fn parse_news_data(&self, symbol: &str, article: &AlpacaNewsArticle) -> Result<MarketData> {
        let news_data = NewsData {
            headline: article.headline.clone(),
            summary: article.summary.clone(),
            url: article.url.clone(),
            author: article.author.clone(),
            source: article.source.clone(),
            sentiment: article.sentiment,
            symbols: article.symbols.clone(),
            category: article.category.clone(),
        };

        Ok(MarketData {
            timestamp: Utc::now(),
            symbol: symbol.to_string(),
            price: 0.0, // News doesn't have price
            volume: 0.0, // News doesn't have volume
            high: None,
            low: None,
            open: None,
            source: "alpaca_news".to_string(),
            exchange: "alpaca".to_string(),
            change_24h: None,
            change_percent: None,
            market_cap: None,
            circulating_supply: None,
            options_data: None,
            news_data: Some(news_data),
        })
    }

    /// Stream options data with ULTRA-THREADING
    async fn stream_options_data(&self) -> Result<()> {
        info!("Streaming options data with ULTRA-THREADING...");
        
        // ULTRA-THREADING: Process all options symbols concurrently
        let options_symbols: Vec<_> = self.config.symbols
            .iter()
            .filter(|symbol| self.is_options_symbol(symbol))
            .cloned()
            .collect();
        
        if options_symbols.is_empty() {
            debug!("No options symbols configured");
            return Ok(());
        }
        
        // Create concurrent tasks for each symbol
        let symbol_futures: Vec<_> = options_symbols
            .iter()
            .map(|symbol| {
                let symbol = symbol.clone();
                let streamer = self.clone();
                tokio::spawn(async move {
                    streamer.process_single_options_symbol(&symbol).await
                })
            })
            .collect();
        
        // Execute all symbol processing concurrently
        let results = futures::future::join_all(symbol_futures).await;
        
        // Process results
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok(_)) => {
                    if let Some(symbol) = options_symbols.get(i) {
                        debug!("Successfully processed options data for {}", symbol);
                    }
                }
                Ok(Err(e)) => {
                    if let Some(symbol) = options_symbols.get(i) {
                        warn!("Failed to process options data for {}: {}", symbol, e);
                    }
                }
                Err(e) => {
                    if let Some(symbol) = options_symbols.get(i) {
                        error!("Options processing task failed for {}: {}", symbol, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a single options symbol (helper method for ultra-threading)
    async fn process_single_options_symbol(&self, symbol: &str) -> Result<()> {
        match self.get_options_data(symbol).await {
            Ok(data) => {
                // Update cache
                {
                    let mut market_data = self.market_data.write().await;
                    market_data.insert(symbol.to_string(), data.clone());
                }
                
                // Write to file
                if let Err(e) = self.write_market_data_file(symbol, &data).await {
                    error!("Failed to write options data for {}: {}", symbol, e);
                }
                Ok(())
            }
            Err(e) => {
                warn!("Failed to get options data for {}: {}", symbol, e);
                Err(e)
            }
        }
    }

    /// Stream crypto data with ULTRA-THREADING
    async fn stream_crypto_data(&self) -> Result<()> {
        info!("Streaming crypto data with ULTRA-THREADING...");
        
        // ULTRA-THREADING: Process all crypto symbols concurrently
        let crypto_symbols: Vec<_> = self.config.symbols
            .iter()
            .filter(|symbol| self.is_crypto_symbol(symbol))
            .cloned()
            .collect();
        
        if crypto_symbols.is_empty() {
            debug!("No crypto symbols configured");
            return Ok(());
        }
        
        // Create concurrent tasks for each symbol
        let symbol_futures: Vec<_> = crypto_symbols
            .iter()
            .map(|symbol| {
                let symbol = symbol.clone();
                let streamer = self.clone();
                tokio::spawn(async move {
                    streamer.process_single_crypto_symbol(&symbol).await
                })
            })
            .collect();
        
        // Execute all symbol processing concurrently
        let results = futures::future::join_all(symbol_futures).await;
        
        // Process results
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok(_)) => {
                    if let Some(symbol) = crypto_symbols.get(i) {
                        debug!("Successfully processed crypto data for {}", symbol);
                    }
                }
                Ok(Err(e)) => {
                    if let Some(symbol) = crypto_symbols.get(i) {
                        warn!("Failed to process crypto data for {}: {}", symbol, e);
                    }
                }
                Err(e) => {
                    if let Some(symbol) = crypto_symbols.get(i) {
                        error!("Crypto processing task failed for {}: {}", symbol, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a single crypto symbol (helper method for ultra-threading)
    async fn process_single_crypto_symbol(&self, symbol: &str) -> Result<()> {
        match self.get_market_data(&symbol.to_string()).await {
            Ok(data) => {
                // Update cache
                {
                    let mut market_data = self.market_data.write().await;
                    market_data.insert(symbol.to_string(), data.clone());
                }
                
                // Write to file
                if let Err(e) = self.write_market_data_file(symbol, &data).await {
                    error!("Failed to write crypto data for {}: {}", symbol, e);
                }
                Ok(())
            }
            Err(e) => {
                warn!("Failed to get crypto data for {}: {}", symbol, e);
                Err(e)
            }
        }
    }

    /// Stream news data with ULTRA-THREADING
    async fn stream_news_data(&self) -> Result<()> {
        info!("Streaming news data with ULTRA-THREADING...");
        
        // ULTRA-THREADING: Process all symbols concurrently for news
        let news_symbols: Vec<_> = self.config.symbols.iter().cloned().collect();
        
        if news_symbols.is_empty() {
            debug!("No symbols configured for news");
            return Ok(());
        }
        
        // Create concurrent tasks for each symbol
        let symbol_futures: Vec<_> = news_symbols
            .iter()
            .map(|symbol| {
                let symbol = symbol.clone();
                let streamer = self.clone();
                tokio::spawn(async move {
                    streamer.process_single_news_symbol(&symbol).await
                })
            })
            .collect();
        
        // Execute all symbol processing concurrently
        let results = futures::future::join_all(symbol_futures).await;
        
        // Process results
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok(_)) => {
                    if let Some(symbol) = news_symbols.get(i) {
                        debug!("Successfully processed news data for {}", symbol);
                    }
                }
                Ok(Err(e)) => {
                    if let Some(symbol) = news_symbols.get(i) {
                        warn!("Failed to process news data for {}: {}", symbol, e);
                    }
                }
                Err(e) => {
                    if let Some(symbol) = news_symbols.get(i) {
                        error!("News processing task failed for {}: {}", symbol, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a single news symbol (helper method for ultra-threading)
    async fn process_single_news_symbol(&self, symbol: &str) -> Result<()> {
        match self.get_news_data(symbol).await {
            Ok(data) => {
                // Update cache
                {
                    let mut market_data = self.market_data.write().await;
                    market_data.insert(symbol.to_string(), data.clone());
                }
                
                // Write to file
                if let Err(e) = self.write_market_data_file(symbol, &data).await {
                    error!("Failed to write news data for {}: {}", symbol, e);
                }
                Ok(())
            }
            Err(e) => {
                warn!("Failed to get news data for {}: {}", symbol, e);
                Err(e)
            }
        }
    }

    /// Stream stocks data with ULTRA-THREADING
    async fn stream_stocks_data(&self) -> Result<()> {
        info!("Streaming stocks data with ULTRA-THREADING...");
        
        // ULTRA-THREADING: Process all stock symbols concurrently
        let stock_symbols: Vec<_> = self.config.symbols
            .iter()
            .filter(|symbol| self.is_stock_symbol(symbol))
            .cloned()
            .collect();
        
        if stock_symbols.is_empty() {
            debug!("No stock symbols configured");
            return Ok(());
        }
        
        // Create concurrent tasks for each symbol
        let symbol_futures: Vec<_> = stock_symbols
            .iter()
            .map(|symbol| {
                let symbol = symbol.clone();
                let streamer = self.clone();
                tokio::spawn(async move {
                    streamer.process_single_stock_symbol(&symbol).await
                })
            })
            .collect();
        
        // Execute all symbol processing concurrently
        let results = futures::future::join_all(symbol_futures).await;
        
        // Process results
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok(_)) => {
                    if let Some(symbol) = stock_symbols.get(i) {
                        debug!("Successfully processed stock data for {}", symbol);
                    }
                }
                Ok(Err(e)) => {
                    if let Some(symbol) = stock_symbols.get(i) {
                        warn!("Failed to process stock data for {}: {}", symbol, e);
                    }
                }
                Err(e) => {
                    if let Some(symbol) = stock_symbols.get(i) {
                        error!("Stock processing task failed for {}: {}", symbol, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a single stock symbol (helper method for ultra-threading)
    async fn process_single_stock_symbol(&self, symbol: &str) -> Result<()> {
        match self.get_market_data(&symbol.to_string()).await {
            Ok(data) => {
                // Update cache
                {
                    let mut market_data = self.market_data.write().await;
                    market_data.insert(symbol.to_string(), data.clone());
                }
                
                // Write to file
                if let Err(e) = self.write_market_data_file(symbol, &data).await {
                    error!("Failed to write stock data for {}: {}", symbol, e);
                }
                Ok(())
            }
            Err(e) => {
                warn!("Failed to get stock data for {}: {}", symbol, e);
                Err(e)
            }
        }
    }

    /// Check if symbol is options
    fn is_options_symbol(&self, symbol: &str) -> bool {
        // Options symbols typically contain expiration and strike info
        symbol.contains('C') || symbol.contains('P') || symbol.contains('2')
    }

    /// Check if symbol is crypto
    fn is_crypto_symbol(&self, symbol: &str) -> bool {
        symbol.contains('/') && (symbol.contains("BTC") || symbol.contains("ETH") || symbol.contains("USD"))
    }

    /// Check if symbol is stock
    fn is_stock_symbol(&self, symbol: &str) -> bool {
        !self.is_options_symbol(symbol) && !self.is_crypto_symbol(symbol) && symbol.len() <= 5
    }

    /// Write market data to JSON file
    async fn write_market_data_file(&self, symbol: &str, data: &MarketData) -> Result<()> {
        let filename = if symbol.contains("BTC") {
            "market_data_btc.json"
        } else if symbol.contains("ETH") {
            "market_data_eth.json"
        } else {
            &format!("market_data_{}.json", symbol.to_lowercase().replace('/', "_"))
        };

        let file_path = self.data_dir.join(filename);
        let json_content = serde_json::to_string_pretty(data)?;
        
        fs::write(file_path, json_content).await?;
        debug!("Updated market data file: {}", filename);
        
        Ok(())
    }

    /// Get current market data for a symbol
    pub async fn get_current_data(&self, symbol: &Symbol) -> Option<MarketData> {
        let market_data = self.market_data.read().await;
        market_data.get(symbol).cloned()
    }

    /// Get options data for a symbol
    async fn get_options_data(&self, symbol: &str) -> Result<MarketData> {
        let endpoint = format!("{}/v2/options/quotes/latest", self.config.base_url);
        
        let response = self
            .client
            .get(&endpoint)
            .header("APCA-API-KEY-ID", &self.config.api_key)
            .header("APCA-API-SECRET-KEY", &self.config.secret_key)
            .query(&[("symbols", symbol)])
            .send()
            .await?;

        if response.status().is_success() {
            let options_data: AlpacaOptionsQuote = response.json().await?;
            self.parse_options_data(symbol, options_data).await
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Options API request failed: {}", error_text))
        }
    }

    /// Get news data for a symbol
    async fn get_news_data(&self, symbol: &str) -> Result<MarketData> {
        let endpoint = format!("{}/v2/news", self.config.base_url);
        
        let response = self
            .client
            .get(&endpoint)
            .header("APCA-API-KEY-ID", &self.config.api_key)
            .header("APCA-API-SECRET-KEY", &self.config.secret_key)
            .query(&[("symbols", symbol), ("limit", "1")])
            .send()
            .await?;

        if response.status().is_success() {
            let news_data: Vec<AlpacaNewsArticle> = response.json().await?;
            if let Some(article) = news_data.first() {
                self.parse_news_data(symbol, article).await
            } else {
                Err(anyhow::anyhow!("No news data found for {}", symbol))
            }
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("News API request failed: {}", error_text))
        }
    }

    /// Get all current market data
    pub async fn get_all_market_data(&self) -> HashMap<Symbol, MarketData> {
        let market_data = self.market_data.read().await;
        market_data.clone()
    }

    /// Check if streaming is active
    pub async fn is_running(&self) -> bool {
        let running = self.running.read().await;
        *running
    }
}

/// Load Alpaca configuration from environment
pub fn load_alpaca_config() -> Result<AlpacaConfig> {
    let api_key = std::env::var("ALPACA_API_KEY")
        .or_else(|_| std::env::var("ALPACA_API_KEY"))
        .unwrap_or_else(|_| "your_alpaca_api_key_here".to_string());

    let secret_key = std::env::var("ALPACA_SECRET_KEY")
        .or_else(|_| std::env::var("ALPACA_SECRET_KEY"))
        .unwrap_or_else(|_| "your_alpaca_secret_key_here".to_string());

    let base_url = std::env::var("ALPACA_BASE_URL")
        .unwrap_or_else(|_| "https://paper-api.alpaca.markets".to_string());

    let websocket_url = std::env::var("ALPACA_WEBSOCKET_URL")
        .unwrap_or_else(|_| "wss://stream.data.alpaca.markets/v2/iex".to_string());

    let symbols = std::env::var("TRADING_SYMBOLS")
        .unwrap_or_else(|_| "BTC/USD,ETH/USD,AAPL,SPY240920C00500000".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let update_interval_ms = std::env::var("UPDATE_INTERVAL_MS")
        .unwrap_or_else(|_| "1000".to_string())
        .parse()
        .unwrap_or(1000);

    let data_retention_hours = std::env::var("DATA_RETENTION_HOURS")
        .unwrap_or_else(|_| "24".to_string())
        .parse()
        .unwrap_or(24);

    let stream_types = std::env::var("STREAM_TYPES")
        .unwrap_or_else(|_| "Options,Crypto,News,Stocks".to_string())
        .split(',')
        .map(|s| match s.trim() {
            "Options" => StreamType::Options,
            "Crypto" => StreamType::Crypto,
            "News" => StreamType::News,
            "Stocks" => StreamType::Stocks,
            _ => StreamType::Stocks,
        })
        .collect();

    Ok(AlpacaConfig {
        api_key,
        secret_key,
        base_url,
        websocket_url,
        symbols,
        update_interval_ms,
        data_retention_hours,
        stream_types,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_alpaca_streamer_creation() {
        let config = AlpacaConfig {
            api_key: "test_key".to_string(),
            secret_key: "test_secret".to_string(),
            base_url: "https://test.api".to_string(),
            websocket_url: "wss://test.ws".to_string(),
            symbols: vec!["BTC/USD".to_string()],
            update_interval_ms: 1000,
            data_retention_hours: 24,
            stream_types: vec![StreamType::Crypto],
        };

        let temp_dir = tempdir().unwrap();
        let streamer = AlpacaStreamer::new(config, temp_dir.path().to_path_buf());
        assert!(streamer.is_ok());
    }

    #[test]
    fn test_load_alpaca_config() {
        std::env::set_var("ALPACA_API_KEY", "test_key");
        std::env::set_var("ALPACA_SECRET_KEY", "test_secret");
        
        let config = load_alpaca_config().unwrap();
        assert_eq!(config.api_key, "test_key");
        assert_eq!(config.secret_key, "test_secret");
        
        // Clean up
        std::env::remove_var("ALPACA_API_KEY");
        std::env::remove_var("ALPACA_SECRET_KEY");
    }
}
