pub mod ollama_client;
pub mod ollama_config;
pub mod ollama_receipt;
pub mod ai_model_manager;
pub mod consensus_engine;
pub mod conversation_manager;


// Re-export the main types for easier importing
pub use ollama_client::OllamaClient;
pub use ollama_config::Config;
pub use ai_model_manager::{AIModelManager, ModelConfig, ModelRole, ConsensusResult};
pub use consensus_engine::{ConsensusEngine, ConsensusRequest, AnalysisType, UrgencyLevel};
pub use ollama_receipt::OllamaReceipt;