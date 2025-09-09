use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

/// Conversation message types for Ollama chat API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

/// Individual message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub content: String,
}

/// Conversation request for Ollama chat API
#[derive(Debug, Serialize)]
pub struct ConversationRequest {
    pub model: String,
    pub messages: Vec<ConversationMessage>,
    pub stream: bool,
    pub options: ConversationOptions,
}

/// Options for conversation requests
#[derive(Debug, Serialize)]
pub struct ConversationOptions {
    pub temperature: f32,
    pub max_tokens: i32,
    pub top_k: i32,
    pub top_p: f32,
    pub repeat_penalty: f32,
}

/// Conversation response from Ollama
#[derive(Debug, Deserialize)]
pub struct ConversationResponse {
    pub message: ConversationMessage,
    pub done: bool,
    pub total_duration: Option<u64>,
    pub load_duration: Option<u64>,
    pub prompt_eval_count: Option<u32>,
    pub prompt_eval_duration: Option<u64>,
    pub eval_count: Option<u32>,
    pub eval_duration: Option<u64>,
}

/// Conversation manager for maintaining model-specific conversations
pub struct ConversationManager {
    conversations: HashMap<String, Vec<ConversationMessage>>, // model_name -> conversation history
    max_history: usize,
}

impl ConversationManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            conversations: HashMap::new(),
            max_history,
        }
    }

    /// Initialize a conversation for a model with role-specific system prompt
    pub fn initialize_conversation(&mut self, model_name: &str, role: &crate::ollama::ai_model_manager::ModelRole) {
        let system_prompt = self.get_role_system_prompt(role);
        let messages = vec![
            ConversationMessage {
                role: MessageRole::System,
                content: system_prompt,
            }
        ];
        self.conversations.insert(model_name.to_string(), messages);
    }

    /// Add a user message to a model's conversation
    pub fn add_user_message(&mut self, model_name: &str, content: String) {
        if let Some(conversation) = self.conversations.get_mut(model_name) {
            conversation.push(ConversationMessage {
                role: MessageRole::User,
                content,
            });
            Self::trim_conversation_internal(conversation, self.max_history);
        }
    }

    /// Add an assistant response to a model's conversation
    pub fn add_assistant_message(&mut self, model_name: &str, content: String) {
        if let Some(conversation) = self.conversations.get_mut(model_name) {
            conversation.push(ConversationMessage {
                role: MessageRole::Assistant,
                content,
            });
            Self::trim_conversation_internal(conversation, self.max_history);
        }
    }

    /// Get the current conversation for a model
    pub fn get_conversation(&self, model_name: &str) -> Option<&Vec<ConversationMessage>> {
        self.conversations.get(model_name)
    }

    /// Clear conversation history for a model
    pub fn clear_conversation(&mut self, model_name: &str) {
        self.conversations.remove(model_name);
    }

    /// Get role-specific system prompt
    fn get_role_system_prompt(&self, role: &crate::ollama::ai_model_manager::ModelRole) -> String {
        match role {
            crate::ollama::ai_model_manager::ModelRole::TechnicalAnalysis => {
                "You are an expert technical analyst specializing in stock market analysis. \
                Your expertise includes RSI, MACD, Bollinger Bands, moving averages, support/resistance levels, \
                chart patterns, and trend analysis. Always provide data-driven technical insights with specific \
                indicators and confidence levels. Focus on mathematical precision and pattern recognition."
            }
            crate::ollama::ai_model_manager::ModelRole::SentimentAnalysis => {
                "You are a market sentiment analyst with expertise in news analysis, market psychology, \
                fear/greed indicators, and social sentiment. You excel at interpreting market mood, news impact, \
                and investor behavior patterns. Provide nuanced sentiment analysis with confidence levels and \
                reasoning based on qualitative factors."
            }
            crate::ollama::ai_model_manager::ModelRole::RiskManagement => {
                "You are a conservative risk management specialist focused on portfolio protection, \
                position sizing, stop losses, and risk assessment. Your primary concern is capital preservation \
                and risk-adjusted returns. Always prioritize safety and provide conservative recommendations \
                with clear risk explanations."
            }
            crate::ollama::ai_model_manager::ModelRole::MarketRegime => {
                "You are a market regime analyst specializing in identifying bull, bear, and sideways markets, \
                volatility analysis, and market cycle detection. You excel at determining overall market conditions \
                and adjusting strategies accordingly. Focus on macro trends and regime changes."
            }
            crate::ollama::ai_model_manager::ModelRole::MomentumAnalysis => {
                "You are a momentum analyst specializing in price momentum, volume analysis, and trend strength. \
                You excel at identifying momentum shifts, volume patterns, and short-term price movements. \
                Focus on velocity and acceleration of price changes with technical momentum indicators."
            }
            crate::ollama::ai_model_manager::ModelRole::GeneralPurpose => {
                "You are a general trading AI providing balanced analysis across all market factors. \
                You consider technical, fundamental, sentiment, and risk factors to provide comprehensive \
                trading recommendations. Maintain objectivity and consider multiple perspectives."
            }
        }.to_string()
    }

    /// Trim conversation to maintain max history
    fn trim_conversation_internal(conversation: &mut Vec<ConversationMessage>, max_history: usize) {
        if conversation.len() > max_history {
            // Keep system message and recent messages
            let system_msg = conversation.remove(0);
            conversation.truncate(max_history - 1);
            conversation.insert(0, system_msg);
        }
    }

    /// Create conversation request for a model
    pub fn create_conversation_request(
        &self,
        model_name: &str,
        user_message: String,
        temperature: f32,
        max_tokens: i32,
    ) -> Result<ConversationRequest> {
        let mut messages = self.conversations
            .get(model_name)
            .cloned()
            .unwrap_or_default();
        
        messages.push(ConversationMessage {
            role: MessageRole::User,
            content: user_message,
        });

        Ok(ConversationRequest {
            model: model_name.to_string(),
            messages,
            stream: false,
            options: ConversationOptions {
                temperature,
                max_tokens,
                top_k: 20,
                top_p: 0.9,
                repeat_penalty: 1.1,
            },
        })
    }
}

impl Default for ConversationManager {
    fn default() -> Self {
        Self::new(20) // Default 20 message history
    }
}
