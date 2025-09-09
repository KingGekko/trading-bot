use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Timelike, Weekday, Datelike};

/// Market session types for adaptive timing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MarketSession {
    PreMarket,      // 4:00-9:30 AM
    OpeningBell,    // 9:30-10:00 AM
    RegularTrading, // 10:00 AM - 12:00 PM, 1:00-3:00 PM
    LunchHour,      // 12:00-1:00 PM
    PowerHour,      // 3:00-4:00 PM
    AfterHours,     // 4:00-8:00 PM
    Closed,         // Outside trading hours
}

/// AI processing performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProcessingMetrics {
    pub average_response_time: Duration,
    pub fastest_response: Duration,
    pub slowest_response: Duration,
    pub total_requests: u32,
    pub success_rate: f64,
    pub last_updated: DateTime<Utc>,
}

/// Adaptive timing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveTimingConfig {
    pub base_cycle_duration: Duration,
    pub ai_speed_multiplier: f64,
    pub session_specific_multipliers: std::collections::HashMap<MarketSession, f64>,
    pub min_cycle_duration: Duration,
    pub max_cycle_duration: Duration,
    pub performance_window_size: usize,
}

/// Adaptive timing manager
pub struct AdaptiveTimingManager {
    config: AdaptiveTimingConfig,
    metrics: AIProcessingMetrics,
    recent_response_times: Vec<Duration>,
    current_session: MarketSession,
    last_session_check: Instant,
}

impl AdaptiveTimingManager {
    /// Create new adaptive timing manager
    pub fn new() -> Self {
        let mut session_multipliers = std::collections::HashMap::new();
        session_multipliers.insert(MarketSession::PreMarket, 0.8);      // Faster for gaps
        session_multipliers.insert(MarketSession::OpeningBell, 0.5);   // Much faster for momentum
        session_multipliers.insert(MarketSession::RegularTrading, 1.0); // Normal speed
        session_multipliers.insert(MarketSession::LunchHour, 0.7);     // Faster for reversals
        session_multipliers.insert(MarketSession::PowerHour, 0.6);      // Faster for end-of-day
        session_multipliers.insert(MarketSession::AfterHours, 0.9);    // Slightly faster
        session_multipliers.insert(MarketSession::Closed, 2.0);       // Much slower when closed

        Self {
            config: AdaptiveTimingConfig {
                base_cycle_duration: Duration::from_secs(30),
                ai_speed_multiplier: 1.0,
                session_specific_multipliers: session_multipliers,
                min_cycle_duration: Duration::from_secs(5),
                max_cycle_duration: Duration::from_secs(300),
                performance_window_size: 10,
            },
            metrics: AIProcessingMetrics {
                average_response_time: Duration::from_millis(1000),
                fastest_response: Duration::from_millis(500),
                slowest_response: Duration::from_millis(2000),
                total_requests: 0,
                success_rate: 1.0,
                last_updated: Utc::now(),
            },
            recent_response_times: Vec::new(),
            current_session: MarketSession::Closed,
            last_session_check: Instant::now(),
        }
    }

    /// Update AI processing metrics
    pub fn record_ai_response_time(&mut self, response_time: Duration, success: bool) {
        // Add to recent times
        self.recent_response_times.push(response_time);
        
        // Maintain window size
        if self.recent_response_times.len() > self.config.performance_window_size {
            self.recent_response_times.remove(0);
        }

        // Update metrics
        self.metrics.total_requests += 1;
        if success {
            self.metrics.success_rate = (self.metrics.success_rate * (self.metrics.total_requests - 1) as f64 + 1.0) / self.metrics.total_requests as f64;
        } else {
            self.metrics.success_rate = (self.metrics.success_rate * (self.metrics.total_requests - 1) as f64) / self.metrics.total_requests as f64;
        }

        // Update response time statistics
        if response_time < self.metrics.fastest_response {
            self.metrics.fastest_response = response_time;
        }
        if response_time > self.metrics.slowest_response {
            self.metrics.slowest_response = response_time;
        }

        // Calculate average response time
        let total_time: Duration = self.recent_response_times.iter().sum();
        self.metrics.average_response_time = total_time / self.recent_response_times.len() as u32;
        self.metrics.last_updated = Utc::now();

        // Update AI speed multiplier based on performance
        self.update_ai_speed_multiplier();
    }

