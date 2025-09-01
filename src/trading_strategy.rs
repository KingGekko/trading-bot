use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::market_data::{Asset, Position};

pub mod enhanced_decision_engine;
pub mod ai_decision_engine;
pub use enhanced_decision_engine::{EnhancedDecisionEngine, TradingAction};
pub use ai_decision_engine::AIDecisionEngine;

/// Advanced Trading Strategy with Modern Portfolio Theory, Kelly Criterion, and CAPM
#[derive(Debug, Clone)]
pub struct AdvancedTradingStrategy {
    pub risk_free_rate: f64,           // Risk-free rate (e.g., Treasury yield)
    pub target_return: f64,            // Target portfolio return
    pub max_volatility: f64,           // Maximum portfolio volatility
    pub volatility_lookback: usize,    // Days for volatility calculation
    pub correlation_threshold: f64,    // Maximum correlation between positions
    pub sharpe_ratio_threshold: f64,   // Minimum Sharpe ratio for trades
    pub max_drawdown_limit: f64,       // Maximum allowed drawdown
    pub portfolio_protection: f64,     // Stop loss at starting portfolio value
    pub profit_target: f64,            // Take profit target (5% = 0.05)
    pub options_enabled: bool,         // Enable options trading
    pub max_options_allocation: f64,   // Maximum allocation to options (0.3 = 30%)
}

/// Market data point for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataPoint {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub timestamp: DateTime<Utc>,
}

/// Account data for strategy calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {
    pub cash: f64,
    pub equity: f64,
    pub buying_power: f64,
    pub portfolio_value: f64,
    pub daytrade_count: i32,
    pub pattern_day_trader: bool,
    pub shorting_enabled: bool,
    pub margin_multiplier: f64,
    pub starting_portfolio_value: f64, // Starting value for portfolio protection
}

/// Portfolio allocation recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAllocation {
    pub symbol: String,
    pub allocation_percentage: f64,
    pub expected_return: f64,
    pub volatility: f64,
    pub sharpe_ratio: f64,
    pub position_size: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub asset_type: AssetType,
    pub options_data: Option<OptionsData>,
}

/// Asset type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssetType {
    Stock,
    Option,
    ETF,
    Crypto,
    Bond,
}

/// Options-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsData {
    pub strike_price: f64,
    pub expiration_date: String,
    pub option_type: OptionType, // Call or Put
    pub implied_volatility: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub underlying_symbol: String,
}

/// Option type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptionType {
    Call,
    Put,
}

/// Risk metrics for portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub portfolio_volatility: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown: f64,
    pub var_95: f64,
    pub expected_shortfall: f64,
    pub beta_to_spy: HashMap<String, f64>,
    pub correlation_matrix: HashMap<String, HashMap<String, f64>>,
}

/// Enhanced strategy data including positions and asset universe
#[derive(Debug, Clone)]
pub struct EnhancedStrategyData {
    pub market_data: HashMap<String, MarketDataPoint>,
    pub account_data: AccountData,
    pub historical_data: Vec<MarketDataPoint>,
    pub current_positions: Vec<Position>,
    pub asset_universe: Vec<Asset>,
    pub portfolio_history: Option<Value>,
}

/// Portfolio protection and profit target management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioManager {
    pub starting_value: f64,
    pub current_value: f64,
    pub stop_loss_triggered: bool,
    pub profit_targets: HashMap<String, ProfitTarget>,
    pub protection_level: f64,
    pub profit_target_percentage: f64,
}

/// Profit target for individual positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitTarget {
    pub symbol: String,
    pub entry_price: f64,
    pub target_price: f64,
    pub current_price: f64,
    pub profit_percentage: f64,
    pub target_hit: bool,
    pub position_size: f64,
}

impl AdvancedTradingStrategy {
    /// Create a new advanced trading strategy
    pub fn new() -> Self {
        Self {
            risk_free_rate: 0.04,      // 4% risk-free rate
            target_return: 0.12,       // 12% target return
            max_volatility: 0.20,      // 20% max volatility
            volatility_lookback: 30,   // 30 days lookback
            correlation_threshold: 0.7, // 70% correlation threshold
            sharpe_ratio_threshold: 1.0, // 1.0 Sharpe ratio minimum
            max_drawdown_limit: 0.15,  // 15% max drawdown
            portfolio_protection: 1.0, // 100% of starting value
            profit_target: 0.05,       // 5% profit target
            options_enabled: true,     // Enable options trading
            max_options_allocation: 0.3, // 30% max options allocation
        }
    }

