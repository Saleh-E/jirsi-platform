//! Rate Limiting Middleware
//!
//! Per-tenant rate limiting to prevent abuse and ensure fair usage.

use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per window
    pub requests_per_window: u32,
    /// Window duration in seconds
    pub window_secs: u64,
    /// Burst allowance (extra requests above limit)
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_window: 1000, // 1000 requests
            window_secs: 60,            // per minute
            burst_size: 100,            // with 100 burst
        }
    }
}

/// Tenant rate limit state
#[derive(Debug)]
struct TenantLimitState {
    request_count: u32,
    window_start: Instant,
    burst_remaining: u32,
}

impl TenantLimitState {
    fn new(config: &RateLimitConfig) -> Self {
        Self {
            request_count: 0,
            window_start: Instant::now(),
            burst_remaining: config.burst_size,
        }
    }
}

/// Shared rate limiter state
pub type SharedRateLimiter = Arc<RateLimiter>;

/// Rate limiter
pub struct RateLimiter {
    config: RateLimitConfig,
    tenant_states: RwLock<HashMap<Uuid, TenantLimitState>>,
}

impl RateLimiter {
    /// Create new rate limiter with default config
    pub fn new() -> Self {
        Self::with_config(RateLimitConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            config,
            tenant_states: RwLock::new(HashMap::new()),
        }
    }

    /// Check if request is allowed for tenant
    pub async fn check_limit(&self, tenant_id: Uuid) -> RateLimitResult {
        let mut states = self.tenant_states.write().await;
        
        let state = states.entry(tenant_id)
            .or_insert_with(|| TenantLimitState::new(&self.config));

        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_secs);

        // Check if window has expired
        if now.duration_since(state.window_start) > window_duration {
            // Reset window
            state.request_count = 0;
            state.window_start = now;
            state.burst_remaining = self.config.burst_size;
        }

        state.request_count += 1;

        if state.request_count <= self.config.requests_per_window {
            // Within normal limit
            RateLimitResult::Allowed {
                remaining: self.config.requests_per_window - state.request_count,
                reset_at: state.window_start + window_duration,
            }
        } else if state.burst_remaining > 0 {
            // Using burst allowance
            state.burst_remaining -= 1;
            RateLimitResult::Allowed {
                remaining: 0,
                reset_at: state.window_start + window_duration,
            }
        } else {
            // Rate limited
            let retry_after = window_duration
                .checked_sub(now.duration_since(state.window_start))
                .unwrap_or(Duration::ZERO);

            RateLimitResult::Limited {
                retry_after_secs: retry_after.as_secs(),
            }
        }
    }

    /// Get current usage stats for a tenant
    pub async fn get_usage(&self, tenant_id: Uuid) -> Option<(u32, u32)> {
        let states = self.tenant_states.read().await;
        states.get(&tenant_id).map(|s| (s.request_count, self.config.requests_per_window))
    }

    /// Clear expired entries (call periodically)
    pub async fn cleanup(&self) {
        let mut states = self.tenant_states.write().await;
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_secs * 2); // Keep for 2x window

        states.retain(|_, state| {
            now.duration_since(state.window_start) < window_duration
        });
    }
}

/// Rate limit check result
#[derive(Debug)]
pub enum RateLimitResult {
    Allowed {
        remaining: u32,
        reset_at: Instant,
    },
    Limited {
        retry_after_secs: u64,
    },
}

/// Rate limiting middleware layer
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Response<Body> {
    // Extract tenant ID from request extensions (set by auth middleware)
    let tenant_id = request.extensions()
        .get::<Uuid>()
        .copied();

    let Some(tenant_id) = tenant_id else {
        // No tenant ID - skip rate limiting (might be unauthenticated endpoint)
        return next.run(request).await;
    };

    // Get rate limiter from extensions
    let rate_limiter = request.extensions()
        .get::<SharedRateLimiter>()
        .cloned();

    let Some(rate_limiter) = rate_limiter else {
        // No rate limiter configured
        return next.run(request).await;
    };

    // Check rate limit
    match rate_limiter.check_limit(tenant_id).await {
        RateLimitResult::Allowed { remaining, reset_at } => {
            let mut response = next.run(request).await;
            
            // Add rate limit headers
            let headers = response.headers_mut();
            headers.insert("X-RateLimit-Remaining", remaining.to_string().parse().unwrap());
            headers.insert("X-RateLimit-Limit", rate_limiter.config.requests_per_window.to_string().parse().unwrap());
            
            response
        }
        RateLimitResult::Limited { retry_after_secs } => {
            warn!(
                tenant_id = %tenant_id,
                retry_after = retry_after_secs,
                "Rate limit exceeded"
            );

            let mut response = Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .body(Body::from("Rate limit exceeded"))
                .unwrap();

            response.headers_mut()
                .insert("Retry-After", retry_after_secs.to_string().parse().unwrap());
            
            response
        }
    }
}

/// Rate limit error response
#[derive(Debug, serde::Serialize)]
pub struct RateLimitError {
    pub error: String,
    pub retry_after_secs: u64,
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("Content-Type", "application/json")
            .header("Retry-After", self.retry_after_secs.to_string())
            .body(Body::from(body))
            .unwrap()
    }
}
