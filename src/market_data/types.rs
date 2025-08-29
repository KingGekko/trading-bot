use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Type alias for trading symbols
pub type Symbol = String;

/// Market data structure for all stream types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    /// Timestamp of the data
    pub timestamp: DateTime<Utc>,
    /// Trading symbol (e.g., "BTC/USD", "ETH/USD", "AAPL", "SPY240920C00500000")
    pub symbol: String,
    /// Current price
    pub price: f64,
    /// Trading volume
    pub volume: f64,
    /// High price (optional, mainly for crypto/stocks)
    pub high: Option<f64>,
    /// Low price (optional, mainly for crypto/stocks)
    pub low: Option<f64>,
    /// Open price (optional, mainly for crypto/stocks)
    pub open: Option<f64>,
    /// Data source identifier
    pub source: String,
    /// Exchange name
    pub exchange: String,
    /// 24-hour price change (optional)
    pub change_24h: Option<f64>,
    /// 24-hour percentage change (optional)
    pub change_percent: Option<f64>,
    /// Market capitalization (optional)
    pub market_cap: Option<f64>,
    /// Circulating supply (optional)
    pub circulating_supply: Option<f64>,
    /// Additional data for options
    pub options_data: Option<OptionsData>,
    /// Additional data for news
    pub news_data: Option<NewsData>,
}

/// Options-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsData {
    /// Strike price
    pub strike: f64,
    /// Expiration date
    pub expiration: DateTime<Utc>,
    /// Option type (call/put)
    pub option_type: String,
    /// Underlying symbol
    pub underlying: String,
    /// Implied volatility
    pub implied_volatility: Option<f64>,
    /// Delta
    pub delta: Option<f64>,
    /// Gamma
    pub gamma: Option<f64>,
    /// Theta
    pub theta: Option<f64>,
    /// Vega
    pub vega: Option<f64>,
    /// Open interest
    pub open_interest: Option<i64>,
    /// Bid price
    pub bid: Option<f64>,
    /// Ask price
    pub ask: Option<f64>,
}

/// News-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsData {
    /// News headline
    pub headline: String,
    /// News summary
    pub summary: Option<String>,
    /// News URL
    pub url: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Source
    pub source: String,
    /// Sentiment score
    pub sentiment: Option<f64>,
    /// Related symbols
    pub symbols: Vec<String>,
    /// News category
    pub category: Option<String>,
}

/// Market data error types
#[derive(Debug, Clone, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum MarketDataError {
    #[error("API request failed: {0}")]
    ApiRequestFailed(String),
    
    #[error("Data parsing failed: {0}")]
    DataParsingFailed(String),
    
    #[error("File I/O error: {0}")]
    FileIoError(String),
    
    #[error("JSON serialization error: {0}")]
    JsonError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),
    
    #[error("Data not available for symbol: {0}")]
    DataNotAvailable(String),
}

/// Market data source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    AlpacaOptions,
    AlpacaCrypto,
    AlpacaStocks,
    AlpacaNews,
    Custom(String),
}

impl fmt::Display for DataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataSource::AlpacaOptions => write!(f, "alpaca_options"),
            DataSource::AlpacaCrypto => write!(f, "alpaca_crypto"),
            DataSource::AlpacaStocks => write!(f, "alpaca_stocks"),
            DataSource::AlpacaNews => write!(f, "alpaca_news"),
            DataSource::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Market data update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataUpdate {
    /// Symbol that was updated
    pub symbol: Symbol,
    /// Previous market data (if available)
    pub previous: Option<MarketData>,
    /// Current market data
    pub current: MarketData,
    /// Update timestamp
    pub update_time: DateTime<Utc>,
    /// Update type
    pub update_type: UpdateType,
}

/// Types of market data updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateType {
    /// New data received
    New,
    /// Data updated
    Updated,
    /// Data removed/expired
    Removed,
    /// Error occurred
    Error(MarketDataError),
}

/// Market data statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataStats {
    /// Total symbols being tracked
    pub total_symbols: usize,
    /// Symbols with recent updates
    pub active_symbols: usize,
    /// Last update time
    pub last_update: Option<DateTime<Utc>>,
    /// Total data points received
    pub total_updates: u64,
    /// Errors encountered
    pub error_count: u64,
    /// Average update frequency (updates per minute)
    pub avg_update_frequency: f64,
}

/// Market data filter criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataFilter {
    /// Minimum price threshold
    pub min_price: Option<f64>,
    /// Maximum price threshold
    pub max_price: Option<f64>,
    /// Minimum volume threshold
    pub min_volume: Option<f64>,
    /// Maximum volume threshold
    pub max_volume: Option<f64>,
    /// Data sources to include
    pub sources: Option<Vec<DataSource>>,
    /// Exchanges to include
    pub exchanges: Option<Vec<String>>,
    /// Symbols to include
    pub symbols: Option<Vec<Symbol>>,
    /// Time range (from)
    pub from_time: Option<DateTime<Utc>>,
    /// Time range (to)
    pub to_time: Option<DateTime<Utc>>,
}

