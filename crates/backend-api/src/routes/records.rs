use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
    routing::{get, post, patch},
    Router,
};
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;
use crate::state::AppState;
use crate::middleware::tenant::ResolvedTenant;
use core_models::{FieldDef, FieldType}; // Need to verify if core_models is accessible/re-exported or add imports

// Validation helper
fn validate_and_process_payload(
    fields: &[FieldDef], 
    payload: &Value, 
    is_update: bool
) -> Result<Value, String> {
    let mut processed = payload.clone();

    // If payload is not an object, fail (unless it's an update and we allow partials? No, record data must be object)
    if !processed.is_object() {
        return Err("Payload must be a JSON object".to_string());
    }
    
    let obj = processed.as_object_mut().unwrap();

    for field in fields {
        let value = obj.get(&field.name);

        // 1. apply default if missing and not update (or if update and we want to set default? usually only on create)
        if value.is_none() && !is_update {
            if let Some(default) = &field.default_value {
                obj.insert(field.name.clone(), default.clone());
            }
        }
        
        let value = obj.get(&field.name); // Get again after default

        // 2. Check required
        if field.is_required && !is_update {
            if value.is_none() || value.unwrap().is_null() {
                 return Err(format!("Field '{}' is required", field.label));
            }
        }

        // 3. Type Check & Validation (Skip if missing/null unless required checked above)
        if let Some(v) = value {
            if !v.is_null() {
                validate_field_type(field, v)?;
            }
        }
    }
    
    // TODO: Strip unknown fields? Or allow flexible schema? 
    // For now allow flexibility, or strict? stricter is better for "metadata driven".
    // strict: remove keys not in fields?
    
    Ok(processed)
}

fn validate_field_type(field: &FieldDef, value: &Value) -> Result<(), String> {
    match field.field_type {
        FieldType::Text | FieldType::TextArea | FieldType::RichText | FieldType::Email | FieldType::Phone | FieldType::Url => {
            if !value.is_string() {
                return Err(format!("Field '{}' must be a string", field.label));
            }
            // Length check
            if let Some(val) = &field.validation {
                let s = value.as_str().unwrap();
                if let Some(min) = val.min_length {
                    if (s.len() as i32) < min {
                         return Err(format!("Field '{}' is too short (min {})", field.label, min));
                    }
                }
                if let Some(max) = val.max_length {
                     if (s.len() as i32) > max {
                         return Err(format!("Field '{}' is too long (max {})", field.label, max));
                    }
                }
                // Regex pattern
                if let Some(pattern) = &val.pattern {
                     // TODO: Compile regex and check. proper implementation needs cached regex.
                     // For MVP, maybe skip or simple check?
                }
            }
        },
        FieldType::Number { .. } | FieldType::Money { .. } | FieldType::Score { .. } => {
             if !value.is_number() {
                 return Err(format!("Field '{}' must be a number", field.label));
             }
             // Min/Max value
             if let Some(val) = &field.validation {
                 let n = value.as_f64().unwrap();
                 if let Some(min) = val.min_value {
                     if n < min {
                         return Err(format!("Field '{}' must be >= {}", field.label, min));
                     }
                 }
                 if let Some(max) = val.max_value {
                     if n > max {
                         return Err(format!("Field '{}' must be <= {}", field.label, max));
                     }
                 }
             }
        },
        FieldType::Boolean => {
            if !value.is_boolean() {
                return Err(format!("Field '{}' must be a boolean", field.label));
            }
        },
        // TODO: Other types
        _ => {} 
    }
    Ok(())
}

async fn get_rls_connection(pool: &PgPool, tenant_id: Uuid) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    
    // Set the session variable
    sqlx::query("SELECT set_config('app.current_tenant', $1::text, false)")
        .bind(tenant_id)
        .execute(&mut *conn)
        .await?;
        
    Ok(conn)
}

// Routes configuration
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/:entity_code/records", post(create_record))
        .route("/:entity_code/records", get(list_records))
        .route("/:entity_code/records/:id", get(get_record))
        .route("/:entity_code/records/:id", patch(update_record))
}

// Handlers