    /// Create strategy with custom parameters
    pub fn with_parameters(
        risk_free_rate: f64,
        target_return: f64,
        max_volatility: f64,
        portfolio_protection: f64,
        profit_target: f64,
        options_enabled: bool,
    ) -> Self {
        Self {
            risk_free_rate,
            target_return,
            max_volatility,
            volatility_lookback: 30,
            correlation_threshold: 0.7,
            sharpe_ratio_threshold: 1.0,
            max_drawdown_limit: 0.15,
            portfolio_protection,
            profit_target,
            options_enabled,
            max_options_allocation: 0.3,
        }
    }

    /// Calculate optimal portfolio allocation with portfolio protection
    pub fn calculate_optimal_allocation(
        &self,
        market_data: &HashMap<String, MarketDataPoint>,
        account_data: &AccountData,
        historical_data: &[MarketDataPoint],
    ) -> Result<Vec<PortfolioAllocation>> {
        let mut allocations = Vec::new();
        
        // Calculate expected returns and volatilities
        let mut expected_returns = HashMap::new();
        let mut volatilities = HashMap::new();
        
        for (symbol, data) in market_data {
            let expected_return = self.calculate_expected_return(data, historical_data)?;
            let volatility = self.calculate_volatility(symbol, historical_data)?;
            
            expected_returns.insert(symbol.clone(), expected_return);
            volatilities.insert(symbol.clone(), volatility);
        }
        
        // Apply Modern Portfolio Theory
        let optimal_weights = self.apply_modern_portfolio_theory(
            &expected_returns,
            &volatilities,
            market_data,
        )?;
        
        // Apply Kelly Criterion for position sizing
        let kelly_weights = self.apply_kelly_criterion(
            &expected_returns,
            &volatilities,
            &optimal_weights,
        )?;
        
        // Apply portfolio protection
        let protected_weights = self.apply_portfolio_protection(
            &kelly_weights,
            account_data,
            &expected_returns,
        )?;
        
        // Generate allocations with stop losses and profit targets
        for (symbol, weight) in protected_weights {
            if let Some(data) = market_data.get(&symbol) {
                let allocation = self.create_allocation(
                    &symbol,
                    weight,
                    &expected_returns,
                    &volatilities,
                    data,
                    account_data,
                )?;
                allocations.push(allocation);
            }
        }
        
        // Sort by Sharpe ratio
        allocations.sort_by(|a, b| b.sharpe_ratio.partial_cmp(&a.sharpe_ratio).unwrap());
        
        Ok(allocations)
    }

    /// Apply portfolio protection to prevent going below starting value
    fn apply_portfolio_protection(
        &self,
        weights: &HashMap<String, f64>,
        account_data: &AccountData,
        expected_returns: &HashMap<String, f64>,
    ) -> Result<HashMap<String, f64>> {
        let mut protected_weights = weights.clone();
        let starting_value = account_data.starting_portfolio_value;
        let current_value = account_data.portfolio_value;
        
        // Calculate portfolio protection level
        let protection_level = starting_value * self.portfolio_protection;
        
        // If current value is below protection level, reduce risk
        if current_value < protection_level {
            let risk_reduction_factor = (current_value / protection_level).min(1.0);
            
            // Reduce position sizes to protect capital
            for (symbol, weight) in protected_weights.iter_mut() {
                if let Some(expected_return) = expected_returns.get(symbol) {
                    // Reduce allocation for positions with negative expected returns
                    if *expected_return < 0.0 {
                        *weight *= risk_reduction_factor * 0.5; // Additional reduction for losing positions
                    } else {
                        *weight *= risk_reduction_factor;
                    }
                }
            }
            
            println!("🛡️ Portfolio protection activated: Reducing risk by {:.1}%", 
                (1.0 - risk_reduction_factor) * 100.0);
        }
        
        Ok(protected_weights)
    }

