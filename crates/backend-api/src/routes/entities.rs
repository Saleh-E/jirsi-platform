use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;
use crate::state::AppState;
use crate::middleware::tenant::ResolvedTenant;
use core_models::{FieldDef, FieldType}; 

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub search: Option<String>,
    pub view_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub data: Vec<Value>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Deserialize)]
pub struct LookupQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LookupResult {
    pub id: Uuid,
    pub label: String,
    pub sub_label: Option<String>,
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // Record CRUD
        // Mounted at /api/v1/entities usually, but user requested /records. 
        // We will support BOTH standard REST /:entity_code and explicit /records/:entity_code 
        // to satisfy all requirements/legacy compatibility if needed.
        // However, standard REST convention for "entities" module usually implies mounting at /entities.
        // If main.rs mounts this at /records, then paths are /:entity_code.
        // If main.rs mounts this at /api/v1, then full paths needed.
        // Let's assume we replace the old entities router which was mounted at /entities or /api/v1.
        
        // For compliance with "Action A: POST /api/v1/records/:entity_code", we define absolute paths here
        // assuming this router is merged into the api_v1 router.
        
        // Generic CRUD
        .route("/records/:entity_code", get(list_records).post(create_record))
        .route("/records/:entity_code/:id", get(get_record).put(update_record).delete(delete_record))
        
        // Legacy/Standard alias (so /entities/contact also works if needed, or for fetching contacts via old frontend)
        .route("/entities/:entity_code", get(list_records).post(create_record))
        .route("/entities/:entity_code/:id", get(get_record).put(update_record).delete(delete_record))
        
