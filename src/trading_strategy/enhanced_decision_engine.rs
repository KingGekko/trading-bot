use crate::trading_strategy::{MarketDataPoint, AccountData, Position};

use anyhow::Result;
use std::collections::HashMap;

/// Enhanced decision engine for sophisticated long/short trading decisions
pub struct EnhancedDecisionEngine {
    pub risk_free_rate: f64,
    pub market_regime: String,
    pub regime_confidence: f64,
    pub volatility_regime: String,
    pub momentum_threshold: f64,
    pub mean_reversion_threshold: f64,
    pub contrarian_threshold: f64,
}

impl EnhancedDecisionEngine {
    pub fn new(risk_free_rate: f64) -> Self {
        Self {
            risk_free_rate,
            market_regime: "unknown".to_string(),
            regime_confidence: 0.0,
            volatility_regime: "normal".to_string(),
            momentum_threshold: 0.02, // 2% momentum threshold
            mean_reversion_threshold: 0.05, // 5% mean reversion threshold
            contrarian_threshold: 0.8, // 80% contrarian threshold
        }
    }

    /// Analyze market regime and determine optimal strategy
    pub fn analyze_market_regime(&mut self, market_data: &HashMap<String, MarketDataPoint>) -> Result<()> {
        let mut total_momentum = 0.0;
        let mut total_volatility = 0.0;
        let mut asset_count = 0;

        for (_, data) in market_data {
            let momentum = (data.price - data.open) / data.open;
            let volatility = (data.high - data.low) / data.open;
            
            total_momentum += momentum;
            total_volatility += volatility;
            asset_count += 1;
        }

        if asset_count > 0 {
            let avg_momentum = total_momentum / asset_count as f64;
            let avg_volatility = total_volatility / asset_count as f64;

            // Determine market regime
            self.market_regime = if avg_momentum > self.momentum_threshold {
                "bull_market".to_string()
            } else if avg_momentum < -self.momentum_threshold {
                "bear_market".to_string()
            } else if avg_volatility > 0.03 {
                "high_volatility".to_string()
            } else if avg_volatility < 0.01 {
                "low_volatility".to_string()
            } else {
                "sideways_market".to_string()
            };

            // Determine volatility regime
            self.volatility_regime = if avg_volatility > 0.03 {
                "high".to_string()
            } else if avg_volatility < 0.01 {
                "low".to_string()
            } else {
                "normal".to_string()
            };

            // Calculate regime confidence
            self.regime_confidence = (avg_momentum.abs() + avg_volatility).min(1.0);
        }

        Ok(())
    }

