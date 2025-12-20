use axum::{
    Router,
    routing::{get, post, delete, patch},
    extract::{State, Path, Query},
    Json,
};
use core_models::{AppDef, EntityType, FieldDef, ViewDef};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::tenant::ResolvedTenant;
use crate::state::AppState;
use crate::error::ApiError;
use axum::Extension;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/apps", get(list_apps))
        .route("/entities", get(list_entities).post(create_entity))
        .route("/entities/:name", get(get_entity))
        .route("/entities/:name/fields", get(get_fields).post(create_field))
        .route("/entities/:name/fields/:field_id", patch(update_field))
        .route("/entities/:name/fields/:field_id/options", post(add_field_option))
        .route("/entities/:name/fields/:field_id/options/:option_value", delete(delete_field_option))
        .route("/entities/:name/views", get(get_views).post(create_view))
}


#[derive(Debug, Deserialize)]
pub struct TenantQuery {
    pub tenant_id: Uuid,
}

async fn list_apps(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<ResolvedTenant>,
) -> Result<Json<Vec<AppDef>>, ApiError> {
    let apps = state.metadata.list_apps(tenant.id).await?;
    Ok(Json(apps))
}

#[derive(Debug, Deserialize)]
pub struct ListEntitiesQuery {
    pub app_id: Option<String>,
}

async fn list_entities(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<ResolvedTenant>,
    Query(query): Query<ListEntitiesQuery>,
) -> Result<Json<Vec<EntityType>>, ApiError> {
    let entities = state.metadata
        .list_entity_types(tenant.id, query.app_id.as_deref())
        .await?;
    Ok(Json(entities))
}

async fn get_entity(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Extension(tenant): Extension<ResolvedTenant>,
) -> Result<Json<EntityType>, ApiError> {
    let entity = state.metadata
        .get_entity_type(tenant.id, &name)
        .await?;
    Ok(Json(entity))
}

#[derive(Debug, Deserialize)]
pub struct CreateEntityRequest {
    pub app_id: String,
    pub name: String,
    pub label: String,
    pub icon: Option<String>,
    pub description: Option<String>,
}

async fn create_entity(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<ResolvedTenant>,
    Json(payload): Json<CreateEntityRequest>,
) -> Result<Json<Uuid>, ApiError> {
    let now = chrono::Utc::now();
    let id = Uuid::new_v4();

    sqlx::query(
        r#"INSERT INTO entity_types 
           (id, tenant_id, app_id, name, label, label_plural, icon, description, flags, soft_delete, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '{}', true, $9, $10)"#
    )
    .bind(id)
    .bind(tenant.id)
    .bind(&payload.app_id)
    .bind(&payload.name)
    .bind(&payload.label)
    .bind(format!("{}s", payload.label)) // Simple pluralization default
    .bind(&payload.icon)
    .bind(&payload.description)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create entity: {}", e);
        ApiError::Internal(e.to_string())
    })?;

    Ok(Json(id))
}


async fn get_fields(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Extension(tenant): Extension<ResolvedTenant>,
) -> Result<Json<Vec<FieldDef>>, ApiError> {
    let fields = state.metadata
        .get_fields_by_entity_name(tenant.id, &name)
        .await?;
    Ok(Json(fields))
}

