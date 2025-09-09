use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Market regime types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MarketRegime {
    BullMarket,        // Strong uptrend, high confidence
    BearMarket,        // Strong downtrend, defensive
    SidewaysMarket,    // Range-bound, mean reversion
    HighVolatility,    // Elevated volatility, risk management
    LowVolatility,     // Low volatility, trend following
    TrendingUp,        // Moderate uptrend
    TrendingDown,      // Moderate downtrend
    Consolidation,     // Tight range, breakout potential
    Reversal,          // Potential trend change
    Unknown,           // Insufficient data
}

/// Market regime characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeCharacteristics {
    pub regime: MarketRegime,
    pub volatility_level: f64,        // 0.0 to 1.0
    pub trend_strength: f64,          // -1.0 to 1.0
    pub momentum: f64,                // -1.0 to 1.0
    pub volume_profile: f64,          // 0.0 to 1.0
    pub market_breadth: f64,          // 0.0 to 1.0
    pub confidence: f64,              // 0.0 to 1.0
    pub duration_days: u32,
    pub regime_score: f64,            // Overall regime strength
}

/// Regime-specific strategy parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeStrategyParams {
    pub regime: MarketRegime,
    pub position_size_multiplier: f64,
    pub stop_loss_multiplier: f64,
    pub take_profit_multiplier: f64,
    pub max_positions: u32,
    pub preferred_timeframes: Vec<String>,
    pub risk_tolerance: f64,
    pub strategy_focus: String,
}

/// Market regime adaptation manager
pub struct MarketRegimeAdaptationManager {
    current_regime: MarketRegime,
    regime_history: Vec<RegimeCharacteristics>,
    strategy_params: HashMap<MarketRegime, RegimeStrategyParams>,
    regime_detection_window: usize,
    enabled: bool,
    trading_mode: String,
}

impl MarketRegimeAdaptationManager {
    /// Create new market regime adaptation manager
    pub fn new(trading_mode: String) -> Self {
        let mut manager = Self {
            current_regime: MarketRegime::Unknown,
            regime_history: Vec::new(),
            strategy_params: HashMap::new(),
            regime_detection_window: 20,
            enabled: true, // Available in both modes
            trading_mode,
        };

        // Initialize regime-specific strategy parameters
        manager.initialize_strategy_params();
        manager
    }

