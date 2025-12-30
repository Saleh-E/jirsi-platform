//! Circuit Breaker - Failure Protection and Rate Limiting
//!
//! Implements the Circuit Breaker pattern to:
//! - Prevent cascading failures
//! - Enforce rate limits
//! - Auto-recover after failures

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Normal operation - requests allowed
    Closed,
    /// Circuit tripped - requests blocked
    Open,
    /// Testing recovery - limited requests allowed
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Number of successes in half-open to close
    pub success_threshold: u32,
    /// Time before attempting recovery
    pub timeout: Duration,
    /// Maximum requests per minute
    pub rate_limit: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
            rate_limit: 60,
        }
    }
}

/// Circuit breaker instance
#[derive(Debug)]
struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure: Option<Instant>,
    last_success: Option<Instant>,
    config: CircuitBreakerConfig,
    
    // Rate limiting
    request_count: u32,
    window_start: Instant,
}

impl CircuitBreaker {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure: None,
            last_success: None,
            config,
            request_count: 0,
            window_start: Instant::now(),
        }
    }
    
    fn can_execute(&mut self) -> Result<(), CircuitBreakerError> {
        // Check rate limit first
        let now = Instant::now();
        if now.duration_since(self.window_start) >= Duration::from_secs(60) {
            // Reset window
            self.window_start = now;
            self.request_count = 0;
        }
        
        if self.request_count >= self.config.rate_limit {
            return Err(CircuitBreakerError::RateLimited);
        }
        
        // Check circuit state
        match self.state {
            CircuitState::Closed => {
                self.request_count += 1;
                Ok(())
            }
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = self.last_failure {
                    if now.duration_since(last_failure) >= self.config.timeout {
                        // Transition to half-open
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        self.request_count += 1;
                        Ok(())
                    } else {
                        Err(CircuitBreakerError::CircuitOpen)
                    }
                } else {
                    // No last failure recorded, close circuit
                    self.state = CircuitState::Closed;
                    self.request_count += 1;
                    Ok(())
                }
            }
            CircuitState::HalfOpen => {
                self.request_count += 1;
                Ok(())
            }
        }
    }
    
    fn record_success(&mut self) {
        self.last_success = Some(Instant::now());
        
        match self.state {
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    // Enough successes, close the circuit
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                }
            }
            CircuitState::Closed => {
                // Decay failure count on success
                if self.failure_count > 0 {
                    self.failure_count -= 1;
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but reset just in case
                self.state = CircuitState::Closed;
                self.failure_count = 0;
            }
        }
    }
    
    fn record_failure(&mut self) {
        self.last_failure = Some(Instant::now());
        self.failure_count += 1;
        
        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open reopens the circuit
                self.state = CircuitState::Open;
                self.success_count = 0;
            }
            CircuitState::Open => {
                // Already open, just update last_failure
            }
        }
    }
}

/// Circuit breaker errors
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit is open - service unavailable")]
    CircuitOpen,
    #[error("Rate limit exceeded")]
    RateLimited,
}

/// Circuit breaker manager - manages multiple circuits
pub struct CircuitBreakerManager {
    circuits: RwLock<HashMap<String, CircuitBreaker>>,
    default_config: CircuitBreakerConfig,
}

impl CircuitBreakerManager {
    pub fn new(default_config: CircuitBreakerConfig) -> Self {
        Self {
            circuits: RwLock::new(HashMap::new()),
            default_config,
        }
    }
    
    /// Get or create a circuit breaker for the given key
    pub async fn get_or_create(&self, key: &str) -> CircuitState {
        let circuits = self.circuits.read().await;
        if let Some(cb) = circuits.get(key) {
            return cb.state;
        }
        drop(circuits);
        
        let mut circuits = self.circuits.write().await;
        circuits.entry(key.to_string())
            .or_insert_with(|| CircuitBreaker::new(self.default_config.clone()))
            .state
    }
    
    /// Check if a request can be executed
    pub async fn can_execute(&self, key: &str) -> Result<(), CircuitBreakerError> {
        let mut circuits = self.circuits.write().await;
        let cb = circuits.entry(key.to_string())
            .or_insert_with(|| CircuitBreaker::new(self.default_config.clone()));
        cb.can_execute()
    }
    
    /// Record a successful execution
    pub async fn record_success(&self, key: &str) {
        let mut circuits = self.circuits.write().await;
        if let Some(cb) = circuits.get_mut(key) {
            cb.record_success();
        }
    }
    
    /// Record a failed execution
    pub async fn record_failure(&self, key: &str) {
        let mut circuits = self.circuits.write().await;
        if let Some(cb) = circuits.get_mut(key) {
            cb.record_failure();
        }
    }
    
    /// Execute a function with circuit breaker protection
    pub async fn execute<F, T, E>(&self, key: &str, f: F) -> Result<T, CircuitBreakerError>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        self.can_execute(key).await?;
        
        match f.await {
            Ok(result) => {
                self.record_success(key).await;
                Ok(result)
            }
            Err(_) => {
                self.record_failure(key).await;
                Err(CircuitBreakerError::CircuitOpen)
            }
        }
    }
    
    /// Get circuit state
    pub async fn get_state(&self, key: &str) -> Option<CircuitState> {
        let circuits = self.circuits.read().await;
        circuits.get(key).map(|cb| cb.state)
    }
    
    /// Get all circuit states (for monitoring)
    pub async fn get_all_states(&self) -> HashMap<String, CircuitState> {
        let circuits = self.circuits.read().await;
        circuits.iter()
            .map(|(k, v)| (k.clone(), v.state))
            .collect()
    }
}

/// Workflow execution guard - prevents infinite loops
pub struct WorkflowExecutionGuard {
    max_loops: u32,
    current_count: u32,
    execution_id: uuid::Uuid,
}

impl WorkflowExecutionGuard {
    pub fn new(execution_id: uuid::Uuid, max_loops: u32) -> Self {
        Self {
            max_loops,
            current_count: 0,
            execution_id,
        }
    }
    
    /// Check if execution can continue (not exceeded max loops)
    pub fn can_continue(&mut self) -> Result<(), String> {
        self.current_count += 1;
        
        if self.current_count > self.max_loops {
            Err(format!(
                "Workflow execution {} exceeded maximum loop count of {}",
                self.execution_id, self.max_loops
            ))
        } else {
            Ok(())
        }
    }
    
    /// Get current loop count
    pub fn loop_count(&self) -> u32 {
        self.current_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let manager = CircuitBreakerManager::new(config);
        
        // Record failures
        for _ in 0..3 {
            manager.record_failure("test").await;
        }
        
        assert_eq!(
            manager.get_state("test").await,
            Some(CircuitState::Open)
        );
    }
    
    #[test]
    fn test_workflow_guard_prevents_infinite_loops() {
        let mut guard = WorkflowExecutionGuard::new(uuid::Uuid::new_v4(), 5);
        
        for _ in 0..5 {
            assert!(guard.can_continue().is_ok());
        }
        
        // 6th iteration should fail
        assert!(guard.can_continue().is_err());
    }
}
