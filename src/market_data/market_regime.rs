use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use log::info;
use crate::market_data::Asset;

/// Market regime types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarketRegime {
    BullMarket,           // Strong upward trend
    BearMarket,           // Strong downward trend
    SidewaysMarket,       // Range-bound, low volatility
    HighVolatility,       // High volatility, uncertain direction
    LowVolatility,        // Low volatility, stable
    Crisis,               // Extreme volatility, panic selling
    Recovery,             // Bouncing back from crisis
    Consolidation,        // Building base for next move
    Momentum,             // Strong momentum in one direction
    MeanReversion,        // Reverting to historical averages
}

/// Market regime analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRegimeAnalysis {
    pub current_regime: MarketRegime,
    pub confidence_score: f64,
    pub volatility_regime: VolatilityRegime,
    pub trend_strength: f64,
    pub correlation_regime: CorrelationRegime,
    pub sector_analysis: SectorAnalysis,
    pub regime_duration: i32, // Days in current regime
    pub regime_probability: f64,
    pub regime_indicators: RegimeIndicators,
    pub timestamp: DateTime<Utc>,
}

/// Volatility regime classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityRegime {
    pub regime_type: String, // "Low", "Normal", "High", "Extreme"
    pub vix_equivalent: f64,
    pub volatility_percentile: f64,
    pub volatility_trend: String, // "Increasing", "Decreasing", "Stable"
}

/// Correlation regime analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationRegime {
    pub average_correlation: f64,
    pub correlation_trend: String, // "Increasing", "Decreasing", "Stable"
    pub diversification_benefit: f64,
    pub sector_correlation: HashMap<String, f64>,
}

/// Sector analysis for regime detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorAnalysis {
    pub leading_sectors: Vec<String>,
    pub lagging_sectors: Vec<String>,
    pub sector_rotation: String, // "Defensive", "Cyclical", "Mixed"
    pub sector_momentum: HashMap<String, f64>,
    pub sector_volatility: HashMap<String, f64>,
}

/// Key regime indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeIndicators {
    pub sp500_trend: f64,
    pub vix_level: f64,
    pub treasury_yield: f64,
    pub dollar_strength: f64,
    pub commodity_trend: f64,
    pub bond_equity_correlation: f64,
    pub market_breadth: f64,
    pub momentum_score: f64,
    pub volatility_score: f64,
    pub correlation_score: f64,
}

/// Market Regime Detector
pub struct MarketRegimeDetector {
    asset_universe: Vec<Asset>,
    historical_data: HashMap<String, Vec<PricePoint>>,
    regime_history: Vec<MarketRegimeAnalysis>,
}

/// Price point for historical analysis
#[derive(Debug, Clone)]
pub struct PricePoint {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>,
}

impl MarketRegimeDetector {
    /// Create a new market regime detector
    pub fn new(asset_universe: Vec<Asset>) -> Self {
        Self {
            asset_universe,
            historical_data: HashMap::new(),
            regime_history: Vec::new(),
        }
    }

    /// Analyze current market regime using asset universe
    pub async fn analyze_market_regime(&mut self, market_data: &HashMap<String, f64>) -> Result<MarketRegimeAnalysis> {
        info!("🔍 Analyzing market regime using asset universe...");
        
        // Calculate key regime indicators
        let regime_indicators = self.calculate_regime_indicators(market_data)?;
        
        // Determine volatility regime
        let volatility_regime = self.analyze_volatility_regime(&regime_indicators)?;
        
        // Analyze correlation regime
        let correlation_regime = self.analyze_correlation_regime(market_data)?;
        
        // Perform sector analysis
        let sector_analysis = self.analyze_sectors(market_data)?;
        
        // Determine trend strength
        let trend_strength = self.calculate_trend_strength(&regime_indicators)?;
        
        // Classify market regime
        let (current_regime, confidence_score) = self.classify_market_regime(
            &regime_indicators,
            &volatility_regime,
            &correlation_regime,
            trend_strength,
        )?;
        
        // Calculate regime probability
        let regime_probability = self.calculate_regime_probability(&current_regime, &regime_indicators)?;
        
        // Determine regime duration
        let regime_duration = self.calculate_regime_duration(&current_regime)?;
        
        let analysis = MarketRegimeAnalysis {
            current_regime,
            confidence_score,
            volatility_regime,
            trend_strength,
            correlation_regime,
            sector_analysis,
            regime_duration,
            regime_probability,
            regime_indicators,
            timestamp: Utc::now(),
        };
        
        // Store in history
        self.regime_history.push(analysis.clone());
        
        info!("✅ Market regime analysis complete: {:?}", analysis.current_regime);
        Ok(analysis)
    }

