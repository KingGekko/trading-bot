use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Advanced technical indicator types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdvancedIndicator {
    IchimokuCloud,      // Comprehensive trend and support/resistance
    VolumeProfile,      // Volume at price levels
    MarketProfile,      // Time-price opportunities
    OrderFlow,          // Buy/sell pressure analysis
    SmartMoney,         // Institutional flow tracking
    FibonacciRetracement, // Key retracement levels
    ElliottWave,        // Wave pattern analysis
    WyckoffMethod,      // Accumulation/distribution phases
    VWAP,               // Volume weighted average price
    AnchoredVWAP,       // VWAP from specific point
    Supertrend,         // Trend following indicator
    ParabolicSAR,       // Stop and reverse points
    StochasticRSI,     // Momentum oscillator
    WilliamsR,          // Momentum indicator
    CommodityChannelIndex, // Cycle indicator
}

/// Advanced indicator data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedIndicatorData {
    pub indicator_type: AdvancedIndicator,
    pub symbol: String,
    pub timeframe: String,
    pub current_value: f64,
    pub signal: IndicatorSignal,
    pub strength: f64, // 0.0 to 1.0
    pub confidence: f64, // 0.0 to 1.0
    pub historical_values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

/// Indicator signal types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndicatorSignal {
    StrongBuy,
    Buy,
    Neutral,
    Sell,
    StrongSell,
    Accumulation,
    Distribution,
    Breakout,
    Breakdown,
    Consolidation,
}

/// Advanced indicators manager
pub struct AdvancedIndicatorsManager {
    indicators: HashMap<String, Vec<AdvancedIndicatorData>>,
    enabled: bool,
    trading_mode: String,
}

impl AdvancedIndicatorsManager {
    /// Create new advanced indicators manager
    pub fn new(trading_mode: String) -> Self {
        Self {
            indicators: HashMap::new(),
            enabled: true, // Available in both modes
            trading_mode,
        }
    }

    /// Check if advanced indicators are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get status message
    pub fn get_status(&self) -> String {
        if self.is_enabled() {
            format!("✅ Advanced Indicators ENABLED - {} symbols tracked", self.indicators.len())
        } else {
            "❌ Advanced Indicators DISABLED".to_string()
        }
    }

    /// Calculate all advanced indicators for a symbol
    pub async fn calculate_advanced_indicators(&mut self, symbol: &str, price_data: &[f64], volume_data: &[f64]) -> Result<Vec<AdvancedIndicatorData>> {
        if !self.is_enabled() {
            return Ok(vec![]);
        }

        let mut indicators = Vec::new();

        // Calculate Ichimoku Cloud
        if let Ok(ichimoku) = self.calculate_ichimoku_cloud(symbol, price_data) {
            indicators.push(ichimoku);
        }

        // Calculate Volume Profile
        if let Ok(volume_profile) = self.calculate_volume_profile(symbol, price_data, volume_data) {
            indicators.push(volume_profile);
        }

        // Calculate VWAP
        if let Ok(vwap) = self.calculate_vwap(symbol, price_data, volume_data) {
            indicators.push(vwap);
        }

        // Calculate Supertrend
        if let Ok(supertrend) = self.calculate_supertrend(symbol, price_data) {
            indicators.push(supertrend);
        }

        // Calculate Stochastic RSI
        if let Ok(stoch_rsi) = self.calculate_stochastic_rsi(symbol, price_data) {
            indicators.push(stoch_rsi);
        }

        // Calculate Fibonacci Retracement
        if let Ok(fibonacci) = self.calculate_fibonacci_retracement(symbol, price_data) {
            indicators.push(fibonacci);
        }

        // Store indicators
        self.indicators.insert(symbol.to_string(), indicators.clone());

        Ok(indicators)
    }

