//! Structured Logging Configuration

use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Initialize structured logging
pub fn init_tracing() {
    // Environment filter for log levels
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,backend_api=debug,sqlx=warn"));
    
    // JSON format for production
    let json_layer = if std::env::var("LOG_FORMAT").unwrap_or_default() == "json" {
        Some(
            fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(true)
        )
    } else {
        None
    };
    
    // Pretty format for development
    let pretty_layer = if json_layer.is_none() {
        Some(
            fmt::layer()
                .pretty()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
        )
    } else {
        None
    };
    
    // Build subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(json_layer)
        .with(pretty_layer)
        .init();
    
    tracing::info!("Tracing initialized");
}

/// Log request middleware
use axum::{
    extract::{Request},
    middleware::Next,
    response::Response,
};

pub async fn logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();
    
    // Create span for this request
    let span = tracing::info_span!(
        "http_request",
        method = %method,
        uri = %uri,
    );
    
    let _guard = span.enter();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    tracing::info!(
        status = %status,
        duration_ms = duration.as_millis(),
        "Request completed"
    );
    
    response
}
