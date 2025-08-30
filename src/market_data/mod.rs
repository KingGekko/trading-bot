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

// Re-export main types for easy access
pub use unified_websocket::{UnifiedAlpacaWebSocket, StreamType, load_unified_websocket_config};
pub use account_verifier::{AccountVerifier, AccountVerification, AccountType};
