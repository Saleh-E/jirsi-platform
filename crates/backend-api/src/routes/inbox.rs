//! Inbox API routes - Unified messaging inbox

use axum::{
    Router,
    routing::{get, post},
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
        .route("/threads", get(list_threads))
        .route("/threads/:entity_id/messages", get(get_thread_messages))
        .route("/threads/:entity_id/reply", post(send_reply))
}

#[derive(Debug, Deserialize)]
pub struct ThreadsQuery {
    pub tenant_id: Uuid,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub entity_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InboxThreadResponse {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub entity_name: String,
    pub last_message_preview: String,
    pub last_message_at: DateTime<Utc>,
    pub unread_count: i64,
    pub last_interaction_type: String,
}

#[derive(Debug, Serialize)]
pub struct ThreadListResponse {
    pub data: Vec<InboxThreadResponse>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct ThreadMessageResponse {
    pub id: Uuid,
    pub interaction_type: String,
    pub title: String,
    pub content: Option<String>,
    pub created_by: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub direction: String,
    pub duration_minutes: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct MessagesListResponse {
    pub data: Vec<ThreadMessageResponse>,
    pub entity_name: String,
    pub entity_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ReplyRequest {
    pub tenant_id: Uuid,
    pub interaction_type: String,
    pub title: String,
    #[serde(default)]
    pub content: Option<String>,
    pub created_by: Uuid,
}

/// GET /inbox/threads - List all conversation threads
async fn list_threads(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ThreadsQuery>,
) -> Result<Json<ThreadListResponse>, ApiError> {
    use sqlx::Row;

    // Query to aggregate interactions into threads
    // Groups by record_id (entity), gets latest message info
    let sql = r#"
        WITH thread_summary AS (
            SELECT 
                i.record_id as entity_id,
                i.entity_type,
                MAX(i.occurred_at) as last_message_at,
                COUNT(*) as message_count
            FROM interactions i
            WHERE i.tenant_id = $1
            GROUP BY i.record_id, i.entity_type
            ORDER BY MAX(i.occurred_at) DESC
            LIMIT 50
        ),
        latest_messages AS (
            SELECT DISTINCT ON (i.record_id)
                i.record_id as entity_id,
                i.title,
                i.content,
                i.interaction_type
            FROM interactions i
            WHERE i.tenant_id = $1
            ORDER BY i.record_id, i.occurred_at DESC
        )
        SELECT 
            ts.entity_id,
            ts.entity_type,
            ts.last_message_at,
            ts.message_count,
            lm.title as last_title,
            lm.content as last_content,
            lm.interaction_type as last_interaction_type
        FROM thread_summary ts
        JOIN latest_messages lm ON ts.entity_id = lm.entity_id
        ORDER BY ts.last_message_at DESC
    "#;

    let rows = sqlx::query(sql)
        .bind(query.tenant_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut threads: Vec<InboxThreadResponse> = Vec::new();

    for row in rows {
        let entity_id: Uuid = row.try_get("entity_id").unwrap_or_default();
        let entity_type: String = row.try_get("entity_type").unwrap_or_default();
        let last_message_at: DateTime<Utc> = row.try_get("last_message_at").unwrap_or_else(|_| Utc::now());
        let last_title: String = row.try_get("last_title").unwrap_or_default();
        let last_content: Option<String> = row.try_get("last_content").ok();
        let last_interaction_type: String = row.try_get("last_interaction_type").unwrap_or_default();

        // Fetch entity name based on type
        let entity_name = get_entity_name(&state.pool, &entity_type, entity_id).await;

        // Create preview from content or title
        let preview = last_content
            .as_deref()
            .unwrap_or(&last_title);
        let last_message_preview = truncate_preview(preview, 80);

        threads.push(InboxThreadResponse {
            entity_id,
            entity_type,
            entity_name,
            last_message_preview,
            last_message_at,
            unread_count: 0, // TODO: Implement read tracking
            last_interaction_type,
        });
    }

    let total = threads.len() as i64;

    Ok(Json(ThreadListResponse { data: threads, total }))
}

/// GET /inbox/threads/:entity_id/messages - Get full message history for a thread
async fn get_thread_messages(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
    Query(query): Query<ThreadsQuery>,
) -> Result<Json<MessagesListResponse>, ApiError> {
    use sqlx::Row;

    // First get entity info
    let entity_type_row = sqlx::query(
        "SELECT DISTINCT entity_type FROM interactions WHERE record_id = $1 AND tenant_id = $2 LIMIT 1"
    )
    .bind(entity_id)
    .bind(query.tenant_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let entity_type: String = entity_type_row
        .map(|r| r.try_get("entity_type").unwrap_or_default())
        .unwrap_or_else(|| "contact".to_string());

    let entity_name = get_entity_name(&state.pool, &entity_type, entity_id).await;

    // Fetch all messages for this thread
    let sql = r#"
        SELECT 
            id,
            interaction_type,
            title,
            content,
            created_by,
            occurred_at,
            duration_minutes
        FROM interactions
        WHERE record_id = $1 AND tenant_id = $2
        ORDER BY occurred_at ASC
    "#;

    let rows = sqlx::query(sql)
        .bind(entity_id)
        .bind(query.tenant_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let messages: Vec<ThreadMessageResponse> = rows.iter().map(|row| {
        let interaction_type: String = row.try_get("interaction_type").unwrap_or_default();
        
        // Determine direction based on interaction type
        // Inbound: email, message (from customer)
        // Outbound: note (internal), call (agent initiated)
        let direction = match interaction_type.to_lowercase().as_str() {
            "email" | "message" => "inbound",
            _ => "outbound",
        };

        ThreadMessageResponse {
            id: row.try_get("id").unwrap_or_default(),
            interaction_type,
            title: row.try_get("title").unwrap_or_default(),
            content: row.try_get("content").ok(),
            created_by: row.try_get("created_by").unwrap_or_default(),
            occurred_at: row.try_get("occurred_at").unwrap_or_else(|_| Utc::now()),
            direction: direction.to_string(),
            duration_minutes: row.try_get("duration_minutes").ok(),
        }
    }).collect();

    Ok(Json(MessagesListResponse {
        data: messages,
        entity_name,
        entity_type,
    }))
}

/// POST /inbox/threads/:entity_id/reply - Send a reply in a thread
async fn send_reply(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
    Json(req): Json<ReplyRequest>,
) -> Result<Json<ThreadMessageResponse>, ApiError> {
    let now = Utc::now();
    let id = Uuid::new_v4();

    // Determine entity type from existing interactions
    let entity_type_row = sqlx::query(
        "SELECT DISTINCT entity_type FROM interactions WHERE record_id = $1 AND tenant_id = $2 LIMIT 1"
    )
    .bind(entity_id)
    .bind(req.tenant_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let entity_type: String = entity_type_row
        .map(|r| {
            use sqlx::Row;
            r.try_get("entity_type").unwrap_or_default()
        })
        .unwrap_or_else(|| "contact".to_string());

    // Create the interaction
    sqlx::query(
        r#"
        INSERT INTO interactions (id, tenant_id, entity_type, record_id, interaction_type, title, content, created_by, occurred_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(id)
    .bind(req.tenant_id)
    .bind(&entity_type)
    .bind(entity_id)
    .bind(&req.interaction_type)
    .bind(&req.title)
    .bind(&req.content)
    .bind(req.created_by)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    // TODO: Trigger "On Reply" workflows here

    Ok(Json(ThreadMessageResponse {
        id,
        interaction_type: req.interaction_type,
        title: req.title,
        content: req.content,
        created_by: req.created_by,
        occurred_at: now,
        direction: "outbound".to_string(),
        duration_minutes: None,
    }))
}

/// Helper to get entity name by type and ID
async fn get_entity_name(pool: &sqlx::PgPool, entity_type: &str, entity_id: Uuid) -> String {
    use sqlx::Row;

    match entity_type {
        "contact" => {
            let row = sqlx::query(
                "SELECT first_name, last_name FROM contacts WHERE id = $1"
            )
            .bind(entity_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

            row.map(|r| {
                let first: String = r.try_get("first_name").unwrap_or_default();
                let last: String = r.try_get("last_name").unwrap_or_default();
                format!("{} {}", first, last).trim().to_string()
            })
            .unwrap_or_else(|| "Unknown Contact".to_string())
        }
        "company" => {
            let row = sqlx::query("SELECT name FROM companies WHERE id = $1")
                .bind(entity_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();

            row.map(|r| r.try_get("name").unwrap_or_default())
                .unwrap_or_else(|| "Unknown Company".to_string())
        }
        "deal" => {
            let row = sqlx::query("SELECT name FROM deals WHERE id = $1")
                .bind(entity_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();

            row.map(|r| r.try_get("name").unwrap_or_default())
                .unwrap_or_else(|| "Unknown Deal".to_string())
        }
        _ => format!("Entity {}", entity_id),
    }
}

/// Truncate text to max length with ellipsis
fn truncate_preview(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}
