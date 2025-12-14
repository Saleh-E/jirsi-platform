//! Dashboard Analytics - KPIs, Charts, and Funnel Data

use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Utc, Datelike};
use std::collections::HashMap;
use uuid::Uuid;
use crate::targets::{get_target_for_user, MetricType};

/// A single KPI result with optional target and progress
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct KpiResult {
    pub value: f64,
    pub target: Option<f64>,
    pub progress_percent: Option<f64>,
    pub trend_percent: f64,
    pub previous_value: f64,
}

/// Dashboard KPI response with targets
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DashboardKpis {
    pub total_leads: i64,
    pub total_leads_prev: i64,
    pub leads_trend: f64,
    pub leads_target: Option<f64>,
    pub leads_progress: Option<f64>,
    
    pub total_deals: i64,
    pub ongoing_deals: i64,
    pub total_deals_prev: i64,
    pub deals_trend: f64,
    pub deals_target: Option<f64>,
    pub deals_progress: Option<f64>,
    
    pub forecasted_revenue: f64,
    pub forecasted_revenue_prev: f64,
    pub revenue_trend: f64,
    pub revenue_target: Option<f64>,
    pub revenue_progress: Option<f64>,
    
    pub win_rate: f64,
    pub win_rate_prev: f64,
    pub win_rate_trend: f64,
}

/// Sales trend data point
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SalesTrendPoint {
    pub date: String,
    pub leads: i64,
    pub deals: i64,
}

/// Funnel stage data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunnelStage {
    pub stage: String,
    pub count: i64,
}

/// Recent activity item
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityItem {
    pub id: String,
    pub action: String,
    pub entity: String,
    pub entity_name: String,
    pub user: String,
    pub timestamp: String,
}

/// Full dashboard response
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DashboardResponse {
    pub kpis: DashboardKpis,
    pub sales_trend: Vec<SalesTrendPoint>,
    pub funnel_data: Vec<FunnelStage>,
    pub recent_activities: Vec<ActivityItem>,
}


/// Date range for queries
#[derive(Clone, Debug)]
pub struct DateRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

impl DateRange {
    pub fn today() -> Self {
        let today = Utc::now().date_naive();
        Self { from: today, to: today }
    }
    
    pub fn yesterday() -> Self {
        let today = Utc::now().date_naive();
        let yesterday = today.pred_opt().unwrap();
        Self { from: yesterday, to: yesterday }
    }
    
    pub fn this_week() -> Self {
        let today = Utc::now().date_naive();
        let days_from_monday = today.weekday().num_days_from_monday();
        let from = today - chrono::Duration::days(days_from_monday as i64);
        Self { from, to: today }
    }
    
    pub fn last_week() -> Self {
        let today = Utc::now().date_naive();
        let days_from_monday = today.weekday().num_days_from_monday();
        let this_monday = today - chrono::Duration::days(days_from_monday as i64);
        let last_monday = this_monday - chrono::Duration::days(7);
        let last_sunday = this_monday - chrono::Duration::days(1);
        Self { from: last_monday, to: last_sunday }
    }
    
    pub fn this_month() -> Self {
        let today = Utc::now().date_naive();
        let from = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
        Self { from, to: today }
    }
    
    pub fn last_month() -> Self {
        let today = Utc::now().date_naive();
        let last_month = today.checked_sub_months(chrono::Months::new(1)).unwrap();
        let from = NaiveDate::from_ymd_opt(last_month.year(), last_month.month(), 1).unwrap();
        let to = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
            .unwrap()
            .pred_opt()
            .unwrap();
        Self { from, to }
    }
    
    pub fn this_quarter() -> Self {
        let today = Utc::now().date_naive();
        let quarter_start_month = ((today.month() - 1) / 3) * 3 + 1;
        let from = NaiveDate::from_ymd_opt(today.year(), quarter_start_month, 1).unwrap();
        Self { from, to: today }
    }
    
