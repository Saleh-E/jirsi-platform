use axum::{
    Router,
    routing::{get, post, delete},
    extract::{State, Path, Query},
    Json,
};
use core_models::{AppDef, EntityType, FieldDef, ViewDef};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::AppState;
use crate::error::ApiError;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/apps", get(list_apps))
        .route("/entities", get(list_entities))
        .route("/entities/:name", get(get_entity))
        .route("/entities/:name/fields", get(get_fields))
        .route("/entities/:name/fields/:field_id/options", post(add_field_option))
        .route("/entities/:name/fields/:field_id/options/:option_value", delete(delete_field_option))
        .route("/entities/:name/views", get(get_views))
        .route("/entities/:name/views", post(create_view))
}

#[derive(Debug, Deserialize)]
pub struct TenantQuery {
    pub tenant_id: Uuid,
}

async fn list_apps(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<Vec<AppDef>>, ApiError> {
    let apps = state.metadata.list_apps(query.tenant_id).await?;
    Ok(Json(apps))
}

#[derive(Debug, Deserialize)]
pub struct ListEntitiesQuery {
    pub tenant_id: Uuid,
    pub app_id: Option<String>,
}

async fn list_entities(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListEntitiesQuery>,
) -> Result<Json<Vec<EntityType>>, ApiError> {
    let entities = state.metadata
        .list_entity_types(query.tenant_id, query.app_id.as_deref())
        .await?;
    Ok(Json(entities))
}

async fn get_entity(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<EntityType>, ApiError> {
    let entity = state.metadata
        .get_entity_type(query.tenant_id, &name)
        .await?;
    Ok(Json(entity))
}

async fn get_fields(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<Vec<FieldDef>>, ApiError> {
    let fields = state.metadata
        .get_fields_by_entity_name(query.tenant_id, &name)
        .await?;
    Ok(Json(fields))
}

async fn get_views(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<Vec<ViewDef>>, ApiError> {
    let entity = state.metadata
        .get_entity_type(query.tenant_id, &name)
        .await?;
    let views = state.metadata
        .get_views(query.tenant_id, entity.id)
        .await?;
    Ok(Json(views))
}

/// Request body for creating a personal view
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields used in future view creation logic
pub struct CreateViewRequest {
    pub name: String,
    pub label: String,
    pub view_type: String,
    pub columns: Option<Vec<String>>,
    pub filters: Option<serde_json::Value>,
    pub sort: Option<serde_json::Value>,
    pub group_by: Option<String>,
    pub settings: Option<serde_json::Value>,
}

/// Create a personal view for the current user
async fn create_view(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(query): Query<TenantQuery>,
    Json(payload): Json<CreateViewRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Get entity type
    let entity = state.metadata
        .get_entity_type(query.tenant_id, &name)
        .await?;
    
    let view_id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();
    
    // Insert view into database
    sqlx::query(
        r#"INSERT INTO view_defs 
           (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, 
            columns, filters, sort, settings, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, false, false, $7, $8, $9, $10, $11, $12)"#
    )
    .bind(view_id)
    .bind(query.tenant_id)
    .bind(entity.id)
    .bind(&payload.name)
    .bind(&payload.label)
    .bind(&payload.view_type)
    .bind(serde_json::to_value(payload.columns.unwrap_or_default()).unwrap())
    .bind(payload.filters.unwrap_or_else(|| serde_json::json!([])))
    .bind(payload.sort.unwrap_or_else(|| serde_json::json!([])))
    .bind(payload.settings.unwrap_or_else(|| serde_json::json!({})))
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;
    
    Ok(Json(serde_json::json!({
        "id": view_id,
        "name": payload.name,
        "created": true
    })))
}

/// Request body for adding a field option
#[derive(Debug, Deserialize)]
pub struct AddFieldOptionRequest {
    pub value: String,
    pub label: Option<String>,
    pub color: Option<String>,
}

/// Add a new option to a Select/Status field's options list
async fn add_field_option(
    State(state): State<Arc<AppState>>,
    Path((entity_name, field_id)): Path<(String, Uuid)>,
    Query(query): Query<TenantQuery>,
    Json(payload): Json<AddFieldOptionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let now = chrono::Utc::now();
    
    // Build the new option object
    let label = payload.label.unwrap_or_else(|| payload.value.clone());
    let mut new_option = serde_json::json!({
        "value": payload.value,
        "label": label
    });
    if let Some(color) = payload.color {
        new_option["color"] = serde_json::json!(color);
    }
    
    // Debug: Log what we're trying to insert
    tracing::debug!(
        field_id = %field_id,
        new_option = %new_option,
        "Attempting to add new option to field"
    );
    
    // Update the field's options by appending the new option to the JSON array
    // The options column stores JSONB, we use jsonb_insert or array concatenation
    let result = sqlx::query(
        r#"UPDATE field_defs 
           SET options = COALESCE(options, '[]'::jsonb) || $1::jsonb,
               updated_at = $2
           WHERE id = $3 AND tenant_id = $4"#
    )
    .bind(serde_json::json!([new_option]))
    .bind(now)
    .bind(field_id)
    .bind(query.tenant_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "SQL error while adding option");
        ApiError::Internal(e.to_string())
    })?;
    
    tracing::debug!(
        rows_affected = result.rows_affected(),
        "SQL UPDATE result for adding option"
    );
    
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound(format!("Field {} not found", field_id)));
    }
    
    tracing::info!(
        entity = %entity_name,
        field_id = %field_id,
        option = %payload.value,
        "Successfully added new option to field"
    );
    
    // IMPORTANT: Invalidate the fields cache so the updated options appear on next fetch
    // First get the entity type to get its ID for cache invalidation
    if let Ok(entity) = state.metadata.get_entity_type(query.tenant_id, &entity_name).await {
        state.metadata.invalidate_fields(entity.id);
        tracing::debug!(
            entity_type_id = %entity.id,
            "Invalidated fields cache after adding option"
        );
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "option": new_option
    })))
}