#[derive(Debug, Deserialize)]
pub struct CreateFieldRequest {
    pub name: String,
    pub label: String,
    pub field_type: serde_json::Value, // Accept generic JSON for the Enum
    pub is_required: Option<bool>,
    pub is_unique: Option<bool>,
    pub show_in_list: Option<bool>,
    pub show_in_card: Option<bool>,
    pub validation: Option<serde_json::Value>,
    pub ui_hints: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

async fn create_field(
    State(state): State<Arc<AppState>>,
    Path(entity_name): Path<String>,
    Extension(tenant): Extension<ResolvedTenant>,
    Json(payload): Json<CreateFieldRequest>,
) -> Result<Json<Uuid>, ApiError> {
    // 1. Resolve Entity ID
    let entity = state.metadata.get_entity_type(tenant.id, &entity_name).await?;
    
    // 2. Insert Field
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    
    // Parse field_type logic matches repository: split type string and options
    let (type_str, options_val) = match &payload.field_type {
        serde_json::Value::String(s) => (s.clone(), serde_json::json!({})),
        serde_json::Value::Object(map) => {
            let t = map.get("type").and_then(|v| v.as_str()).unwrap_or("Text").to_string();
            let c = map.get("config").cloned().unwrap_or(serde_json::json!({}));
            (t, c)
        },
        _ => ("Text".to_string(), serde_json::json!({})),
    };
    
    // If payload has explicit options/validation, merge or prefer them?
    // Request struct has 'options', 'validation' etc? No, request struct missing 'options'.
    // Let's rely on the Config from field_type, OR add options to request struct.
    // The previous request struct didn't have 'options' field! 
    // Ideally we support both formats. Let's assume options come from the field_type.
    
    sqlx::query(
        r#"INSERT INTO field_defs 
           (id, tenant_id, entity_type_id, name, label, field_type, is_required, is_unique, 
            show_in_list, show_in_card, validation, ui_hints, options, sort_order, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)"#
    )
    .bind(id)
    .bind(tenant.id)
    .bind(entity.id)
    .bind(&payload.name)
    .bind(&payload.label)
    .bind(type_str)
    .bind(payload.is_required.unwrap_or(false))
    .bind(payload.is_unique.unwrap_or(false))
    .bind(payload.show_in_list.unwrap_or(false))
    .bind(payload.show_in_card.unwrap_or(false))
    .bind(payload.validation)
    .bind(payload.ui_hints)
    .bind(options_val)
    .bind(payload.sort_order.unwrap_or(0))
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Invalidate cache
    state.metadata.invalidate_fields(entity.id);

    Ok(Json(id))
}

#[derive(Debug, Deserialize)]
pub struct UpdateFieldRequest {
    pub label: Option<String>,
    pub is_required: Option<bool>,
    pub is_unique: Option<bool>,
    pub show_in_list: Option<bool>,
    pub show_in_card: Option<bool>,
    pub validation: Option<serde_json::Value>,
    pub ui_hints: Option<serde_json::Value>,
    pub options: Option<serde_json::Value>,
    pub sort_order: Option<i32>,
}

async fn update_field(
    State(state): State<Arc<AppState>>,
    Path((entity_name, field_id)): Path<(String, Uuid)>,
    Extension(tenant): Extension<ResolvedTenant>,
    Json(payload): Json<UpdateFieldRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let entity = state.metadata.get_entity_type(tenant.id, &entity_name).await?;
    let now = chrono::Utc::now();

    // build dynamic query? Or just update all present fields using COALESCE?
    // Using explicit implementation for clarity.
    
    sqlx::query(
        r#"UPDATE field_defs SET
           label = COALESCE($1, label),
           is_required = COALESCE($2, is_required),
           is_unique = COALESCE($3, is_unique),
           show_in_list = COALESCE($4, show_in_list),
           show_in_card = COALESCE($5, show_in_card),
           validation = COALESCE($6, validation),
           ui_hints = COALESCE($7, ui_hints),
           options = COALESCE($8, options),
           sort_order = COALESCE($9, sort_order),
           updated_at = $10
           WHERE id = $11 AND tenant_id = $12"#
    )
    .bind(payload.label)
    .bind(payload.is_required)
    .bind(payload.is_unique)
    .bind(payload.show_in_list)
    .bind(payload.show_in_card)
    .bind(payload.validation)
    .bind(payload.ui_hints)
    .bind(payload.options) // Explicit options update
    .bind(payload.sort_order)
    .bind(now)
    .bind(field_id)
    .bind(tenant.id)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    state.metadata.invalidate_fields(entity.id);

    Ok(Json(serde_json::json!({"status": "updated"})))
}



async fn get_views(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Extension(tenant): Extension<ResolvedTenant>,
) -> Result<Json<Vec<ViewDef>>, ApiError> {
    let entity = state.metadata
        .get_entity_type(tenant.id, &name)
        .await?;
    let views = state.metadata
        .get_views(tenant.id, entity.id)
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
    Extension(tenant): Extension<ResolvedTenant>,
    Json(payload): Json<CreateViewRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Get entity type
    let entity = state.metadata
        .get_entity_type(tenant.id, &name)
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
    .bind(tenant.id)
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
    Extension(tenant): Extension<ResolvedTenant>,
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
    .bind(tenant.id)
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
    if let Ok(entity) = state.metadata.get_entity_type(tenant.id, &entity_name).await {
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
    Extension(tenant): Extension<ResolvedTenant>,
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
    .bind(tenant.id)
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
    if let Ok(entity) = state.metadata.get_entity_type(tenant.id, &entity_name).await {
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