        // Lookup
        .route("/lookup/:entity_code", get(lookup_entity))
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /records/:entity_code
async fn list_records(
    State(state): State<Arc<AppState>>,
    Path(entity_code): Path<String>,
    Query(query): Query<ListQuery>,
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
) -> impl IntoResponse {
    let mut conn = match get_rls_connection(&state.pool, tenant.id).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 1. Resolve Entity Type
    let entity_type = match state.metadata.get_entity_type(tenant.id, &entity_code).await {
        Ok(e) => e,
        Err(_) => return (StatusCode::NOT_FOUND, format!("Entity type '{}' not found", entity_code)).into_response(),
    };

    // 2. Pagination
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(25).min(100).max(1);
    let offset = (page - 1) * per_page;

    // 3. View Logic (Filtering/Sorting)
    let sort_clause = "ORDER BY created_at DESC".to_string();
    
    // 4. Query
    let count_sql = "SELECT COUNT(*) as count FROM entity_records WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL";
    let count_row = sqlx::query(count_sql)
        .bind(tenant.id)
        .bind(entity_type.id)
        .fetch_one(&mut *conn)
        .await;

    let total: i64 = match count_row {
        Ok(r) => r.try_get("count").unwrap_or(0),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let sql = format!(
        "SELECT id, data, created_at, updated_at FROM entity_records WHERE tenant_id = $1 AND entity_type_id = $2 AND deleted_at IS NULL {} LIMIT $3 OFFSET $4",
        sort_clause
    );

    let rows = sqlx::query(&sql)
        .bind(tenant.id)
        .bind(entity_type.id)
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&mut *conn)
        .await;

    match rows {
        Ok(results) => {
             let data: Vec<Value> = results.iter().map(|row| {
                let mut map = row.try_get::<Value, _>("data").unwrap_or(serde_json::json!({})).as_object().unwrap_or(&serde_json::Map::new()).clone();
                // Inject system fields
                map.insert("id".to_string(), serde_json::json!(row.get::<Uuid, _>("id")));
                map.insert("created_at".to_string(), serde_json::json!(row.get::<chrono::DateTime<chrono::Utc>, _>("created_at")));
                map.insert("updated_at".to_string(), serde_json::json!(row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at")));
                Value::Object(map)
            }).collect();

            Json(ListResponse { data, total, page, per_page }).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// POST /records/:entity_code
async fn create_record(
    State(state): State<Arc<AppState>>,
    Path(entity_code): Path<String>,
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let mut conn = match get_rls_connection(&state.pool, tenant.id).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 1. Resolve Metadata
    let fields = match state.metadata.get_fields_by_entity_name(tenant.id, &entity_code).await {
        Ok(f) => f,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid entity type: {}", e)).into_response(),
    };
    
    let entity_type = state.metadata.get_entity_type(tenant.id, &entity_code).await.unwrap();

    // 2. Validate
    let processed_data = match validate_and_process_payload(&fields, &payload, false) {
        Ok(d) => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))).into_response(),
    };

    // 3. Insert
    let new_id = Uuid::new_v4();
    let result = sqlx::query(
        "INSERT INTO entity_records (id, tenant_id, entity_type_id, data) VALUES ($1, $2, $3, $4) RETURNING id"
    )
    .bind(new_id)
    .bind(tenant.id)
    .bind(entity_type.id)
    .bind(processed_data)
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(row) => {
            Json(serde_json::json!({
                "id": row.get::<Uuid, _>("id"),
                "status": "created"
            })).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// GET /records/:entity_code/:id
async fn get_record(
    State(state): State<Arc<AppState>>,
    Path((_entity_code, id)): Path<(String, Uuid)>,
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
) -> impl IntoResponse {
    let mut conn = match get_rls_connection(&state.pool, tenant.id).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let result = sqlx::query("SELECT id, data FROM entity_records WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL")
        .bind(id)
        .bind(tenant.id)
        .fetch_optional(&mut *conn)
        .await;

    match result {
        Ok(Some(row)) => {
            let mut data = row.get::<Value, _>("data");
            if let Some(obj) = data.as_object_mut() {
                obj.insert("id".to_string(), serde_json::json!(row.get::<Uuid, _>("id")));
            }
            Json(data).into_response()
        },
        Ok(None) => (StatusCode::NOT_FOUND, "Record not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// PUT /records/:entity_code/:id
async fn update_record(
    State(state): State<Arc<AppState>>,
    Path((entity_code, id)): Path<(String, Uuid)>,
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let mut conn = match get_rls_connection(&state.pool, tenant.id).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let fields = match state.metadata.get_fields_by_entity_name(tenant.id, &entity_code).await {
        Ok(f) => f,
        Err(_) => return (StatusCode::NOT_FOUND, "Entity type not found").into_response(),
    };

    // Validate (is_update = true -> allow partials)
    let processed_data = match validate_and_process_payload(&fields, &payload, true) {
        Ok(d) => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))).into_response(),
    };

    // Update using JSONB merge (|| operator)
    let result = sqlx::query(
        "UPDATE entity_records SET data = data || $1, updated_at = NOW() WHERE id = $2 AND tenant_id = $3 RETURNING id"
    )
    .bind(processed_data)
    .bind(id)
    .bind(tenant.id)
    .execute(&mut *conn)
    .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                Json(serde_json::json!({"status": "updated"})).into_response()
            } else {
                (StatusCode::NOT_FOUND, "Record not found").into_response()
            }
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// DELETE /records/:entity_code/:id
async fn delete_record(
    State(state): State<Arc<AppState>>,
    Path((_entity_code, id)): Path<(String, Uuid)>,
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
) -> impl IntoResponse {
    let mut conn = match get_rls_connection(&state.pool, tenant.id).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Soft delete
    let result = sqlx::query("UPDATE entity_records SET deleted_at = NOW() WHERE id = $1 AND tenant_id = $2")
        .bind(id)
        .bind(tenant.id)
        .execute(&mut *conn)
        .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                Json(serde_json::json!({"status": "deleted"})).into_response()
            } else {
                 (StatusCode::NOT_FOUND, "Record not found").into_response()
            }
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// GET /lookup/:entity_code?q=search_term
async fn lookup_entity(
    State(state): State<Arc<AppState>>,
    Path(entity_code): Path<String>,
    Query(query): Query<LookupQuery>,
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
) -> impl IntoResponse {
    let mut conn = match get_rls_connection(&state.pool, tenant.id).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let entity_type = match state.metadata.get_entity_type(tenant.id, &entity_code).await {
        Ok(e) => e,
        Err(_) => return (StatusCode::NOT_FOUND, "Entity type not found").into_response(),
    };

    // Determine display field. Currently inferred from code.
    // Future: add `display_field` col to entity_types
    let display_field_sql = match entity_code.as_str() {
        "contact" => "data->>'first_name' || ' ' || data->>'last_name'",
        "company" | "deal" => "data->>'name'",
        "task" | "property" => "data->>'title'",
        "contract" => "data->>'contract_number'",
        _ => "data->>'name'", // Fallback
    };
    
    // Search filter
    let search_term = query.q.unwrap_or_default();
    let search_pattern = format!("%{}%", search_term);
    
    let sql = format!(
        r#"
        SELECT id, {} as label
        FROM entity_records 
        WHERE tenant_id = $1 
          AND entity_type_id = $2 
          AND deleted_at IS NULL
          AND ({} ILIKE $3)
        ORDER BY created_at DESC
        LIMIT 20
        "#,
        display_field_sql, display_field_sql 
    );

    let rows = sqlx::query(&sql)
        .bind(tenant.id)
        .bind(entity_type.id)
        .bind(search_pattern)
        .fetch_all(&mut *conn)
        .await;

    match rows {
        Ok(results) => {
            let data: Vec<LookupResult> = results.iter().map(|row| {
                 LookupResult {
                     id: row.get("id"),
                     label: row.get::<String, _>("label").trim().to_string(),
                     sub_label: None,
                 }
            }).collect();
            Json(data).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ============================================================================
// Helpers
// ============================================================================

async fn get_rls_connection(pool: &PgPool, tenant_id: Uuid) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    sqlx::query("SELECT set_config('app.current_tenant', $1::text, false)")
        .bind(tenant_id)
        .execute(&mut *conn)
        .await?;
    Ok(conn)
}

fn validate_and_process_payload(
    fields: &[FieldDef], 
    payload: &Value, 
    is_update: bool
) -> Result<Value, String> {
    let mut processed = payload.clone();

    if !processed.is_object() {
        return Err("Payload must be a JSON object".to_string());
    }
    
    let obj = processed.as_object_mut().unwrap();

    for field in fields {
        let value = obj.get(&field.name);

        // 1. Default Value (if missing on CREATE)
        if value.is_none() && !is_update {
            if let Some(default) = &field.default_value {
                obj.insert(field.name.clone(), default.clone());
            }
        }
        
        let value = obj.get(&field.name); // Refresh

        // 2. Required Check
        if field.is_required && !is_update {
             if value.is_none() || value.unwrap().is_null() {
                  return Err(format!("Field '{}' is required", field.label));
             }
        }

        // 3. Type Validation
        if let Some(v) = value {
            if !v.is_null() {
                validate_field_type(field, v)?;
            }
        }
    }
    
    Ok(processed)
}

fn validate_field_type(field: &FieldDef, value: &Value) -> Result<(), String> {
    match field.field_type {
        FieldType::Text | FieldType::TextArea | FieldType::Email | FieldType::Phone | FieldType::Url => {
            if !value.is_string() {
                return Err(format!("Field '{}' must be a text string", field.label));
            }
            if let Some(val) = &field.validation {
                let s = value.as_str().unwrap();
                if let Some(min) = val.min_length {
                    if (s.len() as i32) < min { return Err(format!("'{}' strictly needs {} characters", field.label, min)); }
                }
                if let Some(max) = val.max_length {
                     if (s.len() as i32) > max { return Err(format!("'{}' exceeds max {} characters", field.label, max)); }
                }
            }
        },
        FieldType::Number { .. } | FieldType::Money { .. } | FieldType::Score { .. } => {
             if !value.is_number() {
                 return Err(format!("Field '{}' must be a number", field.label));
             }
             if let Some(val) = &field.validation {
                 let n = value.as_f64().unwrap();
                 if let Some(min) = val.min_value {
                     if n < min { return Err(format!("'{}' must be >= {}", field.label, min)); }
                 }
                 if let Some(max) = val.max_value {
                     if n > max { return Err(format!("'{}' must be <= {}", field.label, max)); }
                 }
             }
        },
        FieldType::Boolean => {
             if !value.is_boolean() {
                 return Err(format!("Field '{}' must be a boolean", field.label));
             }
        },
        FieldType::Date => {
             if !value.is_string() { return Err(format!("Date field '{}' must be string", field.label)); }
        },
        FieldType::DateTime => {
             if !value.is_string() { return Err(format!("DateTime field '{}' must be string", field.label)); }
        },
        FieldType::Link { .. } => {
             if !value.is_string() { return Err(format!("Link field '{}' must be a valid ID string", field.label)); }
        }
        _ => {}
    }
    Ok(())
}
