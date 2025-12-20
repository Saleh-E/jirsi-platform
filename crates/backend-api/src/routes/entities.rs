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
    
    // Special handling for migrated entities
    let results = if ["contact", "deal", "property", "task", "viewing"].contains(&entity_type.as_str()) {
        // Resolve entity_type_id
        let et_row = sqlx::query("SELECT id FROM entity_types WHERE name = $1 AND tenant_id = $2")
            .bind(&entity_type)
            .bind(params.tenant_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .ok_or_else(|| ApiError::Internal(format!("Entity type '{}' not found", entity_type)))?;
        let et_id: Uuid = et_row.get("id");

        let display_field = match entity_type.as_str() {
            "contact" => "last_name", // We sort by first name usually but filter by multiple
            "property" => "title",
            "deal" => "name",
            "task" => "title",
            "viewing" => "id", // No good label for viewing
            _ => "id",
        };

        // For contacts we search first/last/email. For others, just the display field.
        let filter_cond = if entity_type == "contact" {
            "(data->>'first_name' ILIKE $3 OR data->>'last_name' ILIKE $3 OR data->>'email' ILIKE $3)"
        } else {
            // dynamic field name in json path require string construction or careful binding
            // simplified: we'll match on specific known fields
            match entity_type.as_str() {
                "property" => "(data->>'title' ILIKE $3 OR data->>'reference' ILIKE $3)",
                "deal" => "(data->>'name' ILIKE $3)",
                "task" => "(data->>'title' ILIKE $3)",
                _ => "true", // fallback
            }
        };

        let sql = format!(
            r#"
            SELECT id, data
            FROM entity_records 
            WHERE tenant_id = $1 
              AND entity_type_id = $2
              AND deleted_at IS NULL
              AND {}
            ORDER BY created_at DESC
            LIMIT 20
            "#,
            filter_cond
        );

        let rows = sqlx::query(&sql)
        .bind(params.tenant_id)
        .bind(et_id)
        .bind(&search_pattern)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        
        rows.iter().map(|row| {
            let id: Uuid = row.try_get("id").unwrap_or_default();
            let data: serde_json::Value = row.try_get("data").unwrap_or(serde_json::json!({}));
            
            let label = match entity_type.as_str() {
                "contact" => {
                    let first = data.get("first_name").and_then(|v| v.as_str()).unwrap_or("");
                    let last = data.get("last_name").and_then(|v| v.as_str()).unwrap_or("");
                    format!("{} {}", first, last).trim().to_string()
                },
                "property" => data.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled Property").to_string(),
                "deal" => data.get("name").and_then(|v| v.as_str()).unwrap_or("Untitled Deal").to_string(),
                "task" => data.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled Task").to_string(),
                _ => id.to_string(),
            };
            
            LookupResult {
                id,
                label: if label.is_empty() { id.to_string() } else { label },
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
    
    // Resolve entity_type_id
    let et_row = sqlx::query("SELECT id FROM entity_types WHERE name = 'contact' AND tenant_id = $1")
        .bind(tenant_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Internal("Contact entity type not found".to_string()))?;
    let et_id: Uuid = et_row.get("id");

    let search_pattern = search.map(|s| format!("%{}%", s));
    
    // Get total count
    let count_sql = if search.is_some() {
        "SELECT COUNT(*) as count FROM entity_records WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL AND (data->>'first_name' ILIKE $3 OR data->>'last_name' ILIKE $3 OR data->>'email' ILIKE $3)"
    } else {
        "SELECT COUNT(*) as count FROM entity_records WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL"
    };
    
    let count_row = if let Some(ref pattern) = search_pattern {
        sqlx::query(count_sql).bind(tenant_id).bind(et_id).bind(pattern).fetch_one(pool).await
    } else {
        sqlx::query(count_sql).bind(tenant_id).bind(et_id).fetch_one(pool).await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    // Get records
    let rows = if let Some(ref pattern) = search_pattern {
        sqlx::query(
            r#"
            SELECT id, data, created_at, updated_at
            FROM entity_records 
            WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL AND (data->>'first_name' ILIKE $5 OR data->>'last_name' ILIKE $5 OR data->>'email' ILIKE $5)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id)
        .bind(et_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .bind(pattern)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query(
            r#"
            SELECT id, data, created_at, updated_at
            FROM entity_records 
            WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id)
        .bind(et_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|row| {
        let id: Uuid = row.try_get("id").unwrap_or_default();
        let mut data: serde_json::Map<String, serde_json::Value> = row.try_get::<serde_json::Value, _>("data").unwrap_or(serde_json::json!({})).as_object().unwrap_or(&serde_json::Map::new()).clone();
        
        // Ensure ID and metadata are included in the flat response if needed, 
        // but frontend might expect them at top level. 
        // The migration put everything in `data`, so `first_name` etc are there.
        // `id` needs to be distinct.
        data.insert("id".to_string(), serde_json::json!(id));
        
        serde_json::Value::Object(data)
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

// Helper to extract fields from JSONB for deals
async fn query_deals(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    limit: i32,
    offset: i32,
    search: Option<&str>,
) -> Result<(Vec<serde_json::Value>, i64), ApiError> {
    use sqlx::Row;

    // Resolve entity_type_id for 'deal'
    let et_row = sqlx::query("SELECT id FROM entity_types WHERE name = 'deal' AND tenant_id = $1")
        .bind(tenant_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Internal("Entity type 'deal' not found".to_string()))?;
    let et_id: Uuid = et_row.get("id");

    let search_pattern = search.map(|s| format!("%{}%", s));

     // Get total count
    let count_sql = if search.is_some() {
        "SELECT COUNT(*) as count FROM entity_records WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL AND (data->>'name' ILIKE $3)"
    } else {
        "SELECT COUNT(*) as count FROM entity_records WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL"
    };
    
    let count_row = if let Some(ref pattern) = search_pattern {
        sqlx::query(count_sql).bind(tenant_id).bind(et_id).bind(pattern).fetch_one(pool).await
    } else {
        sqlx::query(count_sql).bind(tenant_id).bind(et_id).fetch_one(pool).await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    // Get records
    let rows = if let Some(ref pattern) = search_pattern {
        sqlx::query(
            r#"
            SELECT id, data, created_at, updated_at
            FROM entity_records 
            WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL AND (data->>'name' ILIKE $5)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id)
        .bind(et_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .bind(pattern)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query(
            r#"
            SELECT id, data, created_at, updated_at
            FROM entity_records 
            WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id)
        .bind(et_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        let mut map = row.try_get::<serde_json::Value, _>("data").unwrap_or(serde_json::json!({})).as_object().unwrap_or(&serde_json::Map::new()).clone();
        map.insert("id".to_string(), serde_json::Value::String(row.try_get::<Uuid, _>("id").unwrap_or_default().to_string()));
        map.insert("created_at".to_string(), serde_json::Value::String(row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at").unwrap_or_default().to_rfc3339()));
        map.insert("updated_at".to_string(), serde_json::Value::String(row.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at").unwrap_or_default().to_rfc3339()));
        serde_json::Value::Object(map)
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
    
    // Resolve entity_type_id for 'property'
    let et_row = sqlx::query("SELECT id FROM entity_types WHERE name = 'property' AND tenant_id = $1")
        .bind(tenant_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Internal("Entity type 'property' not found".to_string()))?;
    let et_id: Uuid = et_row.get("id");

    let search_pattern = search.map(|s| format!("%{}%", s));
    
    // Get total count
    let count_sql = if search.is_some() {
        "SELECT COUNT(*) as count FROM entity_records WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL AND (data->>'title' ILIKE $3 OR data->>'reference' ILIKE $3 OR data->>'city' ILIKE $3)"
    } else {
        "SELECT COUNT(*) as count FROM entity_records WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL"
    };
    
    let count_row = if let Some(ref pattern) = search_pattern {
        sqlx::query(count_sql).bind(tenant_id).bind(et_id).bind(pattern).fetch_one(pool).await
    } else {
        sqlx::query(count_sql).bind(tenant_id).bind(et_id).fetch_one(pool).await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    // Get records
    let rows = if let Some(ref pattern) = search_pattern {
        sqlx::query(
            r#"
            SELECT id, data, created_at, updated_at
            FROM entity_records 
            WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL AND (data->>'title' ILIKE $5 OR data->>'reference' ILIKE $5 OR data->>'city' ILIKE $5)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id)
        .bind(et_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .bind(pattern)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query(
            r#"
            SELECT id, data, created_at, updated_at
            FROM entity_records 
            WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id)
        .bind(et_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        let mut map = row.try_get::<serde_json::Value, _>("data").unwrap_or(serde_json::json!({})).as_object().unwrap_or(&serde_json::Map::new()).clone();
        map.insert("id".to_string(), serde_json::Value::String(row.try_get::<Uuid, _>("id").unwrap_or_default().to_string()));
        map.insert("created_at".to_string(), serde_json::Value::String(row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at").unwrap_or_default().to_rfc3339()));
        map.insert("updated_at".to_string(), serde_json::Value::String(row.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at").unwrap_or_default().to_rfc3339()));
        serde_json::Value::Object(map)
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

// Helper to extract fields from JSONB for viewings
async fn query_viewings(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    limit: i32,
    offset: i32,
    _search: Option<&str>,
) -> Result<(Vec<serde_json::Value>, i64), ApiError> {
    use sqlx::Row;

    // Resolve entity_type_id for 'viewing'
    let et_row = sqlx::query("SELECT id FROM entity_types WHERE name = 'viewing' AND tenant_id = $1")
        .bind(tenant_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Internal("Entity type 'viewing' not found".to_string()))?;
    let et_id: Uuid = et_row.get("id");
    
    // Get total count
    let count_sql = "SELECT COUNT(*) as count FROM entity_records WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL";
    
    let count_row = sqlx::query(count_sql).bind(tenant_id).bind(et_id).fetch_one(pool).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    // Get records
    let rows = sqlx::query(
        r#"
        SELECT id, data, created_at, updated_at
        FROM entity_records 
        WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL
        ORDER BY data->>'scheduled_at' DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(tenant_id)
    .bind(et_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        let mut map = row.try_get::<serde_json::Value, _>("data").unwrap_or(serde_json::json!({})).as_object().unwrap_or(&serde_json::Map::new()).clone();
        map.insert("id".to_string(), serde_json::Value::String(row.try_get::<Uuid, _>("id").unwrap_or_default().to_string()));
        map.insert("created_at".to_string(), serde_json::Value::String(row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at").unwrap_or_default().to_rfc3339()));
        map.insert("updated_at".to_string(), serde_json::Value::String(row.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at").unwrap_or_default().to_rfc3339()));
        serde_json::Value::Object(map)
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
    
    // Resolve entity_type_id for 'task'
    let et_row = sqlx::query("SELECT id FROM entity_types WHERE name = 'task' AND tenant_id = $1")
        .bind(tenant_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Internal("Entity type 'task' not found".to_string()))?;
    let et_id: Uuid = et_row.get("id");

    let search_pattern = search.map(|s| format!("%{}%", s));
    
    // Get total count
    let count_sql = if search.is_some() {
        "SELECT COUNT(*) as count FROM entity_records WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL AND (data->>'title' ILIKE $3)"
    } else {
        "SELECT COUNT(*) as count FROM entity_records WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL"
    };

    let count_row = if let Some(ref pattern) = search_pattern {
        sqlx::query(count_sql).bind(tenant_id).bind(et_id).bind(pattern).fetch_one(pool).await
    } else {
        sqlx::query(count_sql).bind(tenant_id).bind(et_id).fetch_one(pool).await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    // Get records
    let rows = if let Some(ref pattern) = search_pattern {
        sqlx::query(
            r#"
            SELECT id, data, created_at, updated_at
            FROM entity_records 
            WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL AND (data->>'title' ILIKE $5)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id)
        .bind(et_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .bind(pattern)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query(
            r#"
            SELECT id, data, created_at, updated_at
            FROM entity_records 
            WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tenant_id)
        .bind(et_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        let mut map = row.try_get::<serde_json::Value, _>("data").unwrap_or(serde_json::json!({})).as_object().unwrap_or(&serde_json::Map::new()).clone();
        map.insert("id".to_string(), serde_json::Value::String(row.try_get::<Uuid, _>("id").unwrap_or_default().to_string()));
        map.insert("created_at".to_string(), serde_json::Value::String(row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at").unwrap_or_default().to_rfc3339()));
        map.insert("updated_at".to_string(), serde_json::Value::String(row.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at").unwrap_or_default().to_rfc3339()));
        serde_json::Value::Object(map)
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
                r#"INSERT INTO entity_records (id, tenant_id, entity_type_id, data, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6)"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(_entity.id)
            .bind(&data)
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
            sqlx::query(
                r#"INSERT INTO entity_records (id, tenant_id, entity_type_id, data, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6)"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(_entity.id)
            .bind(&data)
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "property" => {
            sqlx::query(
                r#"INSERT INTO entity_records (id, tenant_id, entity_type_id, data, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6)"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(_entity.id)
            .bind(&data)
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "task" => {
            sqlx::query(
                r#"INSERT INTO entity_records (id, tenant_id, entity_type_id, data, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6)"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(_entity.id)
            .bind(&data)
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "viewing" => {
            sqlx::query(
                r#"INSERT INTO entity_records (id, tenant_id, entity_type_id, data, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6)"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(_entity.id)
            .bind(&data)
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
            sqlx::query("SELECT id, data FROM entity_records WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL")
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

        "deal" | "task" | "property" | "viewing" => {
            sqlx::query("SELECT id, data FROM entity_records WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL")
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
                "contact" | "deal" | "task" | "property" | "viewing" => {
                    let mut data: serde_json::Map<String, serde_json::Value> = r.try_get::<serde_json::Value, _>("data").unwrap_or(serde_json::json!({})).as_object().unwrap_or(&serde_json::Map::new()).clone();
                    data.insert("id".to_string(), serde_json::json!(r.try_get::<Uuid, _>("id").unwrap_or_default()));
                    serde_json::Value::Object(data)
                },
                "company" => serde_json::json!({
                    "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
                    "name": r.try_get::<String, _>("name").unwrap_or_default(),
                    "domain": r.try_get::<Option<String>, _>("domain").ok().flatten(),
                    "industry": r.try_get::<Option<String>, _>("industry").ok().flatten(),
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
                r#"UPDATE entity_records SET 
                   data = data || $3,
                   updated_at = $4
                   WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(&data)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        "property" | "deal" | "task" | "viewing" => {
             sqlx::query(
                r#"UPDATE entity_records SET 
                   data = data || $3,
                   updated_at = $4
                   WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"#
            )
            .bind(id)
            .bind(query.tenant_id)
            .bind(&data)
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

    // Migrated entities (Entity Records)
    if ["contact", "deal", "task", "property", "viewing"].contains(&entity_type.as_str()) {
        sqlx::query("UPDATE entity_records SET deleted_at = $3 WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(query.tenant_id)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        return Ok(Json(serde_json::json!({ "deleted": true })));
    }

    // Legacy tables handling
    let table = match entity_type.as_str() {
        "company" => "companies",
        "listing" => "listings",
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
