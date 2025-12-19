//! Generic entity CRUD routes

use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::{State, Path, Query},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::state::AppState;
use crate::error::ApiError;
use super::workflows::execute_triggered_workflows;

/// Helper function to parse date string from JSON value
/// Handles "YYYY-MM-DD" format and ISO datetime strings
fn parse_date_field(data: &serde_json::Value, field_name: &str) -> Option<chrono::NaiveDate> {
    data.get(field_name)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .and_then(|s| {
            // Try YYYY-MM-DD format first
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .or_else(|| {
                    // Also try ISO datetime format (take just the date part)
                    if s.len() >= 10 {
                        chrono::NaiveDate::parse_from_str(&s[..10], "%Y-%m-%d").ok()
                    } else {
                        None
                    }
                })
        })
}

/// Helper to parse datetime field from JSON value
fn parse_datetime_field(data: &serde_json::Value, field_name: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    data.get(field_name)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/:entity_type", get(list_records))
        .route("/:entity_type", post(create_record))
        .route("/:entity_type/lookup", get(lookup_entity))
        .route("/:entity_type/:id", get(get_record))
        .route("/:entity_type/:id", put(update_record))
        .route("/:entity_type/:id", delete(delete_record))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ListQuery {
    pub tenant_id: Uuid,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub data: Vec<serde_json::Value>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

// ============================================================================
// LOOKUP ENDPOINT - Universal lookup for Link fields
// ============================================================================

#[derive(Debug, Serialize)]
pub struct LookupResult {
    pub id: Uuid,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct LookupQuery {
    pub tenant_id: Uuid,
    /// Search term for filtering results
    pub q: Option<String>,
}

/// Get the display field for an entity type (used for lookup labels)
fn get_display_field_for_entity(entity_type: &str) -> &'static str {
    match entity_type {
        "contact" => "first_name",  // Will combine with last_name
        "company" => "name",
        "deal" => "name",
        "property" => "title",
        "listing" => "channel_name",
        "viewing" => "id",  // Fallback
        "offer" => "id",
        "contract" => "contract_number",
        "task" => "title",
        "user" => "email",
        _ => "name",
    }
}

/// Get the table name for an entity type
fn get_table_name_for_entity(entity_type: &str) -> &'static str {
    match entity_type {
        "contact" => "contacts",
        "company" => "companies",
        "deal" => "deals",
        "property" => "properties",
        "listing" => "listings",
        "viewing" => "viewings",
        "offer" => "offers",
        "contract" => "contracts",
        "task" => "tasks",
        "user" => "users",
        _ => "unknown",
    }
}

/// Universal lookup endpoint for fetching options for Link fields
pub async fn lookup_entity(
    State(state): State<Arc<AppState>>,
    Path(entity_type): Path<String>,
    Query(params): Query<LookupQuery>,
) -> Result<Json<Vec<LookupResult>>, ApiError> {
    use sqlx::Row;
    
    // Validate entity type exists
    let _entity = state.metadata
        .get_entity_type(params.tenant_id, &entity_type)
        .await?;
    
    let search_pattern = params.q.as_ref()
        .map(|q| format!("%{}%", q))
        .unwrap_or_else(|| "%".to_string());
    
    // Special handling for contacts (combine first_name + last_name)
    let results = if entity_type == "contact" {
        let rows = sqlx::query(
            r#"
            SELECT id, first_name, last_name 
            FROM contacts 
            WHERE tenant_id = $1 
              AND deleted_at IS NULL
              AND (first_name ILIKE $2 OR last_name ILIKE $2 OR email ILIKE $2)
            ORDER BY first_name, last_name
            LIMIT 20
            "#
        )
        .bind(params.tenant_id)
        .bind(&search_pattern)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        
        rows.iter().map(|row| {
            let id: Uuid = row.try_get("id").unwrap_or_default();
            let first: String = row.try_get("first_name").unwrap_or_default();
            let last: String = row.try_get("last_name").unwrap_or_default();
            LookupResult {
                id,
                label: format!("{} {}", first, last).trim().to_string(),
            }
        }).collect()
    } else {
        // Generic lookup for other entities
        let table_name = get_table_name_for_entity(&entity_type);
        let display_field = get_display_field_for_entity(&entity_type);
        
        if table_name == "unknown" {
            return Ok(Json(vec![]));
        }
        
        // Build query based on entity type
        let query = match entity_type.as_str() {
            "user" => {
                // Users table doesn't have deleted_at
                format!(
                    "SELECT id, {} as label FROM {} WHERE tenant_id = $1 AND {} ILIKE $2 ORDER BY {} LIMIT 20",
                    display_field, table_name, display_field, display_field
                )
            }
            _ => {
                format!(
                    "SELECT id, {} as label FROM {} WHERE tenant_id = $1 AND deleted_at IS NULL AND {} ILIKE $2 ORDER BY {} LIMIT 20",
                    display_field, table_name, display_field, display_field
                )
            }
        };
        
        let rows = sqlx::query(&query)
            .bind(params.tenant_id)
            .bind(&search_pattern)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        
        rows.iter().map(|row| {
            let id: Uuid = row.try_get("id").unwrap_or_default();
            let label: String = row.try_get("label").unwrap_or_else(|_| id.to_string());
            LookupResult { id, label }
        }).collect()
    };
    
    Ok(Json(results))
}


async fn list_records(
    State(state): State<Arc<AppState>>,
    Path(entity_type): Path<String>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse>, ApiError> {
    
    
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(25).min(100);
    let offset = (page - 1) * per_page;

    // Validate entity type exists
    let _entity = state.metadata
        .get_entity_type(query.tenant_id, &entity_type)
        .await?;

    let search = query.search.clone();

    // Map entity type to table and query 
    let (data, total) = match entity_type.as_str() {
        "contact" => query_contacts(&state.pool, query.tenant_id, per_page, offset, search.as_deref()).await?,
        "company" => query_companies(&state.pool, query.tenant_id, per_page, offset, search.as_deref()).await?,
        "deal" => query_deals(&state.pool, query.tenant_id, per_page, offset, search.as_deref()).await?,
        "property" => query_properties(&state.pool, query.tenant_id, per_page, offset, search.as_deref()).await?,
        "listing" => query_listings(&state.pool, query.tenant_id, per_page, offset, search.as_deref()).await?,
        "viewing" => query_viewings(&state.pool, query.tenant_id, per_page, offset, search.as_deref()).await?,
        "offer" => query_offers(&state.pool, query.tenant_id, per_page, offset, search.as_deref()).await?,
        "contract" => query_contracts(&state.pool, query.tenant_id, per_page, offset, search.as_deref()).await?,
        "task" => query_tasks(&state.pool, query.tenant_id, per_page, offset, search.as_deref()).await?,
        _ => (Vec::new(), 0),
    };

    Ok(Json(ListResponse {
        data,
        total,
        page,
        per_page,
    }))
}

async fn query_contacts(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    limit: i32,
    offset: i32,
    search: Option<&str>,
) -> Result<(Vec<serde_json::Value>, i64), ApiError> {
    use sqlx::Row;
    
    let search_pattern = search.map(|s| format!("%{}%", s));
    
    // Get total count
    let count_sql = if search.is_some() {
        "SELECT COUNT(*) as count FROM contacts WHERE tenant_id = $1 AND deleted_at IS NULL AND (first_name ILIKE $2 OR last_name ILIKE $2 OR email ILIKE $2)"
    } else {
        "SELECT COUNT(*) as count FROM contacts WHERE tenant_id = $1 AND deleted_at IS NULL"
    };
    
    let count_row = if let Some(ref pattern) = search_pattern {
        sqlx::query(count_sql).bind(tenant_id).bind(pattern).fetch_one(pool).await
    } else {
        sqlx::query(count_sql).bind(tenant_id).fetch_one(pool).await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    // Get records
    let rows = if let Some(ref pattern) = search_pattern {
        sqlx::query(
            r#"
            SELECT id, first_name, last_name, email, phone, lifecycle_stage, created_at, updated_at
            FROM contacts 
            WHERE tenant_id = $1 AND deleted_at IS NULL AND (first_name ILIKE $4 OR last_name ILIKE $4 OR email ILIKE $4)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .bind(pattern)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query(
            r#"
            SELECT id, first_name, last_name, email, phone, lifecycle_stage, created_at, updated_at
            FROM contacts 
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
            "first_name": row.try_get::<String, _>("first_name").unwrap_or_default(),
            "last_name": row.try_get::<String, _>("last_name").unwrap_or_default(),
            "email": row.try_get::<Option<String>, _>("email").ok().flatten(),
            "phone": row.try_get::<Option<String>, _>("phone").ok().flatten(),
            "lifecycle_stage": row.try_get::<String, _>("lifecycle_stage").unwrap_or_default(),
        })
    }).collect();

    Ok((data, total))
}

async fn query_companies(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    limit: i32,
    offset: i32,
    _search: Option<&str>,
) -> Result<(Vec<serde_json::Value>, i64), ApiError> {
    use sqlx::Row;
    
    let count_row = sqlx::query("SELECT COUNT(*) as count FROM companies WHERE tenant_id = $1 AND deleted_at IS NULL")
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    let rows = sqlx::query(
        r#"
        SELECT id, name, domain, industry, phone, created_at, updated_at
        FROM companies 
        WHERE tenant_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(tenant_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
            "name": row.try_get::<String, _>("name").unwrap_or_default(),
            "domain": row.try_get::<Option<String>, _>("domain").ok().flatten(),
            "industry": row.try_get::<Option<String>, _>("industry").ok().flatten(),
            "phone": row.try_get::<Option<String>, _>("phone").ok().flatten(),
        })
    }).collect();

    Ok((data, total))
}

