//! Metadata routes

use axum::{
    Router,
    routing::get,
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
