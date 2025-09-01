use crate::order_execution::order_types::*;
use crate::market_data::{AssetUniverseManager, Position};
use anyhow::Result;

pub struct LiquidationManager {
    pub profit_target_percentage: f64,
    pub stop_loss_percentage: f64,
    pub starting_portfolio_value: f64,
}

impl LiquidationManager {
    pub fn new(profit_target_percentage: f64, stop_loss_percentage: f64, starting_portfolio_value: f64) -> Self {
        Self {
            profit_target_percentage,
            stop_loss_percentage,
            starting_portfolio_value,
        }
    }

    /// Analyze current positions and determine if liquidation is needed
    pub async fn analyze_liquidation_needs(
        &self,
        data_dir: &str,
        current_portfolio_value: f64,
    ) -> Result<Vec<LiquidationTrigger>> {
        let mut triggers = Vec::new();

        // Load current positions
        let positions = AssetUniverseManager::load_positions(data_dir).await?;
        
        // Check portfolio-level stop loss
        if current_portfolio_value < self.starting_portfolio_value {
            let loss_percentage = ((self.starting_portfolio_value - current_portfolio_value) / self.starting_portfolio_value) * 100.0;
            if loss_percentage >= self.stop_loss_percentage {
                // Portfolio stop loss triggered - liquidate all positions
                for position in &positions {
                    if let Ok(qty) = position.qty.parse::<f64>() {
                        if qty > 0.0 {
                            triggers.push(LiquidationTrigger {
                                symbol: position.symbol.clone(),
                                trigger_type: LiquidationType::StopLoss,
                                current_price: 0.0, // Will be filled by order executor
                                target_price: 0.0,
                                profit_percentage: -loss_percentage,
                                reason: format!("Portfolio stop loss triggered: {:.2}% loss", loss_percentage),
                            });
                        }
                    }
                }
            }
        }

        // Check individual position profit targets
        for position in &positions {
            if let Ok(qty) = position.qty.parse::<f64>() {
                if qty > 0.0 {
                    if let Ok(market_value) = position.market_value.parse::<f64>() {
                        if let Ok(avg_cost) = position.avg_entry_price.parse::<f64>() {
                            let current_price = market_value / qty;
                            let profit_percentage = ((current_price - avg_cost) / avg_cost) * 100.0;
                            
                            // Check if profit target is reached
                            if profit_percentage >= self.profit_target_percentage {
                                triggers.push(LiquidationTrigger {
                                    symbol: position.symbol.clone(),
                                    trigger_type: LiquidationType::ProfitTarget,
                                    current_price,
                                    target_price: current_price,
                                    profit_percentage,
                                    reason: format!("Profit target reached: {:.2}% profit", profit_percentage),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(triggers)
    }

    /// Convert positions to liquidation analysis format
    pub fn convert_positions_for_analysis(&self, positions: &[Position]) -> Vec<PositionForLiquidation> {
        let mut analysis_positions = Vec::new();

        for position in positions {
            if let Ok(qty) = position.qty.parse::<f64>() {
                if qty > 0.0 {
                    if let Ok(market_value) = position.market_value.parse::<f64>() {
                        if let Ok(avg_cost) = position.avg_entry_price.parse::<f64>() {
                            let current_price = market_value / qty;
                            let unrealized_pl = market_value - (avg_cost * qty);
                            let unrealized_plpc = (unrealized_pl / (avg_cost * qty)) * 100.0;

                            analysis_positions.push(PositionForLiquidation {
                                symbol: position.symbol.clone(),
                                quantity: qty,
                                current_price,
                                average_cost: avg_cost,
                                market_value,
                                unrealized_pl,
                                unrealized_plpc,
                            });
                        }
                    }
                }
            }
        }

        analysis_positions
    }

    /// Check if any position meets liquidation criteria
    pub fn should_liquidate_position(&self, position: &PositionForLiquidation) -> Option<LiquidationTrigger> {
        // Check profit target
        if position.unrealized_plpc >= self.profit_target_percentage {
            return Some(LiquidationTrigger {
                symbol: position.symbol.clone(),
                trigger_type: LiquidationType::ProfitTarget,
                current_price: position.current_price,
                target_price: position.current_price,
                profit_percentage: position.unrealized_plpc,
                reason: format!("Profit target reached: {:.2}% profit", position.unrealized_plpc),
            });
        }

        // Check stop loss (if position is losing money)
        if position.unrealized_plpc < -self.stop_loss_percentage {
            return Some(LiquidationTrigger {
                symbol: position.symbol.clone(),
                trigger_type: LiquidationType::StopLoss,
                current_price: position.current_price,
                target_price: position.current_price,
                profit_percentage: position.unrealized_plpc,
                reason: format!("Stop loss triggered: {:.2}% loss", position.unrealized_plpc),
            });
        }

        None
    }

    /// Get liquidation summary for logging
    pub fn get_liquidation_summary(&self, triggers: &[LiquidationTrigger]) -> String {
        if triggers.is_empty() {
            return "No liquidation triggers found".to_string();
        }

        let mut summary = format!("Found {} liquidation trigger(s):\n", triggers.len());
        
        for trigger in triggers {
            summary.push_str(&format!(
                "  - {}: {} ({:.2}% {}) - {}\n",
                trigger.symbol,
                match trigger.trigger_type {
                    LiquidationType::ProfitTarget => "PROFIT TARGET",
                    LiquidationType::StopLoss => "STOP LOSS",
                    LiquidationType::RiskManagement => "RISK MANAGEMENT",
                    LiquidationType::StrategySignal => "STRATEGY SIGNAL",
                },
                trigger.profit_percentage.abs(),
                if trigger.profit_percentage >= 0.0 { "profit" } else { "loss" },
                trigger.reason
            ));
        }

        summary
    }
}
