//! Observability - Monitoring, Metrics, and Logging

use axum::{
    extract::State,
    http::StatusCode,
    Json, Router,
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::Utc;

pub mod metrics;
pub mod tracing_config;

use crate::state::AppState;

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: String,
    pub checks: HealthChecks,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthChecks {
    pub database: CheckStatus,
    pub redis: CheckStatus,
    pub job_queue: CheckStatus,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health check endpoint
async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, (StatusCode, String)> {
    let start_time = std::time::SystemTime::UNIX_EPOCH;
    let uptime = std::time::SystemTime::now()
        .duration_since(start_time)
        .unwrap()
        .as_secs();
    
    
    // Check database
    let db_status = match sqlx::query("SELECT 1").fetch_one(&state.pool).await {
        Ok(_) => CheckStatus::Healthy,
        Err(_) => CheckStatus::Unhealthy,
    };
    
    // Redis status check - degraded since we don't have direct cache access in AppState
    let redis_status = CheckStatus::Degraded;
    
    // Check job queue
    let job_status = CheckStatus::Healthy; // TODO: Implement actual check
    
    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        timestamp: Utc::now().to_rfc3339(),
        checks: HealthChecks {
            database: db_status,
            redis: redis_status,
            job_queue: job_status,
        },
    }))
}

/// Readiness check (K8s)
async fn readiness_check(
    State(state): State<AppState>,
) -> Result<&'static str, StatusCode> {
    // Check if database is accessible
    sqlx::query("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    
    Ok("ready")
}

/// Liveness check (K8s)
async fn liveness_check() -> &'static str {
    "alive"
}

/// Observability routes
pub fn observability_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
        .route("/metrics", get(metrics::metrics_handler))
}
