use crate::ollama::{ai_model_manager::*, ollama_client::OllamaClient, conversation_manager::ConversationManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

/// Consensus Engine for Multi-Model AI Decision Making
pub struct ConsensusEngine {
    model_manager: AIModelManager,
    ollama_client: OllamaClient,
    conversation_manager: ConversationManager,
    consensus_history: Vec<ConsensusResult>,
    max_history: usize,
}

/// Consensus Analysis Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRequest {
    pub market_data: String,
    pub portfolio_data: String,
    pub trading_context: String,
    pub analysis_type: AnalysisType,
    pub symbols: Vec<String>,
    pub urgency: UrgencyLevel,
}

/// Type of analysis requested
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisType {
    BuySignal,      // Should we buy?
    SellSignal,     // Should we sell?
    HoldSignal,     // Should we hold?
    RiskAssessment, // Risk evaluation
    MarketRegime,   // Market condition analysis
    PositionSizing, // How much to buy/sell
    PortfolioReview, // Overall portfolio analysis
}

/// Urgency level for consensus
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UrgencyLevel {
    Low,     // Can wait for full consensus
    Medium,  // Standard consensus process
    High,    // Fast consensus with fewer models
    Critical, // Immediate decision with best available model
}

impl ConsensusEngine {
    /// Create new consensus engine
    pub fn new(ollama_client: OllamaClient) -> Self {
        Self {
            model_manager: AIModelManager::new(),
            ollama_client,
            conversation_manager: ConversationManager::new(20),
            consensus_history: Vec::new(),
            max_history: 100,
        }
    }

    /// Initialize with available models
    pub async fn initialize(&mut self, available_models: Vec<String>) -> Result<()> {
        // Auto-assign roles to available models
        self.model_manager.auto_assign_roles(available_models.clone())?;
        
        // Initialize conversations for each model
        for model_name in &available_models {
            if let Some(model_config) = self.model_manager.get_model(model_name) {
                self.conversation_manager.initialize_conversation(model_name, &model_config.role);
            }
        }
        
        println!("🤖 AI Model Manager Initialized:");
        let role_summary = self.model_manager.get_role_summary();
        for (role, count) in role_summary {
            println!("   {:?}: {} models", role, count);
        }
        
        println!("💬 Conversation contexts initialized for all models");
        Ok(())
    }

    /// Get consensus from multiple models
    pub async fn get_consensus(&mut self, request: ConsensusRequest) -> Result<ConsensusResult> {
        println!("🔄 Getting consensus for {:?} analysis...", request.analysis_type);
        
        // Determine which models to use based on urgency
        let models_to_use = self.select_models_for_request(&request);
        
        if models_to_use.is_empty() {
            return Err(anyhow::anyhow!("No models available for consensus"));
        }
        
        // Get individual model responses
        let mut individual_responses = HashMap::new();
        let mut responses = Vec::new();
        
        // Clone model configs to avoid borrow issues
        let model_configs: Vec<ModelConfig> = models_to_use.iter().map(|c| (*c).clone()).collect();
        
        for model_config in &model_configs {
            match self.get_model_response(model_config, &request).await {
                Ok(response) => {
                    individual_responses.insert(model_config.name.clone(), response.clone());
                    responses.push(response);
                }
                Err(e) => {
                    println!("⚠️  Model {} failed: {}", model_config.name, e);
                }
            }
        }
        
        if responses.is_empty() {
            return Err(anyhow::anyhow!("All models failed to respond"));
        }
        
        // Calculate consensus
        let consensus = self.calculate_consensus(&responses, &request);
        
        // Store in history
        self.consensus_history.push(consensus.clone());
        if self.consensus_history.len() > self.max_history {
            self.consensus_history.remove(0);
        }
        
        println!("✅ Consensus reached with {:.1}% confidence", consensus.confidence * 100.0);
        
        Ok(consensus)
    }

    /// Select models based on request urgency and type
    fn select_models_for_request(&self, request: &ConsensusRequest) -> Vec<&ModelConfig> {
        let mut selected_models = Vec::new();
        
        match request.urgency {
            UrgencyLevel::Critical => {
                // Use only the highest priority model for each critical role
                let critical_roles = [ModelRole::RiskManagement, ModelRole::TechnicalAnalysis];
                for role in critical_roles {
                    if let Some(model) = self.model_manager.get_models_for_role(&role).first() {
                        selected_models.push(*model);
                    }
                }
            }
            UrgencyLevel::High => {
                // Use top 2 models for each role
                for role in self.get_relevant_roles(&request.analysis_type) {
                    let models = self.model_manager.get_models_for_role(&role);
                    selected_models.extend(models.iter().take(2));
                }
            }
            UrgencyLevel::Medium => {
                // Use top 3 models for each role
                for role in self.get_relevant_roles(&request.analysis_type) {
                    let models = self.model_manager.get_models_for_role(&role);
                    selected_models.extend(models.iter().take(3));
                }
            }
            UrgencyLevel::Low => {
                // Use all available models
                selected_models.extend(self.model_manager.get_enabled_models());
            }
        }
        
        selected_models
    }