async fn list_records(
    State(state): State<Arc<AppState>>,
    Path(entity_code): Path<String>,
    // We get the tenant from the middleware extension
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
) -> impl IntoResponse {
    let mut conn = match get_rls_connection(&state.pool, tenant.id).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 1. Resolve entity_type_id from code (e.g. 'crm.contact')
    // We should cache this or look it up. For now, just query.
    // Note: this query also needs to be tenant-scoped if entity_types is tenant-scoped!
    // existing schema says entity_types has tenant_id.
    
    // WARNING: 'entity_types' doesn't have RLS enabled yet in my migration, but it should be scoped by tenant_id manually if not.
    // Or we use the RLS connection and existing RLS policy?
    // I didn't enable RLS on entity_types. So I must use WHERE tenant_id = $1.
    
    let entity_type = sqlx::query("SELECT id FROM entity_types WHERE name = $1 AND tenant_id = $2")
        .bind(&entity_code)
        .bind(tenant.id)
        .fetch_optional(&mut *conn)
        .await;

    let entity_type_id = match entity_type {
        Ok(Some(row)) => row.get::<Uuid, _>("id"),
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Entity type {} not found", entity_code)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // 2. Query records
    // Since we are using RLS connection, we just say WHERE entity_type_id = $1
    // The tenant filter is automatic via RLS policy 'tenant_isolation_policy'.
    
    let records = sqlx::query("SELECT * FROM entity_records WHERE entity_type_id = $1 ORDER BY created_at DESC LIMIT 100")
        .bind(entity_type_id)
        .fetch_all(&mut *conn)
        .await;

    match records {
        Ok(rows) => {
            let result: Vec<Value> = rows.into_iter().map(|row| {
                serde_json::json!({
                    "id": row.get::<Uuid, _>("id"),
                    "human_id": row.get::<Option<String>, _>("human_id"),
                    "data": row.get::<Value, _>("data"),
                    "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                })
            }).collect();
            Json(result).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

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

    // Resolve Type
    let entity_type = match state.metadata.get_entity_type(tenant.id, &entity_code).await {
        Ok(e) => e,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Validate
    let fields = match state.metadata.get_fields_by_entity_name(tenant.id, &entity_code).await {
        Ok(f) => f,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let processed_data = match validate_and_process_payload(&fields, &payload, false) {
        Ok(d) => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))).into_response(),
    };

    // Insert
    // RLS policy checks tenant_id = current_setting.
    // We must provide tenant_id in INSERT as well, and it must match!
    let new_id = Uuid::new_v4();
    
    let result = sqlx::query(
        "INSERT INTO entity_records (id, tenant_id, entity_type_id, data) VALUES ($1, $2, $3, $4) RETURNING id, created_at"
    )
    .bind(new_id)
    .bind(tenant.id)
    .bind(entity_type.id)
    .bind(&processed_data)

    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(row) => Json(serde_json::json!({
            "id": row.get::<Uuid, _>("id"),
            "status": "created"
        })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_record(
    State(state): State<Arc<AppState>>,
    Path((_entity_code, id)): Path<(String, Uuid)>,
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
) -> impl IntoResponse {
    let mut conn = match get_rls_connection(&state.pool, tenant.id).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // RLS handles visibility
    let result = sqlx::query("SELECT * FROM entity_records WHERE id = $1")
        .bind(id)
        .fetch_optional(&mut *conn)
        .await;

    match result {
        Ok(Some(row)) => Json(serde_json::json!({
            "id": row.get::<Uuid, _>("id"),
            "data": row.get::<Value, _>("data")
        })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

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
    
    // We merge data? or replace? Usually PATCH is merge.
    // Postgres jsonb_concat (||) or explicit merge.
    // For proper validation, we need to merge in application code then validate the FULL object?
    // Or just validate the fields present in PATCH?
    // Usually partial validation.
    
    // Fetch generic fields to validate inputs
     let entity_type = match state.metadata.get_entity_type(tenant.id, &entity_code).await {
         Ok(e) => e,
         Err(e) => return (StatusCode::NOT_FOUND, format!("Entity type not found: {}", e)).into_response(),
     };
     
     let fields = match state.metadata.get_fields_by_entity_name(tenant.id, &entity_code).await {
         Ok(f) => f,
         Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load fields: {}", e)).into_response(),
     };
     
     // Validate partial payload
     let validated_patch = match validate_and_process_payload(&fields, &payload, true) {
         Ok(d) => d,
         Err(msg) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))).into_response(),
     };

    let result = sqlx::query("UPDATE entity_records SET data = data || $1 WHERE id = $2 RETURNING id")
        .bind(validated_patch)

        .bind(id)
        .fetch_optional(&mut *conn) // RLS applies
        .await;
        
    match result {
        Ok(Some(_)) => Json(serde_json::json!({"status": "updated"})).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
