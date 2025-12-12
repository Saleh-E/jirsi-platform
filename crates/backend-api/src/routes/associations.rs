//! Association API routes - Runtime linking between records

use axum::{
    Router,
    routing::{get, post, delete},
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
        .route("/", get(list_associations))
        .route("/", post(create_association))
        .route("/:id", delete(delete_association))
        .route("/defs", get(list_association_defs))
}

#[derive(Debug, Deserialize)]
pub struct AssociationQuery {
    pub tenant_id: Uuid,
    /// Filter by source entity type
    pub source_entity: Option<String>,
    /// Filter by source record ID
    pub source_id: Option<Uuid>,
    /// Filter by target entity type
    pub target_entity: Option<String>,
    /// Filter by target record ID
    pub target_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct AssociationResponse {
    pub id: Uuid,
    pub association_def_id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub role: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssociationRequest {
    pub tenant_id: Uuid,
    pub association_def_id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
}

#[derive(Debug, Serialize)]
pub struct AssociationDefResponse {
    pub id: Uuid,
    pub name: String,
    pub source_entity: String,
    pub target_entity: String,
    pub label_source: String,
    pub label_target: String,
    pub cardinality: String,
}

async fn list_association_defs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AssociationQuery>,
) -> Result<Json<Vec<AssociationDefResponse>>, ApiError> {
    use sqlx::Row;
    
    let rows = sqlx::query(
        r#"
        SELECT id, name, source_entity, target_entity, label_source, label_target, cardinality
        FROM association_defs
        WHERE tenant_id = $1
        ORDER BY name
        "#,
    )
    .bind(query.tenant_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let defs = rows.iter().map(|row| {
        AssociationDefResponse {
            id: row.try_get("id").unwrap_or_default(),
            name: row.try_get("name").unwrap_or_default(),
            source_entity: row.try_get("source_entity").unwrap_or_default(),
            target_entity: row.try_get("target_entity").unwrap_or_default(),
            label_source: row.try_get("label_source").unwrap_or_default(),
            label_target: row.try_get("label_target").unwrap_or_default(),
            cardinality: row.try_get("cardinality").unwrap_or_default(),
        }
    }).collect();

    Ok(Json(defs))
}

async fn list_associations(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AssociationQuery>,
) -> Result<Json<Vec<AssociationResponse>>, ApiError> {
    use sqlx::Row;
    
    let mut sql = String::from(
        "SELECT id, association_def_id, source_id, target_id, role, is_primary FROM associations WHERE tenant_id = $1"
    );
    
    if query.source_id.is_some() {
        sql.push_str(" AND source_id = $2");
    }
    if query.target_id.is_some() {
        sql.push_str(" AND target_id = $3");
    }

    let mut q = sqlx::query(&sql).bind(query.tenant_id);
    
    if let Some(source_id) = query.source_id {
        q = q.bind(source_id);
    }
    if let Some(target_id) = query.target_id {
        q = q.bind(target_id);
    }

    let rows = q.fetch_all(&state.pool).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let associations = rows.iter().map(|row| {
        AssociationResponse {
            id: row.try_get("id").unwrap_or_default(),
            association_def_id: row.try_get("association_def_id").unwrap_or_default(),
            source_id: row.try_get("source_id").unwrap_or_default(),
            target_id: row.try_get("target_id").unwrap_or_default(),
            role: row.try_get("role").ok(),
            is_primary: row.try_get("is_primary").unwrap_or(false),
        }
    }).collect();

    Ok(Json(associations))
}

async fn create_association(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAssociationRequest>,
) -> Result<Json<AssociationResponse>, ApiError> {
    let now = Utc::now();
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO associations (id, tenant_id, association_def_id, source_id, target_id, role, is_primary, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(id)
    .bind(req.tenant_id)
    .bind(req.association_def_id)
    .bind(req.source_id)
    .bind(req.target_id)
    .bind(&req.role)
    .bind(req.is_primary)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(AssociationResponse {
        id,
        association_def_id: req.association_def_id,
        source_id: req.source_id,
        target_id: req.target_id,
        role: req.role,
        is_primary: req.is_primary,
    }))
}

async fn delete_association(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    sqlx::query("DELETE FROM associations WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