    /// Create allocation with stop loss and profit target
    fn create_allocation(
        &self,
        symbol: &str,
        weight: f64,
        expected_returns: &HashMap<String, f64>,
        volatilities: &HashMap<String, f64>,
        data: &MarketDataPoint,
        account_data: &AccountData,
    ) -> Result<PortfolioAllocation> {
        let current_price = data.price;
        let expected_return = expected_returns.get(symbol).unwrap_or(&0.0);
        let volatility = volatilities.get(symbol).unwrap_or(&0.0);
        
        // Calculate Sharpe ratio
        let sharpe_ratio = if *volatility > 0.0 {
            (*expected_return - self.risk_free_rate) / volatility
        } else {
            0.0
        };
        
        // Calculate position size
        let position_size = weight * account_data.portfolio_value;
        
        // Calculate stop loss (2x volatility below current price)
        let stop_loss = current_price * (1.0 - (2.0 * volatility));
        
        // Calculate take profit (5% above current price)
        let take_profit = current_price * (1.0 + self.profit_target);
        
        // Determine asset type
        let asset_type = self.determine_asset_type(symbol);
        
        // Create options data if applicable
        let options_data = if asset_type == AssetType::Option {
            Some(self.create_options_data(symbol, current_price, volatility))
        } else {
            None
        };
        
        Ok(PortfolioAllocation {
            symbol: symbol.to_string(),
            allocation_percentage: weight * 100.0,
            expected_return: *expected_return,
            volatility: *volatility,
            sharpe_ratio,
            position_size,
            stop_loss,
            take_profit,
            asset_type,
            options_data,
        })
    }

    /// Determine asset type based on symbol
    fn determine_asset_type(&self, symbol: &str) -> AssetType {
        if symbol.contains("$") || symbol.contains("CALL") || symbol.contains("PUT") {
            AssetType::Option
        } else if symbol.ends_with("X") || symbol.contains("ETF") {
            AssetType::ETF
        } else if symbol.contains("USD") || symbol.contains("BTC") || symbol.contains("ETH") {
            AssetType::Crypto
        } else if symbol.contains("BOND") || symbol.contains("T") {
            AssetType::Bond
        } else {
            AssetType::Stock
        }
    }

    /// Create options data for options trading
    fn create_options_data(
        &self,
        symbol: &str,
        _current_price: f64,
        volatility: &f64,
    ) -> OptionsData {
        // Parse options symbol to extract strike and expiration
        let (strike_price, expiration_date, option_type, underlying_symbol) = 
            self.parse_options_symbol(symbol);
        
        // Calculate Greeks (simplified)
        let delta = if option_type == OptionType::Call { 0.6 } else { -0.4 };
        let gamma = 0.02;
        let theta = -0.01;
        let vega = 0.15;
        
        OptionsData {
            strike_price,
            expiration_date,
            option_type,
            implied_volatility: *volatility,
            delta,
            gamma,
            theta,
            vega,
            underlying_symbol,
        }
    }

    /// Parse options symbol to extract components
    fn parse_options_symbol(&self, symbol: &str) -> (f64, String, OptionType, String) {
        // Simplified parsing - in practice would use more sophisticated parsing
        if symbol.contains("CALL") {
            let parts: Vec<&str> = symbol.split("CALL").collect();
            let underlying = parts[0].to_string();
            let strike_str = parts[1].replace("$", "");
            let strike = strike_str.parse().unwrap_or(100.0);
            let expiration = "2024-12-20".to_string(); // Default expiration
            
            (strike, expiration, OptionType::Call, underlying)
        } else if symbol.contains("PUT") {
            let parts: Vec<&str> = symbol.split("PUT").collect();
            let underlying = parts[0].to_string();
            let strike_str = parts[1].replace("$", "");
            let strike = strike_str.parse().unwrap_or(100.0);
            let expiration = "2024-12-20".to_string(); // Default expiration
            
            (strike, expiration, OptionType::Put, underlying)
        } else {
            // Default values for unknown format
            (100.0, "2024-12-20".to_string(), OptionType::Call, symbol.to_string())
        }
    }

    /// Calculate expected return using CAPM
    fn calculate_expected_return(
        &self,
        data: &MarketDataPoint,
        _historical_data: &[MarketDataPoint],
    ) -> Result<f64> {
        // Simplified CAPM calculation
        let market_return = 0.10; // 10% market return
        let beta = 1.0; // Default beta
        
        let expected_return = self.risk_free_rate + beta * (market_return - self.risk_free_rate);
        
        // Add momentum factor
        let momentum = (data.price - data.open) / data.open;
        let adjusted_return = expected_return + (momentum * 0.1);
        
        Ok(adjusted_return)
    }

