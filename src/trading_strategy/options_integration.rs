use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Options strategy types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptionsStrategy {
    CoveredCall,        // Sell calls against long stock
    ProtectivePut,     // Buy puts to protect long stock
    CashSecuredPut,     // Sell puts for premium income
    Straddle,          // Buy call and put at same strike
    Strangle,          // Buy call and put at different strikes
    IronCondor,        // Limited risk, limited reward spread
    Butterfly,          // Neutral strategy with defined risk
    CalendarSpread,     // Different expiration dates
    DiagonalSpread,     // Different strikes and expirations
}

/// Options position data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsPosition {
    pub symbol: String,
    pub strategy: OptionsStrategy,
    pub strike_price: f64,
    pub expiration_date: String,
    pub option_type: String, // "call" or "put"
    pub quantity: i32,
    pub premium_paid: f64,
    pub current_value: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub implied_volatility: f64,
}

/// Options market data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsMarketData {
    pub symbol: String,
    pub underlying_price: f64,
    pub options_chain: Vec<OptionsContract>,
    pub implied_volatility_surface: HashMap<String, f64>,
    pub volume: i64,
    pub open_interest: i64,
    pub bid_ask_spread: f64,
}

/// Individual options contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsContract {
    pub strike: f64,
    pub expiration: String,
    pub option_type: String,
    pub bid: f64,
    pub ask: f64,
    pub last_price: f64,
    pub volume: i64,
    pub open_interest: i64,
    pub implied_volatility: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
}

/// Options strategy manager for live trading only
pub struct OptionsStrategyManager {
    positions: Vec<OptionsPosition>,
    market_data: HashMap<String, OptionsMarketData>,
    enabled: bool,
    trading_mode: String,
}

impl OptionsStrategyManager {
    /// Create new options strategy manager
    pub fn new(trading_mode: String) -> Self {
        let enabled = trading_mode == "live";
        
        Self {
            positions: Vec::new(),
            market_data: HashMap::new(),
            enabled,
            trading_mode,
        }
    }

    /// Check if options trading is enabled (live mode only)
    pub fn is_enabled(&self) -> bool {
        self.enabled && self.trading_mode == "live"
    }

    /// Get options trading status
    pub fn get_status(&self) -> String {
        if self.is_enabled() {
            format!("✅ Options trading ENABLED (Live mode) - {} positions", self.positions.len())
        } else {
            format!("❌ Options trading DISABLED (Paper mode) - Live mode required")
        }
    }

    /// Analyze options opportunities for a given symbol
    pub async fn analyze_options_opportunities(&mut self, symbol: &str, underlying_price: f64) -> Result<Vec<OptionsStrategyRecommendation>> {
        if !self.is_enabled() {
            return Ok(vec![OptionsStrategyRecommendation {
                strategy: OptionsStrategy::CoveredCall,
                symbol: symbol.to_string(),
                recommendation: "Options analysis requires live trading mode".to_string(),
                confidence: 0.0,
                expected_return: 0.0,
                max_risk: 0.0,
                reasoning: "Paper trading mode does not support options analysis".to_string(),
            }]);
        }

        // Simulate options market data (in real implementation, this would fetch from Alpaca)
        let options_data = self.simulate_options_data(symbol, underlying_price);
        
        let mut recommendations = Vec::new();

        // Analyze different strategies
        recommendations.extend(self.analyze_covered_call(&options_data));
        recommendations.extend(self.analyze_protective_put(&options_data));
        recommendations.extend(self.analyze_cash_secured_put(&options_data));
        recommendations.extend(self.analyze_straddle(&options_data));
        recommendations.extend(self.analyze_iron_condor(&options_data));

        Ok(recommendations)
    }