async fn query_deals(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    limit: i32,
    offset: i32,
    _search: Option<&str>,
) -> Result<(Vec<serde_json::Value>, i64), ApiError> {
    use sqlx::Row;
    
    let count_row = sqlx::query("SELECT COUNT(*) as count FROM deals WHERE tenant_id = $1 AND deleted_at IS NULL")
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    let rows = sqlx::query(
        r#"
        SELECT id, name, amount, stage, expected_close_date, created_at, updated_at
        FROM deals 
        WHERE tenant_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(tenant_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
            "name": row.try_get::<String, _>("name").unwrap_or_default(),
            "amount": row.try_get::<Option<i64>, _>("amount").ok().flatten(),
            "stage": row.try_get::<String, _>("stage").unwrap_or_default(),
            "expected_close_date": row.try_get::<Option<chrono::NaiveDate>, _>("expected_close_date").ok().flatten(),
        })
    }).collect();

    Ok((data, total))
}

async fn query_properties(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    limit: i32,
    offset: i32,
    search: Option<&str>,
) -> Result<(Vec<serde_json::Value>, i64), ApiError> {
    use sqlx::Row;
    
    let search_pattern = search.map(|s| format!("%{}%", s));
    
    // Get total count
    let count_sql = if search.is_some() {
        "SELECT COUNT(*) as count FROM properties WHERE tenant_id = $1 AND deleted_at IS NULL AND (title ILIKE $2 OR reference ILIKE $2 OR city ILIKE $2)"
    } else {
        "SELECT COUNT(*) as count FROM properties WHERE tenant_id = $1 AND deleted_at IS NULL"
    };
    
    let count_row = if let Some(ref pattern) = search_pattern {
        sqlx::query(count_sql).bind(tenant_id).bind(pattern).fetch_one(pool).await
    } else {
        sqlx::query(count_sql).bind(tenant_id).fetch_one(pool).await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    // Get records
    let rows = if let Some(ref pattern) = search_pattern {
        sqlx::query(
            r#"
            SELECT id, reference, title, property_type, usage, status, city, area, 
                   bedrooms, bathrooms, size_sqm, price, rent_amount, currency,
                   created_at, updated_at
            FROM properties 
            WHERE tenant_id = $1 AND deleted_at IS NULL AND (title ILIKE $4 OR reference ILIKE $4 OR city ILIKE $4)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .bind(pattern)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query(
            r#"
            SELECT id, reference, title, property_type, usage, status, city, area, 
                   bedrooms, bathrooms, size_sqm, price, rent_amount, currency,
                   created_at, updated_at
            FROM properties 
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
            "reference": row.try_get::<Option<String>, _>("reference").ok().flatten(),
            "title": row.try_get::<String, _>("title").unwrap_or_default(),
            "property_type": row.try_get::<Option<String>, _>("property_type").ok().flatten(),
            "usage": row.try_get::<Option<String>, _>("usage").ok().flatten(),
            "status": row.try_get::<Option<String>, _>("status").ok().flatten(),
            "city": row.try_get::<Option<String>, _>("city").ok().flatten(),
            "area": row.try_get::<Option<String>, _>("area").ok().flatten(),
            "bedrooms": row.try_get::<Option<i32>, _>("bedrooms").ok().flatten(),
            "bathrooms": row.try_get::<Option<i32>, _>("bathrooms").ok().flatten(),
            "price": row.try_get::<Option<f64>, _>("price").ok().flatten(),
            "rent_amount": row.try_get::<Option<f64>, _>("rent_amount").ok().flatten(),
        })
    }).collect();

    Ok((data, total))
}