    /// Calculate key regime indicators
    fn calculate_regime_indicators(&self, market_data: &HashMap<String, f64>) -> Result<RegimeIndicators> {
        // SP500 trend (using SPY as proxy)
        let sp500_trend = self.calculate_sp500_trend(market_data)?;
        
        // VIX equivalent (using volatility of major indices)
        let vix_level = self.calculate_vix_equivalent(market_data)?;
        
        // Treasury yield (using bond ETFs)
        let treasury_yield = self.calculate_treasury_yield(market_data)?;
        
        // Dollar strength (using currency ETFs)
        let dollar_strength = self.calculate_dollar_strength(market_data)?;
        
        // Commodity trend (using commodity ETFs)
        let commodity_trend = self.calculate_commodity_trend(market_data)?;
        
        // Bond-equity correlation
        let bond_equity_correlation = self.calculate_bond_equity_correlation(market_data)?;
        
        // Market breadth
        let market_breadth = self.calculate_market_breadth(market_data)?;
        
        // Momentum score
        let momentum_score = self.calculate_momentum_score(market_data)?;
        
        // Volatility score
        let volatility_score = self.calculate_volatility_score(market_data)?;
        
        // Correlation score
        let correlation_score = self.calculate_correlation_score(market_data)?;
        
        Ok(RegimeIndicators {
            sp500_trend,
            vix_level,
            treasury_yield,
            dollar_strength,
            commodity_trend,
            bond_equity_correlation,
            market_breadth,
            momentum_score,
            volatility_score,
            correlation_score,
        })
    }

    /// Calculate SP500 trend
    fn calculate_sp500_trend(&self, market_data: &HashMap<String, f64>) -> Result<f64> {
        // Use SPY as SP500 proxy
        if let Some(spy_price) = market_data.get("SPY") {
            // Simplified trend calculation (in practice would use historical data)
            let trend = (*spy_price - 400.0) / 400.0; // Assuming baseline of 400
            Ok(trend.max(-1.0).min(1.0))
        } else {
            Ok(0.0)
        }
    }

    /// Calculate VIX equivalent using volatility of major indices
    fn calculate_vix_equivalent(&self, market_data: &HashMap<String, f64>) -> Result<f64> {
        let major_indices = vec!["SPY", "QQQ", "IWM", "VTI"];
        let mut total_volatility = 0.0;
        let mut count = 0;
        
        for index in major_indices {
            if let Some(price) = market_data.get(index) {
                // Simplified volatility calculation
                let volatility = (*price * 0.02).min(50.0); // 2% volatility assumption
                total_volatility += volatility;
                count += 1;
            }
        }
        
        let avg_volatility = if count > 0 { total_volatility / count as f64 } else { 20.0 };
        Ok(avg_volatility)
    }

    /// Calculate treasury yield using bond ETFs
    fn calculate_treasury_yield(&self, market_data: &HashMap<String, f64>) -> Result<f64> {
        // Use BND as treasury proxy
        if let Some(bnd_price) = market_data.get("BND") {
            // Simplified yield calculation (inverse relationship with price)
            let yield_rate = (100.0 - *bnd_price) / 100.0 * 5.0; // 5% max yield
            Ok(yield_rate.max(0.0).min(10.0))
        } else {
            Ok(4.0) // Default yield
        }
    }

    /// Calculate dollar strength
    fn calculate_dollar_strength(&self, _market_data: &HashMap<String, f64>) -> Result<f64> {
        // Simplified dollar strength calculation
        Ok(0.5) // Neutral dollar strength
    }

