use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Technical indicators calculated from 15-minute historical data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIndicators {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    
    // Moving Averages
    pub sma_5: f64,      // 5-minute Simple Moving Average
    pub sma_10: f64,     // 10-minute Simple Moving Average
    pub sma_15: f64,     // 15-minute Simple Moving Average
    pub ema_5: f64,      // 5-minute Exponential Moving Average
    pub ema_10: f64,     // 10-minute Exponential Moving Average
    
    // Momentum Indicators
    pub rsi: f64,        // Relative Strength Index (14-period)
    pub macd: f64,       // MACD Line
    pub macd_signal: f64, // MACD Signal Line
    pub macd_histogram: f64, // MACD Histogram
    
    // Volatility Indicators
    pub bollinger_upper: f64,  // Bollinger Bands Upper
    pub bollinger_middle: f64, // Bollinger Bands Middle (SMA 20)
    pub bollinger_lower: f64,  // Bollinger Bands Lower
    pub atr: f64,             // Average True Range
    
    // Volume Indicators
    pub volume_sma: f64,      // Volume Simple Moving Average
    pub volume_ratio: f64,    // Current Volume / Average Volume
    
    // Price Action
    pub price_change_1min: f64,    // 1-minute price change
    pub price_change_5min: f64,    // 5-minute price change
    pub price_change_15min: f64,   // 15-minute price change
    pub volatility: f64,           // Price volatility (standard deviation)
}

/// Market data point for technical analysis
#[derive(Debug, Clone)]
pub struct MarketDataPoint {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl TechnicalIndicators {
    /// Calculate technical indicators from 15-minute historical data
    pub fn calculate_from_data(symbol: &str, data_points: &[MarketDataPoint]) -> Result<Self> {
        if data_points.is_empty() {
            return Err(anyhow::anyhow!("No data points provided for technical analysis"));
        }

        let latest_point = &data_points[data_points.len() - 1];
        let current_price = latest_point.close;
        
        // Sort data points by timestamp to ensure chronological order
        let mut sorted_data = data_points.to_vec();
        sorted_data.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        // Calculate moving averages
        let sma_5 = Self::calculate_sma(&sorted_data, 5);
        let sma_10 = Self::calculate_sma(&sorted_data, 10);
        let sma_15 = Self::calculate_sma(&sorted_data, 15);
        let ema_5 = Self::calculate_ema(&sorted_data, 5);
        let ema_10 = Self::calculate_ema(&sorted_data, 10);
        
        // Calculate momentum indicators
        let rsi = Self::calculate_rsi(&sorted_data, 14);
        let (macd, macd_signal, macd_histogram) = Self::calculate_macd(&sorted_data);
        
        // Calculate volatility indicators
        let (bollinger_upper, bollinger_middle, bollinger_lower) = Self::calculate_bollinger_bands(&sorted_data, 20);
        let atr = Self::calculate_atr(&sorted_data, 14);
        
        // Calculate volume indicators
        let volume_sma = Self::calculate_volume_sma(&sorted_data, 10);
        let volume_ratio = if volume_sma > 0.0 {
            latest_point.volume / volume_sma
        } else {
            1.0
        };
        
        // Calculate price changes
        let price_change_1min = Self::calculate_price_change(&sorted_data, 1);
        let price_change_5min = Self::calculate_price_change(&sorted_data, 5);
        let price_change_15min = Self::calculate_price_change(&sorted_data, 15);
        
        // Calculate volatility (standard deviation of returns)
        let volatility = Self::calculate_volatility(&sorted_data);
        
        Ok(TechnicalIndicators {
            symbol: symbol.to_string(),
            timestamp: latest_point.timestamp,
            sma_5,
            sma_10,
            sma_15,
            ema_5,
            ema_10,
            rsi,
            macd,
            macd_signal,
            macd_histogram,
            bollinger_upper,
            bollinger_middle,
            bollinger_lower,
            atr,
            volume_sma,
            volume_ratio,
            price_change_1min,
            price_change_5min,
            price_change_15min,
            volatility,
        })
    }
    
    /// Calculate Simple Moving Average
    fn calculate_sma(data: &[MarketDataPoint], period: usize) -> f64 {
        if data.len() < period {
            return data.last().map(|p| p.close).unwrap_or(0.0);
        }
        
        let recent_data = &data[data.len() - period..];
        let sum: f64 = recent_data.iter().map(|p| p.close).sum();
        sum / period as f64
    }
    
    /// Calculate Exponential Moving Average
    fn calculate_ema(data: &[MarketDataPoint], period: usize) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        
        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = data[0].close;
        
        for point in data.iter().skip(1) {
            ema = (point.close * multiplier) + (ema * (1.0 - multiplier));
        }
        
        ema
    }
    