async fn query_listings(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    limit: i32,
    offset: i32,
    _search: Option<&str>,
) -> Result<(Vec<serde_json::Value>, i64), ApiError> {
    use sqlx::Row;
    
    let count_row = sqlx::query("SELECT COUNT(*) as count FROM listings WHERE tenant_id = $1 AND deleted_at IS NULL")
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    let rows = sqlx::query(
        r#"SELECT id, property_id, channel, channel_name, listing_price, listing_currency,
                  start_date, end_date, status, featured, created_at
           FROM listings WHERE tenant_id = $1 AND deleted_at IS NULL
           ORDER BY created_at DESC LIMIT $2 OFFSET $3"#
    )
    .bind(tenant_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
            "property_id": row.try_get::<Option<Uuid>, _>("property_id").ok().flatten(),
            "channel": row.try_get::<Option<String>, _>("channel").ok().flatten(),
            "channel_name": row.try_get::<Option<String>, _>("channel_name").ok().flatten(),
            "listing_price": row.try_get::<Option<f64>, _>("listing_price").ok().flatten(),
            "listing_currency": row.try_get::<Option<String>, _>("listing_currency").ok().flatten(),
            "start_date": row.try_get::<Option<chrono::NaiveDate>, _>("start_date").ok().flatten(),
            "end_date": row.try_get::<Option<chrono::NaiveDate>, _>("end_date").ok().flatten(),
            "status": row.try_get::<Option<String>, _>("status").ok().flatten(),
            "featured": row.try_get::<Option<bool>, _>("featured").ok().flatten(),
        })
    }).collect();

    Ok((data, total))
}

async fn query_viewings(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    limit: i32,
    offset: i32,
    _search: Option<&str>,
) -> Result<(Vec<serde_json::Value>, i64), ApiError> {
    use sqlx::Row;
    
    let count_row = sqlx::query("SELECT COUNT(*) as count FROM viewings WHERE tenant_id = $1 AND deleted_at IS NULL")
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    let rows = sqlx::query(
        r#"SELECT id, property_id, contact_id, agent_id, scheduled_at, duration_minutes, 
                  status, feedback, rating, created_at
           FROM viewings WHERE tenant_id = $1 AND deleted_at IS NULL
           ORDER BY scheduled_at DESC LIMIT $2 OFFSET $3"#
    )
    .bind(tenant_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
            "property_id": row.try_get::<Option<Uuid>, _>("property_id").ok().flatten(),
            "contact_id": row.try_get::<Option<Uuid>, _>("contact_id").ok().flatten(),
            "scheduled_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("scheduled_at").ok().flatten(),
            "status": row.try_get::<Option<String>, _>("status").ok().flatten(),
            "duration_minutes": row.try_get::<Option<i32>, _>("duration_minutes").ok().flatten(),
        })
    }).collect();

    Ok((data, total))
}

async fn query_offers(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    limit: i32,
    offset: i32,
    _search: Option<&str>,
) -> Result<(Vec<serde_json::Value>, i64), ApiError> {
    use sqlx::Row;
    
    let count_row = sqlx::query("SELECT COUNT(*) as count FROM offers WHERE tenant_id = $1 AND deleted_at IS NULL")
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    let rows = sqlx::query(
        r#"SELECT id, property_id, contact_id, offer_amount, currency, status, 
                  submitted_at, expires_at, created_at
           FROM offers WHERE tenant_id = $1 AND deleted_at IS NULL
           ORDER BY created_at DESC LIMIT $2 OFFSET $3"#
    )
    .bind(tenant_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
            "property_id": row.try_get::<Option<Uuid>, _>("property_id").ok().flatten(),
            "contact_id": row.try_get::<Option<Uuid>, _>("contact_id").ok().flatten(),
            "offer_amount": row.try_get::<Option<f64>, _>("offer_amount").ok().flatten(),
            "currency": row.try_get::<Option<String>, _>("currency").ok().flatten(),
            "status": row.try_get::<Option<String>, _>("status").ok().flatten(),
        })
    }).collect();

    Ok((data, total))
}

