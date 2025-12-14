//! Dashboard Analytics - KPIs, Charts, and Funnel Data

use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Utc, Datelike};
use std::collections::HashMap;
use uuid::Uuid;

/// Dashboard KPI response
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DashboardKpis {
    pub total_leads: i64,
    pub total_leads_prev: i64,
    pub leads_trend: f64,
    
    pub total_deals: i64,
    pub ongoing_deals: i64,
    pub total_deals_prev: i64,
    pub deals_trend: f64,
    
    pub forecasted_revenue: f64,
    pub forecasted_revenue_prev: f64,
    pub revenue_trend: f64,
    
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
    
    pub fn this_year() -> Self {
        let today = Utc::now().date_naive();
        let from = NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap();
        Self { from, to: today }
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

/// Get full dashboard data
pub async fn get_dashboard(
    pool: &PgPool,
    tenant_id: &str,
    range_name: &str,
) -> Result<DashboardResponse, sqlx::Error> {
    // Determine date ranges
    let (current_range, prev_range) = match range_name {
        "this_month" | "month" => (DateRange::this_month(), DateRange::last_month()),
        "this_quarter" | "quarter" => (DateRange::this_quarter(), DateRange::this_month()),
        "this_year" | "year" => (DateRange::this_year(), DateRange::this_quarter()),
        _ => (DateRange::this_month(), DateRange::last_month()),
    };
    
    // Fetch all KPIs in parallel (tokio::join! for better performance)
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
    
    // Calculate trends
    let leads_trend = calculate_trend(total_leads as f64, total_leads_prev as f64);
    let deals_trend = calculate_trend(total_deals as f64, total_deals_prev as f64);
    let win_rate_trend = win_rate - win_rate_prev;
    
    Ok(DashboardResponse {
        kpis: DashboardKpis {
            total_leads,
            total_leads_prev,
            leads_trend,
            total_deals,
            ongoing_deals,
            total_deals_prev,
            deals_trend,
            forecasted_revenue,
            forecasted_revenue_prev: 0.0, // Would need another query
            revenue_trend: 0.0,
            win_rate,
            win_rate_prev,
            win_rate_trend,
        },
        sales_trend,
        funnel_data,
        recent_activities,
    })
}



