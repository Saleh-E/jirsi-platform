//! Interactions API routes - Activity timeline for entities

use axum::{
    Router,
    routing::{get, post, delete},
    extract::{State, Path, Query},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::state::AppState;
use crate::error::ApiError;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_interactions))
        .route("/", post(create_interaction))
        .route("/summary/:entity_type/:record_id", get(get_interaction_summary))
        .route("/:id", get(get_interaction))
        .route("/:id", delete(delete_interaction))
}

use crate::middleware::database::RlsConn;
use crate::middleware::tenant::ResolvedTenant;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct InteractionQuery {
    pub entity_type: Option<String>,
    pub record_id: Option<Uuid>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct InteractionResponse {
    pub id: Uuid,
    pub entity_type: String,
    pub record_id: Uuid,
    pub interaction_type: String,
    pub title: String,
    pub content: Option<String>,
    pub created_by: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub duration_minutes: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInteractionRequest {
    pub entity_type: String,
    pub record_id: Uuid,
    pub interaction_type: String,
    pub title: String,
    #[serde(default)]
    pub content: Option<String>,
    pub created_by: Uuid,
    #[serde(default)]
    pub duration_minutes: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct InteractionListResponse {
    pub data: Vec<InteractionResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

async fn list_interactions(
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    Query(query): Query<InteractionQuery>,
    mut conn: RlsConn,
) -> Result<Json<InteractionListResponse>, ApiError> {
    use sqlx::Row;
    
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(25).min(100);
    let offset = (page - 1) * per_page;

    // Build query based on filters
    let (count_sql, data_sql, has_record_filter) = if query.record_id.is_some() {
        (
            "SELECT COUNT(*) as count FROM interactions WHERE tenant_id = $1 AND record_id = $2",
            r#"
            SELECT id, entity_type, record_id, interaction_type, title, content, created_by, occurred_at, duration_minutes
            FROM interactions 
            WHERE tenant_id = $1 AND record_id = $2
            ORDER BY occurred_at DESC
            LIMIT $3 OFFSET $4
            "#,
            true,
        )
    } else {
        (
            "SELECT COUNT(*) as count FROM interactions WHERE tenant_id = $1",
            r#"
            SELECT id, entity_type, record_id, interaction_type, title, content, created_by, occurred_at, duration_minutes
            FROM interactions 
            WHERE tenant_id = $1
            ORDER BY occurred_at DESC
            LIMIT $2 OFFSET $3
            "#,
            false,
        )
    };

    let count_row = if has_record_filter {
        sqlx::query(count_sql)
            .bind(tenant.id)
            .bind(query.record_id.unwrap())
            .fetch_one(&mut **conn)
            .await
    } else {
        sqlx::query(count_sql)
            .bind(tenant.id)
            .fetch_one(&mut **conn)
            .await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    let rows = if has_record_filter {
        sqlx::query(data_sql)
            .bind(tenant.id)
            .bind(query.record_id.unwrap())
            .bind(per_page as i64)
            .bind(offset as i64)
            .fetch_all(&mut **conn)
            .await
    } else {
        sqlx::query(data_sql)
            .bind(tenant.id)
            .bind(per_page as i64)
            .bind(offset as i64)
            .fetch_all(&mut **conn)
            .await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;

    let data: Vec<InteractionResponse> = rows.iter().map(|row| {
        InteractionResponse {
            id: row.try_get("id").unwrap_or_default(),
            entity_type: row.try_get("entity_type").unwrap_or_default(),
            record_id: row.try_get("record_id").unwrap_or_default(),
            interaction_type: row.try_get("interaction_type").unwrap_or_default(),
            title: row.try_get("title").unwrap_or_default(),
            content: row.try_get("content").ok(),
            created_by: row.try_get("created_by").unwrap_or_default(),
            occurred_at: row.try_get("occurred_at").unwrap_or_else(|_| Utc::now()),
            duration_minutes: row.try_get("duration_minutes").ok(),
        }
    }).collect();

    Ok(Json(InteractionListResponse {
        data,
        total,
        page,
        per_page,
    }))
}

async fn create_interaction(
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    mut conn: RlsConn,
    Json(req): Json<CreateInteractionRequest>,
) -> Result<Json<InteractionResponse>, ApiError> {
    let now = Utc::now();
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO interactions (id, tenant_id, entity_type, record_id, interaction_type, title, content, created_by, occurred_at, duration_minutes, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
    )
    .bind(id)
    .bind(tenant.id)
    .bind(&req.entity_type)
    .bind(req.record_id)
    .bind(&req.interaction_type)
    .bind(&req.title)
    .bind(&req.content)
    .bind(req.created_by)
    .bind(now)
    .bind(req.duration_minutes)
    .bind(now)
    .bind(now)
    .execute(&mut **conn)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(InteractionResponse {
        id,
        entity_type: req.entity_type,
        record_id: req.record_id,
        interaction_type: req.interaction_type,
        title: req.title,
        content: req.content,
        created_by: req.created_by,
        occurred_at: now,
        duration_minutes: req.duration_minutes,
    }))
}

async fn get_interaction(
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    mut conn: RlsConn,
    Path(id): Path<Uuid>,
) -> Result<Json<InteractionResponse>, ApiError> {
    use sqlx::Row;

    let row = sqlx::query(
        "SELECT id, entity_type, record_id, interaction_type, title, content, created_by, occurred_at, duration_minutes FROM interactions WHERE id = $1 AND tenant_id = $2"
    )
    .bind(id)
    .bind(tenant.id)
    .fetch_optional(&mut **conn)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    match row {
        Some(r) => Ok(Json(InteractionResponse {
            id: r.try_get("id").unwrap_or_default(),
            entity_type: r.try_get("entity_type").unwrap_or_default(),
            record_id: r.try_get("record_id").unwrap_or_default(),
            interaction_type: r.try_get("interaction_type").unwrap_or_default(),
            title: r.try_get("title").unwrap_or_default(),
            content: r.try_get("content").ok(),
            created_by: r.try_get("created_by").unwrap_or_default(),
            occurred_at: r.try_get("occurred_at").unwrap_or_else(|_| Utc::now()),
            duration_minutes: r.try_get("duration_minutes").ok(),
        })),
        None => Err(ApiError::NotFound(format!("Interaction {} not found", id))),
    }
}

async fn delete_interaction(
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    mut conn: RlsConn,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    sqlx::query("DELETE FROM interactions WHERE id = $1 AND tenant_id = $2")
        .bind(id)
        .bind(tenant.id)
        .execute(&mut **conn)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

#[derive(Debug, Serialize)]
pub struct InteractionSummary {
    pub total_count: i64,
    pub last_interaction: Option<DateTime<Utc>>,
    pub counts_by_type: std::collections::HashMap<String, i64>,
}

async fn get_interaction_summary(
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    Path((entity_type, record_id)): Path<(String, Uuid)>,
    mut conn: RlsConn,
) -> Result<Json<InteractionSummary>, ApiError> {
    use sqlx::Row;

    // Total count and last interaction
    let row = sqlx::query(
        "SELECT COUNT(*) as total, MAX(occurred_at) as last FROM interactions WHERE tenant_id = $1 AND entity_type = $2 AND record_id = $3"
    )
    .bind(tenant.id)
    .bind(&entity_type)
    .bind(record_id)
    .fetch_one(&mut **conn)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let total_count: i64 = row.try_get("total").unwrap_or(0);
    let last_interaction: Option<DateTime<Utc>> = row.try_get("last").ok();

    // Counts by type
    let type_rows = sqlx::query(
        "SELECT interaction_type, COUNT(*) as count FROM interactions WHERE tenant_id = $1 AND entity_type = $2 AND record_id = $3 GROUP BY interaction_type"
    )
    .bind(tenant.id)
    .bind(&entity_type)
    .bind(record_id)
    .fetch_all(&mut **conn)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut counts_by_type = std::collections::HashMap::new();
    for row in type_rows {
        let itype: String = row.try_get("interaction_type").unwrap_or_default();
        let count: i64 = row.try_get("count").unwrap_or(0);
        counts_by_type.insert(itype, count);
    }

    Ok(Json(InteractionSummary {
        total_count,
        last_interaction,
        counts_by_type,
    }))
}