async fn query_contracts(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    limit: i32,
    offset: i32,
    _search: Option<&str>,
) -> Result<(Vec<serde_json::Value>, i64), ApiError> {
    use sqlx::Row;
    
    let count_row = sqlx::query("SELECT COUNT(*) as count FROM contracts WHERE tenant_id = $1 AND deleted_at IS NULL")
        .bind(tenant_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    let rows = sqlx::query(
        r#"SELECT id, contract_number, contract_type, property_id, start_date, end_date,
                  amount, currency, status, signed_at, created_at
           FROM contracts WHERE tenant_id = $1 AND deleted_at IS NULL
           ORDER BY created_at DESC LIMIT $2 OFFSET $3"#
    )
    .bind(tenant_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
            "contract_number": row.try_get::<Option<String>, _>("contract_number").ok().flatten(),
            "contract_type": row.try_get::<Option<String>, _>("contract_type").ok().flatten(),
            "property_id": row.try_get::<Option<Uuid>, _>("property_id").ok().flatten(),
            "start_date": row.try_get::<Option<chrono::NaiveDate>, _>("start_date").ok().flatten(),
            "end_date": row.try_get::<Option<chrono::NaiveDate>, _>("end_date").ok().flatten(),
            "amount": row.try_get::<Option<f64>, _>("amount").ok().flatten(),
            "status": row.try_get::<Option<String>, _>("status").ok().flatten(),
        })
    }).collect();

    Ok((data, total))
}