    /// Calculate commodity trend
    fn calculate_commodity_trend(&self, market_data: &HashMap<String, f64>) -> Result<f64> {
        // Use GLD and SLV as commodity proxies
        let mut commodity_trend = 0.0;
        let mut count = 0;
        
        for commodity in vec!["GLD", "SLV"] {
            if let Some(price) = market_data.get(commodity) {
                let trend = (*price - 100.0) / 100.0; // Assuming baseline of 100
                commodity_trend += trend;
                count += 1;
            }
        }
        
        let avg_trend = if count > 0 { commodity_trend / count as f64 } else { 0.0 };
        Ok(avg_trend.max(-1.0).min(1.0))
    }

    /// Calculate bond-equity correlation
    fn calculate_bond_equity_correlation(&self, _market_data: &HashMap<String, f64>) -> Result<f64> {
        // Simplified correlation calculation
        Ok(-0.3) // Typical negative correlation
    }

    /// Calculate market breadth
    fn calculate_market_breadth(&self, market_data: &HashMap<String, f64>) -> Result<f64> {
        let mut advancing = 0;
        let mut total = 0;
        
        for (symbol, price) in market_data {
            if symbol.len() <= 5 { // Focus on major stocks, not ETFs
                total += 1;
                if *price > 50.0 { // Simplified advancing condition
                    advancing += 1;
                }
            }
        }
        
        let breadth = if total > 0 { advancing as f64 / total as f64 } else { 0.5 };
        Ok(breadth)
    }

    /// Calculate momentum score
    fn calculate_momentum_score(&self, market_data: &HashMap<String, f64>) -> Result<f64> {
        let mut total_momentum = 0.0;
        let mut count = 0;
        
        for (_, price) in market_data {
            let momentum = (*price - 100.0) / 100.0; // Simplified momentum
            total_momentum += momentum;
            count += 1;
        }
        
        let avg_momentum = if count > 0 { total_momentum / count as f64 } else { 0.0 };
        Ok(avg_momentum.max(-1.0).min(1.0))
    }

    /// Calculate volatility score
    fn calculate_volatility_score(&self, market_data: &HashMap<String, f64>) -> Result<f64> {
        let mut total_volatility = 0.0;
        let mut count = 0;
        
        for (_, price) in market_data {
            let volatility = (*price * 0.02).min(50.0); // 2% volatility assumption
            total_volatility += volatility;
            count += 1;
        }
        
        let avg_volatility = if count > 0 { total_volatility / count as f64 } else { 20.0 };
        let volatility_score = (avg_volatility - 20.0) / 30.0; // Normalize to 0-1
        Ok(volatility_score.max(0.0).min(1.0))
    }

    /// Calculate correlation score
    fn calculate_correlation_score(&self, _market_data: &HashMap<String, f64>) -> Result<f64> {
        // Simplified correlation score
        Ok(0.6) // Moderate correlation
    }

    /// Analyze volatility regime
    fn analyze_volatility_regime(&self, indicators: &RegimeIndicators) -> Result<VolatilityRegime> {
        let vix_level = indicators.vix_level;
        
        let (regime_type, volatility_percentile) = if vix_level < 15.0 {
            ("Low", 0.25)
        } else if vix_level < 25.0 {
            ("Normal", 0.5)
        } else if vix_level < 35.0 {
            ("High", 0.75)
        } else {
            ("Extreme", 0.95)
        };
        
        let volatility_trend = if indicators.volatility_score > 0.7 {
            "Increasing"
        } else if indicators.volatility_score < 0.3 {
            "Decreasing"
        } else {
            "Stable"
        };
        
        Ok(VolatilityRegime {
            regime_type: regime_type.to_string(),
            vix_equivalent: vix_level,
            volatility_percentile,
            volatility_trend: volatility_trend.to_string(),
        })
    }

    /// Analyze correlation regime
    fn analyze_correlation_regime(&self, _market_data: &HashMap<String, f64>) -> Result<CorrelationRegime> {
        let average_correlation = 0.6; // Simplified
        let correlation_trend = "Stable".to_string();
        let diversification_benefit = 1.0 - average_correlation;
        
        let mut sector_correlation = HashMap::new();
        sector_correlation.insert("Technology".to_string(), 0.7);
        sector_correlation.insert("Financial".to_string(), 0.6);
        sector_correlation.insert("Healthcare".to_string(), 0.5);
        sector_correlation.insert("Energy".to_string(), 0.4);
        
        Ok(CorrelationRegime {
            average_correlation,
            correlation_trend,
            diversification_benefit,
            sector_correlation,
        })
    }