    /// Get relevant roles for analysis type
    fn get_relevant_roles(&self, analysis_type: &AnalysisType) -> Vec<ModelRole> {
        match analysis_type {
            AnalysisType::BuySignal | AnalysisType::SellSignal => {
                vec![
                    ModelRole::TechnicalAnalysis,
                    ModelRole::SentimentAnalysis,
                    ModelRole::RiskManagement,
                    ModelRole::MomentumAnalysis,
                ]
            }
            AnalysisType::HoldSignal => {
                vec![
                    ModelRole::TechnicalAnalysis,
                    ModelRole::MarketRegime,
                    ModelRole::RiskManagement,
                ]
            }
            AnalysisType::RiskAssessment => {
                vec![
                    ModelRole::RiskManagement,
                    ModelRole::MarketRegime,
                    ModelRole::TechnicalAnalysis,
                ]
            }
            AnalysisType::MarketRegime => {
                vec![
                    ModelRole::MarketRegime,
                    ModelRole::TechnicalAnalysis,
                    ModelRole::SentimentAnalysis,
                ]
            }
            AnalysisType::PositionSizing => {
                vec![
                    ModelRole::RiskManagement,
                    ModelRole::TechnicalAnalysis,
                    ModelRole::MomentumAnalysis,
                ]
            }
            AnalysisType::PortfolioReview => {
                vec![
                    ModelRole::RiskManagement,
                    ModelRole::MarketRegime,
                    ModelRole::GeneralPurpose,
                ]
            }
        }
    }

    /// Get individual model response using conversation context
    async fn get_model_response(
        &mut self,
        model_config: &ModelConfig,
        request: &ConsensusRequest,
    ) -> Result<ModelResponse> {
        let user_message = self.build_prompt_for_role(&model_config.role, request);
        
        // Add user message to conversation
        self.conversation_manager.add_user_message(&model_config.name, user_message);
        
        // Get conversation for this model
        let conversation = self.conversation_manager
            .get_conversation(&model_config.name)
            .ok_or_else(|| anyhow::anyhow!("No conversation found for model: {}", model_config.name))?
            .clone();
        
        // Generate response using conversation context
        let response = self.ollama_client
            .chat_with_model(
                &model_config.name,
                conversation.clone(),
                model_config.temperature as f32,
                model_config.max_tokens as i32,
            )
            .await?;
        
        // Add assistant response to conversation
        self.conversation_manager.add_assistant_message(&model_config.name, response.clone());
        
        // Parse response and extract decision components
        let (decision, confidence, reasoning) = self.parse_model_response(&response, &model_config.role);
        
        Ok(ModelResponse {
            model_name: model_config.name.clone(),
            role: model_config.role.clone(),
            decision,
            confidence,
            reasoning,
            weight: model_config.weight,
        })
    }

    /// Build role-specific prompt
    fn build_prompt_for_role(&self, role: &ModelRole, request: &ConsensusRequest) -> String {
        let base_context = format!(
            "Market Data: {}\nPortfolio: {}\nContext: {}\nSymbols: {:?}\nAnalysis Type: {:?}",
            request.market_data,
            request.portfolio_data,
            request.trading_context,
            request.symbols,
            request.analysis_type
        );
        
        match role {
            ModelRole::TechnicalAnalysis => {
                format!(
                    "You are a technical analysis expert. Analyze the following data and provide a trading decision based on technical indicators, patterns, and market structure.\n\n{}\n\nProvide: 1) Decision (BUY/SELL/HOLD), 2) Confidence (0.0-1.0), 3) Technical reasoning",
                    base_context
                )
            }
            ModelRole::SentimentAnalysis => {
                format!(
                    "You are a market sentiment analyst. Analyze news, market mood, and sentiment indicators to provide a trading decision.\n\n{}\n\nProvide: 1) Decision (BUY/SELL/HOLD), 2) Confidence (0.0-1.0), 3) Sentiment reasoning",
                    base_context
                )
            }
            ModelRole::RiskManagement => {
                format!(
                    "You are a risk management expert. Focus on position sizing, risk assessment, and portfolio protection.\n\n{}\n\nProvide: 1) Decision (BUY/SELL/HOLD), 2) Confidence (0.0-1.0), 3) Risk reasoning",
                    base_context
                )
            }
            ModelRole::MarketRegime => {
                format!(
                    "You are a market regime analyst. Determine if we're in a bull, bear, or sideways market and adjust strategy accordingly.\n\n{}\n\nProvide: 1) Decision (BUY/SELL/HOLD), 2) Confidence (0.0-1.0), 3) Regime reasoning",
                    base_context
                )
            }
            ModelRole::MomentumAnalysis => {
                format!(
                    "You are a momentum analyst. Focus on price momentum, volume analysis, and trend strength.\n\n{}\n\nProvide: 1) Decision (BUY/SELL/HOLD), 2) Confidence (0.0-1.0), 3) Momentum reasoning",
                    base_context
                )
            }
            ModelRole::GeneralPurpose => {
                format!(
                    "You are a general trading AI. Provide a balanced trading decision considering all factors.\n\n{}\n\nProvide: 1) Decision (BUY/SELL/HOLD), 2) Confidence (0.0-1.0), 3) General reasoning",
                    base_context
                )
            }
        }
    }