async fn query_tasks(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    limit: i32,
    offset: i32,
    search: Option<&str>,
) -> Result<(Vec<serde_json::Value>, i64), ApiError> {
    use sqlx::Row;
    
    let search_pattern = search.map(|s| format!("%{}%", s));
    
    let count_sql = if search.is_some() {
        "SELECT COUNT(*) as count FROM tasks WHERE tenant_id = $1 AND title ILIKE $2"
    } else {
        "SELECT COUNT(*) as count FROM tasks WHERE tenant_id = $1"
    };
    
    let count_row = if let Some(ref pattern) = search_pattern {
        sqlx::query(count_sql).bind(tenant_id).bind(pattern).fetch_one(pool).await
    } else {
        sqlx::query(count_sql).bind(tenant_id).fetch_one(pool).await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    let rows = if let Some(ref pattern) = search_pattern {
        sqlx::query(
            r#"SELECT id, title, description, status, priority, due_date, completed_at, assignee_id, created_at
               FROM tasks WHERE tenant_id = $1 AND title ILIKE $4
               ORDER BY created_at DESC LIMIT $2 OFFSET $3"#
        )
        .bind(tenant_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .bind(pattern)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query(
            r#"SELECT id, title, description, status, priority, due_date, completed_at, assignee_id, created_at
               FROM tasks WHERE tenant_id = $1
               ORDER BY created_at DESC LIMIT $2 OFFSET $3"#
        )
        .bind(tenant_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|row| {
        serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
            "title": row.try_get::<String, _>("title").unwrap_or_default(),
            "description": row.try_get::<Option<String>, _>("description").ok().flatten(),
            "status": row.try_get::<Option<String>, _>("status").ok().flatten().unwrap_or("pending".to_string()),
            "priority": row.try_get::<Option<String>, _>("priority").ok().flatten().unwrap_or("medium".to_string()),
            "due_date": row.try_get::<Option<chrono::NaiveDate>, _>("due_date").ok().flatten(),
            "completed_at": row.try_get::<Option<chrono::DateTime<Utc>>, _>("completed_at").ok().flatten(),
            "assignee_id": row.try_get::<Option<Uuid>, _>("assignee_id").ok().flatten(),
        })
    }).collect();

    Ok((data, total))
}

async fn create_record(
    State(state): State<Arc<AppState>>,
    Path(entity_type): Path<String>,
    Query(query): Query<ListQuery>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let now = Utc::now();
    let id = Uuid::new_v4();

    // Validate entity type
    let _entity = state.metadata
        .get_entity_type(query.tenant_id, &entity_type)
        .await?;

    match entity_type.as_str() {
        "contact" => {
            sqlx::query(
                r#"INSERT INTO contacts (id, tenant_id, first_name, last_name, email, phone, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(data.get("first_name").and_then(|v| v.as_str()).unwrap_or(""))
            .bind(data.get("last_name").and_then(|v| v.as_str()).unwrap_or(""))
            .bind(data.get("email").and_then(|v| v.as_str()))
            .bind(data.get("phone").and_then(|v| v.as_str()))
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "company" => {
            sqlx::query(
                r#"INSERT INTO companies (id, tenant_id, name, domain, industry, phone, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(data.get("name").and_then(|v| v.as_str()).unwrap_or(""))
            .bind(data.get("domain").and_then(|v| v.as_str()))
            .bind(data.get("industry").and_then(|v| v.as_str()))
            .bind(data.get("phone").and_then(|v| v.as_str()))
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "deal" => {
            let close_date: Option<chrono::NaiveDate> = data.get("expected_close_date")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            
            // Get default pipeline_id for this tenant
            let pipeline_id: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM pipelines WHERE tenant_id = $1 LIMIT 1"
            )
            .bind(query.tenant_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

            let pipeline_id = pipeline_id.ok_or_else(|| 
                ApiError::BadRequest("No pipeline found. Please create a pipeline first.".to_string())
            )?;
                
            sqlx::query(
                r#"INSERT INTO deals (id, tenant_id, pipeline_id, name, amount, stage, expected_close_date, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(pipeline_id)
            .bind(data.get("name").and_then(|v| v.as_str()).unwrap_or(""))
            .bind(data.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .bind(data.get("stage").and_then(|v| v.as_str()).unwrap_or("prospecting"))
            .bind(close_date)
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "property" => {
            sqlx::query(
                r#"INSERT INTO properties (id, tenant_id, reference, title, property_type, usage, status, 
                   country, city, area, address, bedrooms, bathrooms, size_sqm, price, rent_amount, 
                   currency, description, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(data.get("reference").and_then(|v| v.as_str()))
            .bind(data.get("title").and_then(|v| v.as_str()).unwrap_or(""))
            .bind(data.get("property_type").and_then(|v| v.as_str()))
            .bind(data.get("usage").and_then(|v| v.as_str()))
            .bind(data.get("status").and_then(|v| v.as_str()).unwrap_or("draft"))
            .bind(data.get("country").and_then(|v| v.as_str()))
            .bind(data.get("city").and_then(|v| v.as_str()))
            .bind(data.get("area").and_then(|v| v.as_str()))
            .bind(data.get("address").and_then(|v| v.as_str()))
            .bind(data.get("bedrooms").and_then(|v| v.as_i64()).map(|v| v as i32))
            .bind(data.get("bathrooms").and_then(|v| v.as_i64()).map(|v| v as i32))
            .bind(data.get("size_sqm").and_then(|v| v.as_f64()))
            .bind(data.get("price").and_then(|v| v.as_f64()))
            .bind(data.get("rent_amount").and_then(|v| v.as_f64()))
            .bind(data.get("currency").and_then(|v| v.as_str()).unwrap_or("USD"))
            .bind(data.get("description").and_then(|v| v.as_str()))
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "task" => {
            // Parse due_date as DateTime
            let due_date: Option<chrono::DateTime<Utc>> = data.get("due_date")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                        .ok()
                        .map(|d| d.and_hms_opt(12, 0, 0).unwrap().and_utc())));
            
            // Get default user for created_by (required field)
            let created_by: Option<Uuid> = data.get("created_by")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .or_else(|| {
                    // Will be fetched asynchronously below
                    None
                });
            
            // If no created_by provided, get first user for this tenant
            let created_by = if let Some(uid) = created_by {
                uid
            } else {
                let default_user: Option<Uuid> = sqlx::query_scalar(
                    "SELECT id FROM users WHERE tenant_id = $1 LIMIT 1"
                )
                .bind(query.tenant_id)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?;
                
                default_user.ok_or_else(|| 
                    ApiError::BadRequest("No users found. Please create a user first.".to_string())
                )?
            };
            
            sqlx::query(
                r#"INSERT INTO tasks (id, tenant_id, title, description, status, priority, task_type, due_date, created_by, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(data.get("title").and_then(|v| v.as_str()).unwrap_or(""))
            .bind(data.get("description").and_then(|v| v.as_str()))
            .bind(data.get("status").and_then(|v| v.as_str()).unwrap_or("open"))
            .bind(data.get("priority").and_then(|v| v.as_str()).unwrap_or("normal"))
            .bind(data.get("task_type").and_then(|v| v.as_str()).unwrap_or("todo"))
            .bind(due_date)
            .bind(created_by)
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "viewing" => {
            // Parse dates - support both datetime and start/end time fields
            let scheduled_at: Option<chrono::DateTime<Utc>> = data.get("scheduled_at")
                .or_else(|| data.get("start_time"))
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M")
                        .ok()
                        .map(|ndt| ndt.and_utc())));
            
            let scheduled_end: Option<chrono::DateTime<Utc>> = data.get("scheduled_end")
                .or_else(|| data.get("end_time"))
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M")
                        .ok()
                        .map(|ndt| ndt.and_utc())));
            
            // Get property_id - required
            let property_id: Option<Uuid> = data.get("property_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());
            
            // Get contact_id - required
            let contact_id: Option<Uuid> = data.get("contact_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());
            
            // Get agent_id - optional
            let agent_id: Option<Uuid> = data.get("agent_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());
            
            sqlx::query(
                r#"INSERT INTO viewings (id, tenant_id, property_id, contact_id, agent_id, scheduled_at, 
                   scheduled_start, scheduled_end, duration_minutes, status, feedback, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(property_id)
            .bind(contact_id)
            .bind(agent_id)
            .bind(scheduled_at.unwrap_or(now))
            .bind(scheduled_at)
            .bind(scheduled_end)
            .bind(data.get("duration_minutes").and_then(|v| v.as_i64()).unwrap_or(30) as i32)
            .bind(data.get("status").and_then(|v| v.as_str()).unwrap_or("scheduled"))
            .bind(data.get("feedback").and_then(|v| v.as_str()))
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "listing" => {
            // Get property_id - required
            let property_id: Option<Uuid> = data.get("property_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());
            
            // Parse dates
            let start_date: chrono::NaiveDate = data.get("start_date")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .unwrap_or_else(|| now.date_naive());
            
            let end_date: Option<chrono::NaiveDate> = data.get("end_date")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            
            sqlx::query(
                r#"INSERT INTO listings (id, tenant_id, property_id, channel, channel_name, external_url,
                   listing_price, listing_currency, start_date, end_date, status, headline, promo_price, 
                   description, featured, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(property_id)
            .bind(data.get("channel").and_then(|v| v.as_str()))
            .bind(data.get("channel_name").and_then(|v| v.as_str()))
            .bind(data.get("external_url").and_then(|v| v.as_str()))
            .bind(data.get("listing_price").or_else(|| data.get("list_price")).and_then(|v| v.as_f64()))
            .bind(data.get("listing_currency").and_then(|v| v.as_str()).unwrap_or("AED"))
            .bind(start_date)
            .bind(end_date)
            .bind(data.get("status").and_then(|v| v.as_str()).unwrap_or("draft"))
            .bind(data.get("headline").and_then(|v| v.as_str()))
            .bind(data.get("promo_price").or_else(|| data.get("promotional_price")).and_then(|v| v.as_f64()))
            .bind(data.get("description").and_then(|v| v.as_str()))
            .bind(data.get("featured").and_then(|v| v.as_bool()).unwrap_or(false))
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "offer" => {
            // Get property_id - required
            let property_id: Option<Uuid> = data.get("property_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());
            
            // Get contact_id - required
            let contact_id: Option<Uuid> = data.get("contact_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());
            
            // Get deal_id - optional
            let deal_id: Option<Uuid> = data.get("deal_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());
            
            sqlx::query(
                r#"INSERT INTO offers (id, tenant_id, property_id, contact_id, deal_id, offer_amount,
                   currency, status, offer_type, financing_type, deposit_amount, conditions, 
                   created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(property_id)
            .bind(contact_id)
            .bind(deal_id)
            .bind(data.get("offer_amount").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .bind(data.get("currency").and_then(|v| v.as_str()).unwrap_or("USD"))
            .bind(data.get("status").and_then(|v| v.as_str()).unwrap_or("draft"))
            .bind(data.get("offer_type").and_then(|v| v.as_str()))
            .bind(data.get("financing_type").or_else(|| data.get("finance_type")).and_then(|v| v.as_str()))
            .bind(data.get("deposit_amount").and_then(|v| v.as_f64()))
            .bind(data.get("conditions").and_then(|v| v.as_str()))
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        _ => return Err(ApiError::NotFound(format!("Entity type {} not supported for create", entity_type))),
    }

    // Trigger workflows asynchronously (fire and forget - don't block HTTP response)
    let pool = state.pool.clone();
    let tenant = query.tenant_id;
    let et = entity_type.clone();
    let new_data = data.clone();
    tokio::spawn(async move {
        let _ = execute_triggered_workflows(
            &pool,
            tenant,
            "record_created",
            &et,
            id,
            None,
            new_data,
        ).await;
    });

    Ok(Json(serde_json::json!({ "id": id, "created": true })))
}

async fn get_record(
    State(state): State<Arc<AppState>>,
    Path((entity_type, id)): Path<(String, Uuid)>,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use sqlx::Row;

    let row = match entity_type.as_str() {
        "contact" => {
            sqlx::query("SELECT id, first_name, last_name, email, phone, lifecycle_stage FROM contacts WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(query.tenant_id)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?
        }
        "company" => {
            sqlx::query("SELECT id, name, domain, industry, phone FROM companies WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(query.tenant_id)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?
        }
        "property" => {
            sqlx::query(
                r#"SELECT id, reference, title, property_type, usage, status, country, city, area, address,
                   latitude, longitude, bedrooms, bathrooms, size_sqm, floor, total_floors, year_built,
                   price, rent_amount, currency, service_charge, commission_percent, description,
                   owner_id, agent_id, developer_id, listed_at, expires_at, created_at, updated_at
                   FROM properties WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"#
            )
                .bind(id)
                .bind(query.tenant_id)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?
        }
        "deal" => {
            sqlx::query(
                r#"SELECT id, name, amount, stage, expected_close_date, created_at, updated_at
                   FROM deals WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"#
            )
                .bind(id)
                .bind(query.tenant_id)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?
        }
        "task" => {
            sqlx::query(
                r#"SELECT id, title, description, status, priority, due_date, completed_at, assignee_id, created_at, updated_at
                   FROM tasks WHERE id = $1 AND tenant_id = $2"#
            )
                .bind(id)
                .bind(query.tenant_id)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?
        }
        "listing" => {
            sqlx::query(
                r#"SELECT id, property_id, channel, channel_name, listing_price, listing_currency,
                          start_date, end_date, status, featured, created_at, updated_at
                   FROM listings WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"#
            )
                .bind(id)
                .bind(query.tenant_id)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?
        }
        _ => None,
    };

    match row {
        Some(r) => {
            let data = match entity_type.as_str() {
                "contact" => serde_json::json!({
                    "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
                    "first_name": r.try_get::<String, _>("first_name").unwrap_or_default(),
                    "last_name": r.try_get::<String, _>("last_name").unwrap_or_default(),
                    "email": r.try_get::<Option<String>, _>("email").ok().flatten(),
                    "phone": r.try_get::<Option<String>, _>("phone").ok().flatten(),
                }),
                "company" => serde_json::json!({
                    "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
                    "name": r.try_get::<String, _>("name").unwrap_or_default(),
                    "domain": r.try_get::<Option<String>, _>("domain").ok().flatten(),
                    "industry": r.try_get::<Option<String>, _>("industry").ok().flatten(),
                }),
                "property" => serde_json::json!({
                    "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
                    "reference": r.try_get::<Option<String>, _>("reference").ok().flatten(),
                    "title": r.try_get::<String, _>("title").unwrap_or_default(),
                    "property_type": r.try_get::<Option<String>, _>("property_type").ok().flatten(),
                    "usage": r.try_get::<Option<String>, _>("usage").ok().flatten(),
                    "status": r.try_get::<Option<String>, _>("status").ok().flatten(),
                    "country": r.try_get::<Option<String>, _>("country").ok().flatten(),
                    "city": r.try_get::<Option<String>, _>("city").ok().flatten(),
                    "area": r.try_get::<Option<String>, _>("area").ok().flatten(),
                    "address": r.try_get::<Option<String>, _>("address").ok().flatten(),
                    "bedrooms": r.try_get::<Option<i32>, _>("bedrooms").ok().flatten(),
                    "bathrooms": r.try_get::<Option<i32>, _>("bathrooms").ok().flatten(),
                    "size_sqm": r.try_get::<Option<f64>, _>("size_sqm").ok().flatten(),
                    "price": r.try_get::<Option<f64>, _>("price").ok().flatten(),
                    "rent_amount": r.try_get::<Option<f64>, _>("rent_amount").ok().flatten(),
                    "currency": r.try_get::<Option<String>, _>("currency").ok().flatten(),
                    "description": r.try_get::<Option<String>, _>("description").ok().flatten(),
                }),
                "deal" => serde_json::json!({
                    "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
                    "name": r.try_get::<String, _>("name").unwrap_or_default(),
                    "amount": r.try_get::<Option<i64>, _>("amount").ok().flatten(),
                    "stage": r.try_get::<String, _>("stage").unwrap_or_default(),
                    "expected_close_date": r.try_get::<Option<chrono::NaiveDate>, _>("expected_close_date").ok().flatten(),
                }),
                "task" => serde_json::json!({
                    "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
                    "title": r.try_get::<String, _>("title").unwrap_or_default(),
                    "description": r.try_get::<Option<String>, _>("description").ok().flatten(),
                    "status": r.try_get::<Option<String>, _>("status").ok().flatten().unwrap_or("pending".to_string()),
                    "priority": r.try_get::<Option<String>, _>("priority").ok().flatten().unwrap_or("medium".to_string()),
                    "due_date": r.try_get::<Option<chrono::DateTime<Utc>>, _>("due_date").ok().flatten(),
                    "completed_at": r.try_get::<Option<chrono::DateTime<Utc>>, _>("completed_at").ok().flatten(),
                    "assignee_id": r.try_get::<Option<Uuid>, _>("assignee_id").ok().flatten(),
                }),
                "listing" => serde_json::json!({
                    "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
                    "property_id": r.try_get::<Option<Uuid>, _>("property_id").ok().flatten(),
                    "channel": r.try_get::<Option<String>, _>("channel").ok().flatten(),
                    "channel_name": r.try_get::<Option<String>, _>("channel_name").ok().flatten(),
                    "listing_price": r.try_get::<Option<f64>, _>("listing_price").ok().flatten(),
                    "listing_currency": r.try_get::<Option<String>, _>("listing_currency").ok().flatten(),
                    "start_date": r.try_get::<Option<chrono::NaiveDate>, _>("start_date").ok().flatten(),
                    "end_date": r.try_get::<Option<chrono::NaiveDate>, _>("end_date").ok().flatten(),
                    "status": r.try_get::<Option<String>, _>("status").ok().flatten(),
                    "featured": r.try_get::<Option<bool>, _>("featured").ok().flatten(),
                }),
                _ => serde_json::json!({}),
            };
            Ok(Json(data))
        }
        None => Err(ApiError::NotFound(format!("Record {} not found", id))),
    }
}

async fn update_record(
    State(state): State<Arc<AppState>>,
    Path((entity_type, id)): Path<(String, Uuid)>,
    Query(query): Query<ListQuery>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let now = Utc::now();

    match entity_type.as_str() {
        "contact" => {
            sqlx::query(
                r#"UPDATE contacts SET 
                   first_name = COALESCE($3, first_name),
                   last_name = COALESCE($4, last_name),
                   email = COALESCE($5, email),
                   phone = COALESCE($6, phone),
                   updated_at = $7
                   WHERE id = $1 AND tenant_id = $2"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(data.get("first_name").and_then(|v| v.as_str()))
            .bind(data.get("last_name").and_then(|v| v.as_str()))
            .bind(data.get("email").and_then(|v| v.as_str()))
            .bind(data.get("phone").and_then(|v| v.as_str()))
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "property" => {
            sqlx::query(
                r#"UPDATE properties SET 
                   reference = COALESCE($3, reference),
                   title = COALESCE($4, title),
                   property_type = COALESCE($5, property_type),
                   usage = COALESCE($6, usage),
                   status = COALESCE($7, status),
                   city = COALESCE($8, city),
                   area = COALESCE($9, area),
                   bedrooms = COALESCE($10, bedrooms),
                   bathrooms = COALESCE($11, bathrooms),
                   price = COALESCE($12, price),
                   rent_amount = COALESCE($13, rent_amount),
                   description = COALESCE($14, description),
                   updated_at = $15
                   WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(data.get("reference").and_then(|v| v.as_str()))
            .bind(data.get("title").and_then(|v| v.as_str()))
            .bind(data.get("property_type").and_then(|v| v.as_str()))
            .bind(data.get("usage").and_then(|v| v.as_str()))
            .bind(data.get("status").and_then(|v| v.as_str()))
            .bind(data.get("city").and_then(|v| v.as_str()))
            .bind(data.get("area").and_then(|v| v.as_str()))
            .bind(data.get("bedrooms").and_then(|v| v.as_i64()).map(|v| v as i32))
            .bind(data.get("bathrooms").and_then(|v| v.as_i64()).map(|v| v as i32))
            .bind(data.get("price").and_then(|v| v.as_f64()))
            .bind(data.get("rent_amount").and_then(|v| v.as_f64()))
            .bind(data.get("description").and_then(|v| v.as_str()))
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "company" => {
            sqlx::query(
                r#"UPDATE companies SET 
                   name = COALESCE($3, name),
                   domain = COALESCE($4, domain),
                   industry = COALESCE($5, industry),
                   phone = COALESCE($6, phone),
                   updated_at = $7
                   WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(data.get("name").and_then(|v| v.as_str()))
            .bind(data.get("domain").and_then(|v| v.as_str()))
            .bind(data.get("industry").and_then(|v| v.as_str()))
            .bind(data.get("phone").and_then(|v| v.as_str()))
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "deal" => {
            let expected_close_date = parse_date_field(&data, "expected_close_date");
            sqlx::query(
                r#"UPDATE deals SET 
                   name = COALESCE($3, name),
                   amount = COALESCE($4, amount),
                   stage = COALESCE($5, stage),
                   expected_close_date = COALESCE($6, expected_close_date),
                   updated_at = $7
                   WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(data.get("name").and_then(|v| v.as_str()))
            .bind(data.get("amount").and_then(|v| v.as_f64()))
            .bind(data.get("stage").and_then(|v| v.as_str()))
            .bind(expected_close_date)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "task" => {
            let due_date = parse_date_field(&data, "due_date");
            sqlx::query(
                r#"UPDATE tasks SET 
                   title = COALESCE($3, title),
                   description = COALESCE($4, description),
                   status = COALESCE($5, status),
                   priority = COALESCE($6, priority),
                   due_date = COALESCE($7, due_date),
                   updated_at = $8
                   WHERE id = $1 AND tenant_id = $2"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(data.get("title").and_then(|v| v.as_str()))
            .bind(data.get("description").and_then(|v| v.as_str()))
            .bind(data.get("status").and_then(|v| v.as_str()))
            .bind(data.get("priority").and_then(|v| v.as_str()))
            .bind(due_date)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "viewing" => {
            sqlx::query(
                r#"UPDATE viewings SET 
                   status = COALESCE($3, status),
                   feedback = COALESCE($4, feedback),
                   rating = COALESCE($5, rating),
                   outcome = COALESCE($6, outcome),
                   updated_at = $7
                   WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(data.get("status").and_then(|v| v.as_str()))
            .bind(data.get("feedback").and_then(|v| v.as_str()))
            .bind(data.get("rating").and_then(|v| v.as_i64()).map(|v| v as i32))
            .bind(data.get("outcome").and_then(|v| v.as_str()))
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "listing" => {
            // Parse date fields using helper
            let start_date = parse_date_field(&data, "start_date");
            let end_date = parse_date_field(&data, "end_date");

            sqlx::query(
                r#"UPDATE listings SET 
                   status = COALESCE($3, status),
                   listing_price = COALESCE($4, listing_price),
                   listing_currency = COALESCE($5, listing_currency),
                   featured = COALESCE($6, featured),
                   channel = COALESCE($7, channel),
                   channel_name = COALESCE($8, channel_name),
                   start_date = COALESCE($9, start_date),
                   end_date = COALESCE($10, end_date),
                   updated_at = $11
                   WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(data.get("status").and_then(|v| v.as_str()))
            .bind(data.get("listing_price").and_then(|v| v.as_f64()))
            .bind(data.get("listing_currency").and_then(|v| v.as_str()))
            .bind(data.get("featured").and_then(|v| v.as_bool()))
            .bind(data.get("channel").and_then(|v| v.as_str()))
            .bind(data.get("channel_name").and_then(|v| v.as_str()))
            .bind(start_date)
            .bind(end_date)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "offer" => {
            sqlx::query(
                r#"UPDATE offers SET 
                   status = COALESCE($3, status),
                   offer_amount = COALESCE($4, offer_amount),
                   currency = COALESCE($5, currency),
                   conditions = COALESCE($6, conditions),
                   financing_type = COALESCE($7, financing_type),
                   updated_at = $8
                   WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(data.get("status").and_then(|v| v.as_str()))
            .bind(data.get("offer_amount").and_then(|v| v.as_f64()))
            .bind(data.get("currency").and_then(|v| v.as_str()))
            .bind(data.get("conditions").and_then(|v| v.as_str()))
            .bind(data.get("financing_type").and_then(|v| v.as_str()))
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        _ => return Err(ApiError::NotFound(format!("Entity type {} not supported for update", entity_type))),
    }

    // Trigger workflows asynchronously (fire and forget - don't block HTTP response)
    let pool = state.pool.clone();
    let tenant = query.tenant_id;
    let et = entity_type.clone();
    let new_data = data.clone();
    tokio::spawn(async move {
        let _ = execute_triggered_workflows(
            &pool,
            tenant,
            "field_changed",
            &et,
            id,
            None, // TODO: fetch old_values before update for comparison
            new_data,
        ).await;
    });

    Ok(Json(serde_json::json!({ "id": id, "updated": true })))
}

async fn delete_record(
    State(state): State<Arc<AppState>>,
    Path((entity_type, id)): Path<(String, Uuid)>,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let now = Utc::now();

    // Soft delete
    let table = match entity_type.as_str() {
        "contact" => "contacts",
        "company" => "companies",
        "deal" => "deals",
        "property" => "properties",
        "listing" => "listings",
        "viewing" => "viewings",
        "offer" => "offers",
        "contract" => "contracts",
        _ => return Err(ApiError::NotFound(format!("Entity type {} not found", entity_type))),
    };

    let sql = format!("UPDATE {} SET deleted_at = $3 WHERE id = $1 AND tenant_id = $2", table);
    sqlx::query(&sql)
        .bind(id)
        .bind(query.tenant_id)
        .bind(now)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
