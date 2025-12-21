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
#[allow(dead_code)]
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

use crate::middleware::database::RlsConn;
use crate::middleware::tenant::ResolvedTenant;

#[derive(Debug, Serialize)]
pub struct AssociationResponse {
    pub id: Uuid,
    pub association_def_id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub role: Option<String>,
    pub is_primary: bool,
    pub target_label: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct CreateAssociationRequest {
    pub association_def_id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub role: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
}

// ...

async fn list_association_defs(
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    mut conn: RlsConn,
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
    .bind(tenant.id)
    .fetch_all(&mut **conn)
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
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    Query(query): Query<AssociationQuery>,
    mut conn: RlsConn,
) -> Result<Json<Vec<AssociationResponse>>, ApiError> {
    use sqlx::Row;
    
    let mut sql = String::from(
        r#"
        SELECT 
            a.id, a.association_def_id, a.source_id, a.target_id, a.role, a.is_primary,
            r.data->>'name' as target_name,
            r.data->>'title' as target_title,
            r.data->>'first_name' as target_fn,
            r.data->>'last_name' as target_ln
        FROM associations a
        LEFT JOIN entity_records r ON a.target_id = r.id
        WHERE a.tenant_id = $1
        "#
    );
    
    let mut param_idx = 2;
    let mut rows = if let Some(source_id) = query.source_id {
        sql.push_str(&format!(" AND a.source_id = ${}", param_idx));
        sqlx::query(&sql).bind(tenant.id).bind(source_id).fetch_all(&mut **conn).await
    } else {
        sqlx::query(&sql).bind(tenant.id).fetch_all(&mut **conn).await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;

    let associations = rows.iter().map(|row| {
        let fn_str: Option<String> = row.try_get("target_fn").ok();
        let ln_str: Option<String> = row.try_get("target_ln").ok();
        let name_str: Option<String> = row.try_get("target_name").ok();
        let title_str: Option<String> = row.try_get("target_title").ok();
        
        let label = if let (Some(f), Some(l)) = (fn_str, ln_str) {
            Some(format!("{} {}", f, l))
        } else {
            name_str.or(title_str)
        };

        AssociationResponse {
            id: row.try_get("id").unwrap_or_default(),
            association_def_id: row.try_get("association_def_id").unwrap_or_default(),
            source_id: row.try_get("source_id").unwrap_or_default(),
            target_id: row.try_get("target_id").unwrap_or_default(),
            role: row.try_get("role").ok(),
            is_primary: row.try_get("is_primary").unwrap_or(false),
            target_label: label,
        }
    }).collect();

    Ok(Json(associations))
}

async fn create_association(
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    mut conn: RlsConn,
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
    .bind(tenant.id)
    .bind(req.association_def_id)
    .bind(req.source_id)
    .bind(req.target_id)
    .bind(&req.role)
    .bind(req.is_primary)
    .bind(now)
    .bind(now)
    .execute(&mut **conn)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(AssociationResponse {
        id,
        association_def_id: req.association_def_id,
        source_id: req.source_id,
        target_id: req.target_id,
        role: req.role,
        is_primary: req.is_primary,
        target_label: None,
    }))
}

async fn delete_association(
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    mut conn: RlsConn,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    sqlx::query("DELETE FROM associations WHERE id = $1 AND tenant_id = $2")
        .bind(id)
        .bind(tenant.id)
        .execute(&mut **conn)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
