use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Market sectors for rotation analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MarketSector {
    Technology,
    Healthcare,
    Financials,
    ConsumerDiscretionary,
    ConsumerStaples,
    Industrials,
    Energy,
    Materials,
    RealEstate,
    Utilities,
    CommunicationServices,
}

/// Sector performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorPerformance {
    pub sector: MarketSector,
    pub symbol: String,
    pub current_price: f64,
    pub change_1d: f64,
    pub change_5d: f64,
    pub change_1m: f64,
    pub volume: i64,
    pub relative_strength: f64,
    pub momentum_score: f64,
    pub volatility: f64,
}

/// Sector rotation phase
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SectorRotationPhase {
    EarlyCycle,        // Technology, Consumer Discretionary leading
    MidCycle,          // Industrials, Materials gaining
    LateCycle,         // Energy, Materials, Financials strong
    Recession,         // Utilities, Consumer Staples defensive
    Recovery,          // Technology, Healthcare leading recovery
    Sideways,          // No clear rotation pattern
}

/// Sector rotation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorRotationAnalysis {
    pub current_phase: SectorRotationPhase,
    pub phase_confidence: f64,
    pub leading_sectors: Vec<MarketSector>,
    pub lagging_sectors: Vec<MarketSector>,
    pub rotation_trend: f64, // -1.0 to 1.0
    pub recommendations: Vec<SectorRecommendation>,
    pub market_regime: String,
}

/// Sector recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorRecommendation {
    pub sector: MarketSector,
    pub action: String, // "BUY", "SELL", "HOLD", "OVERWEIGHT", "UNDERWEIGHT"
    pub confidence: f64,
    pub reasoning: String,
    pub expected_return: f64,
    pub risk_level: String, // "LOW", "MEDIUM", "HIGH"
}

/// Sector rotation manager
pub struct SectorRotationManager {
    sector_data: HashMap<MarketSector, SectorPerformance>,
    historical_phases: Vec<SectorRotationPhase>,
    enabled: bool,
    trading_mode: String,
}

impl SectorRotationManager {
    /// Create new sector rotation manager (live mode only)
    pub fn new(trading_mode: String) -> Self {
        Self {
            sector_data: HashMap::new(),
            historical_phases: Vec::new(),
            enabled: true, // Only active in live mode
            trading_mode,
        }
    }

    /// Check if sector rotation analysis is enabled (live mode only)
    pub fn is_enabled(&self) -> bool {
        self.enabled && self.trading_mode == "live"
    }

    /// Get status message
    pub fn get_status(&self) -> String {
        if self.is_enabled() {
            format!("✅ Sector Rotation Analysis ENABLED - {} sectors tracked", self.sector_data.len())
        } else {
            "❌ Sector Rotation Analysis DISABLED".to_string()
        }
    }

    /// Update sector performance data (live mode only)
    pub async fn update_sector_data(&mut self, sector_performances: Vec<SectorPerformance>) -> Result<()> {
        // Only update sector data in live mode
        if !self.is_enabled() {
            return Ok(());
        }

        for performance in sector_performances {
            self.sector_data.insert(performance.sector.clone(), performance);
        }

        Ok(())
    }

