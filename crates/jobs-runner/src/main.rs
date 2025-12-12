//! Background Jobs Worker
//!
//! Processes async jobs like node graph executions, email sending, etc.

use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod jobs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "jobs_runner=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting jobs worker");

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/saas_platform".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("Connected to database");

    // Job processing loop
    loop {
        // Check for pending jobs
        match jobs::process_pending_jobs(&pool).await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("Processed {} jobs", count);
                }
            }
            Err(e) => {
                tracing::error!("Error processing jobs: {}", e);
            }
        }

        // Sleep before next check
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