    /// Parse model response to extract decision components
    fn parse_model_response(&self, response: &str, role: &ModelRole) -> (String, f64, String) {
        // Simple parsing - in production, use more sophisticated NLP
        let lines: Vec<&str> = response.lines().collect();
        
        let mut decision = "HOLD".to_string();
        let mut confidence = 0.5;
        let reasoning = response.to_string();
        
        for line in lines {
            let line_lower = line.to_lowercase();
            if line_lower.contains("decision:") || line_lower.contains("decision ") {
                if line_lower.contains("buy") {
                    decision = "BUY".to_string();
                } else if line_lower.contains("sell") {
                    decision = "SELL".to_string();
                } else if line_lower.contains("hold") {
                    decision = "HOLD".to_string();
                }
            } else if line_lower.contains("confidence:") || line_lower.contains("confidence ") {
                if let Some(conf_str) = line.split(':').nth(1) {
                    if let Ok(conf) = conf_str.trim().parse::<f64>() {
                        confidence = conf.clamp(0.0, 1.0);
                    }
                }
            }
        }
        
        // Adjust confidence based on role expertise
        let role_multiplier = match role {
            ModelRole::RiskManagement => 1.1,      // Risk models are more confident
            ModelRole::TechnicalAnalysis => 1.0,   // Standard confidence
            ModelRole::MarketRegime => 0.9,        // Regime detection is uncertain
            ModelRole::SentimentAnalysis => 0.8,   // Sentiment is subjective
            ModelRole::MomentumAnalysis => 0.9,    // Momentum can be volatile
            ModelRole::GeneralPurpose => 0.7,      // General purpose is less confident
        };
        
        confidence = (confidence * role_multiplier).min(1.0);
        
        (decision, confidence, reasoning)
    }

    /// Calculate consensus from individual responses
    fn calculate_consensus(&self, responses: &[ModelResponse], request: &ConsensusRequest) -> ConsensusResult {
        if responses.is_empty() {
            return ConsensusResult {
                final_decision: "HOLD".to_string(),
                confidence: 0.0,
                individual_responses: HashMap::new(),
                consensus_breakdown: ConsensusBreakdown {
                    technical_consensus: 0.0,
                    sentiment_consensus: 0.0,
                    risk_consensus: 0.0,
                    regime_consensus: 0.0,
                    momentum_consensus: 0.0,
                    general_consensus: 0.0,
                },
                reasoning: "No models responded".to_string(),
            };
        }
        
        // Group responses by role
        let mut role_responses: HashMap<ModelRole, Vec<&ModelResponse>> = HashMap::new();
        for response in responses {
            role_responses
                .entry(response.role.clone())
                .or_insert_with(Vec::new)
                .push(response);
        }
        
        // Calculate consensus by role
        let mut consensus_breakdown = ConsensusBreakdown {
            technical_consensus: 0.0,
            sentiment_consensus: 0.0,
            risk_consensus: 0.0,
            regime_consensus: 0.0,
            momentum_consensus: 0.0,
            general_consensus: 0.0,
        };
        
        for (role, role_responses) in &role_responses {
            let consensus = self.calculate_role_consensus(role_responses);
            match role {
                ModelRole::TechnicalAnalysis => consensus_breakdown.technical_consensus = consensus,
                ModelRole::SentimentAnalysis => consensus_breakdown.sentiment_consensus = consensus,
                ModelRole::RiskManagement => consensus_breakdown.risk_consensus = consensus,
                ModelRole::MarketRegime => consensus_breakdown.regime_consensus = consensus,
                ModelRole::MomentumAnalysis => consensus_breakdown.momentum_consensus = consensus,
                ModelRole::GeneralPurpose => consensus_breakdown.general_consensus = consensus,
            }
        }
        
        // Calculate weighted final decision
        let (final_decision, overall_confidence) = self.calculate_weighted_decision(responses);
        
        // Build reasoning
        let reasoning = self.build_consensus_reasoning(&consensus_breakdown, responses);
        
        // Store individual responses
        let individual_responses: HashMap<String, ModelResponse> = responses
            .iter()
            .map(|r| (r.model_name.clone(), r.clone()))
            .collect();
        
        ConsensusResult {
            final_decision,
            confidence: overall_confidence,
            individual_responses,
            consensus_breakdown,
            reasoning,
        }
    }