    /// Initialize strategy parameters for each regime
    fn initialize_strategy_params(&mut self) {
        // Bull Market parameters
        self.strategy_params.insert(MarketRegime::BullMarket, RegimeStrategyParams {
            regime: MarketRegime::BullMarket,
            position_size_multiplier: 1.2,
            stop_loss_multiplier: 0.8,
            take_profit_multiplier: 1.5,
            max_positions: 8,
            preferred_timeframes: vec!["1h".to_string(), "4h".to_string()],
            risk_tolerance: 0.7,
            strategy_focus: "Trend following, momentum strategies".to_string(),
        });

        // Bear Market parameters
        self.strategy_params.insert(MarketRegime::BearMarket, RegimeStrategyParams {
            regime: MarketRegime::BearMarket,
            position_size_multiplier: 0.6,
            stop_loss_multiplier: 1.2,
            take_profit_multiplier: 0.8,
            max_positions: 4,
            preferred_timeframes: vec!["15m".to_string(), "1h".to_string()],
            risk_tolerance: 0.3,
            strategy_focus: "Defensive, short-term, risk management".to_string(),
        });

        // Sideways Market parameters
        self.strategy_params.insert(MarketRegime::SidewaysMarket, RegimeStrategyParams {
            regime: MarketRegime::SidewaysMarket,
            position_size_multiplier: 0.8,
            stop_loss_multiplier: 1.0,
            take_profit_multiplier: 1.0,
            max_positions: 6,
            preferred_timeframes: vec!["5m".to_string(), "15m".to_string()],
            risk_tolerance: 0.5,
            strategy_focus: "Mean reversion, range trading".to_string(),
        });

        // High Volatility parameters
        self.strategy_params.insert(MarketRegime::HighVolatility, RegimeStrategyParams {
            regime: MarketRegime::HighVolatility,
            position_size_multiplier: 0.5,
            stop_loss_multiplier: 1.5,
            take_profit_multiplier: 1.2,
            max_positions: 3,
            preferred_timeframes: vec!["1m".to_string(), "5m".to_string()],
            risk_tolerance: 0.2,
            strategy_focus: "Volatility trading, risk management".to_string(),
        });

        // Low Volatility parameters
        self.strategy_params.insert(MarketRegime::LowVolatility, RegimeStrategyParams {
            regime: MarketRegime::LowVolatility,
            position_size_multiplier: 1.1,
            stop_loss_multiplier: 0.9,
            take_profit_multiplier: 1.3,
            max_positions: 7,
            preferred_timeframes: vec!["1h".to_string(), "4h".to_string()],
            risk_tolerance: 0.6,
            strategy_focus: "Trend following, breakout strategies".to_string(),
        });

        // Trending Up parameters
        self.strategy_params.insert(MarketRegime::TrendingUp, RegimeStrategyParams {
            regime: MarketRegime::TrendingUp,
            position_size_multiplier: 1.0,
            stop_loss_multiplier: 0.9,
            take_profit_multiplier: 1.4,
            max_positions: 6,
            preferred_timeframes: vec!["15m".to_string(), "1h".to_string()],
            risk_tolerance: 0.6,
            strategy_focus: "Momentum, trend continuation".to_string(),
        });

        // Trending Down parameters
        self.strategy_params.insert(MarketRegime::TrendingDown, RegimeStrategyParams {
            regime: MarketRegime::TrendingDown,
            position_size_multiplier: 0.7,
            stop_loss_multiplier: 1.1,
            take_profit_multiplier: 0.9,
            max_positions: 5,
            preferred_timeframes: vec!["5m".to_string(), "15m".to_string()],
            risk_tolerance: 0.4,
            strategy_focus: "Short-term, defensive positioning".to_string(),
        });

        // Consolidation parameters
        self.strategy_params.insert(MarketRegime::Consolidation, RegimeStrategyParams {
            regime: MarketRegime::Consolidation,
            position_size_multiplier: 0.9,
            stop_loss_multiplier: 1.0,
            take_profit_multiplier: 1.1,
            max_positions: 5,
            preferred_timeframes: vec!["5m".to_string(), "15m".to_string()],
            risk_tolerance: 0.5,
            strategy_focus: "Breakout preparation, range trading".to_string(),
        });

        // Reversal parameters
        self.strategy_params.insert(MarketRegime::Reversal, RegimeStrategyParams {
            regime: MarketRegime::Reversal,
            position_size_multiplier: 0.6,
            stop_loss_multiplier: 1.3,
            take_profit_multiplier: 1.6,
            max_positions: 4,
            preferred_timeframes: vec!["1m".to_string(), "5m".to_string()],
            risk_tolerance: 0.3,
            strategy_focus: "Early reversal detection, risk management".to_string(),
        });
    }

    /// Check if market regime adaptation is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get status message
    pub fn get_status(&self) -> String {
        if self.is_enabled() {
            format!("✅ Market Regime Adaptation ENABLED - Current: {:?}", self.current_regime)
        } else {
            "❌ Market Regime Adaptation DISABLED".to_string()
        }
    }