    /// Update AI speed multiplier based on recent performance
    fn update_ai_speed_multiplier(&mut self) {
        let avg_response_ms = self.metrics.average_response_time.as_millis() as f64;
        
        // Adjust multiplier based on AI speed
        if avg_response_ms < 500.0 {
            // Very fast AI - can handle more frequent analysis
            self.config.ai_speed_multiplier = 0.7;
        } else if avg_response_ms < 1000.0 {
            // Fast AI - slightly more frequent
            self.config.ai_speed_multiplier = 0.8;
        } else if avg_response_ms < 2000.0 {
            // Normal AI speed
            self.config.ai_speed_multiplier = 1.0;
        } else if avg_response_ms < 5000.0 {
            // Slow AI - less frequent
            self.config.ai_speed_multiplier = 1.5;
        } else {
            // Very slow AI - much less frequent
            self.config.ai_speed_multiplier = 2.0;
        }
    }

    /// Get current market session
    pub fn get_current_session(&mut self) -> MarketSession {
        // Check session every minute to avoid excessive computation
        if self.last_session_check.elapsed() > Duration::from_secs(60) {
            self.current_session = self.detect_market_session();
            self.last_session_check = Instant::now();
        }
        self.current_session.clone()
    }

    /// Detect current market session based on time
    fn detect_market_session(&self) -> MarketSession {
        let now = Utc::now();
        let hour = now.hour();
        let minute = now.minute();
        let weekday = now.weekday();

        // Check if it's a weekend
        if weekday == Weekday::Sat || weekday == Weekday::Sun {
            return MarketSession::Closed;
        }

        // Convert to minutes since midnight for easier comparison
        let current_minutes = hour * 60 + minute;

        match current_minutes {
            240..=569 => MarketSession::PreMarket,      // 4:00 AM - 9:29 AM
            570..=629 => MarketSession::OpeningBell,    // 9:30 AM - 10:29 AM
            630..=719 => MarketSession::RegularTrading, // 10:30 AM - 11:59 AM
            720..=779 => MarketSession::LunchHour,      // 12:00 PM - 12:59 PM
            780..=1079 => MarketSession::RegularTrading, // 1:00 PM - 2:59 PM
            1080..=1139 => MarketSession::PowerHour,    // 3:00 PM - 3:59 PM
            1140..=1279 => MarketSession::AfterHours,   // 4:00 PM - 7:59 PM
            _ => MarketSession::Closed,                  // Outside trading hours
        }
    }

    /// Calculate optimal cycle duration based on AI performance and market session
    pub fn calculate_optimal_cycle_duration(&mut self) -> Duration {
        let session = self.get_current_session();
        let session_multiplier = self.config.session_specific_multipliers
            .get(&session)
            .unwrap_or(&1.0);

        // Base calculation: AI speed * Session multiplier * Base duration
        let mut optimal_duration = self.config.base_cycle_duration.as_millis() as f64
            * self.config.ai_speed_multiplier
            * session_multiplier;

        // Apply bounds
        optimal_duration = optimal_duration.max(self.config.min_cycle_duration.as_millis() as f64);
        optimal_duration = optimal_duration.min(self.config.max_cycle_duration.as_millis() as f64);

        Duration::from_millis(optimal_duration as u64)
    }

