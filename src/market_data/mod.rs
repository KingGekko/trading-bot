//! Market data module for handling real-time financial data streams
//! 
//! This module provides:
//! - Real-time market data streaming from Alpaca API
//! - Support for multiple data types: Options, Crypto, News, Stocks
//! - Data parsing and transformation
//! - File-based data persistence
//! - WebSocket streaming capabilities

pub mod types;
pub mod streamer;

// Re-export main types for easy access
pub use types::{
    MarketData, Symbol, DataSource, MarketDataUpdate,
    UpdateType, MarketDataStats, MarketDataFilter, MarketDataAggregate, AggregationPeriod,
};
pub use streamer::{AlpacaStreamer, AlpacaConfig, load_alpaca_config};
