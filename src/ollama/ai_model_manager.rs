use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

/// AI Model Role for specialized analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelRole {
    TechnicalAnalysis,  // RSI, MACD, Bollinger Bands, trend analysis
    SentimentAnalysis,  // News sentiment, market mood, fear/greed
    RiskManagement,     // Position sizing, stop losses, portfolio risk
    MarketRegime,       // Bull/bear/sideways detection, volatility
    MomentumAnalysis,   // Price momentum, volume analysis
    GeneralPurpose,     // General trading decisions
}

/// AI Model Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub role: ModelRole,
    pub weight: f64,           // Weight in consensus (0.0-1.0)
    pub temperature: f64,      // Model temperature
    pub max_tokens: u32,       // Max response tokens
    pub enabled: bool,         // Whether model is active
    pub priority: u8,          // Priority for role assignment (1-10)
}

/// Multi-Model Consensus Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub final_decision: String,
    pub confidence: f64,       // Overall confidence (0.0-1.0)
    pub individual_responses: HashMap<String, ModelResponse>,
    pub consensus_breakdown: ConsensusBreakdown,
    pub reasoning: String,
}

/// Individual Model Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub model_name: String,
    pub role: ModelRole,
    pub decision: String,
    pub confidence: f64,
    pub reasoning: String,
    pub weight: f64,
}

/// Consensus Breakdown by Role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusBreakdown {
    pub technical_consensus: f64,
    pub sentiment_consensus: f64,
    pub risk_consensus: f64,
    pub regime_consensus: f64,
    pub momentum_consensus: f64,
    pub general_consensus: f64,
}

/// Enhanced AI Model Manager
pub struct AIModelManager {
    models: HashMap<String, ModelConfig>,
    role_assignments: HashMap<ModelRole, Vec<String>>, // Role -> Model names
    consensus_threshold: f64,
    max_models_per_role: usize,
}