    /// Analyze market regime based on market data
    pub async fn analyze_market_regime(&mut self, market_data: &HashMap<String, f64>) -> Result<RegimeCharacteristics> {
        if !self.is_enabled() {
            return Ok(RegimeCharacteristics {
                regime: MarketRegime::Unknown,
                volatility_level: 0.0,
                trend_strength: 0.0,
                momentum: 0.0,
                volume_profile: 0.0,
                market_breadth: 0.0,
                confidence: 0.0,
                duration_days: 0,
                regime_score: 0.0,
            });
        }

        // Calculate regime characteristics
        let volatility_level = self.calculate_volatility_level(market_data);
        let trend_strength = self.calculate_trend_strength(market_data);
        let momentum = self.calculate_momentum(market_data);
        let volume_profile = self.calculate_volume_profile(market_data);
        let market_breadth = self.calculate_market_breadth(market_data);

        // Determine regime based on characteristics
        let regime = self.determine_regime(volatility_level, trend_strength, momentum, volume_profile, market_breadth);
        
        // Calculate confidence and duration
        let confidence = self.calculate_regime_confidence(volatility_level, trend_strength, momentum);
        let duration_days = self.calculate_regime_duration(&regime);
        
        // Calculate overall regime score
        let regime_score = (volatility_level + trend_strength.abs() + momentum.abs() + volume_profile + market_breadth) / 5.0;

        let characteristics = RegimeCharacteristics {
            regime: regime.clone(),
            volatility_level,
            trend_strength,
            momentum,
            volume_profile,
            market_breadth,
            confidence,
            duration_days,
            regime_score,
        };

        // Update current regime and history
        self.current_regime = regime;
        self.regime_history.push(characteristics.clone());
        
        // Maintain history window
        if self.regime_history.len() > self.regime_detection_window {
            self.regime_history.remove(0);
        }

        Ok(characteristics)
    }

    /// Calculate volatility level
    fn calculate_volatility_level(&self, market_data: &HashMap<String, f64>) -> f64 {
        if market_data.is_empty() {
            return 0.0;
        }

        let values: Vec<f64> = market_data.values().cloned().collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        // Normalize to 0-1 scale
        (std_dev / mean).min(1.0)
    }

    /// Calculate trend strength
    fn calculate_trend_strength(&self, market_data: &HashMap<String, f64>) -> f64 {
        if market_data.len() < 2 {
            return 0.0;
        }

        let mut sorted_data: Vec<_> = market_data.iter().collect();
        sorted_data.sort_by(|a, b| a.0.cmp(b.0));

        let first_price = sorted_data[0].1;
        let last_price = sorted_data[sorted_data.len() - 1].1;
        
        // Calculate trend as percentage change
        let trend = (last_price - first_price) / first_price;
        
        // Normalize to -1 to 1 scale
        trend.clamp(-1.0, 1.0)
    }

    /// Calculate momentum
    fn calculate_momentum(&self, market_data: &HashMap<String, f64>) -> f64 {
        if market_data.len() < 3 {
            return 0.0;
        }

        let mut sorted_data: Vec<_> = market_data.iter().collect();
        sorted_data.sort_by(|a, b| a.0.cmp(b.0));

        let recent_change = sorted_data[sorted_data.len() - 1].1 - sorted_data[sorted_data.len() - 2].1;
        let previous_change = sorted_data[sorted_data.len() - 2].1 - sorted_data[sorted_data.len() - 3].1;

        // Momentum is acceleration of price change
        let momentum = recent_change - previous_change;
        
        // Normalize to -1 to 1 scale
        momentum.clamp(-1.0, 1.0)
    }

    /// Calculate volume profile
    fn calculate_volume_profile(&self, _market_data: &HashMap<String, f64>) -> f64 {
        // Simplified volume profile calculation
        // In real implementation, this would analyze volume patterns
        0.5 // Placeholder
    }

    /// Calculate market breadth
    fn calculate_market_breadth(&self, market_data: &HashMap<String, f64>) -> f64 {
        if market_data.is_empty() {
            return 0.0;
        }

        let values: Vec<f64> = market_data.values().cloned().collect();
        let positive_count = values.iter().filter(|&&x| x > 0.0).count();
        
        positive_count as f64 / values.len() as f64
    }

    /// Determine regime based on characteristics
    fn determine_regime(&self, volatility: f64, trend: f64, momentum: f64, _volume: f64, breadth: f64) -> MarketRegime {
        // High volatility regimes
        if volatility > 0.7 {
            return MarketRegime::HighVolatility;
        }

        // Low volatility regimes
        if volatility < 0.3 {
            if trend.abs() > 0.5 {
                return if trend > 0.0 { MarketRegime::TrendingUp } else { MarketRegime::TrendingDown };
            } else {
                return MarketRegime::LowVolatility;
            }
        }

        // Trend-based regimes
        if trend > 0.6 && breadth > 0.6 {
            return MarketRegime::BullMarket;
        } else if trend < -0.6 && breadth < 0.4 {
            return MarketRegime::BearMarket;
        } else if trend.abs() > 0.3 {
            return if trend > 0.0 { MarketRegime::TrendingUp } else { MarketRegime::TrendingDown };
        }

        // Momentum-based regimes
        if momentum.abs() > 0.5 {
            return MarketRegime::Reversal;
        }

        // Range-bound regimes
        if trend.abs() < 0.2 && volatility < 0.5 {
            return MarketRegime::SidewaysMarket;
        } else if trend.abs() < 0.1 {
            return MarketRegime::Consolidation;
        }

        MarketRegime::Unknown
    }

