use crate::trading_strategy::{MarketDataPoint, AccountData, Position};
use crate::market_data::TechnicalIndicators;
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

    /// Generate AI-enhanced trading decisions with 15-minute technical analysis
    pub async fn generate_ai_enhanced_decisions_with_technical_analysis(
        &mut self,
        market_data: &HashMap<String, MarketDataPoint>,
        account_data: &AccountData,
        current_positions: &[Position],
        portfolio_data: &Value,
        technical_indicators: &HashMap<String, TechnicalIndicators>,
    ) -> Result<Vec<AITradingDecision>> {
        // Step 1: Run mathematical analysis
        self.enhanced_decision_engine.analyze_market_regime(market_data)?;
        let mathematical_decisions = self.enhanced_decision_engine.generate_enhanced_decisions(
            market_data,
            account_data,
            current_positions,
        )?;

        // Step 2: Create enhanced AI prompt with technical analysis
        let ai_prompt = self.create_enhanced_ai_analysis_prompt(
            &mathematical_decisions,
            portfolio_data,
            &self.enhanced_decision_engine.market_regime,
            technical_indicators,
        );

        // Step 3: Get AI insights
        let ai_response = self.ollama_client.generate_optimized(&self.model, &ai_prompt).await?;

        // Step 4: Combine mathematical and AI insights
        let ai_decisions = self.combine_mathematical_and_ai_insights(
            &mathematical_decisions,
            &ai_response,
            technical_indicators,
        )?;

        Ok(ai_decisions)
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

    /// Create enhanced AI analysis prompt with technical indicators
    fn create_enhanced_ai_analysis_prompt(
        &self,
        mathematical_decisions: &[crate::trading_strategy::enhanced_decision_engine::TradingDecision],
        portfolio_data: &Value,
        market_regime: &str,
        technical_indicators: &HashMap<String, TechnicalIndicators>,
    ) -> String {
        let decisions_summary = mathematical_decisions.iter()
            .map(|d| format!(
                "{}: {} (Confidence: {:.2}, Expected Return: {:.2}%, Position Size: ${:.2})",
                d.symbol, d.action, d.confidence_score, d.expected_return * 100.0, d.position_size.abs()
            ))
            .collect::<Vec<_>>()
            .join("\n");

        let mut technical_analysis = String::new();
        
        for (_symbol, indicators) in technical_indicators {
            technical_analysis.push_str(&format!("{}\n", indicators.to_ai_analysis()));
        }

        format!(
            "You are an Elite Quantitative Trading Analyst specializing in profit multiplication and market transcendence.

🎯 MISSION: Analyze the following data and provide precise trading recommendations for maximum profit generation.

📊 TECHNICAL ANALYSIS (15-MINUTE DATA):
{}

📈 MATHEMATICAL ANALYSIS:
Market Regime: {}
Mathematical Decisions:
{}

💰 PORTFOLIO DATA:
{}

🧠 ANALYSIS REQUIREMENTS:
1. Analyze 15-minute technical indicators for short-term momentum
2. Identify price patterns and trend reversals
3. Consider volume confirmation for trade signals
4. Evaluate RSI overbought/oversold conditions
5. Use MACD for trend confirmation
6. Apply Bollinger Bands for volatility analysis
7. Consider ATR for position sizing

🎯 RESPONSE FORMAT:
Provide specific trading recommendations in this exact format:
- BUY [SYMBOL]: [QUANTITY] shares at [PRICE] - [REASONING]
- SELL [SYMBOL]: [QUANTITY] shares at [PRICE] - [REASONING]
- HOLD [SYMBOL] - [REASONING]

Focus on high-probability trades with clear technical confirmation.",
            technical_analysis,
            market_regime,
            decisions_summary,
            serde_json::to_string_pretty(portfolio_data).unwrap_or_default()
        )
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
            r#"You are an Elite quantitative trading analyst specializing in algorithmic trading and portfolio optimization. 

ANALYZE THE FOLLOWING TRADING DATA AND PROVIDE SPECIFIC, ACTIONABLE TRADING RECOMMENDATIONS:

MARKET REGIME: {}
CURRENT MATHEMATICAL DECISIONS:
{}

PORTFOLIO DATA:
{}

REQUIRED OUTPUT FORMAT:

1. MARKET REGIME ASSESSMENT:
   - Is the mathematical regime detection accurate?
   - Additional market insights for profit optimization
   - Current market sentiment analysis

2. DECISION VALIDATION:
   - Which mathematical decisions do you agree/disagree with?
   - Specific reasoning for each decision
   - Confidence adjustments for each symbol

3. RISK ASSESSMENT:
   - Additional risks not captured by mathematical model
   - Risk level for each symbol (LOW/MEDIUM/HIGH)
   - Portfolio-level risk considerations

4. TRADING RECOMMENDATIONS:
   - Specific BUY/SELL/HOLD actions for each symbol
   - Target entry/exit prices
   - Position sizes (as % of portfolio)
   - Stop loss levels
   - Time horizon for each trade

5. PORTFOLIO OPTIMIZATION:
   - Rebalancing suggestions
   - Asset allocation recommendations
   - Risk management strategies

6. EXECUTION PRIORITY:
   - Order of execution for recommendations
   - Market timing considerations
   - Position sizing strategy

Provide your response in a structured format with clear sections and specific actionable recommendations. Focus on profit maximization and risk management."#,
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
            let _ai_reasoning = self.extract_ai_reasoning(&ai_response, &math_decision.symbol);
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
                confidence_score: combined_confidence,
                reasoning: format!("Mathematical: {:.2}, AI Boost: {:.2}", math_decision.confidence_score, ai_confidence_boost),
                ai_insights: ai_response.to_string(),
                technical_analysis: String::new(),
                stop_loss: math_decision.stop_loss,
                take_profit: math_decision.take_profit,
                market_regime: math_decision.market_regime,
                volatility_regime: math_decision.volatility_regime,
                timestamp: Utc::now(),
            };

            ai_enhanced_decisions.push(ai_decision);
        }

        // Sort by combined confidence
        ai_enhanced_decisions.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score)
            .unwrap_or(std::cmp::Ordering::Equal));

        Ok(ai_enhanced_decisions)
    }

    /// Extract AI confidence boost for a specific symbol (improved parsing)
    fn extract_ai_confidence_boost(&self, ai_response: &str, symbol: &str) -> f64 {
        let response_lower = ai_response.to_lowercase();
        let symbol_lower = symbol.to_lowercase();
        
        // Look for structured trading recommendations
        if response_lower.contains(&format!("{}: buy", symbol_lower)) || 
           response_lower.contains(&format!("buy {}", symbol_lower)) ||
           response_lower.contains(&format!("{} buy", symbol_lower)) {
            return 0.15; // Higher boost for AI-confirmed buy signals
        } else if response_lower.contains(&format!("{}: sell", symbol_lower)) ||
                  response_lower.contains(&format!("sell {}", symbol_lower)) ||
                  response_lower.contains(&format!("{} sell", symbol_lower)) {
            return 0.15; // Higher boost for AI-confirmed sell signals
        } else if response_lower.contains(&format!("{}: hold", symbol_lower)) ||
                  response_lower.contains(&format!("hold {}", symbol_lower)) ||
                  response_lower.contains(&format!("{} hold", symbol_lower)) {
            return 0.08; // Moderate boost for hold recommendations
        }
        
        // Look for confidence indicators
        if response_lower.contains("high confidence") || response_lower.contains("strong buy") {
            return 0.12;
        } else if response_lower.contains("medium confidence") || response_lower.contains("moderate") {
            return 0.08;
        } else if response_lower.contains("low confidence") || response_lower.contains("weak") {
            return 0.03;
        }
        
        // Look for positive sentiment around the symbol
        let positive_indicators = ["bullish", "positive", "strong", "good", "favorable", "opportunity"];
        let negative_indicators = ["bearish", "negative", "weak", "poor", "unfavorable", "risk"];
        
        let positive_count = positive_indicators.iter()
            .filter(|&&indicator| response_lower.contains(indicator))
            .count();
        let negative_count = negative_indicators.iter()
            .filter(|&&indicator| response_lower.contains(indicator))
            .count();
        
        if positive_count > negative_count {
            return 0.05; // Small boost for positive sentiment
        } else if negative_count > positive_count {
            return -0.05; // Small reduction for negative sentiment
        }
        
        0.0 // No boost if AI doesn't provide clear signals
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

    /// Combine mathematical analysis with AI insights and technical indicators
    fn combine_mathematical_and_ai_insights(
        &self,
        mathematical_decisions: &[crate::trading_strategy::enhanced_decision_engine::TradingDecision],
        ai_response: &str,
        technical_indicators: &HashMap<String, TechnicalIndicators>,
    ) -> Result<Vec<AITradingDecision>> {
        let mut ai_enhanced_decisions = Vec::new();

        for decision in mathematical_decisions {
            // Extract AI confidence boost for this symbol
            let ai_confidence_boost = self.extract_ai_confidence_boost(ai_response, &decision.symbol);
            
            // Get technical indicators for this symbol
            let technical_boost = if let Some(indicators) = technical_indicators.get(&decision.symbol) {
                self.calculate_technical_confidence_boost(indicators)
            } else {
                0.0
            };

            // Combine all confidence factors
            let final_confidence = (decision.confidence_score + ai_confidence_boost + technical_boost).min(1.0);

            ai_enhanced_decisions.push(AITradingDecision {
                symbol: decision.symbol.clone(),
                action: decision.action.clone(),
                confidence_score: final_confidence,
                expected_return: decision.expected_return,
                position_size: decision.position_size,
                reasoning: format!(
                    "Mathematical: {:.2}, AI Boost: {:.2}, Technical Boost: {:.2}",
                    decision.confidence_score, ai_confidence_boost, technical_boost
                ),
                ai_insights: ai_response.to_string(),
                technical_analysis: technical_indicators.get(&decision.symbol)
                    .map(|ti| ti.to_ai_analysis())
                    .unwrap_or_default(),
                stop_loss: decision.stop_loss,
                take_profit: decision.take_profit,
                market_regime: "Unknown".to_string(),
                volatility_regime: "Unknown".to_string(),
                timestamp: Utc::now(),
            });
        }

        Ok(ai_enhanced_decisions)
    }

    /// Calculate technical confidence boost based on indicators
    fn calculate_technical_confidence_boost(&self, indicators: &TechnicalIndicators) -> f64 {
        let mut boost: f64 = 0.0;
        
        // RSI analysis
        if indicators.rsi < 30.0 {
            boost += 0.1; // Oversold - bullish signal
        } else if indicators.rsi > 70.0 {
            boost -= 0.1; // Overbought - bearish signal
        }
        
        // MACD analysis
        if indicators.macd > indicators.macd_signal && indicators.macd_histogram > 0.0 {
            boost += 0.05; // Bullish MACD
        } else if indicators.macd < indicators.macd_signal && indicators.macd_histogram < 0.0 {
            boost -= 0.05; // Bearish MACD
        }
        
        // Bollinger Bands analysis
        if indicators.bollinger_lower > 0.0 && indicators.bollinger_upper > 0.0 {
            let current_price = (indicators.bollinger_upper + indicators.bollinger_lower) / 2.0;
            let band_width = indicators.bollinger_upper - indicators.bollinger_lower;
            let position_in_band = (current_price - indicators.bollinger_lower) / band_width;
            
            if position_in_band < 0.2 {
                boost += 0.05; // Near lower band - potential bounce
            } else if position_in_band > 0.8 {
                boost -= 0.05; // Near upper band - potential pullback
            }
        }
        
        // Volume analysis
        if indicators.volume_ratio > 1.5 {
            boost += 0.03; // High volume confirmation
        } else if indicators.volume_ratio < 0.5 {
            boost -= 0.03; // Low volume - weak signal
        }
        
        boost.max(-0.2f64).min(0.2f64) // Cap the boost between -20% and +20%
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
                            "high_confidence_decisions": ai_decisions.iter().filter(|d| d.confidence_score > 0.7).count(),
            "medium_confidence_decisions": ai_decisions.iter().filter(|d| d.confidence_score > 0.4 && d.confidence_score <= 0.7).count(),
            "low_confidence_decisions": ai_decisions.iter().filter(|d| d.confidence_score <= 0.4).count(),
                "total_position_value": ai_decisions.iter().map(|d| d.position_size.abs()).sum::<f64>(),
            }
        });

        Ok(report)
    }
}

/// AI-Enhanced Trading Decision with Technical Analysis
#[derive(Debug, Clone, serde::Serialize)]
pub struct AITradingDecision {
    pub symbol: String,
    pub action: crate::trading_strategy::enhanced_decision_engine::TradingAction,
    pub confidence_score: f64,
    pub expected_return: f64,
    pub position_size: f64,
    pub reasoning: String,
    pub ai_insights: String,
    pub technical_analysis: String,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub market_regime: String,
    pub volatility_regime: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