    /// Calculate Ichimoku Cloud
    fn calculate_ichimoku_cloud(&self, symbol: &str, price_data: &[f64]) -> Result<AdvancedIndicatorData> {
        if price_data.len() < 52 {
            return Err(anyhow::anyhow!("Insufficient data for Ichimoku Cloud"));
        }

        let _current_price = price_data[price_data.len() - 1];
        
        // Simplified Ichimoku calculation
        let tenkan_sen = (price_data[price_data.len() - 9..].iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() + 
                          price_data[price_data.len() - 9..].iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()) / 2.0;
        
        let kijun_sen = (price_data[price_data.len() - 26..].iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() + 
                         price_data[price_data.len() - 26..].iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()) / 2.0;
        
        let senkou_span_a = (tenkan_sen + kijun_sen) / 2.0;
        let senkou_span_b = (price_data[price_data.len() - 52..].iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() + 
                             price_data[price_data.len() - 52..].iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()) / 2.0;

        let signal = if _current_price > senkou_span_a && _current_price > senkou_span_b {
            IndicatorSignal::StrongBuy
        } else if _current_price > tenkan_sen && _current_price > kijun_sen {
            IndicatorSignal::Buy
        } else if _current_price < senkou_span_a && _current_price < senkou_span_b {
            IndicatorSignal::StrongSell
        } else if _current_price < tenkan_sen && _current_price < kijun_sen {
            IndicatorSignal::Sell
        } else {
            IndicatorSignal::Neutral
        };

        let strength = if _current_price > senkou_span_a && _current_price > senkou_span_b {
            ((_current_price - senkou_span_a) / senkou_span_a).min(1.0)
        } else if _current_price < senkou_span_a && _current_price < senkou_span_b {
            ((senkou_span_a - _current_price) / senkou_span_a).min(1.0)
        } else {
            0.5
        };

        Ok(AdvancedIndicatorData {
            indicator_type: AdvancedIndicator::IchimokuCloud,
            symbol: symbol.to_string(),
            timeframe: "1h".to_string(),
            current_value: _current_price,
            signal,
            strength,
            confidence: 0.85,
            historical_values: price_data.to_vec(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("tenkan_sen".to_string(), tenkan_sen.to_string());
                meta.insert("kijun_sen".to_string(), kijun_sen.to_string());
                meta.insert("senkou_span_a".to_string(), senkou_span_a.to_string());
                meta.insert("senkou_span_b".to_string(), senkou_span_b.to_string());
                meta
            },
        })
    }

    /// Calculate Volume Profile
    fn calculate_volume_profile(&self, symbol: &str, price_data: &[f64], volume_data: &[f64]) -> Result<AdvancedIndicatorData> {
        if price_data.len() != volume_data.len() || price_data.len() < 20 {
            return Err(anyhow::anyhow!("Insufficient data for Volume Profile"));
        }

        let _current_price = price_data[price_data.len() - 1];
        
        // Create price levels and aggregate volume
        let min_price = price_data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_price = price_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let price_range = max_price - min_price;
        let num_levels = 20;
        let level_size = price_range / num_levels as f64;

        let mut volume_at_price = vec![0.0; num_levels];
        
        for (i, &price) in price_data.iter().enumerate() {
            let level_index = ((price - min_price) / level_size).floor() as usize;
            if level_index < num_levels {
                volume_at_price[level_index] += volume_data[i];
            }
        }

        // Find POC (Point of Control) - price level with highest volume
        let max_volume_index = volume_at_price.iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap().0;
        
        let poc_price = min_price + (max_volume_index as f64 * level_size);
        
        // Determine signal based on current price relative to POC
        let signal = if _current_price > poc_price {
            IndicatorSignal::Breakout
        } else if _current_price < poc_price {
            IndicatorSignal::Breakdown
        } else {
            IndicatorSignal::Consolidation
        };

        let strength = volume_at_price[max_volume_index] / volume_data.iter().sum::<f64>();

        Ok(AdvancedIndicatorData {
            indicator_type: AdvancedIndicator::VolumeProfile,
            symbol: symbol.to_string(),
            timeframe: "1h".to_string(),
            current_value: poc_price,
            signal,
            strength,
            confidence: 0.80,
            historical_values: volume_at_price.clone(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("poc_price".to_string(), poc_price.to_string());
                meta.insert("max_volume".to_string(), volume_at_price[max_volume_index].to_string());
                meta
            },
        })
    }

