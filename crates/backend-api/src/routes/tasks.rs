//! Tasks API routes - Entity-linked task management

use axum::{
    Router,
    routing::{get, post, put, delete},
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
        .route("/", get(list_tasks))
        .route("/", post(create_task))
        .route("/:id", get(get_task))
        .route("/:id", put(update_task))
        .route("/:id", delete(delete_task))
}

use crate::middleware::database::RlsConn;
use crate::middleware::tenant::ResolvedTenant;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TaskQuery {
    pub linked_entity_type: Option<String>,
    pub linked_entity_id: Option<Uuid>,
    pub status: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: String,
    pub status: String,
    pub task_type: String,
    pub linked_entity_type: Option<String>,
    pub linked_entity_id: Option<Uuid>,
    pub assignee_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default)]
    pub task_type: Option<String>,
    #[serde(default)]
    pub linked_entity_type: Option<String>,
    #[serde(default)]
    pub linked_entity_id: Option<Uuid>,
    #[serde(default)]
    pub assignee_id: Option<Uuid>,
    pub created_by: Uuid,
}

fn default_priority() -> String { "normal".to_string() }

#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub data: Vec<TaskResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

async fn list_tasks(
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    Query(query): Query<TaskQuery>,
    mut conn: RlsConn,
) -> Result<Json<TaskListResponse>, ApiError> {
    use sqlx::Row;
    
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(25).min(100);
    let offset = (page - 1) * per_page;

    // Build query with optional filters
    let (count_sql, data_sql, has_entity_filter) = if let Some(entity_id) = query.linked_entity_id {
        (
            "SELECT COUNT(*) as count FROM tasks WHERE tenant_id = $1 AND linked_entity_id = $2",
            "SELECT id, title, description, due_date, priority, status, task_type, linked_entity_type, linked_entity_id, assignee_id, created_by, created_at FROM tasks WHERE tenant_id = $1 AND linked_entity_id = $4 ORDER BY due_date ASC NULLS LAST LIMIT $2 OFFSET $3",
            true
        )
    } else {
        (
            "SELECT COUNT(*) as count FROM tasks WHERE tenant_id = $1",
            "SELECT id, title, description, due_date, priority, status, task_type, linked_entity_type, linked_entity_id, assignee_id, created_by, created_at FROM tasks WHERE tenant_id = $1 ORDER BY due_date ASC NULLS LAST LIMIT $2 OFFSET $3",
            false
        )
    };

    // Get count
    let count_row = if has_entity_filter {
        sqlx::query(count_sql)
            .bind(tenant.id)
            .bind(query.linked_entity_id.unwrap())
            .fetch_one(&mut **conn)
            .await
    } else {
        sqlx::query(count_sql)
            .bind(tenant.id)
            .fetch_one(&mut **conn)
            .await
    }.map_err(|e| ApiError::Internal(e.to_string()))?;
    
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    // Get data
    let rows = if has_entity_filter {
        sqlx::query(data_sql)
            .bind(tenant.id)
            .bind(per_page as i64)
            .bind(offset as i64)
            .bind(query.linked_entity_id.unwrap())
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

    let data: Vec<TaskResponse> = rows.iter().map(|row| {
        TaskResponse {
            id: row.try_get("id").unwrap_or_default(),
            title: row.try_get("title").unwrap_or_default(),
            description: row.try_get("description").ok(),
            due_date: row.try_get("due_date").ok(),
            priority: row.try_get("priority").unwrap_or_default(),
            status: row.try_get("status").unwrap_or_default(),
            task_type: row.try_get("task_type").unwrap_or_default(),
            linked_entity_type: row.try_get("linked_entity_type").ok(),
            linked_entity_id: row.try_get("linked_entity_id").ok(),
            assignee_id: row.try_get("assignee_id").ok(),
            created_by: row.try_get("created_by").unwrap_or_default(),
            created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
        }
    }).collect();

    Ok(Json(TaskListResponse { data, total, page, per_page }))
}

async fn create_task(
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    mut conn: RlsConn,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, ApiError> {
    let now = Utc::now();
    let id = Uuid::new_v4();
    let task_type = req.task_type.unwrap_or_else(|| "todo".to_string());

    sqlx::query(
        r#"
        INSERT INTO tasks (id, tenant_id, title, description, due_date, priority, status, task_type, linked_entity_type, linked_entity_id, assignee_id, created_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, 'open', $7, $8, $9, $10, $11, $12, $13)
        "#,
    )
    .bind(id)
    .bind(tenant.id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(req.due_date)
    .bind(&req.priority)
    .bind(&task_type)
    .bind(&req.linked_entity_type)
    .bind(req.linked_entity_id)
    .bind(req.assignee_id)
    .bind(req.created_by)
    .bind(now)
    .bind(now)
    .execute(&mut **conn)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(TaskResponse {
        id,
        title: req.title,
        description: req.description,
        due_date: req.due_date,
        priority: req.priority,
        status: "open".to_string(),
        task_type,
        linked_entity_type: req.linked_entity_type,
        linked_entity_id: req.linked_entity_id,
        assignee_id: req.assignee_id,
        created_by: req.created_by,
        created_at: now,
    }))
}

async fn get_task(
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    mut conn: RlsConn,
    Path(id): Path<Uuid>,
) -> Result<Json<TaskResponse>, ApiError> {
    use sqlx::Row;

    let row = sqlx::query(
        "SELECT id, title, description, due_date, priority, status, task_type, linked_entity_type, linked_entity_id, assignee_id, created_by, created_at FROM tasks WHERE id = $1 AND tenant_id = $2"
    )
    .bind(id)
    .bind(tenant.id)
    .fetch_optional(&mut **conn)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    match row {
        Some(r) => Ok(Json(TaskResponse {
            id: r.try_get("id").unwrap_or_default(),
            title: r.try_get("title").unwrap_or_default(),
            description: r.try_get("description").ok(),
            due_date: r.try_get("due_date").ok(),
            priority: r.try_get("priority").unwrap_or_default(),
            status: r.try_get("status").unwrap_or_default(),
            task_type: r.try_get("task_type").unwrap_or_default(),
            linked_entity_type: r.try_get("linked_entity_type").ok(),
            linked_entity_id: r.try_get("linked_entity_id").ok(),
            assignee_id: r.try_get("assignee_id").ok(),
            created_by: r.try_get("created_by").unwrap_or_default(),
            created_at: r.try_get("created_at").unwrap_or_else(|_| Utc::now()),
        })),
        None => Err(ApiError::NotFound(format!("Task {} not found", id))),
    }
}

async fn update_task(
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    mut conn: RlsConn,
    Path(id): Path<Uuid>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let now = Utc::now();

    // Update status if provided
    if let Some(status) = data.get("status").and_then(|v| v.as_str()) {
        sqlx::query("UPDATE tasks SET status = $3, updated_at = $4 WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant.id)
            .bind(status)
            .bind(now)
            .execute(&mut **conn)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
    }

    Ok(Json(serde_json::json!({ "id": id, "updated": true })))
}

async fn delete_task(
    axum::Extension(tenant): axum::Extension<ResolvedTenant>,
    mut conn: RlsConn,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    sqlx::query("DELETE FROM tasks WHERE id = $1 AND tenant_id = $2")
        .bind(id)
        .bind(tenant.id)
        .execute(&mut **conn)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
