pub mod ollama_client;
pub mod ollama_config;
pub mod ollama_receipt;

// Re-export the main types for easier importing
pub use ollama_client::OllamaClient;
pub use ollama_config::Config;
pub use ollama_receipt::OllamaReceipt;