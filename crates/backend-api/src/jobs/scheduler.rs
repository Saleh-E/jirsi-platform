//! Job Scheduler - Time-based job scheduling
//!
//! Runs maintenance jobs at specific times:
//! - Snapshot cleanup: 2 AM daily (low-traffic hours)
//! - Scheduled triggers: Every minute

use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{Duration, interval_at, Instant};
use tracing::{info, error, warn};
use chrono::{Utc, Timelike};

use super::snapshot_cleanup::{SnapshotCleanupJob, SnapshotCleanupConfig};
use super::scheduled_trigger_runner::{ScheduledTriggerRunner, ScheduledTriggerConfig};
use crate::workflow_trigger::WorkflowTriggerService;

/// Main scheduler configuration
pub struct SchedulerConfig {
    /// Hour to run cleanup (0-23, default: 2 AM)
    pub cleanup_hour: u32,
    /// Minute to run cleanup (0-59, default: 0)
    pub cleanup_minute: u32,
    /// Enable scheduled jobs
    pub enable_snapshot_cleanup: bool,
    /// Enable trigger runner  
    pub enable_scheduled_triggers: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            cleanup_hour: 2,
            cleanup_minute: 0,
            enable_snapshot_cleanup: true,
            enable_scheduled_triggers: true,
        }
    }
}

/// Main job scheduler
pub struct JobScheduler {
    pool: PgPool,
    config: SchedulerConfig,
    trigger_service: Option<Arc<WorkflowTriggerService>>,
}

impl JobScheduler {
    /// Create new job scheduler
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            config: SchedulerConfig::default(),
            trigger_service: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(pool: PgPool, config: SchedulerConfig) -> Self {
        Self {
            pool,
            config,
            trigger_service: None,
        }
    }

    /// Set trigger service for scheduled workflow execution
    pub fn with_trigger_service(mut self, service: Arc<WorkflowTriggerService>) -> Self {
        self.trigger_service = Some(service);
        self
    }

    /// Start all scheduled jobs
    pub async fn start(self) {
        info!(
            cleanup_hour = self.config.cleanup_hour,
            "Starting job scheduler"
        );

        let pool = self.pool.clone();
        let config = self.config;
        let trigger_service = self.trigger_service;

        // Spawn snapshot cleanup task
        if config.enable_snapshot_cleanup {
            let cleanup_pool = pool.clone();
            let cleanup_hour = config.cleanup_hour;
            let cleanup_minute = config.cleanup_minute;
            
            tokio::spawn(async move {
                run_daily_cleanup(cleanup_pool, cleanup_hour, cleanup_minute).await;
            });
        }

        // Spawn scheduled trigger runner
        if config.enable_scheduled_triggers {
            if let Some(service) = trigger_service {
                let trigger_pool = pool.clone();
                let trigger_config = ScheduledTriggerConfig::default();
                
                tokio::spawn(async move {
                    let runner = ScheduledTriggerRunner::with_config(
                        trigger_pool,
                        service,
                        trigger_config,
                    );
                    runner.start().await;
                });
            } else {
                warn!("Scheduled triggers enabled but no WorkflowTriggerService provided");
            }
        }

        // Keep main scheduler alive
        info!("Job scheduler started successfully");
    }
}

/// Run daily cleanup at specified hour
async fn run_daily_cleanup(pool: PgPool, target_hour: u32, target_minute: u32) {
    loop {
        // Calculate time until next cleanup
        let now = Utc::now();
        let current_hour = now.hour();
        let current_minute = now.minute();
        
        let hours_until = if current_hour < target_hour || 
            (current_hour == target_hour && current_minute < target_minute) {
            target_hour - current_hour
        } else {
            24 - current_hour + target_hour
        };
        
        let minutes_until = if current_minute <= target_minute {
            target_minute - current_minute
        } else {
            60 - current_minute + target_minute
        };

        let total_seconds = (hours_until * 3600 + minutes_until * 60) as u64;
        
        info!(
            hours_until = hours_until,
            minutes_until = minutes_until,
            "Snapshot cleanup scheduled"
        );

        // Sleep until cleanup time
        tokio::time::sleep(Duration::from_secs(total_seconds)).await;

        // Run cleanup
        info!("Starting scheduled snapshot cleanup");
        
        let cleanup_job = SnapshotCleanupJob::with_config(
            pool.clone(),
            SnapshotCleanupConfig {
                cleanup_interval_secs: 0, // One-shot mode
                max_snapshots_per_aggregate: 3,
                max_age_days: 30,
            }
        );

        match cleanup_job.run_cleanup().await {
            Ok((old_deleted, excess_deleted)) => {
                info!(
                    old_deleted = old_deleted,
                    excess_deleted = excess_deleted,
                    "Snapshot cleanup completed successfully"
                );
            }
            Err(e) => {
                error!(error = %e, "Snapshot cleanup failed");
            }
        }

        // Sleep at least 1 hour before checking again
        // This prevents multiple runs if the calculation drifts
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}

/// Utility to start the scheduler from main.rs
pub async fn start_background_jobs(pool: PgPool, trigger_service: Option<Arc<WorkflowTriggerService>>) {
    let mut scheduler = JobScheduler::new(pool);
    
    if let Some(service) = trigger_service {
        scheduler = scheduler.with_trigger_service(service);
    }

    tokio::spawn(async move {
        scheduler.start().await;
    });
}