    /// Simulate options market data (replace with real Alpaca API calls)
    fn simulate_options_data(&self, symbol: &str, underlying_price: f64) -> OptionsMarketData {
        let mut options_chain = Vec::new();
        
        // Generate sample options chain
        for strike_offset in -5i32..=5i32 {
            let strike = underlying_price + (strike_offset as f64 * 5.0);
            if strike > 0.0 {
                // Call option
                options_chain.push(OptionsContract {
                    strike,
                    expiration: "2024-01-19".to_string(),
                    option_type: "call".to_string(),
                    bid: strike * 0.02,
                    ask: strike * 0.025,
                    last_price: strike * 0.0225,
                    volume: 100 + (strike_offset.abs() * 50) as i64,
                    open_interest: 500 + (strike_offset.abs() * 100) as i64,
                    implied_volatility: 0.25 + (strike_offset.abs() as f64 * 0.01),
                    delta: if strike_offset <= 0 { 0.6 } else { 0.4 },
                    gamma: 0.01,
                    theta: -0.05,
                    vega: 0.15,
                });

                // Put option
                options_chain.push(OptionsContract {
                    strike,
                    expiration: "2024-01-19".to_string(),
                    option_type: "put".to_string(),
                    bid: strike * 0.015,
                    ask: strike * 0.02,
                    last_price: strike * 0.0175,
                    volume: 80 + (strike_offset.abs() * 40) as i64,
                    open_interest: 400 + (strike_offset.abs() * 80) as i64,
                    implied_volatility: 0.23 + (strike_offset.abs() as f64 * 0.01),
                    delta: if strike_offset <= 0 { -0.4 } else { -0.6 },
                    gamma: 0.01,
                    theta: -0.04,
                    vega: 0.14,
                });
            }
        }

        OptionsMarketData {
            symbol: symbol.to_string(),
            underlying_price,
            options_chain,
            implied_volatility_surface: HashMap::new(),
            volume: 1000,
            open_interest: 5000,
            bid_ask_spread: 0.05,
        }
    }

    /// Analyze covered call opportunities
    fn analyze_covered_call(&self, data: &OptionsMarketData) -> Vec<OptionsStrategyRecommendation> {
        let mut recommendations = Vec::new();
        
        // Find ATM or slightly OTM calls
        for contract in &data.options_chain {
            if contract.option_type == "call" && contract.strike >= data.underlying_price {
                let premium_income = contract.bid;
                let _max_profit = premium_income + (contract.strike - data.underlying_price);
                let max_risk = data.underlying_price - premium_income;
                let return_rate = premium_income / data.underlying_price;

                if return_rate > 0.01 && return_rate < 0.05 { // 1-5% monthly return
                    recommendations.push(OptionsStrategyRecommendation {
                        strategy: OptionsStrategy::CoveredCall,
                        symbol: data.symbol.clone(),
                        recommendation: format!("Sell {} call at ${:.2} strike for ${:.2} premium", 
                            contract.strike, contract.strike, premium_income),
                        confidence: 0.75,
                        expected_return: return_rate,
                        max_risk: max_risk,
                        reasoning: format!("Good premium income with reasonable risk. Strike is {:.1}% OTM", 
                            ((contract.strike - data.underlying_price) / data.underlying_price) * 100.0),
                    });
                }
            }
        }

        recommendations
    }

    /// Analyze protective put opportunities
    fn analyze_protective_put(&self, data: &OptionsMarketData) -> Vec<OptionsStrategyRecommendation> {
        let mut recommendations = Vec::new();
        
        // Find ATM or slightly ITM puts
        for contract in &data.options_chain {
            if contract.option_type == "put" && contract.strike <= data.underlying_price {
                let protection_cost = contract.ask;
                let protection_level = data.underlying_price - contract.strike;
                let cost_percentage = protection_cost / data.underlying_price;

                if cost_percentage < 0.03 && protection_level > data.underlying_price * 0.05 { // <3% cost, >5% protection
                    recommendations.push(OptionsStrategyRecommendation {
                        strategy: OptionsStrategy::ProtectivePut,
                        symbol: data.symbol.clone(),
                        recommendation: format!("Buy {} put at ${:.2} strike for ${:.2} protection", 
                            contract.strike, contract.strike, protection_cost),
                        confidence: 0.70,
                        expected_return: -cost_percentage, // Negative return for insurance
                        max_risk: protection_cost,
                        reasoning: format!("Good downside protection at reasonable cost. Protects {:.1}% downside", 
                            (protection_level / data.underlying_price) * 100.0),
                    });
                }
            }
        }

        recommendations
    }