impl Default for MarketDataFilter {
    fn default() -> Self {
        Self {
            min_price: None,
            max_price: None,
            min_volume: None,
            max_volume: None,
            sources: None,
            exchanges: None,
            symbols: None,
            from_time: None,
            to_time: None,
        }
    }
}

/// Market data aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataAggregate {
    /// Aggregation period
    pub period: AggregationPeriod,
    /// Start time of aggregation
    pub start_time: DateTime<Utc>,
    /// End time of aggregation
    pub end_time: DateTime<Utc>,
    /// Symbol being aggregated
    pub symbol: Symbol,
    /// Open price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Close price
    pub close: f64,
    /// Total volume
    pub volume: f64,
    /// Number of data points
    pub count: u64,
}

/// Aggregation periods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationPeriod {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Custom(u64), // Custom period in seconds
}

impl fmt::Display for AggregationPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregationPeriod::Second => write!(f, "1s"),
            AggregationPeriod::Minute => write!(f, "1m"),
            AggregationPeriod::Hour => write!(f, "1h"),
            AggregationPeriod::Day => write!(f, "1d"),
            AggregationPeriod::Week => write!(f, "1w"),
            AggregationPeriod::Month => write!(f, "1M"),
            AggregationPeriod::Custom(seconds) => write!(f, "{}s", seconds),
        }
    }
}

impl MarketData {
    /// Create a new market data instance
    pub fn new(
        symbol: String,
        price: f64,
        volume: f64,
        source: String,
        exchange: String,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            symbol,
            price,
            volume,
            high: None,
            low: None,
            open: None,
            source,
            exchange,
            change_24h: None,
            change_percent: None,
            market_cap: None,
            circulating_supply: None,
            options_data: None,
            news_data: None,
        }
    }

    /// Calculate price change from previous data
    pub fn calculate_change(&self, previous: &MarketData) -> (f64, f64) {
        let change = self.price - previous.price;
        let change_percent = if previous.price > 0.0 {
            (change / previous.price) * 100.0
        } else {
            0.0
        };
        (change, change_percent)
    }

    /// Check if price has changed significantly
    pub fn has_significant_change(&self, previous: &MarketData, threshold: f64) -> bool {
        let (_, change_percent) = self.calculate_change(previous);
        change_percent.abs() >= threshold
    }

    /// Get formatted price string
    pub fn formatted_price(&self) -> String {
        if self.price >= 1000.0 {
            format!("${:.2}", self.price)
        } else if self.price >= 1.0 {
            format!("${:.4}", self.price)
        } else {
            format!("${:.8}", self.price)
        }
    }

    /// Get formatted volume string
    pub fn formatted_volume(&self) -> String {
        if self.volume >= 1_000_000.0 {
            format!("{:.2}M", self.volume / 1_000_000.0)
        } else if self.volume >= 1_000.0 {
            format!("{:.2}K", self.volume / 1_000.0)
        } else {
            format!("{:.2}", self.volume)
        }
    }
}

impl fmt::Display for MarketData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} @ {} (Vol: {})",
            self.symbol,
            self.formatted_price(),
            self.timestamp.format("%H:%M:%S"),
            self.formatted_volume()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_data_creation() {
        let data = MarketData::new(
            "BTC/USD".to_string(),
            45000.0,
            1250.5,
            "alpaca_crypto".to_string(),
            "alpaca".to_string(),
        );

        assert_eq!(data.symbol, "BTC/USD");
        assert_eq!(data.price, 45000.0);
        assert_eq!(data.volume, 1250.5);
        assert_eq!(data.source, "alpaca_crypto");
        assert_eq!(data.exchange, "alpaca");
        assert!(data.options_data.is_none());
        assert!(data.news_data.is_none());
    }

    #[test]
    fn test_price_change_calculation() {
        let previous = MarketData::new(
            "BTC/USD".to_string(),
            44000.0,
            1200.0,
            "alpaca_crypto".to_string(),
            "alpaca".to_string(),
        );

        let current = MarketData::new(
            "BTC/USD".to_string(),
            45000.0,
            1250.5,
            "alpaca_crypto".to_string(),
            "alpaca".to_string(),
        );

        let (change, change_percent) = current.calculate_change(&previous);
        assert_eq!(change, 1000.0);
        assert!((change_percent - 2.2727).abs() < 0.01);
    }

    #[test]
    fn test_significant_change_detection() {
        let previous = MarketData::new(
            "BTC/USD".to_string(),
            44000.0,
            1200.0,
            "alpaca_crypto".to_string(),
            "alpaca".to_string(),
        );

        let current = MarketData::new(
            "BTC/USD".to_string(),
            45000.0,
            1250.5,
            "alpaca_crypto".to_string(),
            "alpaca".to_string(),
        );

        assert!(current.has_significant_change(&previous, 2.0));
        assert!(!current.has_significant_change(&previous, 5.0));
    }

    #[test]
    fn test_formatted_strings() {
        let data = MarketData::new(
            "BTC/USD".to_string(),
            45000.12345678,
            1250000.5,
            "alpaca_crypto".to_string(),
            "alpaca".to_string(),
        );

        assert_eq!(data.formatted_price(), "$45000.12");
        assert_eq!(data.formatted_volume(), "1.25M");
    }
}
