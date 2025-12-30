//! Plugin Resource Governance
//!
//! Implements resource limits and sandboxing for WASM plugins:
//! - Fuel (instruction count) limits
//! - Memory limits
//! - Execution timeout
//! - Rate limiting

use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Plugin resource limits configuration
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum fuel (WASM instructions) per execution
    pub max_fuel: u64,
    /// Maximum memory in bytes
    pub max_memory_bytes: usize,
    /// Maximum execution time
    pub timeout: Duration,
    /// Maximum HTTP requests per execution
    pub max_http_requests: u32,
    /// Maximum entity operations per execution
    pub max_entity_ops: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_fuel: 1_000_000_000,           // 1 billion instructions
            max_memory_bytes: 64 * 1024 * 1024,  // 64 MB
            timeout: Duration::from_secs(30),
            max_http_requests: 10,
            max_entity_ops: 100,
        }
    }
}

impl ResourceLimits {
    /// Limits for untrusted marketplace plugins
    pub fn untrusted() -> Self {
        Self {
            max_fuel: 100_000_000,              // 100 million instructions
            max_memory_bytes: 16 * 1024 * 1024,  // 16 MB
            timeout: Duration::from_secs(5),
            max_http_requests: 3,
            max_entity_ops: 20,
        }
    }
    
    /// Limits for verified/trusted plugins
    pub fn trusted() -> Self {
        Self {
            max_fuel: 5_000_000_000,            // 5 billion instructions
            max_memory_bytes: 128 * 1024 * 1024, // 128 MB
            timeout: Duration::from_secs(60),
            max_http_requests: 50,
            max_entity_ops: 500,
        }
    }
    
    /// Limits for system/first-party plugins
    pub fn system() -> Self {
        Self {
            max_fuel: u64::MAX,
            max_memory_bytes: 512 * 1024 * 1024, // 512 MB
            timeout: Duration::from_secs(300),
            max_http_requests: u32::MAX,
            max_entity_ops: u32::MAX,
        }
    }
}

/// Resource usage tracker for a single execution
#[derive(Debug)]
pub struct ResourceTracker {
    limits: ResourceLimits,
    start_time: Instant,
    fuel_consumed: u64,
    memory_used: usize,
    http_requests: u32,
    entity_ops: u32,
}

impl ResourceTracker {
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            limits,
            start_time: Instant::now(),
            fuel_consumed: 0,
            memory_used: 0,
            http_requests: 0,
            entity_ops: 0,
        }
    }
    
    /// Check if execution has exceeded time limit
    pub fn check_timeout(&self) -> Result<(), ResourceError> {
        if self.start_time.elapsed() > self.limits.timeout {
            Err(ResourceError::Timeout {
                elapsed: self.start_time.elapsed(),
                limit: self.limits.timeout,
            })
        } else {
            Ok(())
        }
    }
    
    /// Consume fuel and check limits
    pub fn consume_fuel(&mut self, amount: u64) -> Result<(), ResourceError> {
        self.check_timeout()?;
        self.fuel_consumed += amount;
        if self.fuel_consumed > self.limits.max_fuel {
            Err(ResourceError::FuelExhausted {
                consumed: self.fuel_consumed,
                limit: self.limits.max_fuel,
            })
        } else {
            Ok(())
        }
    }
    
    /// Track memory allocation
    pub fn allocate_memory(&mut self, bytes: usize) -> Result<(), ResourceError> {
        self.check_timeout()?;
        self.memory_used += bytes;
        if self.memory_used > self.limits.max_memory_bytes {
            Err(ResourceError::MemoryExceeded {
                used: self.memory_used,
                limit: self.limits.max_memory_bytes,
            })
        } else {
            Ok(())
        }
    }
    
    /// Track HTTP request
    pub fn track_http_request(&mut self) -> Result<(), ResourceError> {
        self.check_timeout()?;
        self.http_requests += 1;
        if self.http_requests > self.limits.max_http_requests {
            Err(ResourceError::HttpLimitExceeded {
                count: self.http_requests,
                limit: self.limits.max_http_requests,
            })
        } else {
            Ok(())
        }
    }
    
    /// Track entity operation
    pub fn track_entity_op(&mut self) -> Result<(), ResourceError> {
        self.check_timeout()?;
        self.entity_ops += 1;
        if self.entity_ops > self.limits.max_entity_ops {
            Err(ResourceError::EntityOpsExceeded {
                count: self.entity_ops,
                limit: self.limits.max_entity_ops,
            })
        } else {
            Ok(())
        }
    }
    
    /// Get usage summary
    pub fn get_usage(&self) -> ResourceUsage {
        ResourceUsage {
            elapsed: self.start_time.elapsed(),
            fuel_consumed: self.fuel_consumed,
            memory_used: self.memory_used,
            http_requests: self.http_requests,
            entity_ops: self.entity_ops,
        }
    }
}

/// Resource usage summary
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub elapsed: Duration,
    pub fuel_consumed: u64,
    pub memory_used: usize,
    pub http_requests: u32,
    pub entity_ops: u32,
}

/// Resource limit errors
#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("Execution timed out after {elapsed:?} (limit: {limit:?})")]
    Timeout {
        elapsed: Duration,
        limit: Duration,
    },
    
    #[error("Fuel exhausted: {consumed} instructions consumed (limit: {limit})")]
    FuelExhausted {
        consumed: u64,
        limit: u64,
    },
    
    #[error("Memory limit exceeded: {used} bytes used (limit: {limit})")]
    MemoryExceeded {
        used: usize,
        limit: usize,
    },
    
    #[error("HTTP request limit exceeded: {count} requests (limit: {limit})")]
    HttpLimitExceeded {
        count: u32,
        limit: u32,
    },
    
    #[error("Entity operation limit exceeded: {count} operations (limit: {limit})")]
    EntityOpsExceeded {
        count: u32,
        limit: u32,
    },
}

/// Plugin execution sandbox
pub struct PluginSandbox {
    limits: ResourceLimits,
    url_allowlist: Vec<String>,
}

impl PluginSandbox {
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            limits,
            url_allowlist: vec![
                "https://api.stripe.com".to_string(),
                "https://api.twilio.com".to_string(),
                "https://api.sendgrid.com".to_string(),
                "https://graph.facebook.com".to_string(),
            ],
        }
    }
    
    /// Check if URL is allowed for external calls
    pub fn is_url_allowed(&self, url: &str) -> bool {
        self.url_allowlist.iter().any(|allowed| url.starts_with(allowed))
    }
    
    /// Add allowed URL prefix
    pub fn allow_url(&mut self, url_prefix: String) {
        self.url_allowlist.push(url_prefix);
    }
    
    /// Get resource limits
    pub fn limits(&self) -> &ResourceLimits {
        &self.limits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fuel_limit() {
        let limits = ResourceLimits {
            max_fuel: 100,
            ..Default::default()
        };
        let mut tracker = ResourceTracker::new(limits);
        
        assert!(tracker.consume_fuel(50).is_ok());
        assert!(tracker.consume_fuel(40).is_ok());
        assert!(tracker.consume_fuel(20).is_err());
    }
    
    #[test]
    fn test_url_allowlist() {
        let sandbox = PluginSandbox::new(ResourceLimits::untrusted());
        
        assert!(sandbox.is_url_allowed("https://api.stripe.com/v1/charges"));
        assert!(!sandbox.is_url_allowed("https://malicious.com/data"));
    }
}
