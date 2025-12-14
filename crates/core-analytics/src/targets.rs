//! Agent Target Resolution Service
//! Fetches and calculates progress against performance goals

use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Utc, Datelike};

/// A resolved target with calculated progress
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ResolvedTarget {
    pub target_value: f64,
    pub current_value: f64,
    pub progress_percent: f64,
}

/// Metric types for targets
#[derive(Clone, Debug, PartialEq)]
pub enum MetricType {
    Revenue,
    DealsWon,
    LeadsCreated,
    CallsMade,
    ViewingsCompleted,
    ListingsAdded,
}

impl MetricType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricType::Revenue => "revenue",
            MetricType::DealsWon => "deals_won",
            MetricType::LeadsCreated => "leads_created",
            MetricType::CallsMade => "calls_made",
            MetricType::ViewingsCompleted => "viewings_completed",
            MetricType::ListingsAdded => "listings_added",
        }
    }
}

/// Get target for a specific user, metric, and date range
pub async fn get_target_for_user(
    pool: &PgPool,
    tenant_id: &str,
    user_id: Option<&str>,
    metric: MetricType,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Option<f64>, sqlx::Error> {
    // If no user_id, get aggregate target for team
    let query = if user_id.is_some() {
        r#"
        SELECT COALESCE(SUM(target_value), 0)::float8 as total_target
        FROM agent_targets
        WHERE tenant_id = $1::uuid
          AND user_id = $2::uuid
          AND metric_type = $3
          AND start_date <= $5
          AND end_date >= $4
        "#
    } else {
        r#"
        SELECT COALESCE(SUM(target_value), 0)::float8 as total_target
        FROM agent_targets
        WHERE tenant_id = $1::uuid
          AND metric_type = $3
          AND start_date <= $5
          AND end_date >= $4
        "#
    };

    let row = if let Some(uid) = user_id {
        sqlx::query(query)
            .bind(tenant_id)
            .bind(uid)
            .bind(metric.as_str())
            .bind(start_date)
            .bind(end_date)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query(query)
            .bind(tenant_id)
            .bind(tenant_id) // dummy for $2
            .bind(metric.as_str())
            .bind(start_date)
            .bind(end_date)
            .fetch_one(pool)
            .await?
    };

    let total: f64 = row.get("total_target");
    if total > 0.0 {
        Ok(Some(total))
    } else {
        Ok(None)
    }
}

/// Get all team targets for a specific metric (for leaderboard)
pub async fn get_team_targets(
    pool: &PgPool,
    tenant_id: &str,
    metric: MetricType,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<UserTarget>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT 
            at.user_id::text,
            u.email as user_email,
            COALESCE(u.first_name, '') || ' ' || COALESCE(u.last_name, '') as user_name,
            at.target_value::float8
        FROM agent_targets at
        JOIN users u ON at.user_id = u.id
        WHERE at.tenant_id = $1::uuid
          AND at.metric_type = $2
          AND at.start_date <= $4
          AND at.end_date >= $3
        ORDER BY at.target_value DESC
        "#
    )
    .bind(tenant_id)
    .bind(metric.as_str())
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|row| UserTarget {
        user_id: row.get("user_id"),
        user_name: row.get("user_name"),
        user_email: row.get("user_email"),
        target_value: row.get("target_value"),
    }).collect())
}

/// User target info for leaderboard
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserTarget {
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
    pub target_value: f64,
}

/// Agent performance for leaderboard
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentPerformance {
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
    pub revenue: f64,
    pub revenue_target: Option<f64>,
    pub revenue_progress: Option<f64>,
    pub deals_won: i64,
    pub deals_won_target: Option<f64>,
    pub deals_progress: Option<f64>,
    pub leads_created: i64,
    pub conversion_rate: f64,
}

/// Get agent leaderboard with performance vs targets
pub async fn get_agent_leaderboard(
    pool: &PgPool,
    tenant_id: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<AgentPerformance>, sqlx::Error> {
    // Get all agents with their performance
    let rows = sqlx::query(
        r#"
        WITH agent_revenue AS (
            SELECT 
                owner_id,
                COALESCE(SUM(amount), 0)::float8 as revenue,
                COUNT(*) FILTER (WHERE stage IN ('Won', 'Closed Won')) as deals_won
            FROM deals
            WHERE tenant_id = $1::uuid
              AND created_at >= $2
              AND created_at <= $3
              AND owner_id IS NOT NULL
            GROUP BY owner_id
        ),
        agent_leads AS (
            SELECT 
                owner_id,
                COUNT(*) as leads_created
            FROM contacts
            WHERE tenant_id = $1::uuid
              AND lifecycle_stage = 'lead'
              AND created_at >= $2
              AND created_at <= $3
              AND owner_id IS NOT NULL
            GROUP BY owner_id
        ),
        agent_targets_agg AS (
            SELECT 
                user_id,
                MAX(CASE WHEN metric_type = 'revenue' THEN target_value END)::float8 as revenue_target,
                MAX(CASE WHEN metric_type = 'deals_won' THEN target_value END)::float8 as deals_target
            FROM agent_targets
            WHERE tenant_id = $1::uuid
              AND start_date <= $3
              AND end_date >= $2
            GROUP BY user_id
        )
        SELECT 
            u.id::text as user_id,
            COALESCE(u.first_name, '') || ' ' || COALESCE(u.last_name, '') as user_name,
            u.email as user_email,
            COALESCE(ar.revenue, 0) as revenue,
            at.revenue_target,
            CASE WHEN at.revenue_target > 0 
                 THEN (COALESCE(ar.revenue, 0) / at.revenue_target * 100) 
                 ELSE NULL END as revenue_progress,
            COALESCE(ar.deals_won, 0) as deals_won,
            at.deals_target as deals_won_target,
            CASE WHEN at.deals_target > 0 
                 THEN (COALESCE(ar.deals_won, 0) / at.deals_target * 100) 
                 ELSE NULL END as deals_progress,
            COALESCE(al.leads_created, 0) as leads_created,
            CASE WHEN COALESCE(al.leads_created, 0) > 0 
                 THEN (COALESCE(ar.deals_won, 0)::float / al.leads_created * 100)
                 ELSE 0 END as conversion_rate
        FROM users u
        LEFT JOIN agent_revenue ar ON u.id = ar.owner_id
        LEFT JOIN agent_leads al ON u.id = al.owner_id
        LEFT JOIN agent_targets_agg at ON u.id = at.user_id
        WHERE u.tenant_id = $1::uuid
          AND u.role IN ('agent', 'admin', 'manager')
        ORDER BY revenue DESC
        "#
    )
    .bind(tenant_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|row| AgentPerformance {
        user_id: row.get("user_id"),
        user_name: row.get("user_name"),
        user_email: row.get("user_email"),
        revenue: row.get("revenue"),
        revenue_target: row.try_get("revenue_target").ok(),
        revenue_progress: row.try_get("revenue_progress").ok(),
        deals_won: row.get("deals_won"),
        deals_won_target: row.try_get("deals_won_target").ok(),
        deals_progress: row.try_get("deals_progress").ok(),
        leads_created: row.get("leads_created"),
        conversion_rate: row.get("conversion_rate"),
    }).collect())
}

/// Calculate progress percentage
pub fn calculate_progress(current: f64, target: f64) -> f64 {
    if target <= 0.0 {
        0.0
    } else {
        (current / target * 100.0).min(200.0) // Cap at 200%
    }
}
