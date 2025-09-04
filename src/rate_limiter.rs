use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    pub max_requests: u32,           // Maximum requests per window
    pub window_duration: Duration,    // Time window for rate limiting
    pub burst_size: u32,              // Maximum burst requests
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,        // 100 requests per window
            window_duration: Duration::from_secs(60), // 1 minute window
            burst_size: 10,           // Allow 10 burst requests
        }
    }
}

/// Request tracking for rate limiting
#[derive(Debug, Clone)]
struct RequestRecord {
    count: u32,
    window_start: Instant,
}

impl RequestRecord {
    fn new() -> Self {
        Self {
            count: 1,
            window_start: Instant::now(),
        }
    }

    fn is_expired(&self, window_duration: Duration) -> bool {
        self.window_start.elapsed() >= window_duration
    }

    fn increment(&mut self) {
        self.count += 1;
    }

    fn reset(&mut self) {
        self.count = 1;
        self.window_start = Instant::now();
    }
}

/// Rate limiter implementation
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, RequestRecord>>>,
    config: RateLimiterConfig,
}

impl RateLimiter {
    /// Create a new rate limiter with default configuration
    pub fn new() -> Self {
        Self::with_config(RateLimiterConfig::default())
    }

    /// Create a new rate limiter with custom configuration
    pub fn with_config(config: RateLimiterConfig) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if a request is allowed for the given key
    pub async fn is_allowed(&self, key: &str) -> bool {
        let mut requests = self.requests.write().await;
        
        // Clean up expired records
        requests.retain(|_, record| !record.is_expired(self.config.window_duration));
        
        if let Some(record) = requests.get_mut(key) {
            if record.is_expired(self.config.window_duration) {
                // Reset expired record
                record.reset();
                true
            } else if record.count < self.config.max_requests {
                // Allow request and increment count
                record.increment();
                true
            } else {
                // Rate limit exceeded
                false
            }
        } else {
            // First request for this key
            requests.insert(key.to_string(), RequestRecord::new());
            true
        }
    }

    /// Get current request count for a key
    pub async fn get_request_count(&self, key: &str) -> u32 {
        let requests = self.requests.read().await;
        if let Some(record) = requests.get(key) {
            if record.is_expired(self.config.window_duration) {
                0
            } else {
                record.count
            }
        } else {
            0
        }
    }

    /// Get remaining requests for a key
    pub async fn get_remaining_requests(&self, key: &str) -> u32 {
        let count = self.get_request_count(key).await;
        if count >= self.config.max_requests {
            0
        } else {
            self.config.max_requests - count
        }
    }

    /// Reset rate limiter for a specific key
    pub async fn reset(&self, key: &str) {
        let mut requests = self.requests.write().await;
        requests.remove(key);
    }

    /// Reset all rate limiters
    pub async fn reset_all(&self) {
        let mut requests = self.requests.write().await;
        requests.clear();
    }

    /// Get rate limiter statistics
    pub async fn get_stats(&self) -> HashMap<String, u32> {
        let requests = self.requests.read().await;
        let mut stats = HashMap::new();
        
        for (key, record) in requests.iter() {
            if !record.is_expired(self.config.window_duration) {
                stats.insert(key.clone(), record.count);
            }
        }
        
        stats
    }
}

/// Rate limiting middleware for API endpoints
pub async fn rate_limit_middleware<F, Fut, T>(
    rate_limiter: &RateLimiter,
    key: &str,
    operation: F,
) -> Result<T, anyhow::Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
{
    if !rate_limiter.is_allowed(key).await {
        return Err(anyhow::anyhow!("Rate limit exceeded"));
    }

    operation().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let config = RateLimiterConfig {
            max_requests: 3,
            window_duration: Duration::from_secs(1),
            burst_size: 1,
        };
        let limiter = RateLimiter::with_config(config);

        // First 3 requests should be allowed
        assert!(limiter.is_allowed("test_key").await);
        assert!(limiter.is_allowed("test_key").await);
        assert!(limiter.is_allowed("test_key").await);

        // 4th request should be blocked
        assert!(!limiter.is_allowed("test_key").await);
    }

    #[tokio::test]
    async fn test_rate_limiter_window_reset() {
        let config = RateLimiterConfig {
            max_requests: 2,
            window_duration: Duration::from_millis(100),
            burst_size: 1,
        };
        let limiter = RateLimiter::with_config(config);

        // Use up the limit
        assert!(limiter.is_allowed("test_key").await);
        assert!(limiter.is_allowed("test_key").await);
        assert!(!limiter.is_allowed("test_key").await);

        // Wait for window to expire
        sleep(Duration::from_millis(150)).await;

        // Should be allowed again
        assert!(limiter.is_allowed("test_key").await);
    }

    #[tokio::test]
    async fn test_rate_limiter_different_keys() {
        let config = RateLimiterConfig {
            max_requests: 1,
            window_duration: Duration::from_secs(1),
            burst_size: 1,
        };
        let limiter = RateLimiter::with_config(config);

        // Different keys should be tracked separately
        assert!(limiter.is_allowed("key1").await);
        assert!(limiter.is_allowed("key2").await);
        assert!(!limiter.is_allowed("key1").await);
        assert!(!limiter.is_allowed("key2").await);
    }
}