    /// Calculate volatility
    fn calculate_volatility(
        &self,
        symbol: &str,
        historical_data: &[MarketDataPoint],
    ) -> Result<f64> {
        let symbol_data: Vec<&MarketDataPoint> = historical_data
            .iter()
            .filter(|d| d.symbol == symbol)
            .collect();
        
        if symbol_data.len() < 2 {
            return Ok(0.02); // Default 2% volatility
        }
        
        let returns: Vec<f64> = symbol_data
            .windows(2)
            .map(|window| {
                let prev_price = window[0].price;
                let curr_price = window[1].price;
                (curr_price - prev_price) / prev_price
            })
            .collect();
        
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        
        let volatility = variance.sqrt();
        Ok(volatility.max(0.01)) // Minimum 1% volatility
    }

    /// Apply Modern Portfolio Theory
    fn apply_modern_portfolio_theory(
        &self,
        expected_returns: &HashMap<String, f64>,
        volatilities: &HashMap<String, f64>,
        market_data: &HashMap<String, MarketDataPoint>,
    ) -> Result<HashMap<String, f64>> {
        let mut weights = HashMap::new();
        let total_assets = expected_returns.len() as f64;
        
        // Equal weight allocation as starting point
        for symbol in expected_returns.keys() {
            weights.insert(symbol.clone(), 1.0 / total_assets);
        }
        
        // Adjust weights based on Sharpe ratios
        let mut total_weight = 0.0;
        for (symbol, expected_return) in expected_returns {
            if let Some(volatility) = volatilities.get(symbol) {
                let sharpe_ratio = if *volatility > 0.0 {
                    (*expected_return - self.risk_free_rate) / volatility
                } else {
                    0.0
                };
                
                // Only include assets with positive Sharpe ratios
                if sharpe_ratio > self.sharpe_ratio_threshold {
                    let weight = sharpe_ratio.max(0.0);
                    weights.insert(symbol.clone(), weight);
                    total_weight += weight;
                } else {
                    weights.insert(symbol.clone(), 0.0);
                }
            }
        }
        
        // Normalize weights
        if total_weight > 0.0 {
            for weight in weights.values_mut() {
                *weight /= total_weight;
            }
        }
        
        Ok(weights)
    }

    /// Apply Kelly Criterion for position sizing
    fn apply_kelly_criterion(
        &self,
        expected_returns: &HashMap<String, f64>,
        volatilities: &HashMap<String, f64>,
        weights: &HashMap<String, f64>,
    ) -> Result<HashMap<String, f64>> {
        let mut kelly_weights = HashMap::new();
        
        for (symbol, weight) in weights {
            if let (Some(expected_return), Some(_volatility)) = 
                (expected_returns.get(symbol), volatilities.get(symbol)) {
                
                // Kelly formula: f = (bp - q) / b
                // where b = odds received, p = probability of win, q = probability of loss
                let win_probability = 0.5 + (*expected_return * 0.5); // Simplified
                let loss_probability = 1.0 - win_probability;
                let odds_received = 1.0; // 1:1 odds
                
                let kelly_fraction = if odds_received > 0.0 {
                    (odds_received * win_probability - loss_probability) / odds_received
                } else {
                    0.0
                };
                
                // Apply Kelly fraction to original weight
                let adjusted_weight = weight * kelly_fraction.max(0.0).min(0.25); // Cap at 25%
                kelly_weights.insert(symbol.clone(), adjusted_weight);
            }
        }
        
        Ok(kelly_weights)
    }