    /// Analyze sector rotation (live mode only)
    pub async fn analyze_sector_rotation(&mut self) -> Result<SectorRotationAnalysis> {
        // Only analyze sector rotation in live mode
        if !self.is_enabled() || self.sector_data.is_empty() {
            return Ok(SectorRotationAnalysis {
                current_phase: SectorRotationPhase::Sideways,
                phase_confidence: 0.0,
                leading_sectors: Vec::new(),
                lagging_sectors: Vec::new(),
                rotation_trend: 0.0,
                recommendations: Vec::new(),
                market_regime: "Unknown".to_string(),
            });
        }

        // Calculate relative strength for each sector
        let mut sector_scores: Vec<(MarketSector, f64)> = self.sector_data.iter()
            .map(|(sector, performance)| (sector.clone(), performance.relative_strength))
            .collect();

        // Sort by relative strength
        sector_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Determine current phase based on leading sectors
        let (current_phase, phase_confidence) = self.determine_rotation_phase(&sector_scores);
        
        // Identify leading and lagging sectors
        let leading_sectors: Vec<MarketSector> = sector_scores.iter()
            .take(3)
            .map(|(sector, _)| sector.clone())
            .collect();

        let lagging_sectors: Vec<MarketSector> = sector_scores.iter()
            .rev()
            .take(3)
            .map(|(sector, _)| sector.clone())
            .collect();

        // Calculate rotation trend
        let rotation_trend = self.calculate_rotation_trend(&sector_scores);

        // Generate recommendations
        let recommendations = self.generate_sector_recommendations(&current_phase, &sector_scores);

        // Determine market regime
        let market_regime = self.determine_market_regime(&current_phase, &rotation_trend);

        let analysis = SectorRotationAnalysis {
            current_phase: current_phase.clone(),
            phase_confidence,
            leading_sectors,
            lagging_sectors,
            rotation_trend,
            recommendations,
            market_regime,
        };

        // Store historical phase
        self.historical_phases.push(current_phase);
        if self.historical_phases.len() > 20 {
            self.historical_phases.remove(0);
        }

        Ok(analysis)
    }

    /// Determine rotation phase based on sector performance
    fn determine_rotation_phase(&self, sector_scores: &[(MarketSector, f64)]) -> (SectorRotationPhase, f64) {
        let mut phase_scores = HashMap::new();
        
        // Score each phase based on current sector leadership
        for (sector, score) in sector_scores {
            match sector {
                MarketSector::Technology => {
                    *phase_scores.entry(SectorRotationPhase::EarlyCycle).or_insert(0.0) += score * 0.4;
                    *phase_scores.entry(SectorRotationPhase::Recovery).or_insert(0.0) += score * 0.3;
                },
                MarketSector::ConsumerDiscretionary => {
                    *phase_scores.entry(SectorRotationPhase::EarlyCycle).or_insert(0.0) += score * 0.3;
                },
                MarketSector::Industrials => {
                    *phase_scores.entry(SectorRotationPhase::MidCycle).or_insert(0.0) += score * 0.4;
                },
                MarketSector::Materials => {
                    *phase_scores.entry(SectorRotationPhase::MidCycle).or_insert(0.0) += score * 0.3;
                    *phase_scores.entry(SectorRotationPhase::LateCycle).or_insert(0.0) += score * 0.3;
                },
                MarketSector::Energy => {
                    *phase_scores.entry(SectorRotationPhase::LateCycle).or_insert(0.0) += score * 0.4;
                },
                MarketSector::Financials => {
                    *phase_scores.entry(SectorRotationPhase::LateCycle).or_insert(0.0) += score * 0.3;
                },
                MarketSector::Utilities => {
                    *phase_scores.entry(SectorRotationPhase::Recession).or_insert(0.0) += score * 0.4;
                },
                MarketSector::ConsumerStaples => {
                    *phase_scores.entry(SectorRotationPhase::Recession).or_insert(0.0) += score * 0.3;
                },
                MarketSector::Healthcare => {
                    *phase_scores.entry(SectorRotationPhase::Recovery).or_insert(0.0) += score * 0.4;
                },
                _ => {} // Other sectors have less impact on phase determination
            }
        }

        // Find the phase with highest score
        let (best_phase, best_score) = phase_scores.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap_or((&SectorRotationPhase::Sideways, &0.0));

        let confidence = (best_score / sector_scores.len() as f64).min(1.0);

        (best_phase.clone(), confidence)
    }

