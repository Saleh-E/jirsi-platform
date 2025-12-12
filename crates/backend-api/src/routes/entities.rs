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

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/:entity_type", get(list_records))
        .route("/:entity_type", post(create_record))
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
        _ => return Err(ApiError::NotFound(format!("Entity type {} not supported for create", entity_type))),
    }

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
        _ => return Err(ApiError::NotFound(format!("Entity type {} not supported for update", entity_type))),
    }

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