    /// Calculate risk metrics for portfolio
    pub fn calculate_risk_metrics(
        &self,
        allocations: &[PortfolioAllocation],
        _historical_data: &[MarketDataPoint],
    ) -> Result<RiskMetrics> {
        let mut portfolio_volatility = 0.0;
        let mut portfolio_return = 0.0;
        let mut total_allocation = 0.0;
        
        // Calculate portfolio return and volatility
        for allocation in allocations {
            portfolio_return += allocation.expected_return * (allocation.allocation_percentage / 100.0);
            portfolio_volatility += allocation.volatility * (allocation.allocation_percentage / 100.0);
            total_allocation += allocation.allocation_percentage / 100.0;
        }
        
        // Normalize
        if total_allocation > 0.0 {
            portfolio_return /= total_allocation;
            portfolio_volatility /= total_allocation;
        }
        
        // Calculate Sharpe ratio
        let sharpe_ratio = if portfolio_volatility > 0.0 {
            (portfolio_return - self.risk_free_rate) / portfolio_volatility
        } else {
            0.0
        };
        
        // Calculate Sortino ratio (simplified)
        let sortino_ratio = if portfolio_volatility > 0.0 {
            (portfolio_return - self.risk_free_rate) / (portfolio_volatility * 0.8)
        } else {
            0.0
        };
        
        // Calculate Value at Risk (95%)
        let var_95 = portfolio_volatility * 1.645;
        
        // Calculate Expected Shortfall
        let expected_shortfall = portfolio_volatility * 2.06;
        
        // Calculate max drawdown (simplified)
        let max_drawdown = portfolio_volatility * 2.0;
        
        // Calculate beta to SPY
        let mut beta_to_spy = HashMap::new();
        for allocation in allocations {
            beta_to_spy.insert(allocation.symbol.clone(), 1.0); // Simplified beta
        }
        
        // Calculate correlation matrix (simplified)
        let mut correlation_matrix = HashMap::new();
        for allocation in allocations {
            let mut correlations = HashMap::new();
            for other_allocation in allocations {
                correlations.insert(other_allocation.symbol.clone(), 0.5); // Simplified correlation
            }
            correlation_matrix.insert(allocation.symbol.clone(), correlations);
        }
        
        Ok(RiskMetrics {
            portfolio_volatility,
            sharpe_ratio,
            sortino_ratio,
            max_drawdown,
            var_95,
            expected_shortfall,
            beta_to_spy,
            correlation_matrix,
        })
    }

    /// Generate enhanced trading recommendations with portfolio protection
    pub fn generate_enhanced_recommendations(
        &self,
        enhanced_data: &EnhancedStrategyData,
    ) -> Result<Value> {
        // Calculate optimal allocations
        let allocations = self.calculate_optimal_allocation(
            &enhanced_data.market_data,
            &enhanced_data.account_data,
            &enhanced_data.historical_data,
        )?;
        
        // Calculate risk metrics
        let risk_metrics = self.calculate_risk_metrics(&allocations, &enhanced_data.historical_data)?;
        
        // Analyze current positions
        let position_analysis = self.analyze_current_positions(&enhanced_data.current_positions)?;
        
        // Generate recommendations with portfolio protection
        let recommendations = self.generate_protected_recommendations(
            &allocations,
            &risk_metrics,
            &enhanced_data.account_data,
            &enhanced_data.current_positions,
        )?;
        
        // Create portfolio summary
        let portfolio_summary = self.create_portfolio_summary(
            &allocations,
            &risk_metrics,
            &enhanced_data.account_data,
            &enhanced_data.asset_universe,
        )?;
        
        // Create asset universe summary
        let asset_universe_summary = self.create_asset_universe_summary(&enhanced_data.asset_universe)?;
        
        let result = json!({
            "strategy": "Enhanced Mathematical Portfolio Optimization with Portfolio Protection",
            "timestamp": Utc::now().to_rfc3339(),
            "account_info": {
                "cash": enhanced_data.account_data.cash,
                "equity": enhanced_data.account_data.equity,
                "buying_power": enhanced_data.account_data.buying_power,
                "daytrade_count": enhanced_data.account_data.daytrade_count,
                "starting_portfolio_value": enhanced_data.account_data.starting_portfolio_value,
                "current_portfolio_value": enhanced_data.account_data.portfolio_value,
                "portfolio_protection_level": enhanced_data.account_data.starting_portfolio_value * self.portfolio_protection,
                "profit_target_percentage": self.profit_target * 100.0,
            },
            "portfolio_protection": {
                "enabled": true,
                "protection_level": enhanced_data.account_data.starting_portfolio_value * self.portfolio_protection,
                "current_value": enhanced_data.account_data.portfolio_value,
                "protection_triggered": enhanced_data.account_data.portfolio_value < (enhanced_data.account_data.starting_portfolio_value * self.portfolio_protection),
                "risk_reduction_factor": if enhanced_data.account_data.portfolio_value < (enhanced_data.account_data.starting_portfolio_value * self.portfolio_protection) {
                    (enhanced_data.account_data.portfolio_value / (enhanced_data.account_data.starting_portfolio_value * self.portfolio_protection)).min(1.0)
                } else {
                    1.0
                }
            },
            "profit_targets": {
                "enabled": true,
                "target_percentage": self.profit_target * 100.0,
                "take_profit_levels": allocations.iter().map(|a| {
                    json!({
                        "symbol": a.symbol,
                        "entry_price": a.take_profit / (1.0 + self.profit_target),
                        "target_price": a.take_profit,
                        "profit_percentage": self.profit_target * 100.0
                    })
                }).collect::<Vec<_>>()
            },
            "options_strategy": {
                "enabled": self.options_enabled,
                "max_allocation": self.max_options_allocation * 100.0,
                "options_positions": allocations.iter().filter(|a| a.asset_type == AssetType::Option).map(|a| {
                    json!({
                        "symbol": a.symbol,
                        "allocation": a.allocation_percentage,
                        "options_data": a.options_data
                    })
                }).collect::<Vec<_>>()
            },
            "asset_universe_summary": asset_universe_summary,
            "position_analysis": position_analysis,
            "portfolio_summary": portfolio_summary,
            "recommendations": recommendations,
            "risk_metrics": {
                "portfolio_volatility": risk_metrics.portfolio_volatility,
                "portfolio_sharpe_ratio": risk_metrics.sharpe_ratio,
                "portfolio_sortino_ratio": risk_metrics.sortino_ratio,
                "max_drawdown": risk_metrics.max_drawdown,
                "value_at_risk_95": risk_metrics.var_95,
                "expected_shortfall": risk_metrics.expected_shortfall,
                "risk_level": if risk_metrics.var_95 > 0.15 { "HIGH" } else if risk_metrics.var_95 > 0.10 { "MEDIUM" } else { "LOW" },
                "current_positions_count": enhanced_data.current_positions.len(),
                "asset_universe_size": enhanced_data.asset_universe.len(),
            }
        });
        
        Ok(result)
    }