    /// Calculate regime confidence
    fn calculate_regime_confidence(&self, volatility: f64, trend: f64, momentum: f64) -> f64 {
        let trend_confidence = trend.abs();
        let momentum_confidence = momentum.abs();
        let volatility_confidence = if volatility > 0.3 && volatility < 0.7 { 1.0 } else { 0.5 };

        (trend_confidence + momentum_confidence + volatility_confidence) / 3.0
    }

    /// Calculate regime duration
    fn calculate_regime_duration(&self, regime: &MarketRegime) -> u32 {
        let mut duration = 1;
        
        for characteristics in self.regime_history.iter().rev() {
            if characteristics.regime == *regime {
                duration += 1;
            } else {
                break;
            }
        }

        duration
    }

    /// Get current regime strategy parameters
    pub fn get_current_strategy_params(&self) -> Option<&RegimeStrategyParams> {
        self.strategy_params.get(&self.current_regime)
    }

    /// Get regime-specific recommendations
    pub fn get_regime_recommendations(&self) -> Vec<String> {
        let params = match self.get_current_strategy_params() {
            Some(p) => p,
            None => return vec!["No regime parameters available".to_string()],
        };

        vec![
            format!("🎯 Strategy Focus: {}", params.strategy_focus),
            format!("📊 Position Size: {:.1}x multiplier", params.position_size_multiplier),
            format!("🛡️ Stop Loss: {:.1}x multiplier", params.stop_loss_multiplier),
            format!("💰 Take Profit: {:.1}x multiplier", params.take_profit_multiplier),
            format!("📈 Max Positions: {}", params.max_positions),
            format!("⏰ Preferred Timeframes: {:?}", params.preferred_timeframes),
            format!("🎲 Risk Tolerance: {:.1}", params.risk_tolerance),
        ]
    }

    /// Get regime transition insights
    pub fn get_regime_insights(&self) -> Vec<String> {
        if self.regime_history.len() < 2 {
            return vec!["Insufficient data for regime insights".to_string()];
        }

        let mut insights = Vec::new();
        let current = &self.regime_history[self.regime_history.len() - 1];
        let previous = &self.regime_history[self.regime_history.len() - 2];

        // Check for regime changes
        if current.regime != previous.regime {
            insights.push(format!("🔄 Regime Change: {:?} → {:?}", previous.regime, current.regime));
        }

        // Check for regime strength changes
        if current.regime_score > previous.regime_score + 0.1 {
            insights.push("📈 Regime strength increasing".to_string());
        } else if current.regime_score < previous.regime_score - 0.1 {
            insights.push("📉 Regime strength decreasing".to_string());
        }

        // Check for volatility changes
        if current.volatility_level > previous.volatility_level + 0.2 {
            insights.push("⚡ Volatility spike detected".to_string());
        } else if current.volatility_level < previous.volatility_level - 0.2 {
            insights.push("😴 Volatility compression".to_string());
        }

        // Check for trend changes
        if current.trend_strength > previous.trend_strength + 0.3 {
            insights.push("🚀 Trend acceleration".to_string());
        } else if current.trend_strength < previous.trend_strength - 0.3 {
            insights.push("🛑 Trend deceleration".to_string());
        }

        insights
    }

    /// Get regime summary
    pub fn get_regime_summary(&self) -> String {
        if let Some(current) = self.regime_history.last() {
            format!(
                "Market Regime: {:?} | Confidence: {:.1}% | Duration: {} days | Score: {:.2}",
                current.regime,
                current.confidence * 100.0,
                current.duration_days,
                current.regime_score
            )
        } else {
            "No regime data available".to_string()
        }
    }
}