    /// Analyze cash secured put opportunities
    fn analyze_cash_secured_put(&self, data: &OptionsMarketData) -> Vec<OptionsStrategyRecommendation> {
        let mut recommendations = Vec::new();
        
        // Find OTM puts with good premium
        for contract in &data.options_chain {
            if contract.option_type == "put" && contract.strike < data.underlying_price {
                let premium_income = contract.bid;
                let assignment_risk = data.underlying_price - contract.strike;
                let return_rate = premium_income / contract.strike;

                if return_rate > 0.02 && assignment_risk < data.underlying_price * 0.1 { // >2% return, <10% assignment risk
                    recommendations.push(OptionsStrategyRecommendation {
                        strategy: OptionsStrategy::CashSecuredPut,
                        symbol: data.symbol.clone(),
                        recommendation: format!("Sell {} put at ${:.2} strike for ${:.2} premium", 
                            contract.strike, contract.strike, premium_income),
                        confidence: 0.65,
                        expected_return: return_rate,
                        max_risk: assignment_risk,
                        reasoning: format!("Good premium income with manageable assignment risk. {:.1}% return on cash", 
                            return_rate * 100.0),
                    });
                }
            }
        }

        recommendations
    }

    /// Analyze straddle opportunities
    fn analyze_straddle(&self, data: &OptionsMarketData) -> Vec<OptionsStrategyRecommendation> {
        let mut recommendations = Vec::new();
        
        // Find ATM calls and puts
        let atm_strike = data.underlying_price.round();
        let mut call_cost = 0.0;
        let mut put_cost = 0.0;

        for contract in &data.options_chain {
            if contract.strike == atm_strike {
                if contract.option_type == "call" {
                    call_cost = contract.ask;
                } else if contract.option_type == "put" {
                    put_cost = contract.ask;
                }
            }
        }

        if call_cost > 0.0 && put_cost > 0.0 {
            let total_cost = call_cost + put_cost;
            let breakeven_up = atm_strike + total_cost;
            let breakeven_down = atm_strike - total_cost;
            let cost_percentage = total_cost / data.underlying_price;

            if cost_percentage < 0.08 { // <8% of underlying price
                recommendations.push(OptionsStrategyRecommendation {
                    strategy: OptionsStrategy::Straddle,
                    symbol: data.symbol.clone(),
                    recommendation: format!("Buy straddle at ${:.2} strike for ${:.2} total cost", 
                        atm_strike, total_cost),
                    confidence: 0.60,
                    expected_return: 0.0, // Directional strategy
                    max_risk: total_cost,
                    reasoning: format!("Volatility play with breakevens at ${:.2} and ${:.2}", 
                        breakeven_up, breakeven_down),
                });
            }
        }

        recommendations
    }

    /// Analyze iron condor opportunities
    fn analyze_iron_condor(&self, data: &OptionsMarketData) -> Vec<OptionsStrategyRecommendation> {
        let mut recommendations = Vec::new();
        
        // Simplified iron condor analysis
        let atm_strike = data.underlying_price.round();
        let spread_width = 10.0; // $10 wide spreads
        
        let put_spread_premium = 0.5; // Estimated credit
        let call_spread_premium = 0.5; // Estimated credit
        let total_premium = put_spread_premium + call_spread_premium;
        let max_risk = spread_width - total_premium;
        let return_rate = total_premium / spread_width;

        if return_rate > 0.15 && max_risk < spread_width * 0.5 { // >15% return, <50% max risk
            recommendations.push(OptionsStrategyRecommendation {
                strategy: OptionsStrategy::IronCondor,
                symbol: data.symbol.clone(),
                recommendation: format!("Iron condor around ${:.2} for ${:.2} credit, ${:.2} max risk", 
                    atm_strike, total_premium, max_risk),
                confidence: 0.55,
                expected_return: return_rate,
                max_risk: max_risk,
                reasoning: format!("Range-bound strategy with {:.1}% return on risk", 
                    return_rate * 100.0),
            });
        }

        recommendations
    }

    /// Get options positions summary
    pub fn get_positions_summary(&self) -> String {
        if self.positions.is_empty() {
            return "No options positions".to_string();
        }

        let total_value: f64 = self.positions.iter().map(|p| p.current_value).sum();
        let total_premium: f64 = self.positions.iter().map(|p| p.premium_paid).sum();
        let unrealized_pnl = total_value - total_premium;

        format!(
            "Options Positions: {} | Total Value: ${:.2} | Unrealized P&L: ${:.2}",
            self.positions.len(),
            total_value,
            unrealized_pnl
        )
    }
}

/// Options strategy recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsStrategyRecommendation {
    pub strategy: OptionsStrategy,
    pub symbol: String,
    pub recommendation: String,
    pub confidence: f64,
    pub expected_return: f64,
    pub max_risk: f64,
    pub reasoning: String,
}
