//! Backend API Server

use axum::{
    Router,
    routing::{get, post},
    extract::State,
    Json,
    response::IntoResponse,
    middleware as axum_middleware,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod routes;
mod state;
mod error;
mod seed;
mod middleware;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "backend_api=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::from_env();

    // Create database pool
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Connected to database");

    // Run migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await?;

    tracing::info!("Migrations complete");

    // Create app state
    let state = Arc::new(AppState::new(pool));

    // Build public routes with tenant middleware
    let public_routes = routes::public::routes()
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::tenant::resolve_tenant,
        ));

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        // Seed endpoint (dev only)
        .route("/seed", post(seed_data))
        // Public routes (with tenant resolution middleware)
        .nest("/public", public_routes)
        // Webhook routes (public, no auth, signature validated)
        .merge(routes::webhooks::routes())
        // API routes (authenticated)
        .nest("/api/v1", routes::api_routes())
        // Add state
        .with_state(state)
        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn seed_data(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match seed::seed_database(&state.pool).await {
        Ok(result) => Json(serde_json::json!({
            "success": true,
            "message": "Database seeded successfully",
            "tenant_id": result.tenant_id,
            "tenant_subdomain": result.tenant_subdomain,
            "admin_email": result.admin_email,
            "admin_password": result.admin_password
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}
