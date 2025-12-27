//! Snapshot Cleanup Job
//!
//! Periodically removes old snapshots to prevent database bloat.

use sqlx::PgPool;
use tokio::time::{Duration, interval};
use tracing::{info, warn, error};
use uuid::Uuid;

/// Snapshot cleanup configuration
pub struct SnapshotCleanupConfig {
    /// How often to run cleanup (default: 24 hours)
    pub cleanup_interval_secs: u64,
    /// Maximum snapshots to keep per aggregate (default: 3)
    pub max_snapshots_per_aggregate: i64,
    /// Delete snapshots older than this many days (default: 30)
    pub max_age_days: i64,
}

impl Default for SnapshotCleanupConfig {
    fn default() -> Self {
        Self {
            cleanup_interval_secs: 24 * 60 * 60, // 24 hours
            max_snapshots_per_aggregate: 3,
            max_age_days: 30,
        }
    }
}

/// Snapshot cleanup job
pub struct SnapshotCleanupJob {
    pool: PgPool,
    config: SnapshotCleanupConfig,
}

impl SnapshotCleanupJob {
    /// Create new cleanup job with default config
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            config: SnapshotCleanupConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(pool: PgPool, config: SnapshotCleanupConfig) -> Self {
        Self { pool, config }
    }

    /// Start the background cleanup loop
    pub async fn start(self) {
        info!(
            interval_hours = self.config.cleanup_interval_secs / 3600,
            "Starting snapshot cleanup job"
        );

        let mut ticker = interval(Duration::from_secs(self.config.cleanup_interval_secs));

        loop {
            ticker.tick().await;

            match self.run_cleanup().await {
                Ok((old_deleted, excess_deleted)) => {
                    if old_deleted > 0 || excess_deleted > 0 {
                        info!(
                            old_deleted = old_deleted,
                            excess_deleted = excess_deleted,
                            "Snapshot cleanup completed"
                        );
                    }
                }
                Err(e) => {
                    error!(error = %e, "Snapshot cleanup failed");
                }
            }
        }
    }

    /// Run a single cleanup pass
    pub async fn run_cleanup(&self) -> Result<(u64, u64), String> {
        let old_deleted = self.delete_old_snapshots().await?;
        let excess_deleted = self.delete_excess_snapshots().await?;
        Ok((old_deleted, excess_deleted))
    }

    /// Delete snapshots older than max_age_days
    async fn delete_old_snapshots(&self) -> Result<u64, String> {
        let result = sqlx::query(
            r#"
            DELETE FROM aggregate_snapshots
            WHERE created_at < NOW() - INTERVAL '1 day' * $1
            "#
        )
        .bind(self.config.max_age_days)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete old snapshots: {}", e))?;

        Ok(result.rows_affected())
    }

    /// Delete excess snapshots beyond max_snapshots_per_aggregate
    async fn delete_excess_snapshots(&self) -> Result<u64, String> {
        // Find aggregates with too many snapshots
        let aggregates: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT aggregate_id
            FROM aggregate_snapshots
            GROUP BY aggregate_id
            HAVING COUNT(*) > $1
            "#
        )
        .bind(self.config.max_snapshots_per_aggregate)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find aggregates: {}", e))?;

        let mut total_deleted = 0u64;

        for (aggregate_id,) in aggregates {
            // Delete all but the N most recent snapshots
            let result = sqlx::query(
                r#"
                DELETE FROM aggregate_snapshots
                WHERE aggregate_id = $1
                  AND id NOT IN (
                      SELECT id FROM aggregate_snapshots
                      WHERE aggregate_id = $1
                      ORDER BY version DESC
                      LIMIT $2
                  )
                "#
            )
            .bind(aggregate_id)
            .bind(self.config.max_snapshots_per_aggregate)
            .execute(&self.pool)
            .await;

            match result {
                Ok(r) => total_deleted += r.rows_affected(),
                Err(e) => {
                    warn!(
                        aggregate_id = %aggregate_id,
                        error = %e,
                        "Failed to cleanup snapshots for aggregate"
                    );
                }
            }
        }

        Ok(total_deleted)
    }
}

/// Batch snapshot creation for aggregates without recent snapshots
pub struct BatchSnapshotCreator {
    pool: PgPool,
    /// Maximum events since last snapshot before creating a new one
    snapshot_threshold: u64,
}

impl BatchSnapshotCreator {
    pub fn new(pool: PgPool, snapshot_threshold: u64) -> Self {
        Self {
            pool,
            snapshot_threshold,
        }
    }

    /// Create snapshots for aggregates that need them
    pub async fn create_missing_snapshots(&self) -> Result<u64, String> {
        // Find aggregates that have many events since last snapshot
        let aggregates: Vec<(Uuid, i64)> = sqlx::query_as(
            r#"
            SELECT 
                e.aggregate_id,
                COUNT(*) as event_count
            FROM events e
            LEFT JOIN aggregate_snapshots s ON e.aggregate_id = s.aggregate_id
            WHERE s.id IS NULL OR e.aggregate_version > COALESCE(s.version, 0)
            GROUP BY e.aggregate_id
            HAVING COUNT(*) > $1
            ORDER BY COUNT(*) DESC
            LIMIT 100
            "#
        )
        .bind(self.snapshot_threshold as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find aggregates needing snapshots: {}", e))?;

        info!(
            count = aggregates.len(),
            "Found aggregates needing snapshots"
        );

        // Note: Actual snapshot creation would require loading and replaying events
        // This is handled by EventStore.create_snapshot()
        // Here we just identify the work needed

        Ok(aggregates.len() as u64)
    }
}
