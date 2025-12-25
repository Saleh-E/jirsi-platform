//! API Gateway - Rate Limiting & Middleware

use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    limits: Arc<RwLock<HashMap<String, TokenBucket>>>,
    max_requests: u32,
    window_secs: i64,
}

struct TokenBucket {
    tokens: u32,
    last_refill: DateTime<Utc>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: i64) -> Self {
        Self {
            limits: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_secs,
        }
    }
    
    pub async fn check_limit(&self, key: &str) -> bool {
        let mut limits = self.limits.write().await;
        
        let bucket = limits.entry(key.to_string()).or_insert(TokenBucket {
            tokens: self.max_requests,
            last_refill: Utc::now(),
        });
        
        // Refill tokens based on time passed
        let now = Utc::now();
        let elapsed = (now - bucket.last_refill).num_seconds();
        
        if elapsed >= self.window_secs {
            bucket.tokens = self.max_requests;
            bucket.last_refill = now;
        } else {
            // Gradual refill
            let tokens_to_add = ((elapsed as f64 / self.window_secs as f64) * self.max_requests as f64) as u32;
            bucket.tokens = (bucket.tokens + tokens_to_add).min(self.max_requests);
        }
        
        // Check and consume token
        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            true
        } else {
            false
        }
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract client identifier (IP or API key)
    let client_id = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous");
    
    // Check rate limit
    if !limiter.check_limit(client_id).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    Ok(next.run(request).await)
}

/// Request metrics collection
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Metrics {
    total_requests: AtomicU64,
    failed_requests: AtomicU64,
    total_duration_ms: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_duration_ms: AtomicU64::new(0),
        }
    }
    
    pub fn record_request(&self, duration_ms: u64, is_error: bool) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        
        if is_error {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    pub fn get_stats(&self) -> MetricsSnapshot {
        let total = self.total_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let duration = self.total_duration_ms.load(Ordering::Relaxed);
        
        MetricsSnapshot {
            total_requests: total,
            failed_requests: failed,
            success_rate: if total > 0 {
                ((total - failed) as f64 / total as f64) * 100.0
            } else {
                100.0
            },
            avg_response_time_ms: if total > 0 {
                duration / total
            } else {
                0
            },
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: u64,
}

/// Metrics middleware
pub async fn metrics_middleware(
    State(metrics): State<Arc<Metrics>>,
    request: Request,
    next: Next,
) -> Response {
    let start = std::time::Instant::now();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed().as_millis() as u64;
    let is_error = response.status().is_client_error() || response.status().is_server_error();
    
    metrics.record_request(duration, is_error);
    
    response
}
