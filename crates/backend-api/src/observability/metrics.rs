//! Prometheus Metrics Exporter - Enhanced for Production
//!
//! Tracks workflow execution, sync latency, API response times,
//! and provides structured logging for critical failures.

use axum::{
    http::StatusCode,
    response::IntoResponse,
    extract::State,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Application metrics
pub struct AppMetrics {
    // Request metrics
    pub http_requests_total: AtomicU64,
    pub http_requests_duration_ms: AtomicU64,
    pub http_requests_errors: AtomicU64,
    
    // Per-endpoint response times (avg in ms)
    pub endpoint_response_times: Arc<RwLock<HashMap<String, EndpointMetrics>>>,
    
    // Database metrics
    pub db_queries_total: AtomicU64,
    pub db_queries_duration_ms: AtomicU64,
    pub db_connections_active: AtomicU64,
    
    // Cache metrics
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    
    // Job queue metrics
    pub jobs_queued: AtomicU64,
    pub jobs_processed: AtomicU64,
    pub jobs_failed: AtomicU64,
    
    // Event store metrics
    pub events_appended: AtomicU64,
    pub events_replayed: AtomicU64,
    pub snapshots_created: AtomicU64,
    pub snapshots_loaded: AtomicU64,
    
    // Workflow execution metrics
    pub workflow_executions_total: AtomicU64,
    pub workflow_executions_success: AtomicU64,
    pub workflow_executions_failed: AtomicU64,
    pub workflow_execution_duration_ms: AtomicU64,
    pub workflow_nodes_executed: AtomicU64,
    
    // Sync metrics
    pub sync_push_total: AtomicU64,
    pub sync_push_success: AtomicU64,
    pub sync_push_conflicts: AtomicU64,
    pub sync_pull_total: AtomicU64,
    pub sync_latency_ms: AtomicU64,
    
    // WebSocket metrics
    pub ws_connections_active: AtomicU64,
    pub ws_messages_sent: AtomicU64,
    pub ws_messages_received: AtomicU64,
}

#[derive(Default, Clone)]
pub struct EndpointMetrics {
    pub count: u64,
    pub total_duration_ms: u64,
    pub errors: u64,
}

impl EndpointMetrics {
    pub fn avg_duration_ms(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.total_duration_ms as f64 / self.count as f64
        }
    }
    
    pub fn error_rate(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            (self.errors as f64 / self.count as f64) * 100.0
        }
    }
}

impl AppMetrics {
    pub fn new() -> Self {
        Self {
            http_requests_total: AtomicU64::new(0),
            http_requests_duration_ms: AtomicU64::new(0),
            http_requests_errors: AtomicU64::new(0),
            endpoint_response_times: Arc::new(RwLock::new(HashMap::new())),
            db_queries_total: AtomicU64::new(0),
            db_queries_duration_ms: AtomicU64::new(0),
            db_connections_active: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            jobs_queued: AtomicU64::new(0),
            jobs_processed: AtomicU64::new(0),
            jobs_failed: AtomicU64::new(0),
            events_appended: AtomicU64::new(0),
            events_replayed: AtomicU64::new(0),
            snapshots_created: AtomicU64::new(0),
            snapshots_loaded: AtomicU64::new(0),
            workflow_executions_total: AtomicU64::new(0),
            workflow_executions_success: AtomicU64::new(0),
            workflow_executions_failed: AtomicU64::new(0),
            workflow_execution_duration_ms: AtomicU64::new(0),
            workflow_nodes_executed: AtomicU64::new(0),
            sync_push_total: AtomicU64::new(0),
            sync_push_success: AtomicU64::new(0),
            sync_push_conflicts: AtomicU64::new(0),
            sync_pull_total: AtomicU64::new(0),
            sync_latency_ms: AtomicU64::new(0),
            ws_connections_active: AtomicU64::new(0),
            ws_messages_sent: AtomicU64::new(0),
            ws_messages_received: AtomicU64::new(0),
        }
    }
    
    // ============ Recording Methods ============
    
