use crate::trading_strategy::{MarketDataPoint, AccountData, Position};
use crate::ollama::OllamaClient;
use serde_json::{json, Value};
use anyhow::Result;
use std::collections::HashMap;
use chrono::Utc;

/// AI-Enhanced Decision Engine that combines mathematical analysis with AI insights
pub struct AIDecisionEngine {
    pub ollama_client: OllamaClient,
    pub model: String,
    pub enhanced_decision_engine: crate::trading_strategy::enhanced_decision_engine::EnhancedDecisionEngine,
}

impl AIDecisionEngine {
    pub fn new(ollama_client: OllamaClient, model: String, risk_free_rate: f64) -> Self {
        Self {
            ollama_client,
            model,
            enhanced_decision_engine: crate::trading_strategy::enhanced_decision_engine::EnhancedDecisionEngine::new(risk_free_rate),
        }
    }

    /// Generate AI-enhanced trading decisions by combining mathematical analysis with AI insights
    pub async fn generate_ai_enhanced_decisions(
        &mut self,
        market_data: &HashMap<String, MarketDataPoint>,
        account_data: &AccountData,
        current_positions: &[Position],
        portfolio_data: &Value,
    ) -> Result<Vec<AITradingDecision>> {
        // Step 1: Run mathematical analysis
        self.enhanced_decision_engine.analyze_market_regime(market_data)?;
        let mathematical_decisions = self.enhanced_decision_engine.generate_enhanced_decisions(
            market_data,
            account_data,
            current_positions,
        )?;

        // Step 2: Create AI prompt with all relevant data
        let ai_prompt = self.create_ai_analysis_prompt(
            &mathematical_decisions,
            portfolio_data,
            &self.enhanced_decision_engine.market_regime,
        );

        // Step 3: Get AI insights
        let ai_response = self.ollama_client.generate_optimized(&self.model, &ai_prompt).await?;

        // Step 4: Combine mathematical and AI insights
        let ai_enhanced_decisions = self.combine_insights(
            mathematical_decisions,
            &ai_response,
            market_data,
        )?;

        Ok(ai_enhanced_decisions)
    }