    /// Calculate rotation trend (-1.0 to 1.0)
    fn calculate_rotation_trend(&self, sector_scores: &[(MarketSector, f64)]) -> f64 {
        if sector_scores.len() < 2 {
            return 0.0;
        }

        // Calculate trend based on cyclical vs defensive sectors
        let cyclical_sectors = vec![
            MarketSector::Technology,
            MarketSector::ConsumerDiscretionary,
            MarketSector::Industrials,
            MarketSector::Materials,
            MarketSector::Energy,
            MarketSector::Financials,
        ];

        let defensive_sectors = vec![
            MarketSector::Utilities,
            MarketSector::ConsumerStaples,
            MarketSector::Healthcare,
            MarketSector::RealEstate,
        ];

        let cyclical_score: f64 = sector_scores.iter()
            .filter(|(sector, _)| cyclical_sectors.contains(sector))
            .map(|(_, score)| score)
            .sum();

        let defensive_score: f64 = sector_scores.iter()
            .filter(|(sector, _)| defensive_sectors.contains(sector))
            .map(|(_, score)| score)
            .sum();

        let total_score = cyclical_score + defensive_score;
        if total_score == 0.0 {
            return 0.0;
        }

        (cyclical_score - defensive_score) / total_score
    }

    /// Generate sector recommendations
    fn generate_sector_recommendations(&self, phase: &SectorRotationPhase, sector_scores: &[(MarketSector, f64)]) -> Vec<SectorRecommendation> {
        let mut recommendations = Vec::new();

        for (sector, score) in sector_scores {
            let (action, reasoning, expected_return, risk_level) = match phase {
                SectorRotationPhase::EarlyCycle => {
                    match sector {
                        MarketSector::Technology | MarketSector::ConsumerDiscretionary => {
                            ("OVERWEIGHT", "Leading early cycle sectors", 0.15, "MEDIUM")
                        },
                        MarketSector::Utilities | MarketSector::ConsumerStaples => {
                            ("UNDERWEIGHT", "Defensive sectors lagging in early cycle", -0.05, "LOW")
                        },
                        _ => ("HOLD", "Neutral positioning", 0.05, "MEDIUM")
                    }
                },
                SectorRotationPhase::MidCycle => {
                    match sector {
                        MarketSector::Industrials | MarketSector::Materials => {
                            ("OVERWEIGHT", "Mid-cycle leaders", 0.12, "MEDIUM")
                        },
                        MarketSector::Technology => {
                            ("HOLD", "Early cycle momentum slowing", 0.08, "MEDIUM")
                        },
                        _ => ("HOLD", "Neutral positioning", 0.06, "MEDIUM")
                    }
                },
                SectorRotationPhase::LateCycle => {
                    match sector {
                        MarketSector::Energy | MarketSector::Materials | MarketSector::Financials => {
                            ("OVERWEIGHT", "Late cycle leaders", 0.10, "HIGH")
                        },
                        MarketSector::Technology | MarketSector::ConsumerDiscretionary => {
                            ("UNDERWEIGHT", "Early cycle sectors weakening", -0.08, "HIGH")
                        },
                        _ => ("HOLD", "Neutral positioning", 0.04, "MEDIUM")
                    }
                },
                SectorRotationPhase::Recession => {
                    match sector {
                        MarketSector::Utilities | MarketSector::ConsumerStaples => {
                            ("OVERWEIGHT", "Defensive sectors leading", 0.08, "LOW")
                        },
                        MarketSector::Energy | MarketSector::Materials => {
                            ("UNDERWEIGHT", "Cyclical sectors under pressure", -0.15, "HIGH")
                        },
                        _ => ("HOLD", "Neutral positioning", 0.02, "MEDIUM")
                    }
                },
                SectorRotationPhase::Recovery => {
                    match sector {
                        MarketSector::Technology | MarketSector::Healthcare => {
                            ("OVERWEIGHT", "Recovery leaders", 0.18, "MEDIUM")
                        },
                        MarketSector::Utilities => {
                            ("UNDERWEIGHT", "Defensive sectors lagging recovery", -0.05, "LOW")
                        },
                        _ => ("HOLD", "Neutral positioning", 0.08, "MEDIUM")
                    }
                },
                SectorRotationPhase::Sideways => {
                    ("HOLD", "No clear rotation pattern", 0.03, "MEDIUM")
                }
            };

            let confidence = if *score > 0.1 { 0.8 } else if *score > 0.0 { 0.6 } else { 0.4 };

            recommendations.push(SectorRecommendation {
                sector: sector.clone(),
                action: action.to_string(),
                confidence,
                reasoning: reasoning.to_string(),
                expected_return,
                risk_level: risk_level.to_string(),
            });
        }

        recommendations
    }