    pub fn last_quarter() -> Self {
        let today = Utc::now().date_naive();
        let current_quarter = (today.month() - 1) / 3;
        let (year, quarter) = if current_quarter == 0 {
            (today.year() - 1, 3)
        } else {
            (today.year(), current_quarter - 1)
        };
        let from_month = quarter * 3 + 1;
        let from = NaiveDate::from_ymd_opt(year, from_month, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(
            if quarter == 3 { year + 1 } else { year },
            if quarter == 3 { 1 } else { from_month + 3 },
            1
        ).unwrap().pred_opt().unwrap();
        Self { from, to }
    }
    
    pub fn this_year() -> Self {
        let today = Utc::now().date_naive();
        let from = NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap();
        Self { from, to: today }
    }
    
    pub fn last_year() -> Self {
        let today = Utc::now().date_naive();
        let last_year = today.year() - 1;
        let from = NaiveDate::from_ymd_opt(last_year, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(last_year, 12, 31).unwrap();
        Self { from, to }
    }
}


/// Get total leads count for a date range
pub async fn get_total_leads(
    pool: &PgPool,
    tenant_id: &str,
    range: &DateRange,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM contacts
        WHERE tenant_id = $1::uuid
          AND lifecycle_stage = 'lead'
          AND created_at >= $2
          AND created_at <= $3
        "#
    )
    .bind(tenant_id)
    .bind(range.from)
    .bind(range.to)
    .fetch_one(pool)
    .await?;
    
    Ok(row.get::<i64, _>("count"))
}

/// Get ongoing deals count (not Won or Lost)
pub async fn get_ongoing_deals(
    pool: &PgPool,
    tenant_id: &str,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM deals
        WHERE tenant_id = $1::uuid
          AND stage IS NOT NULL
          AND stage NOT IN ('Won', 'Lost', 'Closed Won', 'Closed Lost')
        "#
    )
    .bind(tenant_id)
    .fetch_one(pool)
    .await?;
    
    Ok(row.get::<i64, _>("count"))
}

/// Get total deals for date range
pub async fn get_total_deals(
    pool: &PgPool,
    tenant_id: &str,
    range: &DateRange,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM deals
        WHERE tenant_id = $1::uuid
          AND created_at >= $2
          AND created_at <= $3
        "#
    )
    .bind(tenant_id)
    .bind(range.from)
    .bind(range.to)
    .fetch_one(pool)
    .await?;
    
    Ok(row.get::<i64, _>("count"))
}

/// Get forecasted revenue (amount * probability for open deals)
pub async fn get_forecasted_revenue(
    pool: &PgPool,
    tenant_id: &str,
) -> Result<f64, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT COALESCE(SUM(
            COALESCE(amount, 0) * COALESCE(probability, 50) / 100.0
        ), 0)::float8 as forecast
        FROM deals
        WHERE tenant_id = $1::uuid
          AND stage IS NOT NULL
          AND stage NOT IN ('Won', 'Lost', 'Closed Won', 'Closed Lost')
        "#
    )
    .bind(tenant_id)
    .fetch_one(pool)
    .await?;
    
    Ok(row.get::<f64, _>("forecast"))
}

/// Get win rate (won deals / total closed deals * 100)
pub async fn get_win_rate(
    pool: &PgPool,
    tenant_id: &str,
    range: &DateRange,
) -> Result<f64, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT 
            COALESCE(
                SUM(CASE WHEN stage IN ('Won', 'Closed Won') THEN 1 ELSE 0 END)::float /
                NULLIF(SUM(CASE WHEN stage IN ('Won', 'Lost', 'Closed Won', 'Closed Lost') THEN 1 ELSE 0 END), 0)
                * 100.0,
                0
            ) as win_rate
        FROM deals
        WHERE tenant_id = $1::uuid
          AND created_at >= $2
          AND created_at <= $3
        "#
    )
    .bind(tenant_id)
    .bind(range.from)
    .bind(range.to)
    .fetch_one(pool)
    .await?;
    
    Ok(row.get::<f64, _>("win_rate"))
}

