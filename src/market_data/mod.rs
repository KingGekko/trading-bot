//! Market data module for handling real-time financial data streams
//! 
//! This module provides:
//! - Real-time market data streaming from Alpaca API
//! - Support for multiple data types: Options, Crypto, News, Stocks
//! - Data parsing and transformation
//! - File-based data persistence
//! - WebSocket streaming capabilities

pub mod types;
pub mod unified_websocket;
pub mod account_verifier;
pub mod asset_universe;
pub mod market_regime;
pub mod simulated_stream;
pub mod trading_account;
pub mod technical_indicators;

// Re-export main types for easy access
pub use unified_websocket::{UnifiedAlpacaWebSocket, StreamType, load_unified_websocket_config};
// pub use account_verifier::AccountVerifier; // Unused
pub use asset_universe::{AssetUniverseManager, Asset, Position};
pub use market_regime::{MarketRegimeDetector, MarketRegimeAnalysis};
pub use simulated_stream::SimulatedMarketStream;
pub use trading_account::TradingAccountManager;
pub use technical_indicators::{TechnicalIndicators, MarketDataPoint};