    /// Calculate Relative Strength Index (RSI)
    fn calculate_rsi(data: &[MarketDataPoint], period: usize) -> f64 {
        if data.len() < period + 1 {
            return 50.0; // Neutral RSI
        }
        
        let mut gains = Vec::new();
        let mut losses = Vec::new();
        
        for i in 1..data.len() {
            let change = data[i].close - data[i - 1].close;
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }
        
        if gains.len() < period {
            return 50.0;
        }
        
        let recent_gains = &gains[gains.len() - period..];
        let recent_losses = &losses[losses.len() - period..];
        
        let avg_gain: f64 = recent_gains.iter().sum::<f64>() / period as f64;
        let avg_loss: f64 = recent_losses.iter().sum::<f64>() / period as f64;
        
        if avg_loss == 0.0 {
            return 100.0;
        }
        
        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }
    
    /// Calculate MACD (Moving Average Convergence Divergence)
    fn calculate_macd(data: &[MarketDataPoint]) -> (f64, f64, f64) {
        let ema_12 = Self::calculate_ema(data, 12);
        let ema_26 = Self::calculate_ema(data, 26);
        let macd = ema_12 - ema_26;
        
        // For signal line, we'd need more data points, so we'll use a simplified approach
        let macd_signal = macd * 0.9; // Simplified signal line
        let macd_histogram = macd - macd_signal;
        
        (macd, macd_signal, macd_histogram)
    }
    
    /// Calculate Bollinger Bands
    fn calculate_bollinger_bands(data: &[MarketDataPoint], period: usize) -> (f64, f64, f64) {
        let sma = Self::calculate_sma(data, period);
        
        if data.len() < period {
            return (sma, sma, sma);
        }
        
        let recent_data = &data[data.len() - period..];
        let variance: f64 = recent_data.iter()
            .map(|p| (p.close - sma).powi(2))
            .sum::<f64>() / period as f64;
        let std_dev = variance.sqrt();
        
        let upper = sma + (2.0 * std_dev);
        let lower = sma - (2.0 * std_dev);
        
        (upper, sma, lower)
    }
    
    /// Calculate Average True Range (ATR)
    fn calculate_atr(data: &[MarketDataPoint], period: usize) -> f64 {
        if data.len() < period + 1 {
            return 0.0;
        }
        
        let mut true_ranges = Vec::new();
        
        for i in 1..data.len() {
            let high = data[i].high;
            let low = data[i].low;
            let prev_close = data[i - 1].close;
            
            let tr1 = high - low;
            let tr2 = (high - prev_close).abs();
            let tr3 = (low - prev_close).abs();
            
            let true_range = tr1.max(tr2).max(tr3);
            true_ranges.push(true_range);
        }
        
        if true_ranges.len() < period {
            return 0.0;
        }
        
        let recent_trs = &true_ranges[true_ranges.len() - period..];
        recent_trs.iter().sum::<f64>() / period as f64
    }
    
    /// Calculate Volume Simple Moving Average
    fn calculate_volume_sma(data: &[MarketDataPoint], period: usize) -> f64 {
        if data.len() < period {
            return data.last().map(|p| p.volume).unwrap_or(0.0);
        }
        
        let recent_data = &data[data.len() - period..];
        let sum: f64 = recent_data.iter().map(|p| p.volume).sum();
        sum / period as f64
    }
    
    /// Calculate price change over specified number of periods
    fn calculate_price_change(data: &[MarketDataPoint], periods: usize) -> f64 {
        if data.len() < periods + 1 {
            return 0.0;
        }
        
        let current_price = data[data.len() - 1].close;
        let past_price = data[data.len() - 1 - periods].close;
        
        ((current_price - past_price) / past_price) * 100.0
    }
    
    /// Calculate volatility (standard deviation of returns)
    fn calculate_volatility(data: &[MarketDataPoint]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        
        let mut returns = Vec::new();
        for i in 1..data.len() {
            let return_rate = (data[i].close - data[i - 1].close) / data[i - 1].close;
            returns.push(return_rate);
        }
        
        let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance: f64 = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        
        variance.sqrt() * 100.0 // Convert to percentage
    }
    
    /// Generate AI-friendly analysis summary
    pub fn to_ai_analysis(&self) -> String {
        format!(
            "Technical Analysis for {}:
📊 Moving Averages: SMA5=${:.2}, SMA10=${:.2}, SMA15=${:.2}, EMA5=${:.2}, EMA10=${:.2}
📈 Momentum: RSI={:.1}, MACD={:.4}, Signal={:.4}, Histogram={:.4}
📉 Volatility: BB Upper=${:.2}, Middle=${:.2}, Lower=${:.2}, ATR={:.2}
📊 Volume: Current/Avg Ratio={:.2}x, Volume SMA={:.0}
💰 Price Changes: 1min={:.2}%, 5min={:.2}%, 15min={:.2}%, Volatility={:.2}%",
            self.symbol,
            self.sma_5, self.sma_10, self.sma_15, self.ema_5, self.ema_10,
            self.rsi, self.macd, self.macd_signal, self.macd_histogram,
            self.bollinger_upper, self.bollinger_middle, self.bollinger_lower, self.atr,
            self.volume_ratio, self.volume_sma,
            self.price_change_1min, self.price_change_5min, self.price_change_15min, self.volatility
        )
    }
}