    /// Generate protected recommendations with stop losses and profit targets
    fn generate_protected_recommendations(
        &self,
        allocations: &[PortfolioAllocation],
        risk_metrics: &RiskMetrics,
        account_data: &AccountData,
        current_positions: &[Position],
    ) -> Result<Vec<Value>> {
        let mut recommendations = Vec::new();
        
        for allocation in allocations {
            let current_position = current_positions.iter().find(|p| p.symbol == allocation.symbol);
            
            let recommendation = if let Some(position) = current_position {
                // Existing position - check for profit target or stop loss
                let current_value = position.market_value.parse::<f64>().unwrap_or(0.0);
                let qty = position.qty.parse::<f64>().unwrap_or(0.0);
                let entry_value = qty * (allocation.take_profit / (1.0 + self.profit_target));
                let profit_percentage = if entry_value > 0.0 { (current_value - entry_value) / entry_value } else { 0.0 };
                
                if profit_percentage >= self.profit_target {
                    // Take profit
                    json!({
                        "symbol": allocation.symbol,
                        "action": "SELL",
                        "reason": "Profit target reached",
                        "profit_percentage": profit_percentage * 100.0,
                        "current_price": allocation.take_profit / (1.0 + self.profit_target),
                        "target_price": allocation.take_profit,
                        "quantity": position.qty,
                        "asset_type": allocation.asset_type,
                        "options_data": allocation.options_data
                    })
                } else if current_value <= entry_value * (1.0 - (2.0 * allocation.volatility)) {
                    // Stop loss
                    json!({
                        "symbol": allocation.symbol,
                        "action": "SELL",
                        "reason": "Stop loss triggered",
                        "loss_percentage": profit_percentage * 100.0,
                        "current_price": allocation.take_profit / (1.0 + self.profit_target),
                        "stop_loss": allocation.stop_loss,
                        "quantity": position.qty,
                        "asset_type": allocation.asset_type,
                        "options_data": allocation.options_data
                    })
                } else {
                    // Hold position
                    json!({
                        "symbol": allocation.symbol,
                        "action": "HOLD",
                        "reason": "Position within acceptable range",
                        "profit_percentage": profit_percentage * 100.0,
                        "target_profit": self.profit_target * 100.0,
                        "asset_type": allocation.asset_type,
                        "options_data": allocation.options_data
                    })
                }
            } else {
                // New position - check if we should buy
                let can_afford = allocation.position_size <= account_data.buying_power;
                let risk_acceptable = risk_metrics.var_95 <= self.max_drawdown_limit;
                
                if can_afford && risk_acceptable {
                    json!({
                        "symbol": allocation.symbol,
                        "action": "BUY",
                        "reason": "New position based on optimal allocation",
                        "allocation_percentage": allocation.allocation_percentage,
                        "position_size": allocation.position_size,
                        "stop_loss": allocation.stop_loss,
                        "take_profit": allocation.take_profit,
                        "expected_return": allocation.expected_return * 100.0,
                        "sharpe_ratio": allocation.sharpe_ratio,
                        "asset_type": allocation.asset_type,
                        "options_data": allocation.options_data
                    })
                } else {
                    json!({
                        "symbol": allocation.symbol,
                        "action": "SKIP",
                        "reason": if !can_afford { "Insufficient buying power" } else { "Risk too high" },
                        "required_capital": allocation.position_size,
                        "available_capital": account_data.buying_power,
                        "asset_type": allocation.asset_type,
                        "options_data": allocation.options_data
                    })
                }
            };
            
            recommendations.push(recommendation);
        }
        
        Ok(recommendations)
    }