    /// Create comprehensive AI prompt for trading analysis
    fn create_ai_analysis_prompt(
        &self,
        mathematical_decisions: &[crate::trading_strategy::enhanced_decision_engine::TradingDecision],
        portfolio_data: &Value,
        market_regime: &str,
    ) -> String {
        let decisions_summary = mathematical_decisions.iter()
            .map(|d| format!(
                "{}: {} (Confidence: {:.2}, Expected Return: {:.2}%, Position Size: ${:.2})",
                d.symbol, d.action, d.confidence_score, d.expected_return * 100.0, d.position_size.abs()
            ))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"You are an Elite quantitative trading analyst. Analyze the following trading data to transcend in profit multiplication:

MARKET REGIME: {}
CURRENT MATHEMATICAL DECISIONS:
{}

PORTFOLIO DATA:
{}

As an Elite quantitative trading analyst, provide:

1. MARKET REGIME ASSESSMENT: Is the mathematical regime detection accurate? Any additional insights for profit multiplication?
2. DECISION VALIDATION: Which mathematical decisions do you agree/disagree with and why? Focus on profit maximization.
3. RISK ASSESSMENT: Any additional risks not captured by the mathematical model that could impact profit multiplication?
4. OPPORTUNITY IDENTIFICATION: Any trading opportunities the mathematical model missed for exponential profit growth?
5. ENHANCED RECOMMENDATIONS: Specific buy/sell/hold recommendations with reasoning for maximum profit multiplication.
6. MARKET SENTIMENT: Overall market sentiment and implications for profit optimization.
7. PORTFOLIO OPTIMIZATION: Suggestions for portfolio rebalancing to transcend in profit multiplication.

Format your response as structured analysis with clear sections focused on profit multiplication and market transcendence."#,
            market_regime,
            decisions_summary,
            serde_json::to_string_pretty(portfolio_data).unwrap_or_else(|_| "Invalid JSON".to_string())
        )
    }

    /// Combine mathematical decisions with AI insights
    fn combine_insights(
        &self,
        mathematical_decisions: Vec<crate::trading_strategy::enhanced_decision_engine::TradingDecision>,
        ai_response: &str,
        _market_data: &HashMap<String, MarketDataPoint>,
    ) -> Result<Vec<AITradingDecision>> {
        let mut ai_enhanced_decisions = Vec::new();

        for math_decision in mathematical_decisions {
            // Parse AI response for this symbol (simplified - in real implementation, you'd parse the structured response)
            let ai_confidence_boost = self.extract_ai_confidence_boost(&ai_response, &math_decision.symbol);
            let ai_reasoning = self.extract_ai_reasoning(&ai_response, &math_decision.symbol);
            let ai_risk_assessment = self.extract_ai_risk_assessment(&ai_response, &math_decision.symbol);

            // Combine mathematical confidence with AI insights
            let combined_confidence = (math_decision.confidence_score + ai_confidence_boost).min(1.0).max(0.0);
            
            // Adjust position size based on AI risk assessment
            let adjusted_position_size = if ai_risk_assessment > 0.7 {
                math_decision.position_size * 0.8 // Reduce position if AI sees high risk
            } else if ai_risk_assessment < 0.3 {
                math_decision.position_size * 1.2 // Increase position if AI sees low risk
            } else {
                math_decision.position_size
            };

            let ai_decision = AITradingDecision {
                symbol: math_decision.symbol,
                action: math_decision.action,
                position_size: adjusted_position_size,
                expected_return: math_decision.expected_return,
                mathematical_confidence: math_decision.confidence_score,
                ai_confidence_boost,
                combined_confidence,
                mathematical_reasoning: math_decision.reasoning,
                ai_reasoning,
                ai_risk_assessment,
                stop_loss: math_decision.stop_loss,
                take_profit: math_decision.take_profit,
                market_regime: math_decision.market_regime,
                volatility_regime: math_decision.volatility_regime,
                timestamp: Utc::now(),
            };

            ai_enhanced_decisions.push(ai_decision);
        }

        // Sort by combined confidence
        ai_enhanced_decisions.sort_by(|a, b| b.combined_confidence.partial_cmp(&a.combined_confidence).unwrap());

        Ok(ai_enhanced_decisions)
    }

    /// Extract AI confidence boost for a specific symbol (simplified parsing)
    fn extract_ai_confidence_boost(&self, ai_response: &str, symbol: &str) -> f64 {
        // In a real implementation, you'd parse the structured AI response
        // For now, we'll use a simple heuristic based on sentiment
        if ai_response.to_lowercase().contains(&format!("{}: buy", symbol.to_lowercase())) {
            0.1 // Boost confidence for AI-confirmed buy signals
        } else if ai_response.to_lowercase().contains(&format!("{}: sell", symbol.to_lowercase())) {
            0.1 // Boost confidence for AI-confirmed sell signals
        } else if ai_response.to_lowercase().contains(&format!("{}: hold", symbol.to_lowercase())) {
            0.05 // Small boost for hold recommendations
        } else {
            0.0 // No boost if AI doesn't mention the symbol
        }
    }

    /// Extract AI reasoning for a specific symbol
    fn extract_ai_reasoning(&self, ai_response: &str, symbol: &str) -> String {
        // In a real implementation, you'd parse the structured AI response
        // For now, we'll extract relevant sentences mentioning the symbol
        let lines: Vec<&str> = ai_response.lines().collect();
        let mut reasoning = Vec::new();
        
        for line in lines {
            if line.to_lowercase().contains(&symbol.to_lowercase()) {
                reasoning.push(line.trim());
            }
        }
        
        if reasoning.is_empty() {
            "AI analysis did not provide specific reasoning for this symbol.".to_string()
        } else {
            reasoning.join(" ")
        }
    }

    /// Extract AI risk assessment for a specific symbol
    fn extract_ai_risk_assessment(&self, ai_response: &str, symbol: &str) -> f64 {
        // In a real implementation, you'd parse the structured AI response
        // For now, we'll use a simple heuristic based on risk-related keywords
        let risk_keywords = ["high risk", "volatile", "uncertain", "risky"];
        let low_risk_keywords = ["low risk", "stable", "safe", "conservative"];
        
        let response_lower = ai_response.to_lowercase();
        let symbol_context = if response_lower.contains(&symbol.to_lowercase()) {
            // Extract sentences around the symbol mention
            let lines: Vec<&str> = ai_response.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if line.to_lowercase().contains(&symbol.to_lowercase()) {
                    let start = if i > 0 { i - 1 } else { 0 };
                    let end = if i < lines.len() - 1 { i + 2 } else { lines.len() };
                    let context = lines[start..end].join(" ").to_lowercase();
                    // Continue with risk assessment logic
                    let high_risk_count = risk_keywords.iter().filter(|&&keyword| context.contains(keyword)).count();
                    let low_risk_count = low_risk_keywords.iter().filter(|&&keyword| context.contains(keyword)).count();
                    
                    if high_risk_count > low_risk_count {
                        return 0.8; // High risk
                    } else if low_risk_count > high_risk_count {
                        return 0.2; // Low risk
                    } else {
                        return 0.5; // Medium risk
                    }
                }
            }
            response_lower
        } else {
            response_lower
        };

        let high_risk_count = risk_keywords.iter().filter(|&&keyword| symbol_context.contains(keyword)).count();
        let low_risk_count = low_risk_keywords.iter().filter(|&&keyword| symbol_context.contains(keyword)).count();
        
        if high_risk_count > low_risk_count {
            0.8 // High risk
        } else if low_risk_count > high_risk_count {
            0.2 // Low risk
        } else {
            0.5 // Medium risk
        }
    }

    /// Generate comprehensive AI analysis report
    pub async fn generate_ai_analysis_report(
        &mut self,
        market_data: &HashMap<String, MarketDataPoint>,
        account_data: &AccountData,
        current_positions: &[Position],
        portfolio_data: &Value,
    ) -> Result<Value> {
        let ai_decisions = self.generate_ai_enhanced_decisions(
            market_data,
            account_data,
            current_positions,
            portfolio_data,
        ).await?;

        let report = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "model": self.model,
            "market_regime": self.enhanced_decision_engine.market_regime,
            "volatility_regime": self.enhanced_decision_engine.volatility_regime,
            "regime_confidence": self.enhanced_decision_engine.regime_confidence,
            "ai_enhanced_decisions": ai_decisions,
            "summary": {
                "total_decisions": ai_decisions.len(),
                "high_confidence_decisions": ai_decisions.iter().filter(|d| d.combined_confidence > 0.7).count(),
                "medium_confidence_decisions": ai_decisions.iter().filter(|d| d.combined_confidence > 0.4 && d.combined_confidence <= 0.7).count(),
                "low_confidence_decisions": ai_decisions.iter().filter(|d| d.combined_confidence <= 0.4).count(),
                "total_position_value": ai_decisions.iter().map(|d| d.position_size.abs()).sum::<f64>(),
            }
        });

        Ok(report)
    }
}

/// AI-Enhanced Trading Decision
#[derive(Debug, Clone, serde::Serialize)]
pub struct AITradingDecision {
    pub symbol: String,
    pub action: crate::trading_strategy::enhanced_decision_engine::TradingAction,
    pub position_size: f64,
    pub expected_return: f64,
    pub mathematical_confidence: f64,
    pub ai_confidence_boost: f64,
    pub combined_confidence: f64,
    pub mathematical_reasoning: String,
    pub ai_reasoning: String,
    pub ai_risk_assessment: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub market_regime: String,
    pub volatility_regime: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