    /// Generate sophisticated trading decisions with long/short capabilities
    pub fn generate_enhanced_decisions(
        &self,
        market_data: &HashMap<String, MarketDataPoint>,
        account_data: &AccountData,
        current_positions: &[Position],
    ) -> Result<Vec<TradingDecision>> {
        let mut decisions = Vec::new();

        for (symbol, data) in market_data {
            let decision = self.analyze_asset_opportunity(
                symbol,
                data,
                account_data,
                current_positions,
            )?;
            decisions.push(decision);
        }

        // Sort by confidence and expected return
        decisions.sort_by(|a, b| {
            b.confidence_score.partial_cmp(&a.confidence_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
        .then(b.expected_return.partial_cmp(&a.expected_return)
            .unwrap_or(std::cmp::Ordering::Equal))
        });

        Ok(decisions)
    }

    /// Analyze individual asset for trading opportunities
    fn analyze_asset_opportunity(
        &self,
        symbol: &str,
        data: &MarketDataPoint,
        account_data: &AccountData,
        current_positions: &[Position],
    ) -> Result<TradingDecision> {
        let current_position = current_positions.iter().find(|p| p.symbol == symbol);
        
        // Calculate technical indicators
        let momentum = (data.price - data.open) / data.open;
        let volatility = (data.high - data.low) / data.open;
        let volume_ratio = data.volume / 1000000.0; // Normalize volume
        
        // Calculate expected return using multiple factors
        let expected_return = self.calculate_enhanced_expected_return(
            data, momentum, volatility, volume_ratio
        );

        // Determine optimal action based on regime and technicals
        let (action, confidence, reasoning) = self.determine_optimal_action(
            symbol,
            data,
            momentum,
            volatility,
            expected_return,
            current_position,
        );

        // Calculate position size using Kelly Criterion
        let position_size = self.calculate_optimal_position_size(
            expected_return,
            volatility,
            account_data,
            &action,
        );

        Ok(TradingDecision {
            symbol: symbol.to_string(),
            action: action.clone(),
            position_size,
            expected_return,
            confidence_score: confidence,
            reasoning,
            stop_loss: self.calculate_stop_loss(data.price, volatility, &action),
            take_profit: self.calculate_take_profit(data.price, expected_return, &action),
            market_regime: self.market_regime.clone(),
            volatility_regime: self.volatility_regime.clone(),
        })
    }

    /// Calculate enhanced expected return using multiple factors
    fn calculate_enhanced_expected_return(
        &self,
        _data: &MarketDataPoint,
        momentum: f64,
        volatility: f64,
        volume_ratio: f64,
    ) -> f64 {
        // Base expected return from CAPM
        let base_return = self.risk_free_rate + 1.0 * (0.10 - self.risk_free_rate);
        
        // Momentum factor
        let momentum_factor = momentum * 0.3;
        
        // Volatility factor (higher volatility = higher risk premium)
        let volatility_factor = volatility * 0.2;
        
        // Volume factor (higher volume = more conviction)
        let volume_factor = (volume_ratio - 1.0).max(-0.5).min(0.5) * 0.1;
        
        // Regime factor
        let regime_factor = match self.market_regime.as_str() {
            "bull_market" => 0.05,
            "bear_market" => -0.05,
            "high_volatility" => 0.02,
            "low_volatility" => -0.02,
            _ => 0.0,
        };

        base_return + momentum_factor + volatility_factor + volume_factor + regime_factor
    }

    /// Determine optimal action based on comprehensive analysis
    fn determine_optimal_action(
        &self,
        _symbol: &str,
        _data: &MarketDataPoint,
        momentum: f64,
        volatility: f64,
        expected_return: f64,
        current_position: Option<&Position>,
    ) -> (TradingAction, f64, String) {
        // If we have a current position, check for exit conditions
        if let Some(position) = current_position {
            let qty = position.qty.parse::<f64>().unwrap_or(0.0);
            let market_value = position.market_value.parse::<f64>().unwrap_or(0.0);
            let avg_cost = position.avg_entry_price.parse::<f64>().unwrap_or(0.0);
            
            if qty > 0.0 {
                let current_price = market_value / qty;
                let profit_percentage = (current_price - avg_cost) / avg_cost;
                
                // Take profit at 5%
                if profit_percentage >= 0.05 {
                    return (
                        TradingAction::CloseLong,
                        0.9,
                        format!("Take profit: {:.2}% gain", profit_percentage * 100.0)
                    );
                }
                
                // Stop loss at -10%
                if profit_percentage <= -0.10 {
                    return (
                        TradingAction::CloseLong,
                        0.9,
                        format!("Stop loss: {:.2}% loss", profit_percentage * 100.0)
                    );
                }
            } else if qty < 0.0 {
                // Short position
                let current_price = market_value / qty.abs();
                let profit_percentage = (avg_cost - current_price) / avg_cost;
                
                // Take profit on short at 5%
                if profit_percentage >= 0.05 {
                    return (
                        TradingAction::CloseShort,
                        0.9,
                        format!("Take profit on short: {:.2}% gain", profit_percentage * 100.0)
                    );
                }
                
                // Stop loss on short at -10%
                if profit_percentage <= -0.10 {
                    return (
                        TradingAction::CloseShort,
                        0.9,
                        format!("Stop loss on short: {:.2}% loss", profit_percentage * 100.0)
                    );
                }
            }
        }

        // No current position - determine new position
        let (action, confidence, reasoning) = match self.market_regime.as_str() {
            "bull_market" => {
                if expected_return > 0.05 && momentum > 0.01 {
                    (TradingAction::OpenLong, 0.8, "Bull market momentum play".to_string())
                } else if expected_return < -0.05 && momentum < -0.01 {
                    (TradingAction::OpenShort, 0.6, "Bearish divergence in bull market".to_string())
                } else {
                    (TradingAction::Hold, 0.3, "No clear signal in bull market".to_string())
                }
            },
            "bear_market" => {
                if expected_return < -0.05 && momentum < -0.01 {
                    (TradingAction::OpenShort, 0.8, "Bear market continuation".to_string())
                } else if expected_return > 0.05 && momentum > 0.01 {
                    (TradingAction::OpenLong, 0.6, "Bullish reversal in bear market".to_string())
                } else {
                    (TradingAction::Hold, 0.3, "No clear signal in bear market".to_string())
                }
            },
            "high_volatility" => {
                if volatility > 0.05 {
                    (TradingAction::Hold, 0.2, "Too volatile for new positions".to_string())
                } else if expected_return.abs() > 0.08 {
                    if expected_return > 0.0 {
                        (TradingAction::OpenLong, 0.7, "High conviction long in volatile market".to_string())
                    } else {
                        (TradingAction::OpenShort, 0.7, "High conviction short in volatile market".to_string())
                    }
                } else {
                    (TradingAction::Hold, 0.4, "Insufficient conviction in volatile market".to_string())
                }
            },
            "low_volatility" => {
                if expected_return > 0.03 {
                    (TradingAction::OpenLong, 0.6, "Accumulation in low volatility".to_string())
                } else if expected_return < -0.03 {
                    (TradingAction::OpenShort, 0.6, "Distribution in low volatility".to_string())
                } else {
                    (TradingAction::Hold, 0.5, "No clear edge in low volatility".to_string())
                }
            },
            "sideways_market" => {
                // Mean reversion strategy
                if momentum > self.mean_reversion_threshold {
                    (TradingAction::OpenShort, 0.7, "Mean reversion short".to_string())
                } else if momentum < -self.mean_reversion_threshold {
                    (TradingAction::OpenLong, 0.7, "Mean reversion long".to_string())
                } else {
                    (TradingAction::Hold, 0.4, "No mean reversion signal".to_string())
                }
            },
            _ => {
                // Default strategy based on expected return
                if expected_return > 0.05 {
                    (TradingAction::OpenLong, 0.6, "Positive expected return".to_string())
                } else if expected_return < -0.05 {
                    (TradingAction::OpenShort, 0.6, "Negative expected return".to_string())
                } else {
                    (TradingAction::Hold, 0.3, "Insufficient expected return".to_string())
                }
            }
        };

        (action, confidence, reasoning)
    }

    /// Calculate optimal position size using Kelly Criterion
    fn calculate_optimal_position_size(
        &self,
        expected_return: f64,
        volatility: f64,
        account_data: &AccountData,
        action: &TradingAction,
    ) -> f64 {
        // Kelly Criterion: f = (bp - q) / b
        // where b = odds received, p = probability of win, q = probability of loss
        
        let win_probability = if expected_return > 0.0 {
            (expected_return / volatility).min(0.8).max(0.5)
        } else {
            ((-expected_return) / volatility).min(0.8).max(0.5)
        };
        
        let loss_probability = 1.0 - win_probability;
        let odds_received = 1.0; // 1:1 odds
        
        let kelly_fraction = if odds_received > 0.0 {
            (odds_received * win_probability - loss_probability) / odds_received
        } else {
            0.0
        };
        
        // Cap Kelly fraction at 25% of portfolio
        let max_kelly = 0.25;
        let kelly_fraction = kelly_fraction.max(0.0).min(max_kelly);
        
        // Calculate position size
        let position_value = account_data.portfolio_value * kelly_fraction;
        
        // Adjust for action type
        match action {
            TradingAction::OpenLong | TradingAction::CloseShort => position_value,
            TradingAction::OpenShort | TradingAction::CloseLong => -position_value,
            _ => 0.0,
        }
    }

    /// Calculate stop loss level
    fn calculate_stop_loss(&self, current_price: f64, volatility: f64, action: &TradingAction) -> f64 {
        let stop_distance = current_price * volatility * 2.0; // 2x volatility
        
        match action {
            TradingAction::OpenLong | TradingAction::CloseShort => current_price - stop_distance,
            TradingAction::OpenShort | TradingAction::CloseLong => current_price + stop_distance,
            _ => current_price,
        }
    }

    /// Calculate take profit level
    fn calculate_take_profit(&self, current_price: f64, expected_return: f64, action: &TradingAction) -> f64 {
        let profit_distance = current_price * expected_return.abs() * 2.0; // 2x expected return
        
        match action {
            TradingAction::OpenLong | TradingAction::CloseShort => current_price + profit_distance,
            TradingAction::OpenShort | TradingAction::CloseLong => current_price - profit_distance,
            _ => current_price,
        }
    }
}

/// Trading action types
#[derive(Debug, Clone, serde::Serialize)]
pub enum TradingAction {
    OpenLong,    // Buy to open long position
    OpenShort,   // Sell to open short position
    CloseLong,   // Sell to close long position
    CloseShort,  // Buy to close short position
    Hold,        // No action
}

impl std::fmt::Display for TradingAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradingAction::OpenLong => write!(f, "OPEN_LONG"),
            TradingAction::OpenShort => write!(f, "OPEN_SHORT"),
            TradingAction::CloseLong => write!(f, "CLOSE_LONG"),
            TradingAction::CloseShort => write!(f, "CLOSE_SHORT"),
            TradingAction::Hold => write!(f, "HOLD"),
        }
    }
}

/// Enhanced trading decision structure
#[derive(Debug, Clone)]
pub struct TradingDecision {
    pub symbol: String,
    pub action: TradingAction,
    pub position_size: f64,
    pub expected_return: f64,
    pub confidence_score: f64,
    pub reasoning: String,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub market_regime: String,
    pub volatility_regime: String,
}