    /// Get session-specific analysis recommendations
    pub fn get_session_analysis_recommendations(&mut self) -> Vec<String> {
        let session = self.get_current_session();
        
        match session {
            MarketSession::PreMarket => vec![
                "Analyze overnight gaps and pre-market volume".to_string(),
                "Check for earnings announcements and news impact".to_string(),
                "Monitor futures and international market movements".to_string(),
                "Identify potential opening bell momentum plays".to_string(),
            ],
            MarketSession::OpeningBell => vec![
                "Focus on opening gap fills and momentum continuation".to_string(),
                "Monitor volume spikes and price action".to_string(),
                "Watch for institutional order flow patterns".to_string(),
                "Identify breakout and breakdown opportunities".to_string(),
            ],
            MarketSession::RegularTrading => vec![
                "Standard technical analysis and trend following".to_string(),
                "Monitor support/resistance levels".to_string(),
                "Watch for accumulation/distribution patterns".to_string(),
                "Analyze sector rotation and relative strength".to_string(),
            ],
            MarketSession::LunchHour => vec![
                "Watch for lunch hour reversals and consolidation".to_string(),
                "Monitor low-volume price movements".to_string(),
                "Identify potential afternoon breakout setups".to_string(),
                "Check for news-driven midday moves".to_string(),
            ],
            MarketSession::PowerHour => vec![
                "Focus on end-of-day momentum and positioning".to_string(),
                "Monitor institutional closing activity".to_string(),
                "Watch for power hour breakouts".to_string(),
                "Identify potential overnight gap setups".to_string(),
            ],
            MarketSession::AfterHours => vec![
                "Monitor after-hours earnings and news reactions".to_string(),
                "Watch for extended hours volume patterns".to_string(),
                "Analyze overnight position adjustments".to_string(),
                "Prepare for next day's pre-market analysis".to_string(),
            ],
            MarketSession::Closed => vec![
                "Review daily performance and prepare reports".to_string(),
                "Analyze overnight international markets".to_string(),
                "Update watchlists and research".to_string(),
                "Plan next day's strategy".to_string(),
            ],
        }
    }

    /// Get current timing status for display
    pub fn get_timing_status(&mut self) -> String {
        let session = self.get_current_session();
        let cycle_duration = self.calculate_optimal_cycle_duration();
        let avg_response_ms = self.metrics.average_response_time.as_millis();
        
        format!(
            "🕐 Session: {:?} | Cycle: {}s | AI Speed: {}ms avg | Success: {:.1}%",
            session,
            cycle_duration.as_secs(),
            avg_response_ms,
            self.metrics.success_rate * 100.0
        )
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> String {
        format!(
            "📊 AI Performance: {}ms avg ({}ms-{}ms) | {} requests | {:.1}% success",
            self.metrics.average_response_time.as_millis(),
            self.metrics.fastest_response.as_millis(),
            self.metrics.slowest_response.as_millis(),
            self.metrics.total_requests,
            self.metrics.success_rate * 100.0
        )
    }

    /// Check if ultra-high frequency analysis should be enabled (live mode only)
    pub fn should_enable_ultra_high_freq(&self, trading_mode: &str) -> bool {
        trading_mode == "live" && 
        self.metrics.average_response_time < Duration::from_millis(500) &&
        self.metrics.success_rate > 0.95
    }

    /// Get ultra-high frequency recommendations for live mode
    pub fn get_ultra_high_freq_recommendations(&mut self) -> Vec<String> {
        if !self.should_enable_ultra_high_freq("live") {
            return vec!["Ultra-high frequency analysis requires live mode and fast AI".to_string()];
        }

        let session = self.get_current_session();
        match session {
            MarketSession::OpeningBell => vec![
                "⚡ ULTRA-HIGH FREQ: Monitor every price tick for momentum".to_string(),
                "⚡ ULTRA-HIGH FREQ: Track order book changes in real-time".to_string(),
                "⚡ ULTRA-HIGH FREQ: Analyze micro-movements for scalping".to_string(),
            ],
            MarketSession::PowerHour => vec![
                "⚡ ULTRA-HIGH FREQ: Track end-of-day institutional flows".to_string(),
                "⚡ ULTRA-HIGH FREQ: Monitor closing auction dynamics".to_string(),
                "⚡ ULTRA-HIGH FREQ: Analyze final minute positioning".to_string(),
            ],
            _ => vec![
                "Ultra-high frequency analysis available for opening bell and power hour".to_string(),
            ],
        }
    }
}

impl Default for AdaptiveTimingManager {
    fn default() -> Self {
        Self::new()
    }
}