/// Delete an option from a Select/Status field's options list
async fn delete_field_option(
    State(state): State<Arc<AppState>>,
    Path((entity_name, field_id, option_value)): Path<(String, Uuid, String)>,
    Query(query): Query<TenantQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let now = chrono::Utc::now();
    
    // Decode URL-encoded option value (in case it has special characters)
    let option_value = urlencoding::decode(&option_value)
        .map(|s| s.to_string())
        .unwrap_or(option_value);
    
    tracing::debug!(
        field_id = %field_id,
        option_value = %option_value,
        "Attempting to delete option from field"
    );
    
    // Remove the option from the JSONB array where value matches
    // This uses jsonb_path_query_array to filter out the matching element
    let result = sqlx::query(
        r#"UPDATE field_defs 
           SET options = (
               SELECT COALESCE(jsonb_agg(elem), '[]'::jsonb)
               FROM jsonb_array_elements(COALESCE(options, '[]'::jsonb)) AS elem
               WHERE elem->>'value' != $1
           ),
           updated_at = $2
           WHERE id = $3 AND tenant_id = $4"#
    )
    .bind(&option_value)
    .bind(now)
    .bind(field_id)
    .bind(query.tenant_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "SQL error while deleting option");
        ApiError::Internal(e.to_string())
    })?;
    
    tracing::debug!(
        rows_affected = result.rows_affected(),
        "SQL UPDATE result for deleting option"
    );
    
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound(format!("Field {} not found", field_id)));
    }
    
    tracing::info!(
        entity = %entity_name,
        field_id = %field_id,
        option = %option_value,
        "Successfully deleted option from field"
    );
    
    // Invalidate the fields cache
    if let Ok(entity) = state.metadata.get_entity_type(query.tenant_id, &entity_name).await {
        state.metadata.invalidate_fields(entity.id);
        tracing::debug!(
            entity_type_id = %entity.id,
            "Invalidated fields cache after deleting option"
        );
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "deleted_value": option_value
    })))
}
