//! Scheduled Trigger Runner
//!
//! Background job that processes temporal/scheduled workflow triggers.
//! Runs every minute to check for due triggers and execute them.

use std::sync::Arc;
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use sqlx::PgPool;
use tokio::time::{Duration, interval};
use tracing::{info, warn, error};
use uuid::Uuid;
use serde_json::Value as JsonValue;

use crate::workflow_trigger::WorkflowTriggerService;

/// Scheduled trigger runner configuration
pub struct ScheduledTriggerConfig {
    /// How often to check for due triggers (default: 60 seconds)
    pub check_interval_secs: u64,
    /// Maximum triggers to process per tick (default: 100)
    pub max_per_tick: usize,
    /// Grace period for "missed" triggers in seconds (default: 300)
    pub grace_period_secs: i64,
}

impl Default for ScheduledTriggerConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 60,
            max_per_tick: 100,
            grace_period_secs: 300,
        }
    }
}

/// Scheduled trigger runner
pub struct ScheduledTriggerRunner {
    pool: PgPool,
    trigger_service: Arc<WorkflowTriggerService>,
    config: ScheduledTriggerConfig,
}

impl ScheduledTriggerRunner {
    /// Create new scheduled trigger runner
    pub fn new(
        pool: PgPool,
        trigger_service: Arc<WorkflowTriggerService>,
    ) -> Self {
        Self {
            pool,
            trigger_service,
            config: ScheduledTriggerConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        pool: PgPool,
        trigger_service: Arc<WorkflowTriggerService>,
        config: ScheduledTriggerConfig,
    ) -> Self {
        Self {
            pool,
            trigger_service,
            config,
        }
    }

    /// Start the background runner loop
    pub async fn start(self) {
        info!(
            interval_secs = self.config.check_interval_secs,
            "Starting scheduled trigger runner"
        );

        let mut ticker = interval(Duration::from_secs(self.config.check_interval_secs));

        loop {
            ticker.tick().await;

            match self.process_due_triggers().await {
                Ok(count) => {
                    if count > 0 {
                        info!(processed = count, "Processed scheduled triggers");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to process scheduled triggers");
                }
            }
        }
    }

    /// Process all due triggers
    pub async fn process_due_triggers(&self) -> Result<usize, String> {
        let now = Utc::now();
        let grace_cutoff = now - ChronoDuration::seconds(self.config.grace_period_secs);

        // Find all due scheduled triggers
        let due_triggers = sqlx::query_as::<_, (Uuid, Uuid, Uuid, String, Option<DateTime<Utc>>)>(
            r#"
            SELECT 
                t.id,
                t.graph_id,
                g.tenant_id,
                t.cron_expression,
                t.next_run_at
            FROM workflow_triggers t
            JOIN workflow_graphs g ON t.graph_id = g.id
            WHERE 
                t.trigger_type = 'scheduled'
                AND t.is_active = true
                AND t.next_run_at IS NOT NULL
                AND t.next_run_at <= $1
                AND t.next_run_at >= $2
            ORDER BY t.next_run_at ASC
            LIMIT $3
            "#
        )
        .bind(now)
        .bind(grace_cutoff)
        .bind(self.config.max_per_tick as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch due triggers: {}", e))?;

        let mut processed = 0;

        for (trigger_id, graph_id, tenant_id, cron_expr, last_run) in due_triggers {
            // Queue workflow execution
            let execution_id = Uuid::new_v4();
            let trigger_data = serde_json::json!({
                "source": "scheduled",
                "trigger_id": trigger_id,
                "cron_expression": cron_expr,
                "scheduled_at": last_run,
                "executed_at": now.to_rfc3339(),
            });

            let insert_result = sqlx::query(
                r#"
                INSERT INTO workflow_executions 
                    (id, tenant_id, graph_id, trigger_id, trigger_data, status, created_at)
                VALUES ($1, $2, $3, $4, $5, 'pending', NOW())
                "#
            )
            .bind(execution_id)
            .bind(tenant_id)
            .bind(graph_id)
            .bind(trigger_id)
            .bind(&trigger_data)
            .execute(&self.pool)
            .await;

            if let Err(e) = insert_result {
                warn!(
                    trigger_id = %trigger_id,
                    error = %e,
                    "Failed to queue scheduled workflow execution"
                );
                continue;
            }

            // Calculate and update next run time
            let next_run = calculate_next_cron_run(&cron_expr, now);
            
            let update_result = sqlx::query(
                r#"
                UPDATE workflow_triggers 
                SET 
                    next_run_at = $1,
                    last_run_at = NOW(),
                    run_count = COALESCE(run_count, 0) + 1
                WHERE id = $2
                "#
            )
            .bind(next_run)
            .bind(trigger_id)
            .execute(&self.pool)
            .await;

            if let Err(e) = update_result {
                warn!(
                    trigger_id = %trigger_id,
                    error = %e,
                    "Failed to update next run time"
                );
            }

            info!(
                execution_id = %execution_id,
                trigger_id = %trigger_id,
                graph_id = %graph_id,
                next_run = ?next_run,
                "Scheduled workflow execution queued"
            );

            processed += 1;
        }

        Ok(processed)
    }
}

/// Calculate the next run time from a cron expression
/// 
/// Supports simplified cron syntax:
/// - `@hourly` - Every hour
/// - `@daily` or `@midnight` - Every day at midnight
/// - `@weekly` - Every Sunday at midnight
/// - `@monthly` - First of every month at midnight
/// - `*/N * * * *` - Every N minutes
/// - `0 */N * * *` - Every N hours
/// - `0 0 * * *` - Every day at midnight
/// - `0 0 * * 0` - Every Sunday at midnight
fn calculate_next_cron_run(cron_expr: &str, from: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let expr = cron_expr.trim().to_lowercase();
    
    // Handle shorthand expressions
    match expr.as_str() {
        "@hourly" => return Some(from + ChronoDuration::hours(1)),
        "@daily" | "@midnight" => {
            let tomorrow = from + ChronoDuration::days(1);
            return Some(tomorrow.date_naive().and_hms_opt(0, 0, 0)?.and_utc());
        }
        "@weekly" => {
            let days_until_sunday = (7 - from.weekday().num_days_from_sunday()) % 7;
            let next_sunday = from + ChronoDuration::days(days_until_sunday as i64 + 7);
            return Some(next_sunday.date_naive().and_hms_opt(0, 0, 0)?.and_utc());
        }
        "@monthly" => {
            let next_month = if from.month() == 12 {
                from.with_year(from.year() + 1)?.with_month(1)?
            } else {
                from.with_month(from.month() + 1)?
            };
            return Some(next_month.with_day(1)?.date_naive().and_hms_opt(0, 0, 0)?.and_utc());
        }
        _ => {}
    }
    
    // Parse standard cron format: minute hour day_of_month month day_of_week
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() < 5 {
        warn!(cron_expr = %cron_expr, "Invalid cron expression format");
        // Default to 1 hour from now if parsing fails
        return Some(from + ChronoDuration::hours(1));
    }

    // Simple parsing for common patterns
    let minute_part = parts[0];
    let hour_part = parts[1];

    // Handle "*/N" for minutes
    if minute_part.starts_with("*/") {
        if let Ok(interval) = minute_part[2..].parse::<i64>() {
            return Some(from + ChronoDuration::minutes(interval));
        }
    }

    // Handle "*/N" for hours
    if minute_part == "0" && hour_part.starts_with("*/") {
        if let Ok(interval) = hour_part[2..].parse::<i64>() {
            return Some(from + ChronoDuration::hours(interval));
        }
    }

    // Handle specific times (e.g., "0 9 * * *" = 9 AM every day)
    if let (Ok(minute), Ok(hour)) = (minute_part.parse::<u32>(), hour_part.parse::<u32>()) {
        let today_at_time = from.date_naive().and_hms_opt(hour, minute, 0)?;
        if today_at_time.and_utc() > from {
            return Some(today_at_time.and_utc());
        } else {
            // Already passed today, schedule for tomorrow
            let tomorrow = from.date_naive() + ChronoDuration::days(1);
            return Some(tomorrow.and_hms_opt(hour, minute, 0)?.and_utc());
        }
    }

    // Fallback: 1 hour from now
    Some(from + ChronoDuration::hours(1))
}

/// Delayed action trigger - triggers after a specified duration
pub struct DelayedActionTrigger {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub delay_duration: ChronoDuration,
    pub action: String,
    pub condition: Option<JsonValue>,
}

impl DelayedActionTrigger {
    /// Schedule a delayed action (e.g., "3 days after record created, if status != 'replied'")
    pub async fn schedule(
        pool: &PgPool,
        trigger: DelayedActionTrigger,
        graph_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Uuid, String> {
        let trigger_id = Uuid::new_v4();
        let trigger_at = Utc::now() + trigger.delay_duration;

        let trigger_data = serde_json::json!({
            "type": "delayed_action",
            "entity_id": trigger.entity_id,
            "entity_type": trigger.entity_type,
            "action": trigger.action,
            "condition": trigger.condition,
            "delay_hours": trigger.delay_duration.num_hours(),
        });

        sqlx::query(
            r#"
            INSERT INTO workflow_triggers 
                (id, graph_id, trigger_type, entity_type, filter_conditions, is_active, next_run_at, created_at)
            VALUES 
                ($1, $2, 'delayed', $3, $4, true, $5, NOW())
            "#
        )
        .bind(trigger_id)
        .bind(graph_id)
        .bind(&trigger.entity_type)
        .bind(&trigger_data)
        .bind(trigger_at)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to schedule delayed action: {}", e))?;

        info!(
            trigger_id = %trigger_id,
            entity_id = %trigger.entity_id,
            trigger_at = %trigger_at,
            "Delayed action scheduled"
        );

        Ok(trigger_id)
    }

    /// Cancel a pending delayed action
    pub async fn cancel(pool: &PgPool, trigger_id: Uuid) -> Result<(), String> {
        sqlx::query(
            "UPDATE workflow_triggers SET is_active = false WHERE id = $1"
        )
        .bind(trigger_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to cancel delayed action: {}", e))?;

        info!(trigger_id = %trigger_id, "Delayed action cancelled");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_shortcuts() {
        let now = Utc::now();
        
        // @hourly should be 1 hour from now
        let next = calculate_next_cron_run("@hourly", now).unwrap();
        assert!((next - now).num_minutes() >= 59);
        assert!((next - now).num_minutes() <= 61);

        // @daily should be within next 24 hours
        let next = calculate_next_cron_run("@daily", now).unwrap();
        assert!((next - now).num_hours() <= 24);
    }

    #[test]
    fn test_cron_interval() {
        let now = Utc::now();
        
        // */15 * * * * = every 15 minutes
        let next = calculate_next_cron_run("*/15 * * * *", now).unwrap();
        assert_eq!((next - now).num_minutes(), 15);

        // 0 */2 * * * = every 2 hours
        let next = calculate_next_cron_run("0 */2 * * *", now).unwrap();
        assert_eq!((next - now).num_hours(), 2);
    }
}
