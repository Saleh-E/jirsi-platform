//! Metadata routes

use axum::{
    Router,
    routing::{get, post},
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