    /// Calculate consensus for a specific role
    fn calculate_role_consensus(&self, responses: &[&ModelResponse]) -> f64 {
        if responses.is_empty() {
            return 0.0;
        }
        
        // Count decisions
        let mut buy_count = 0;
        let mut sell_count = 0;
        let mut hold_count = 0;
        let mut total_weight = 0.0;
        
        for response in responses {
            total_weight += response.weight;
            match response.decision.as_str() {
                "BUY" => buy_count += 1,
                "SELL" => sell_count += 1,
                "HOLD" => hold_count += 1,
                _ => {}
            }
        }
        
        if total_weight == 0.0 {
            return 0.0;
        }
        
        // Calculate consensus as agreement percentage
        let total_responses = responses.len() as f64;
        let max_agreement = buy_count.max(sell_count).max(hold_count) as f64;
        max_agreement / total_responses
    }

    /// Calculate weighted final decision
    fn calculate_weighted_decision(&self, responses: &[ModelResponse]) -> (String, f64) {
        let mut buy_weight = 0.0;
        let mut sell_weight = 0.0;
        let mut hold_weight = 0.0;
        let mut total_weight = 0.0;
        let mut weighted_confidence = 0.0;
        
        for response in responses {
            let weight = response.weight * response.confidence;
            total_weight += weight;
            weighted_confidence += response.confidence * response.weight;
            
            match response.decision.as_str() {
                "BUY" => buy_weight += weight,
                "SELL" => sell_weight += weight,
                "HOLD" => hold_weight += weight,
                _ => {}
            }
        }
        
        if total_weight == 0.0 {
            return ("HOLD".to_string(), 0.0);
        }
        
        // Determine final decision
        let final_decision = if buy_weight > sell_weight && buy_weight > hold_weight {
            "BUY"
        } else if sell_weight > buy_weight && sell_weight > hold_weight {
            "SELL"
        } else {
            "HOLD"
        };
        
        let overall_confidence = weighted_confidence / total_weight;
        
        (final_decision.to_string(), overall_confidence)
    }

    /// Build consensus reasoning
    fn build_consensus_reasoning(&self, breakdown: &ConsensusBreakdown, responses: &[ModelResponse]) -> String {
        let mut reasoning = String::new();
        
        reasoning.push_str("Consensus Analysis:\n");
        reasoning.push_str(&format!("  Technical: {:.1}%\n", breakdown.technical_consensus * 100.0));
        reasoning.push_str(&format!("  Sentiment: {:.1}%\n", breakdown.sentiment_consensus * 100.0));
        reasoning.push_str(&format!("  Risk: {:.1}%\n", breakdown.risk_consensus * 100.0));
        reasoning.push_str(&format!("  Regime: {:.1}%\n", breakdown.regime_consensus * 100.0));
        reasoning.push_str(&format!("  Momentum: {:.1}%\n", breakdown.momentum_consensus * 100.0));
        reasoning.push_str(&format!("  General: {:.1}%\n", breakdown.general_consensus * 100.0));
        
        reasoning.push_str("\nModel Contributions:\n");
        for response in responses {
            reasoning.push_str(&format!("  {} ({:?}): {} (conf: {:.1}%)\n", 
                response.model_name, 
                response.role, 
                response.decision, 
                response.confidence * 100.0
            ));
        }
        
        reasoning
    }

    /// Get consensus history
    pub fn get_consensus_history(&self) -> &[ConsensusResult] {
        &self.consensus_history
    }

    /// Get model manager reference
    pub fn get_model_manager(&self) -> &AIModelManager {
        &self.model_manager
    }

    /// Get model manager mutable reference
    pub fn get_model_manager_mut(&mut self) -> &mut AIModelManager {
        &mut self.model_manager
    }

    /// Clear conversation history for all models
    pub fn clear_all_conversations(&mut self) {
        for model_name in self.model_manager.get_all_model_names() {
            self.conversation_manager.clear_conversation(&model_name);
        }
        println!("🧹 Cleared conversation history for all models");
    }

    /// Clear conversation history for a specific model
    pub fn clear_model_conversation(&mut self, model_name: &str) {
        self.conversation_manager.clear_conversation(model_name);
        println!("🧹 Cleared conversation history for model: {}", model_name);
    }

    /// Get conversation manager reference
    pub fn get_conversation_manager(&self) -> &ConversationManager {
        &self.conversation_manager
    }
}
