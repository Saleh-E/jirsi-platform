//! Views API routes - User-specific saved views

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
        .route("/", get(list_views))
        .route("/", post(create_view))
        .route("/:id", get(get_view))
        .route("/:id", put(update_view))
        .route("/:id", delete(delete_view))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ViewQuery {
    pub tenant_id: Uuid,
    pub entity_type_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ViewResponse {
    pub id: Uuid,
    pub entity_type_id: Uuid,
    pub name: String,
    pub label: String,
    pub view_type: String,
    pub is_default: bool,
    pub is_system: bool,
    pub created_by: Option<Uuid>,
    pub columns: serde_json::Value,
    pub filters: serde_json::Value,
    pub sort: serde_json::Value,
    pub settings: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CreateViewRequest {
    pub tenant_id: Uuid,
    pub entity_type_id: Uuid,
    pub name: String,
    pub label: String,
    #[serde(default = "default_view_type")]
    pub view_type: String,
    pub created_by: Uuid,
    #[serde(default)]
    pub columns: serde_json::Value,
    #[serde(default)]
    pub filters: serde_json::Value,
    #[serde(default)]
    pub sort: serde_json::Value,
    #[serde(default)]
    pub settings: serde_json::Value,
}

fn default_view_type() -> String { "table".to_string() }

#[derive(Debug, Serialize)]
pub struct ViewListResponse {
    pub data: Vec<ViewResponse>,
    pub total: i64,
}

async fn list_views(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ViewQuery>,
) -> Result<Json<ViewListResponse>, ApiError> {
    use sqlx::Row;

    let mut sql = String::from(
        "SELECT id, entity_type_id, name, label, view_type, is_default, is_system, created_by, columns, filters, sort, settings FROM view_defs WHERE tenant_id = $1"
    );
    
    if query.entity_type_id.is_some() {
        sql.push_str(" AND entity_type_id = $2");
    }
    
    sql.push_str(" ORDER BY is_default DESC, is_system DESC, name ASC");

    let rows = if let Some(entity_id) = query.entity_type_id {
        sqlx::query(&sql)
            .bind(query.tenant_id)
            .bind(entity_id)
            .fetch_all(&state.pool)
            .await
    } else {
        sqlx::query(&sql)
            .bind(query.tenant_id)
            .fetch_all(&state.pool)
            .await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<ViewResponse> = rows.iter().map(|row| {
        ViewResponse {
            id: row.try_get("id").unwrap_or_default(),
            entity_type_id: row.try_get("entity_type_id").unwrap_or_default(),
            name: row.try_get("name").unwrap_or_default(),
            label: row.try_get("label").unwrap_or_default(),
            view_type: row.try_get("view_type").unwrap_or_default(),
            is_default: row.try_get("is_default").unwrap_or(false),
            is_system: row.try_get("is_system").unwrap_or(false),
            created_by: row.try_get("created_by").ok(),
            columns: row.try_get("columns").unwrap_or(serde_json::json!([])),
            filters: row.try_get("filters").unwrap_or(serde_json::json!([])),
            sort: row.try_get("sort").unwrap_or(serde_json::json!([])),
            settings: row.try_get("settings").unwrap_or(serde_json::json!({})),
        }
    }).collect();

    let total = data.len() as i64;
    Ok(Json(ViewListResponse { data, total }))
}

async fn create_view(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateViewRequest>,
) -> Result<Json<ViewResponse>, ApiError> {
    let now = Utc::now();
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, created_by, columns, filters, sort, settings, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, false, false, $7, $8, $9, $10, $11, $12, $13)
        "#,
    )
    .bind(id)
    .bind(req.tenant_id)
    .bind(req.entity_type_id)
    .bind(&req.name)
    .bind(&req.label)
    .bind(&req.view_type)
    .bind(req.created_by)
    .bind(&req.columns)
    .bind(&req.filters)
    .bind(&req.sort)
    .bind(&req.settings)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ViewResponse {
        id,
        entity_type_id: req.entity_type_id,
        name: req.name,
        label: req.label,
        view_type: req.view_type,
        is_default: false,
        is_system: false,
        created_by: Some(req.created_by),
        columns: req.columns,
        filters: req.filters,
        sort: req.sort,
        settings: req.settings,
    }))
}

async fn get_view(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<ViewQuery>,
) -> Result<Json<ViewResponse>, ApiError> {
    use sqlx::Row;

    let row = sqlx::query(
        "SELECT id, entity_type_id, name, label, view_type, is_default, is_system, created_by, columns, filters, sort, settings FROM view_defs WHERE id = $1 AND tenant_id = $2"
    )
    .bind(id)
    .bind(query.tenant_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    match row {
        Some(r) => Ok(Json(ViewResponse {
            id: r.try_get("id").unwrap_or_default(),
            entity_type_id: r.try_get("entity_type_id").unwrap_or_default(),
            name: r.try_get("name").unwrap_or_default(),
            label: r.try_get("label").unwrap_or_default(),
            view_type: r.try_get("view_type").unwrap_or_default(),
            is_default: r.try_get("is_default").unwrap_or(false),
            is_system: r.try_get("is_system").unwrap_or(false),
            created_by: r.try_get("created_by").ok(),
            columns: r.try_get("columns").unwrap_or(serde_json::json!([])),
            filters: r.try_get("filters").unwrap_or(serde_json::json!([])),
            sort: r.try_get("sort").unwrap_or(serde_json::json!([])),
            settings: r.try_get("settings").unwrap_or(serde_json::json!({})),
        })),
        None => Err(ApiError::NotFound(format!("View {} not found", id))),
    }
}

async fn update_view(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<ViewQuery>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let now = Utc::now();

    // Update columns if provided
    if let Some(columns) = data.get("columns") {
        sqlx::query("UPDATE view_defs SET columns = $3, updated_at = $4 WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(query.tenant_id)
            .bind(columns)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
    }

    // Update filters if provided
    if let Some(filters) = data.get("filters") {
        sqlx::query("UPDATE view_defs SET filters = $3, updated_at = $4 WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(query.tenant_id)
            .bind(filters)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
    }

    Ok(Json(serde_json::json!({ "id": id, "updated": true })))
}

async fn delete_view(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<ViewQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Only allow deleting non-system views
    let result = sqlx::query("DELETE FROM view_defs WHERE id = $1 AND tenant_id = $2 AND is_system = false")
        .bind(id)
        .bind(query.tenant_id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::BadRequest("Cannot delete system views".to_string()));
    }

    Ok(Json(serde_json::json!({ "deleted": true })))
}
