//! API module for the trading bot
//! Provides REST endpoints and WebSocket support for streaming JSON data

pub mod json_stream;
pub mod server;
pub mod handlers;

pub use server::start_api_server; 