    /// Calculate VWAP
    fn calculate_vwap(&self, symbol: &str, price_data: &[f64], volume_data: &[f64]) -> Result<AdvancedIndicatorData> {
        if price_data.len() != volume_data.len() || price_data.len() < 10 {
            return Err(anyhow::anyhow!("Insufficient data for VWAP"));
        }

        let _current_price = price_data[price_data.len() - 1];
        
        // Calculate VWAP
        let total_volume_value: f64 = price_data.iter().zip(volume_data.iter())
            .map(|(price, volume)| price * volume)
            .sum();
        let total_volume: f64 = volume_data.iter().sum();
        let vwap = total_volume_value / total_volume;

        // Determine signal
        let signal = if _current_price > vwap * 1.02 {
            IndicatorSignal::StrongBuy
        } else if _current_price > vwap {
            IndicatorSignal::Buy
        } else if _current_price < vwap * 0.98 {
            IndicatorSignal::StrongSell
        } else if _current_price < vwap {
            IndicatorSignal::Sell
        } else {
            IndicatorSignal::Neutral
        };

        let strength = ((_current_price - vwap).abs() / vwap).min(1.0);

        Ok(AdvancedIndicatorData {
            indicator_type: AdvancedIndicator::VWAP,
            symbol: symbol.to_string(),
            timeframe: "1h".to_string(),
            current_value: vwap,
            signal,
            strength,
            confidence: 0.90,
            historical_values: price_data.to_vec(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("vwap".to_string(), vwap.to_string());
                meta.insert("price_vs_vwap".to_string(), ((_current_price - vwap) / vwap * 100.0).to_string());
                meta
            },
        })
    }

    /// Calculate Supertrend
    fn calculate_supertrend(&self, symbol: &str, price_data: &[f64]) -> Result<AdvancedIndicatorData> {
        if price_data.len() < 20 {
            return Err(anyhow::anyhow!("Insufficient data for Supertrend"));
        }

        let _current_price = price_data[price_data.len() - 1];
        let period = 10;
        let multiplier = 3.0;

        // Calculate ATR (simplified)
        let mut tr_values = Vec::new();
        for i in 1..price_data.len() {
            let high = price_data[i];
            let low = price_data[i - 1];
            let tr = (high - low).abs();
            tr_values.push(tr);
        }

        let atr = if tr_values.len() >= period {
            tr_values[tr_values.len() - period..].iter().sum::<f64>() / period as f64
        } else {
            tr_values.iter().sum::<f64>() / tr_values.len() as f64
        };

        // Calculate Supertrend
        let hl2 = (price_data[price_data.len() - 1] + price_data[price_data.len() - 2]) / 2.0;
        let upper_band = hl2 + (multiplier * atr);
        let lower_band = hl2 - (multiplier * atr);

        let signal = if _current_price > upper_band {
            IndicatorSignal::StrongBuy
        } else if _current_price > lower_band {
            IndicatorSignal::Buy
        } else if _current_price < lower_band {
            IndicatorSignal::StrongSell
        } else {
            IndicatorSignal::Sell
        };

        let strength = ((_current_price - hl2).abs() / hl2).min(1.0);

        Ok(AdvancedIndicatorData {
            indicator_type: AdvancedIndicator::Supertrend,
            symbol: symbol.to_string(),
            timeframe: "1h".to_string(),
            current_value: hl2,
            signal,
            strength,
            confidence: 0.75,
            historical_values: price_data.to_vec(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("upper_band".to_string(), upper_band.to_string());
                meta.insert("lower_band".to_string(), lower_band.to_string());
                meta.insert("atr".to_string(), atr.to_string());
                meta
            },
        })
    }