/// Get sales trend (daily leads and deals)
pub async fn get_sales_trend(
    pool: &PgPool,
    tenant_id: &str,
    range: &DateRange,
) -> Result<Vec<SalesTrendPoint>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        WITH dates AS (
            SELECT generate_series($2::date, $3::date, '1 day'::interval)::date as date
        ),
        lead_counts AS (
            SELECT DATE(created_at) as date, COUNT(*) as leads
            FROM contacts
            WHERE tenant_id = $1::uuid AND lifecycle_stage = 'lead'
              AND created_at >= $2 AND created_at <= $3
            GROUP BY DATE(created_at)
        ),
        deal_counts AS (
            SELECT DATE(created_at) as date, COUNT(*) as deals
            FROM deals
            WHERE tenant_id = $1::uuid
              AND created_at >= $2 AND created_at <= $3
            GROUP BY DATE(created_at)
        )
        SELECT 
            d.date,
            COALESCE(l.leads, 0) as leads,
            COALESCE(dc.deals, 0) as deals
        FROM dates d
        LEFT JOIN lead_counts l ON d.date = l.date
        LEFT JOIN deal_counts dc ON d.date = dc.date
        ORDER BY d.date
        "#
    )
    .bind(tenant_id)
    .bind(range.from)
    .bind(range.to)
    .fetch_all(pool)
    .await?;
    
    Ok(rows.iter().map(|row| {
        let date: NaiveDate = row.get("date");
        SalesTrendPoint {
            date: date.format("%b %d").to_string(),
            leads: row.get::<i64, _>("leads"),
            deals: row.get::<i64, _>("deals"),
        }
    }).collect())
}

/// Get funnel conversion (count of deals by stage)
pub async fn get_funnel_conversion(
    pool: &PgPool,
    tenant_id: &str,
) -> Result<Vec<FunnelStage>, sqlx::Error> {
    // Define stage order
    let stage_order = vec!["New", "Qualified", "Proposal", "Negotiation", "Won"];
    
    let rows = sqlx::query(
        r#"
        SELECT 
            COALESCE(stage, 'New') as stage,
            COUNT(*) as count
        FROM deals
        WHERE tenant_id = $1::uuid
        GROUP BY stage
        "#
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;
    
    // Build a map of stage -> count
    let mut stage_counts: HashMap<String, i64> = HashMap::new();
    for row in rows {
        let stage: String = row.get("stage");
        let count: i64 = row.get("count");
        stage_counts.insert(stage, count);
    }
    
    // Return in order
    Ok(stage_order.iter().map(|s| {
        FunnelStage {
            stage: s.to_string(),
            count: *stage_counts.get(*s).unwrap_or(&0),
        }
    }).collect())
}

/// Get recent activities
pub async fn get_recent_activities(
    pool: &PgPool,
    tenant_id: &str,
    limit: i32,
) -> Result<Vec<ActivityItem>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT 
            a.id::text,
            a.activity_type as action,
            COALESCE(a.entity_type, 'Unknown') as entity,
            COALESCE(a.title, 'Untitled') as entity_name,
            COALESCE(u.email, 'System') as user_name,
            a.created_at
        FROM activity_log a
        LEFT JOIN users u ON a.performed_by = u.id
        WHERE a.tenant_id = $1::uuid
        ORDER BY a.created_at DESC
        LIMIT $2
        "#
    )
    .bind(tenant_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    
    Ok(rows.iter().map(|row| {
        let created_at: chrono::DateTime<Utc> = row.get("created_at");
        let now = Utc::now();
        let diff = now.signed_duration_since(created_at);
        
        let timestamp = if diff.num_hours() < 1 {
            format!("{} minutes ago", diff.num_minutes().max(1))
        } else if diff.num_hours() < 24 {
            format!("{} hours ago", diff.num_hours())
        } else if diff.num_days() < 7 {
            format!("{} days ago", diff.num_days())
        } else {
            created_at.format("%b %d, %Y").to_string()
        };
        
        ActivityItem {
            id: row.get("id"),
            action: row.get("action"),
            entity: row.get("entity"),
            entity_name: row.get("entity_name"),
            user: row.get("user_name"),
            timestamp,
        }
    }).collect())
}