    /// Analyze current positions
    fn analyze_current_positions(&self, positions: &[Position]) -> Result<Value> {
        let mut position_analysis = HashMap::new();
        
        for position in positions {
            let market_value = position.market_value.parse::<f64>().unwrap_or(0.0);
            position_analysis.insert(position.symbol.clone(), market_value);
        }
        
        Ok(json!(position_analysis))
    }

    /// Create portfolio summary
    fn create_portfolio_summary(
        &self,
        allocations: &[PortfolioAllocation],
        risk_metrics: &RiskMetrics,
        account_data: &AccountData,
        asset_universe: &[Asset],
    ) -> Result<Value> {
        let total_allocation: f64 = allocations.iter().map(|a| a.allocation_percentage).sum();
        let expected_portfolio_return: f64 = allocations.iter()
            .map(|a| a.expected_return * (a.allocation_percentage / 100.0))
            .sum();
        
        let options_allocation: f64 = allocations.iter()
            .filter(|a| a.asset_type == AssetType::Option)
            .map(|a| a.allocation_percentage)
            .sum();
        
        Ok(json!({
            "total_allocation": total_allocation,
            "expected_portfolio_return": expected_portfolio_return,
            "portfolio_volatility": risk_metrics.portfolio_volatility,
            "portfolio_sharpe_ratio": risk_metrics.sharpe_ratio,
            "value_at_risk_95": risk_metrics.var_95,
            "max_drawdown": risk_metrics.max_drawdown,
            "risk_level": if risk_metrics.var_95 > 0.15 { "HIGH" } else if risk_metrics.var_95 > 0.10 { "MEDIUM" } else { "LOW" },
            "current_positions_count": asset_universe.len(),
            "asset_universe_size": asset_universe.len(),
            "options_allocation_percentage": options_allocation,
            "portfolio_protection_active": account_data.portfolio_value < (account_data.starting_portfolio_value * self.portfolio_protection)
        }))
    }

    /// Create asset universe summary
    fn create_asset_universe_summary(&self, asset_universe: &[Asset]) -> Result<Value> {
        let total_assets = asset_universe.len();
        let tradable_assets = asset_universe.iter().filter(|a| a.tradable).count();
        let marginable_assets = asset_universe.iter().filter(|a| a.marginable).count();
        let shortable_assets = asset_universe.iter().filter(|a| a.shortable).count();
        
        Ok(json!({
            "total_assets": total_assets,
            "tradable_assets": tradable_assets,
            "marginable_assets": marginable_assets,
            "shortable_assets": shortable_assets
        }))
    }
}

impl Default for AdvancedTradingStrategy {
    fn default() -> Self {
        Self::new()
    }
}