    /// Analyze sectors
    fn analyze_sectors(&self, market_data: &HashMap<String, f64>) -> Result<SectorAnalysis> {
        let mut sector_momentum = HashMap::new();
        let mut sector_volatility = HashMap::new();
        
        // Simplified sector analysis
        let sectors = vec![
            ("Technology", vec!["AAPL", "MSFT", "GOOGL", "AMZN", "TSLA"]),
            ("Financial", vec!["JPM", "BAC", "WFC", "GS", "MS"]),
            ("Healthcare", vec!["JNJ", "PFE", "UNH", "ABBV", "MRK"]),
            ("Energy", vec!["XOM", "CVX", "COP", "EOG", "SLB"]),
        ];
        
        for (sector_name, symbols) in sectors {
            let mut sector_momentum_sum = 0.0;
            let mut sector_volatility_sum = 0.0;
            let mut count = 0;
            
            for symbol in symbols {
                if let Some(price) = market_data.get(symbol) {
                    let momentum = (*price - 100.0) / 100.0;
                    let volatility = *price * 0.02;
                    sector_momentum_sum += momentum;
                    sector_volatility_sum += volatility;
                    count += 1;
                }
            }
            
            if count > 0 {
                sector_momentum.insert(sector_name.to_string(), sector_momentum_sum / count as f64);
                sector_volatility.insert(sector_name.to_string(), sector_volatility_sum / count as f64);
            }
        }
        
        // Determine leading and lagging sectors
        let mut sector_pairs: Vec<_> = sector_momentum.iter().collect();
        sector_pairs.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        
        let leading_sectors: Vec<String> = sector_pairs.iter().take(2).map(|(s, _)| (*s).clone()).collect();
        let lagging_sectors: Vec<String> = sector_pairs.iter().rev().take(2).map(|(s, _)| (*s).clone()).collect();
        
        let sector_rotation = if leading_sectors.contains(&"Technology".to_string()) {
            "Cyclical"
        } else if leading_sectors.contains(&"Healthcare".to_string()) {
            "Defensive"
        } else {
            "Mixed"
        };
        
        Ok(SectorAnalysis {
            leading_sectors,
            lagging_sectors,
            sector_rotation: sector_rotation.to_string(),
            sector_momentum,
            sector_volatility,
        })
    }

    /// Calculate trend strength
    fn calculate_trend_strength(&self, indicators: &RegimeIndicators) -> Result<f64> {
        let trend_strength = (indicators.sp500_trend.abs() + indicators.momentum_score.abs()) / 2.0;
        Ok(trend_strength.max(0.0).min(1.0))
    }

    /// Classify market regime
    fn classify_market_regime(
        &self,
        indicators: &RegimeIndicators,
        volatility_regime: &VolatilityRegime,
        correlation_regime: &CorrelationRegime,
        trend_strength: f64,
    ) -> Result<(MarketRegime, f64)> {
        let sp500_trend = indicators.sp500_trend;
        let vix_level = indicators.vix_level;
        let momentum_score = indicators.momentum_score;
        let volatility_score = indicators.volatility_score;
        
        let (regime, confidence) = if vix_level > 35.0 {
            (MarketRegime::Crisis, 0.9)
        } else if vix_level > 25.0 && sp500_trend < -0.1 {
            (MarketRegime::BearMarket, 0.8)
        } else if vix_level > 25.0 && sp500_trend > 0.1 {
            (MarketRegime::HighVolatility, 0.7)
        } else if sp500_trend > 0.15 && momentum_score > 0.1 {
            (MarketRegime::BullMarket, 0.8)
        } else if sp500_trend < -0.15 && momentum_score < -0.1 {
            (MarketRegime::BearMarket, 0.8)
        } else if volatility_score < 0.3 && trend_strength < 0.3 {
            (MarketRegime::SidewaysMarket, 0.7)
        } else if volatility_score < 0.3 {
            (MarketRegime::LowVolatility, 0.6)
        } else if momentum_score.abs() > 0.2 {
            (MarketRegime::Momentum, 0.7)
        } else if correlation_regime.average_correlation > 0.8 {
            (MarketRegime::Crisis, 0.6)
        } else {
            (MarketRegime::Consolidation, 0.5)
        };
        
        Ok((regime, confidence))
    }