/// Calculate trend percentage between current and previous values
fn calculate_trend(current: f64, previous: f64) -> f64 {
    if previous == 0.0 {
        if current > 0.0 { 100.0 } else { 0.0 }
    } else {
        ((current - previous) / previous) * 100.0
    }
}

/// Get full dashboard data with targets
pub async fn get_dashboard(
    pool: &PgPool,
    tenant_id: &str,
    range_name: &str,
) -> Result<DashboardResponse, sqlx::Error> {
    // Determine date ranges
    let (current_range, prev_range) = match range_name {
        "today" => (DateRange::today(), DateRange::yesterday()),
        "this_week" | "week" => (DateRange::this_week(), DateRange::last_week()),
        "this_month" | "month" => (DateRange::this_month(), DateRange::last_month()),
        "this_quarter" | "quarter" => (DateRange::this_quarter(), DateRange::last_quarter()),
        "this_year" | "year" => (DateRange::this_year(), DateRange::last_year()),
        _ => (DateRange::this_month(), DateRange::last_month()),
    };
    
    // Fetch all KPIs in parallel
    let (
        total_leads,
        total_leads_prev,
        total_deals,
        total_deals_prev,
        ongoing_deals,
        forecasted_revenue,
        win_rate,
        win_rate_prev,
        sales_trend,
        funnel_data,
        recent_activities,
    ) = tokio::try_join!(
        get_total_leads(pool, tenant_id, &current_range),
        get_total_leads(pool, tenant_id, &prev_range),
        get_total_deals(pool, tenant_id, &current_range),
        get_total_deals(pool, tenant_id, &prev_range),
        get_ongoing_deals(pool, tenant_id),
        get_forecasted_revenue(pool, tenant_id),
        get_win_rate(pool, tenant_id, &current_range),
        get_win_rate(pool, tenant_id, &prev_range),
        get_sales_trend(pool, tenant_id, &current_range),
        get_funnel_conversion(pool, tenant_id),
        get_recent_activities(pool, tenant_id, 10),
    )?;
    
    // Fetch targets (optional - don't fail if tables don't exist)
    let (leads_target, deals_target, revenue_target) = (
        get_target_for_user(pool, tenant_id, None, MetricType::LeadsCreated, current_range.from, current_range.to).await.ok().flatten(),
        get_target_for_user(pool, tenant_id, None, MetricType::DealsWon, current_range.from, current_range.to).await.ok().flatten(),
        get_target_for_user(pool, tenant_id, None, MetricType::Revenue, current_range.from, current_range.to).await.ok().flatten(),
    );
    
    // Calculate progress percentages
    let leads_progress = leads_target.map(|t| if t > 0.0 { (total_leads as f64 / t * 100.0).min(200.0) } else { 0.0 });
    let deals_progress = deals_target.map(|t| if t > 0.0 { (ongoing_deals as f64 / t * 100.0).min(200.0) } else { 0.0 });
    let revenue_progress = revenue_target.map(|t| if t > 0.0 { (forecasted_revenue / t * 100.0).min(200.0) } else { 0.0 });
    
    // Calculate trends
    let leads_trend = calculate_trend(total_leads as f64, total_leads_prev as f64);
    let deals_trend = calculate_trend(total_deals as f64, total_deals_prev as f64);
    let win_rate_trend = win_rate - win_rate_prev;
    
    Ok(DashboardResponse {
        kpis: DashboardKpis {
            total_leads,
            total_leads_prev,
            leads_trend,
            leads_target,
            leads_progress,
            total_deals,
            ongoing_deals,
            total_deals_prev,
            deals_trend,
            deals_target,
            deals_progress,
            forecasted_revenue,
            forecasted_revenue_prev: 0.0,
            revenue_trend: 0.0,
            revenue_target,
            revenue_progress,
            win_rate,
            win_rate_prev,
            win_rate_trend,
        },
        sales_trend,
        funnel_data,
        recent_activities,
    })
}