    /// Calculate Stochastic RSI
    fn calculate_stochastic_rsi(&self, symbol: &str, price_data: &[f64]) -> Result<AdvancedIndicatorData> {
        if price_data.len() < 21 {
            return Err(anyhow::anyhow!("Insufficient data for Stochastic RSI"));
        }

        let _current_price = price_data[price_data.len() - 1];
        let period = 14;

        // Calculate RSI
        let mut gains = Vec::new();
        let mut losses = Vec::new();
        
        for i in 1..price_data.len() {
            let change = price_data[i] - price_data[i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        let avg_gain = if gains.len() >= period {
            gains[gains.len() - period..].iter().sum::<f64>() / period as f64
        } else {
            gains.iter().sum::<f64>() / gains.len() as f64
        };

        let avg_loss = if losses.len() >= period {
            losses[losses.len() - period..].iter().sum::<f64>() / period as f64
        } else {
            losses.iter().sum::<f64>() / losses.len() as f64
        };

        let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        // Calculate Stochastic RSI
        let stoch_rsi = ((rsi - 20.0) / (80.0 - 20.0)).clamp(0.0, 1.0);

        let signal = if stoch_rsi > 0.8 {
            IndicatorSignal::StrongBuy
        } else if stoch_rsi > 0.6 {
            IndicatorSignal::Buy
        } else if stoch_rsi < 0.2 {
            IndicatorSignal::StrongSell
        } else if stoch_rsi < 0.4 {
            IndicatorSignal::Sell
        } else {
            IndicatorSignal::Neutral
        };

        Ok(AdvancedIndicatorData {
            indicator_type: AdvancedIndicator::StochasticRSI,
            symbol: symbol.to_string(),
            timeframe: "1h".to_string(),
            current_value: stoch_rsi,
            signal,
            strength: stoch_rsi,
            confidence: 0.70,
            historical_values: vec![rsi],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("rsi".to_string(), rsi.to_string());
                meta.insert("stoch_rsi".to_string(), stoch_rsi.to_string());
                meta
            },
        })
    }

    /// Calculate Fibonacci Retracement
    fn calculate_fibonacci_retracement(&self, symbol: &str, price_data: &[f64]) -> Result<AdvancedIndicatorData> {
        if price_data.len() < 20 {
            return Err(anyhow::anyhow!("Insufficient data for Fibonacci Retracement"));
        }

        let _current_price = price_data[price_data.len() - 1];
        
        // Find recent high and low
        let recent_high = price_data[price_data.len() - 20..].iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let recent_low = price_data[price_data.len() - 20..].iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        let range = recent_high - recent_low;
        
        // Calculate Fibonacci levels
        let fib_236 = recent_high - (range * 0.236);
        let fib_382 = recent_high - (range * 0.382);
        let fib_500 = recent_high - (range * 0.500);
        let fib_618 = recent_high - (range * 0.618);
        let fib_786 = recent_high - (range * 0.786);

        // Determine which level current price is closest to
        let levels = vec![fib_236, fib_382, fib_500, fib_618, fib_786];
        let closest_level = levels.iter()
            .min_by(|a, b| (_current_price - **a).abs().partial_cmp(&(_current_price - **b).abs()).unwrap())
            .unwrap();

        let signal = if _current_price > fib_382 {
            IndicatorSignal::StrongBuy
        } else if _current_price > fib_500 {
            IndicatorSignal::Buy
        } else if _current_price < fib_618 {
            IndicatorSignal::StrongSell
        } else if _current_price < fib_500 {
            IndicatorSignal::Sell
        } else {
            IndicatorSignal::Neutral
        };

        let strength = 1.0 - ((_current_price - closest_level).abs() / range).min(1.0);

        Ok(AdvancedIndicatorData {
            indicator_type: AdvancedIndicator::FibonacciRetracement,
            symbol: symbol.to_string(),
            timeframe: "1h".to_string(),
            current_value: *closest_level,
            signal,
            strength,
            confidence: 0.65,
            historical_values: levels,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("recent_high".to_string(), recent_high.to_string());
                meta.insert("recent_low".to_string(), recent_low.to_string());
                meta.insert("fib_236".to_string(), fib_236.to_string());
                meta.insert("fib_382".to_string(), fib_382.to_string());
                meta.insert("fib_500".to_string(), fib_500.to_string());
                meta.insert("fib_618".to_string(), fib_618.to_string());
                meta.insert("fib_786".to_string(), fib_786.to_string());
                meta
            },
        })
    }

    /// Get indicator summary for a symbol
    pub fn get_indicator_summary(&self, symbol: &str) -> String {
        if let Some(indicators) = self.indicators.get(symbol) {
            let strong_signals: Vec<_> = indicators.iter()
                .filter(|i| matches!(i.signal, IndicatorSignal::StrongBuy | IndicatorSignal::StrongSell))
                .collect();
            
            let buy_signals: Vec<_> = indicators.iter()
                .filter(|i| matches!(i.signal, IndicatorSignal::Buy | IndicatorSignal::StrongBuy))
                .collect();
            
            let sell_signals: Vec<_> = indicators.iter()
                .filter(|i| matches!(i.signal, IndicatorSignal::Sell | IndicatorSignal::StrongSell))
                .collect();

            format!(
                "📊 {}: {} indicators | {} strong signals | {} buy | {} sell",
                symbol,
                indicators.len(),
                strong_signals.len(),
                buy_signals.len(),
                sell_signals.len()
            )
        } else {
            format!("📊 {}: No indicators calculated", symbol)
        }
    }

    /// Get all indicators summary
    pub fn get_all_indicators_summary(&self) -> String {
        if self.indicators.is_empty() {
            return "No advanced indicators calculated".to_string();
        }

        let total_indicators: usize = self.indicators.values().map(|v| v.len()).sum();
        format!("Advanced Indicators: {} symbols | {} total indicators", 
                self.indicators.len(), total_indicators)
    }
}