    /// Calculate regime probability
    fn calculate_regime_probability(&self, regime: &MarketRegime, indicators: &RegimeIndicators) -> Result<f64> {
        // Simplified probability calculation based on indicators
        let base_probability = match regime {
            MarketRegime::BullMarket => 0.3,
            MarketRegime::BearMarket => 0.2,
            MarketRegime::SidewaysMarket => 0.2,
            MarketRegime::HighVolatility => 0.1,
            MarketRegime::LowVolatility => 0.1,
            MarketRegime::Crisis => 0.05,
            MarketRegime::Recovery => 0.05,
            MarketRegime::Consolidation => 0.1,
            MarketRegime::Momentum => 0.1,
            MarketRegime::MeanReversion => 0.1,
        };
        
        // Adjust based on current indicators
        let adjusted_probability = base_probability * (1.0 + indicators.momentum_score.abs());
        Ok(adjusted_probability.min(1.0))
    }

    /// Calculate regime duration
    fn calculate_regime_duration(&self, _current_regime: &MarketRegime) -> Result<i32> {
        // Simplified duration calculation
        Ok(5) // Assume 5 days in current regime
    }

    /// Get regime recommendations for trading strategy
    pub fn get_regime_recommendations(&self, analysis: &MarketRegimeAnalysis) -> Result<Value> {
        let recommendations = match analysis.current_regime {
            MarketRegime::BullMarket => json!({
                "strategy": "Momentum and Growth",
                "asset_allocation": {
                    "stocks": 0.8,
                    "bonds": 0.1,
                    "cash": 0.1
                },
                "sectors": "Technology, Consumer Discretionary, Financials",
                "risk_management": "Trailing stops, Take profits at resistance",
                "position_sizing": "Larger positions, Higher leverage acceptable"
            }),
            MarketRegime::BearMarket => json!({
                "strategy": "Defensive and Hedging",
                "asset_allocation": {
                    "stocks": 0.3,
                    "bonds": 0.5,
                    "cash": 0.2
                },
                "sectors": "Consumer Staples, Healthcare, Utilities",
                "risk_management": "Tight stops, Reduce position sizes",
                "position_sizing": "Smaller positions, Higher cash allocation"
            }),
            MarketRegime::SidewaysMarket => json!({
                "strategy": "Range Trading and Mean Reversion",
                "asset_allocation": {
                    "stocks": 0.5,
                    "bonds": 0.3,
                    "cash": 0.2
                },
                "sectors": "Mixed, Focus on individual stock selection",
                "risk_management": "Range-based stops, Buy dips sell rallies",
                "position_sizing": "Moderate positions, Balanced approach"
            }),
            MarketRegime::HighVolatility => json!({
                "strategy": "Volatility Trading and Hedging",
                "asset_allocation": {
                    "stocks": 0.4,
                    "bonds": 0.4,
                    "cash": 0.2
                },
                "sectors": "Defensive sectors, Low beta stocks",
                "risk_management": "Wider stops, Hedge positions",
                "position_sizing": "Reduced positions, Higher cash"
            }),
            MarketRegime::Crisis => json!({
                "strategy": "Capital Preservation",
                "asset_allocation": {
                    "stocks": 0.2,
                    "bonds": 0.6,
                    "cash": 0.2
                },
                "sectors": "Defensive only, Government bonds",
                "risk_management": "Very tight stops, High cash",
                "position_sizing": "Minimal positions, Maximum cash"
            }),
            _ => json!({
                "strategy": "Balanced Approach",
                "asset_allocation": {
                    "stocks": 0.6,
                    "bonds": 0.3,
                    "cash": 0.1
                },
                "sectors": "Diversified across sectors",
                "risk_management": "Standard risk management",
                "position_sizing": "Normal position sizing"
            }),
        };
        
        Ok(recommendations)
    }

    /// Save market regime analysis to file
    pub async fn save_regime_analysis(&self, analysis: &MarketRegimeAnalysis, data_dir: &str) -> Result<()> {
        let file_path = format!("{}/market_regime_analysis.json", data_dir);
        let json_content = serde_json::to_string_pretty(analysis)?;
        tokio::fs::write(&file_path, json_content).await?;
        println!("💾 Saved market regime analysis to: {}", file_path);
        Ok(())
    }
}