    /// Record an HTTP request
    pub fn record_request(&self, endpoint: &str, duration_ms: u64, is_error: bool) {
        self.http_requests_total.fetch_add(1, Ordering::Relaxed);
        self.http_requests_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        
        if is_error {
            self.http_requests_errors.fetch_add(1, Ordering::Relaxed);
        }
        
        // Update per-endpoint metrics (async-safe)
        tokio::spawn({
            let endpoint = endpoint.to_string();
            let endpoint_times = self.endpoint_response_times.clone();
            async move {
                let mut map = endpoint_times.write().await;
                let entry = map.entry(endpoint).or_default();
                entry.count += 1;
                entry.total_duration_ms += duration_ms;
                if is_error {
                    entry.errors += 1;
                }
            }
        });
    }
    
    /// Record workflow execution
    pub fn record_workflow_execution(&self, success: bool, duration_ms: u64, nodes_executed: u64) {
        self.workflow_executions_total.fetch_add(1, Ordering::Relaxed);
        self.workflow_execution_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        self.workflow_nodes_executed.fetch_add(nodes_executed, Ordering::Relaxed);
        
        if success {
            self.workflow_executions_success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.workflow_executions_failed.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// Record sync operation
    pub fn record_sync_push(&self, success: bool, conflict: bool, latency_ms: u64) {
        self.sync_push_total.fetch_add(1, Ordering::Relaxed);
        self.sync_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
        
        if success {
            self.sync_push_success.fetch_add(1, Ordering::Relaxed);
        }
        if conflict {
            self.sync_push_conflicts.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// Record sync pull
    pub fn record_sync_pull(&self, latency_ms: u64) {
        self.sync_pull_total.fetch_add(1, Ordering::Relaxed);
        self.sync_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }
    
    // ============ Calculated Metrics ============
    
    fn calculate_cache_hit_ratio(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            0.0
        } else {
            (hits as f64 / total as f64) * 100.0
        }
    }
    
    fn workflow_success_rate(&self) -> f64 {
        let total = self.workflow_executions_total.load(Ordering::Relaxed);
        let success = self.workflow_executions_success.load(Ordering::Relaxed);
        
        if total == 0 {
            100.0
        } else {
            (success as f64 / total as f64) * 100.0
        }
    }
    
    fn avg_sync_latency_ms(&self) -> f64 {
        let total = self.sync_push_total.load(Ordering::Relaxed) 
            + self.sync_pull_total.load(Ordering::Relaxed);
        let latency = self.sync_latency_ms.load(Ordering::Relaxed);
        
        if total == 0 {
            0.0
        } else {
            latency as f64 / total as f64
        }
    }
    
    fn avg_response_time_ms(&self) -> f64 {
        let total = self.http_requests_total.load(Ordering::Relaxed);
        let duration = self.http_requests_duration_ms.load(Ordering::Relaxed);
        
        if total == 0 {
            0.0
        } else {
            duration as f64 / total as f64
        }
    }
    
    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        format!(
            r#"# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total {}

# HELP http_requests_duration_avg_ms Average HTTP request duration
# TYPE http_requests_duration_avg_ms gauge
http_requests_duration_avg_ms {:.2}

# HELP http_requests_errors_total Total HTTP request errors
# TYPE http_requests_errors_total counter
http_requests_errors_total {}

# HELP db_queries_total Total database queries
# TYPE db_queries_total counter
db_queries_total {}

# HELP db_connections_active Active database connections
# TYPE db_connections_active gauge
db_connections_active {}

# HELP cache_hit_ratio Cache hit ratio percentage
# TYPE cache_hit_ratio gauge
cache_hit_ratio {:.2}

# HELP jobs_queued Jobs currently in queue
# TYPE jobs_queued gauge
jobs_queued {}

# HELP jobs_processed_total Total jobs processed
# TYPE jobs_processed_total counter
jobs_processed_total {}

# HELP jobs_failed_total Total jobs failed
# TYPE jobs_failed_total counter
jobs_failed_total {}

# HELP events_appended_total Total events appended to event store
# TYPE events_appended_total counter
events_appended_total {}

# HELP events_replayed_total Total events replayed
# TYPE events_replayed_total counter
events_replayed_total {}

# HELP snapshots_created_total Aggregate snapshots created
# TYPE snapshots_created_total counter
snapshots_created_total {}

# HELP snapshots_loaded_total Aggregate snapshots loaded
# TYPE snapshots_loaded_total counter
snapshots_loaded_total {}

# HELP workflow_executions_total Total workflow executions
# TYPE workflow_executions_total counter
workflow_executions_total {}

# HELP workflow_success_rate Workflow execution success rate percentage
# TYPE workflow_success_rate gauge
workflow_success_rate {:.2}

# HELP workflow_nodes_executed_total Total workflow nodes executed
# TYPE workflow_nodes_executed_total counter
workflow_nodes_executed_total {}

# HELP sync_push_total Total sync push operations
# TYPE sync_push_total counter
sync_push_total {}

# HELP sync_push_conflicts_total Total sync conflicts
# TYPE sync_push_conflicts_total counter
sync_push_conflicts_total {}

# HELP sync_latency_avg_ms Average sync latency in milliseconds
# TYPE sync_latency_avg_ms gauge
sync_latency_avg_ms {:.2}

# HELP ws_connections_active Active WebSocket connections
# TYPE ws_connections_active gauge
ws_connections_active {}

# HELP ws_messages_total Total WebSocket messages (sent + received)
# TYPE ws_messages_total counter
ws_messages_total {}
"#,
            self.http_requests_total.load(Ordering::Relaxed),
            self.avg_response_time_ms(),
            self.http_requests_errors.load(Ordering::Relaxed),
            self.db_queries_total.load(Ordering::Relaxed),
            self.db_connections_active.load(Ordering::Relaxed),
            self.calculate_cache_hit_ratio(),
            self.jobs_queued.load(Ordering::Relaxed),
            self.jobs_processed.load(Ordering::Relaxed),
            self.jobs_failed.load(Ordering::Relaxed),
            self.events_appended.load(Ordering::Relaxed),
            self.events_replayed.load(Ordering::Relaxed),
            self.snapshots_created.load(Ordering::Relaxed),
            self.snapshots_loaded.load(Ordering::Relaxed),
            self.workflow_executions_total.load(Ordering::Relaxed),
            self.workflow_success_rate(),
            self.workflow_nodes_executed.load(Ordering::Relaxed),
            self.sync_push_total.load(Ordering::Relaxed),
            self.sync_push_conflicts.load(Ordering::Relaxed),
            self.avg_sync_latency_ms(),
            self.ws_connections_active.load(Ordering::Relaxed),
            self.ws_messages_sent.load(Ordering::Relaxed) + self.ws_messages_received.load(Ordering::Relaxed),
        )
    }
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics handler
pub async fn metrics_handler(
    // Extract metrics from state if needed
) -> impl IntoResponse {
    // TODO: Get actual metrics from app state
    let metrics = AppMetrics::new();
    
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        metrics.export_prometheus(),
    )
}

// ============ Structured Logging Helpers ============

/// Log a critical failure with execution context
#[macro_export]
macro_rules! log_critical {
    ($execution_id:expr, $message:expr, $($key:ident = $value:expr),* $(,)?) => {
        tracing::error!(
            execution_id = %$execution_id,
            severity = "CRITICAL",
            $($key = ?$value,)*
            $message
        );
    };
}

/// Log a workflow failure with full context
#[macro_export]
macro_rules! log_workflow_failure {
    ($execution_id:expr, $workflow_id:expr, $node_id:expr, $error:expr) => {
        tracing::error!(
            execution_id = %$execution_id,
            workflow_id = %$workflow_id,
            node_id = %$node_id,
            error = %$error,
            severity = "ERROR",
            category = "workflow",
            "Workflow node execution failed"
        );
    };
}

/// Log a sync conflict
#[macro_export]
macro_rules! log_sync_conflict {
    ($entity_type:expr, $entity_id:expr, $expected_version:expr, $actual_version:expr) => {
        tracing::warn!(
            entity_type = $entity_type,
            entity_id = %$entity_id,
            expected_version = $expected_version,
            actual_version = $actual_version,
            category = "sync",
            "Sync conflict detected"
        );
    };
}

/// Request timing guard - auto-logs on drop
pub struct RequestTimer {
    start: Instant,
    endpoint: String,
    metrics: Arc<AppMetrics>,
}

impl RequestTimer {
    pub fn new(endpoint: &str, metrics: Arc<AppMetrics>) -> Self {
        Self {
            start: Instant::now(),
            endpoint: endpoint.to_string(),
            metrics,
        }
    }
    
    pub fn complete(self, is_error: bool) {
        let duration = self.start.elapsed().as_millis() as u64;
        self.metrics.record_request(&self.endpoint, duration, is_error);
    }
}

impl Drop for RequestTimer {
    fn drop(&mut self) {
        // Auto-record if not explicitly completed
    }
}