impl AIModelManager {
    /// Create new AI Model Manager
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            role_assignments: HashMap::new(),
            consensus_threshold: 0.7, // 70% consensus required
            max_models_per_role: 3,
        }
    }

    /// Add a model to the manager
    pub fn add_model(&mut self, config: ModelConfig) -> Result<()> {
        let model_name = config.name.clone();
        let role = config.role.clone();
        
        // Add model to registry
        self.models.insert(model_name.clone(), config);
        
        // Assign to role
        self.role_assignments
            .entry(role)
            .or_insert_with(Vec::new)
            .push(model_name);
        
        // Sort by priority within role
        self.sort_models_by_priority();
        
        Ok(())
    }

    /// Auto-assign roles based on model capabilities
    pub fn auto_assign_roles(&mut self, available_models: Vec<String>) -> Result<()> {
        // Clear existing assignments
        self.role_assignments.clear();
        
        // Define role assignment rules based on model names
        for model_name in available_models {
            let role = self.detect_model_role(&model_name);
            let config = ModelConfig {
                name: model_name.clone(),
                role: role.clone(),
                weight: self.calculate_default_weight(&role),
                temperature: self.calculate_default_temperature(&role),
                max_tokens: self.calculate_default_tokens(&role),
                enabled: true,
                priority: self.calculate_default_priority(&role),
            };
            
            self.add_model(config)?;
        }
        
        Ok(())
    }

    /// Detect model role based on name
    fn detect_model_role(&self, model_name: &str) -> ModelRole {
        let name_lower = model_name.to_lowercase();
        
        // Technical analysis models (good at math, patterns)
        if name_lower.contains("qwen") || name_lower.contains("llama3") || name_lower.contains("gemma") {
            ModelRole::TechnicalAnalysis
        }
        // Sentiment analysis models (good at language understanding)
        else if name_lower.contains("phi") || name_lower.contains("mistral") {
            ModelRole::SentimentAnalysis
        }
        // Risk management models (conservative, mathematical)
        else if name_lower.contains("llama2") || name_lower.contains("codellama") {
            ModelRole::RiskManagement
        }
        // General purpose models
        else if name_lower.contains("tinyllama") || name_lower.contains("llama") {
            ModelRole::GeneralPurpose
        }
        // Default to general purpose
        else {
            ModelRole::GeneralPurpose
        }
    }

    /// Calculate default weight based on role
    fn calculate_default_weight(&self, role: &ModelRole) -> f64 {
        match role {
            ModelRole::TechnicalAnalysis => 0.25,
            ModelRole::SentimentAnalysis => 0.20,
            ModelRole::RiskManagement => 0.30,
            ModelRole::MarketRegime => 0.15,
            ModelRole::MomentumAnalysis => 0.10,
            ModelRole::GeneralPurpose => 0.20,
        }
    }

    /// Calculate default temperature based on role
    fn calculate_default_temperature(&self, role: &ModelRole) -> f64 {
        match role {
            ModelRole::TechnicalAnalysis => 0.1,  // Low temperature for precise calculations
            ModelRole::SentimentAnalysis => 0.3,  // Medium temperature for nuanced sentiment
            ModelRole::RiskManagement => 0.05,    // Very low temperature for conservative decisions
            ModelRole::MarketRegime => 0.2,       // Low temperature for pattern recognition
            ModelRole::MomentumAnalysis => 0.15,  // Low temperature for technical analysis
            ModelRole::GeneralPurpose => 0.4,     // Medium temperature for general decisions
        }
    }

    /// Calculate default max tokens based on role
    fn calculate_default_tokens(&self, role: &ModelRole) -> u32 {
        match role {
            ModelRole::TechnicalAnalysis => 200,  // Detailed technical analysis
            ModelRole::SentimentAnalysis => 150,  // Concise sentiment summary
            ModelRole::RiskManagement => 100,     // Brief risk assessment
            ModelRole::MarketRegime => 120,       // Market regime description
            ModelRole::MomentumAnalysis => 100,   // Momentum analysis
            ModelRole::GeneralPurpose => 300,     // General trading decision
        }
    }

    /// Calculate default priority based on role
    fn calculate_default_priority(&self, role: &ModelRole) -> u8 {
        match role {
            ModelRole::RiskManagement => 10,      // Highest priority
            ModelRole::TechnicalAnalysis => 8,    // High priority
            ModelRole::MarketRegime => 7,         // High priority
            ModelRole::SentimentAnalysis => 6,    // Medium priority
            ModelRole::MomentumAnalysis => 5,     // Medium priority
            ModelRole::GeneralPurpose => 4,       // Lower priority
        }
    }

    /// Sort models by priority within each role
    fn sort_models_by_priority(&mut self) {
        for models in self.role_assignments.values_mut() {
            models.sort_by(|a, b| {
                let priority_a = self.models.get(a).map(|m| m.priority).unwrap_or(0);
                let priority_b = self.models.get(b).map(|m| m.priority).unwrap_or(0);
                priority_b.cmp(&priority_a) // Higher priority first
            });
        }
    }

    /// Get models for a specific role
    pub fn get_models_for_role(&self, role: &ModelRole) -> Vec<&ModelConfig> {
        self.role_assignments
            .get(role)
            .map(|model_names| {
                model_names
                    .iter()
                    .filter_map(|name| self.models.get(name))
                    .filter(|config| config.enabled)
                    .take(self.max_models_per_role)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all enabled models
    pub fn get_enabled_models(&self) -> Vec<&ModelConfig> {
        self.models
            .values()
            .filter(|config| config.enabled)
            .collect()
    }

    /// Update model configuration
    pub fn update_model(&mut self, model_name: &str, updates: ModelConfig) -> Result<()> {
        if let Some(config) = self.models.get_mut(model_name) {
            *config = updates;
            self.sort_models_by_priority();
        }
        Ok(())
    }

    /// Enable/disable a model
    pub fn set_model_enabled(&mut self, model_name: &str, enabled: bool) -> Result<()> {
        if let Some(config) = self.models.get_mut(model_name) {
            config.enabled = enabled;
        }
        Ok(())
    }

    /// Set consensus threshold
    pub fn set_consensus_threshold(&mut self, threshold: f64) {
        self.consensus_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Get consensus threshold
    pub fn get_consensus_threshold(&self) -> f64 {
        self.consensus_threshold
    }

    /// Get model configuration
    pub fn get_model(&self, model_name: &str) -> Option<&ModelConfig> {
        self.models.get(model_name)
    }

    /// Get all models grouped by role
    pub fn get_models_by_role(&self) -> &HashMap<ModelRole, Vec<String>> {
        &self.role_assignments
    }

    /// Check if we have models for all critical roles
    pub fn has_critical_roles(&self) -> bool {
        let critical_roles = [
            ModelRole::RiskManagement,
            ModelRole::TechnicalAnalysis,
            ModelRole::MarketRegime,
        ];
        
        critical_roles.iter().all(|role| {
            self.role_assignments
                .get(role)
                .map(|models| !models.is_empty())
                .unwrap_or(false)
        })
    }

    /// Get role assignment summary
    pub fn get_role_summary(&self) -> HashMap<ModelRole, usize> {
        self.role_assignments
            .iter()
            .map(|(role, models)| {
                let enabled_count = models
                    .iter()
                    .filter(|name| {
                        self.models
                            .get(*name)
                            .map(|config| config.enabled)
                            .unwrap_or(false)
                    })
                    .count();
                (role.clone(), enabled_count)
            })
            .collect()
    }
}

impl Default for AIModelManager {
    fn default() -> Self {
        Self::new()
    }
}