    /// Determine market regime
    fn determine_market_regime(&self, phase: &SectorRotationPhase, rotation_trend: &f64) -> String {
        match phase {
            SectorRotationPhase::EarlyCycle => "Growth Expansion".to_string(),
            SectorRotationPhase::MidCycle => "Economic Expansion".to_string(),
            SectorRotationPhase::LateCycle => "Late Cycle Growth".to_string(),
            SectorRotationPhase::Recession => "Economic Contraction".to_string(),
            SectorRotationPhase::Recovery => "Economic Recovery".to_string(),
            SectorRotationPhase::Sideways => {
                if *rotation_trend > 0.2 {
                    "Cyclical Leadership".to_string()
                } else if *rotation_trend < -0.2 {
                    "Defensive Leadership".to_string()
                } else {
                    "Mixed Signals".to_string()
                }
            }
        }
    }

    /// Get sector rotation summary
    pub fn get_rotation_summary(&self) -> String {
        if self.sector_data.is_empty() {
            return "No sector data available".to_string();
        }

        let total_sectors = self.sector_data.len();
        let avg_relative_strength: f64 = self.sector_data.values()
            .map(|p| p.relative_strength)
            .sum::<f64>() / total_sectors as f64;

        format!(
            "Sector Rotation: {} sectors | Avg RS: {:.2} | Phases tracked: {}",
            total_sectors,
            avg_relative_strength,
            self.historical_phases.len()
        )
    }

    /// Get top performing sectors
    pub fn get_top_sectors(&self, count: usize) -> Vec<String> {
        let mut sector_performances: Vec<_> = self.sector_data.values().collect();
        sector_performances.sort_by(|a, b| b.relative_strength.partial_cmp(&a.relative_strength).unwrap());

        sector_performances.iter()
            .take(count)
            .map(|p| format!("{}: {:.1}%", format!("{:?}", p.sector), p.change_1d * 100.0))
            .collect()
    }

    /// Get sector rotation insights
    pub fn get_rotation_insights(&self) -> Vec<String> {
        if self.historical_phases.len() < 3 {
            return vec!["Insufficient data for rotation insights".to_string()];
        }

        let recent_phases = &self.historical_phases[self.historical_phases.len() - 3..];
        let mut insights = Vec::new();

        // Check for phase transitions
        if recent_phases[0] != recent_phases[1] {
            insights.push(format!("Phase transition detected: {:?} → {:?}", 
                recent_phases[0], recent_phases[1]));
        }

        // Check for consistent trends
        if recent_phases.iter().all(|p| matches!(p, SectorRotationPhase::EarlyCycle)) {
            insights.push("Sustained early cycle phase - Technology/Consumer Discretionary leadership".to_string());
        } else if recent_phases.iter().all(|p| matches!(p, SectorRotationPhase::LateCycle)) {
            insights.push("Sustained late cycle phase - Energy/Materials leadership".to_string());
        }

        // Check for defensive rotation
        let defensive_count = recent_phases.iter()
            .filter(|p| matches!(p, SectorRotationPhase::Recession))
            .count();
        
        if defensive_count >= 2 {
            insights.push("Defensive rotation pattern - Consider Utilities/Consumer Staples".to_string());
        }

        insights
    }
}
