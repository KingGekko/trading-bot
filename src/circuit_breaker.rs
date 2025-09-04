use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,       // Failing, reject requests
    HalfOpen,  // Testing if service is recovered
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,      // Number of failures before opening
    pub success_threshold: u32,      // Number of successes before closing
    pub timeout_duration: Duration,  // How long to stay open
    pub window_size: Duration,       // Time window for failure counting
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_duration: Duration::from_secs(60),
            window_size: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    last_success_time: Arc<RwLock<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default configuration
    pub fn new() -> Self {
        Self::with_config(CircuitBreakerConfig::default())
    }

    /// Create a new circuit breaker with custom configuration
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: Arc::new(RwLock::new(None)),
            last_success_time: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Check if the circuit breaker allows the operation
    pub async fn is_allowed(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() >= self.config.timeout_duration {
                        // Move to half-open state
                        drop(state);
                        let mut state = self.state.write().await;
                        *state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful operation
    pub async fn record_success(&self) {
        let mut last_success = self.last_success_time.write().await;
        *last_success = Some(Instant::now());

        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                // Increment success count
                let success_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if success_count >= self.config.success_threshold {
                    // Move back to closed state
                    drop(state);
                    let mut state = self.state.write().await;
                    *state = CircuitState::Closed;
                    self.success_count.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::Open => {
                // Should not happen, but handle gracefully
            }
        }
    }

    /// Record a failed operation
    pub async fn record_failure(&self) {
        let mut last_failure = self.last_failure_time.write().await;
        *last_failure = Some(Instant::now());

        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => {
                // Increment failure count
                let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if failure_count >= self.config.failure_threshold {
                    // Move to open state
                    drop(state);
                    let mut state = self.state.write().await;
                    *state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                // Move back to open state
                drop(state);
                let mut state = self.state.write().await;
                *state = CircuitState::Open;
                self.success_count.store(0, Ordering::Relaxed);
            }
            CircuitState::Open => {
                // Already open, just update timestamp
            }
        }
    }

    /// Get current state
    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.clone()
    }

    /// Get failure count
    pub fn get_failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::Relaxed)
    }

    /// Get success count
    pub fn get_success_count(&self) -> u32 {
        self.success_count.load(Ordering::Relaxed)
    }

    /// Reset the circuit breaker to closed state
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Closed;
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
    }
}

/// Execute a function with circuit breaker protection
pub async fn with_circuit_breaker<F, Fut, T, E>(
    circuit_breaker: &CircuitBreaker,
    operation: F,
) -> Result<T, E>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: Clone + std::convert::From<anyhow::Error>,
{
    if !circuit_breaker.is_allowed().await {
        return Err(anyhow::anyhow!("Circuit breaker is open").into());
    }

    match operation().await {
        Ok(result) => {
            circuit_breaker.record_success().await;
            Ok(result)
        }
        Err(error) => {
            circuit_breaker.record_failure().await;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let cb = CircuitBreaker::new();
        assert!(cb.is_allowed().await);
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout_duration: Duration::from_secs(1),
            window_size: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::with_config(config);

        // Record failures
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout_duration: Duration::from_millis(100),
            window_size: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::with_config(config);

        // Trigger open state
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Wait for timeout
        sleep(Duration::from_millis(150)).await;
        assert!(cb.is_allowed().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // Record success to close
        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }
}
