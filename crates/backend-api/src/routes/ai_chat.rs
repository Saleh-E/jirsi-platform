//! AI Chat API - Context-Aware AI Endpoint with RAG
//!
//! Provides:
//! - `/api/ai/chat` - Chat with AI using entity context
//! - RAG (Retrieval-Augmented Generation) integration
//! - Conversation history management

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::ai::embeddings::{
    EmbeddingService, OpenAIEmbeddingProvider, 
    find_similar_entities, entity_to_embedding_content
};
use crate::state::AppState;

/// Chat request
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    /// The user's message
    pub message: String,
    /// Optional conversation ID (for multi-turn conversations)
    pub conversation_id: Option<Uuid>,
    /// Optional entity context (e.g., viewing a contact)
    pub context_entity_id: Option<Uuid>,
    pub context_entity_type: Option<String>,
    /// Whether to include RAG context
    #[serde(default = "default_true")]
    pub use_rag: bool,
}

fn default_true() -> bool { true }

/// Chat response
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub message: String,
    pub conversation_id: Uuid,
    pub rag_context: Option<Vec<RagContextItem>>,
    pub tokens_used: Option<TokenUsage>,
}

/// RAG context item
#[derive(Debug, Serialize)]
pub struct RagContextItem {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub similarity: f32,
    pub preview: String,
}

/// Token usage
#[derive(Debug, Serialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Conversation summary
#[derive(Debug, Serialize)]
pub struct ConversationSummary {
    pub id: Uuid,
    pub title: Option<String>,
    pub created_at: String,
    pub message_count: i64,
}

/// Build AI chat routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/chat", post(chat_handler))
        .route("/conversations", get(list_conversations))
        .route("/conversations/:id", get(get_conversation))
        .route("/conversations/:id/messages", get(get_conversation_messages))
}

/// Main chat handler with RAG integration
async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (axum::http::StatusCode, String)> {
    let tenant_id = get_tenant_id_from_context()?;
    let user_id = get_user_id_from_context()?;
    
    // Get or create conversation
    let conversation_id = match request.conversation_id {
        Some(id) => id,
        None => create_conversation(&state.pool, tenant_id, user_id, request.context_entity_id).await?,
    };
    
    // Build RAG context if enabled
    let rag_context = if request.use_rag {
        build_rag_context(&state, tenant_id, &request.message, request.context_entity_id).await?
    } else {
        vec![]
    };
    
    // Build the prompt with context
    let system_prompt = build_system_prompt(&rag_context, request.context_entity_type.as_deref());
    
    // Get conversation history
    let history = get_recent_messages(&state.pool, conversation_id, 10).await?;
    
    // Call OpenAI
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "mock".to_string());
    let ai_response = call_openai_chat(&api_key, &system_prompt, &history, &request.message).await?;
    
    // Save messages to database
    save_message(&state.pool, conversation_id, "user", &request.message, None).await?;
    save_message(&state.pool, conversation_id, "assistant", &ai_response.content, Some(&rag_context)).await?;
    
    Ok(Json(ChatResponse {
        message: ai_response.content,
        conversation_id,
        rag_context: if rag_context.is_empty() { None } else { Some(rag_context) },
        tokens_used: Some(TokenUsage {
            prompt_tokens: ai_response.prompt_tokens,
            completion_tokens: ai_response.completion_tokens,
            total_tokens: ai_response.prompt_tokens + ai_response.completion_tokens,
        }),
    }))
}

/// Build RAG context by finding similar entities
async fn build_rag_context(
    state: &AppState,
    tenant_id: Uuid,
    query: &str,
    context_entity_id: Option<Uuid>,
) -> Result<Vec<RagContextItem>, (axum::http::StatusCode, String)> {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "mock".to_string());
    
    if api_key == "mock" {
        return Ok(vec![]); // Skip RAG if no API key
    }
    
    let embedding_service = OpenAIEmbeddingProvider::new(api_key);
    
    // Generate embedding for the query
    let query_embedding = embedding_service.embed(query).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Find similar entities
    let similar = find_similar_entities(
        &state.pool,
        tenant_id,
        &query_embedding.embedding,
        None,  // Search all entity types
        5,     // Top 5 results
        0.7,   // Similarity threshold
    ).await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Convert to RAG context items
    let mut context: Vec<RagContextItem> = similar.into_iter()
        .map(|e| RagContextItem {
            entity_id: e.entity_id,
            entity_type: e.entity_type,
            similarity: e.similarity,
            preview: e.content_preview.unwrap_or_default(),
        })
        .collect();
    
    // If there's a specific context entity, ensure it's included
    if let Some(entity_id) = context_entity_id {
        if !context.iter().any(|c| c.entity_id == entity_id) {
            // Fetch the context entity and add it
            if let Ok(Some(entity_data)) = get_entity_data(&state.pool, tenant_id, entity_id).await {
                context.insert(0, RagContextItem {
                    entity_id,
                    entity_type: entity_data.entity_type,
                    similarity: 1.0, // Explicit context
                    preview: entity_data.preview,
                });
            }
        }
    }
    
    Ok(context)
}

/// Build system prompt with RAG context
fn build_system_prompt(rag_context: &[RagContextItem], entity_type: Option<&str>) -> String {
    let mut prompt = String::from(
        "You are a helpful AI assistant for a CRM and real estate management platform called Jirsi. \
        You help users manage contacts, properties, deals, and workflows. \
        Be concise, professional, and helpful.\n\n"
    );
    
    if let Some(et) = entity_type {
        prompt.push_str(&format!("The user is currently viewing a {}.\n\n", et));
    }
    
    if !rag_context.is_empty() {
        prompt.push_str("Relevant context from the database:\n");
        for (i, ctx) in rag_context.iter().enumerate() {
            prompt.push_str(&format!(
                "{}. [{}] (relevance: {:.0}%): {}\n",
                i + 1,
                ctx.entity_type,
                ctx.similarity * 100.0,
                ctx.preview
            ));
        }
        prompt.push_str("\nUse this context to provide accurate, personalized responses.\n");
    }
    
    prompt
}

/// OpenAI chat response
struct OpenAiChatResponse {
    content: String,
    prompt_tokens: u32,
    completion_tokens: u32,
}

/// Call OpenAI Chat API
async fn call_openai_chat(
    api_key: &str,
    system_prompt: &str,
    history: &[(String, String)], // (role, content)
    user_message: &str,
) -> Result<OpenAiChatResponse, (axum::http::StatusCode, String)> {
    if api_key == "mock" {
        return Ok(OpenAiChatResponse {
            content: format!("Mock response to: {}", user_message),
            prompt_tokens: 0,
            completion_tokens: 0,
        });
    }
    
    let client = reqwest::Client::new();
    
    let mut messages = vec![
        serde_json::json!({ "role": "system", "content": system_prompt })
    ];
    
    for (role, content) in history {
        messages.push(serde_json::json!({ "role": role, "content": content }));
    }
    
    messages.push(serde_json::json!({ "role": "user", "content": user_message }));
    
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "gpt-4o-mini",
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 1000
        }))
        .send()
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_default();
        return Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, error));
    }
    
    let json: serde_json::Value = response.json().await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("I couldn't generate a response.")
        .to_string();
    
    let prompt_tokens = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
    let completion_tokens = json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;
    
    Ok(OpenAiChatResponse {
        content,
        prompt_tokens,
        completion_tokens,
    })
}

// ============================================================================
// Database helpers
// ============================================================================

async fn create_conversation(
    pool: &PgPool,
    tenant_id: Uuid,
    user_id: Uuid,
    context_entity_id: Option<Uuid>,
) -> Result<Uuid, (axum::http::StatusCode, String)> {
    let id = Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO ai_conversations (id, tenant_id, user_id, context_entity_id)
        VALUES ($1, $2, $3, $4)
        "#
    )
    .bind(id)
    .bind(tenant_id)
    .bind(user_id)
    .bind(context_entity_id)
    .execute(pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(id)
}

async fn save_message(
    pool: &PgPool,
    conversation_id: Uuid,
    role: &str,
    content: &str,
    rag_context: Option<&Vec<RagContextItem>>,
) -> Result<(), (axum::http::StatusCode, String)> {
    let rag_json = rag_context.map(|c| serde_json::to_value(c).ok()).flatten();
    
    sqlx::query(
        r#"
        INSERT INTO ai_messages (id, conversation_id, role, content, rag_context)
        VALUES ($1, $2, $3, $4, $5)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(conversation_id)
    .bind(role)
    .bind(content)
    .bind(rag_json)
    .execute(pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(())
}

async fn get_recent_messages(
    pool: &PgPool,
    conversation_id: Uuid,
    limit: i32,
) -> Result<Vec<(String, String)>, (axum::http::StatusCode, String)> {
    let rows = sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT role, content
        FROM ai_messages
        WHERE conversation_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#
    )
    .bind(conversation_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Reverse to get chronological order
    Ok(rows.into_iter().rev().collect())
}

/// List conversations handler
async fn list_conversations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ConversationSummary>>, (axum::http::StatusCode, String)> {
    let tenant_id = get_tenant_id_from_context()?;
    let user_id = get_user_id_from_context()?;
    
    let rows = sqlx::query_as::<_, (Uuid, Option<String>, chrono::DateTime<chrono::Utc>, i64)>(
        r#"
        SELECT c.id, c.title, c.created_at, COUNT(m.id) as message_count
        FROM ai_conversations c
        LEFT JOIN ai_messages m ON m.conversation_id = c.id
        WHERE c.tenant_id = $1 AND c.user_id = $2 AND c.is_archived = FALSE
        GROUP BY c.id
        ORDER BY c.created_at DESC
        LIMIT 50
        "#
    )
    .bind(tenant_id)
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(rows.into_iter().map(|(id, title, created_at, message_count)| {
        ConversationSummary {
            id,
            title,
            created_at: created_at.to_rfc3339(),
            message_count,
        }
    }).collect()))
}

/// Get conversation handler
async fn get_conversation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConversationSummary>, (axum::http::StatusCode, String)> {
    let row = sqlx::query_as::<_, (Uuid, Option<String>, chrono::DateTime<chrono::Utc>, i64)>(
        r#"
        SELECT c.id, c.title, c.created_at, COUNT(m.id) as message_count
        FROM ai_conversations c
        LEFT JOIN ai_messages m ON m.conversation_id = c.id
        WHERE c.id = $1
        GROUP BY c.id
        "#
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((axum::http::StatusCode::NOT_FOUND, "Conversation not found".to_string()))?;
    
    Ok(Json(ConversationSummary {
        id: row.0,
        title: row.1,
        created_at: row.2.to_rfc3339(),
        message_count: row.3,
    }))
}

/// Get conversation messages handler
async fn get_conversation_messages(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<serde_json::Value>>, (axum::http::StatusCode, String)> {
    let rows = sqlx::query_as::<_, (Uuid, String, String, chrono::DateTime<chrono::Utc>)>(
        r#"
        SELECT id, role, content, created_at
        FROM ai_messages
        WHERE conversation_id = $1
        ORDER BY created_at ASC
        "#
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(rows.into_iter().map(|(id, role, content, created_at)| {
        serde_json::json!({
            "id": id,
            "role": role,
            "content": content,
            "created_at": created_at.to_rfc3339()
        })
    }).collect()))
}

// ============================================================================
// Helper functions (stubs for now - would use middleware/extractors)
// ============================================================================

fn get_tenant_id_from_context() -> Result<Uuid, (axum::http::StatusCode, String)> {
    // In production, this would come from auth middleware
    Ok(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
}

fn get_user_id_from_context() -> Result<Uuid, (axum::http::StatusCode, String)> {
    // In production, this would come from auth middleware
    Ok(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
}

struct EntityData {
    entity_type: String,
    preview: String,
}

async fn get_entity_data(
    pool: &PgPool,
    tenant_id: Uuid,
    entity_id: Uuid,
) -> Result<Option<EntityData>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, serde_json::Value)>(
        r#"
        SELECT entity_code, data
        FROM entities
        WHERE tenant_id = $1 AND id = $2
        "#
    )
    .bind(tenant_id)
    .bind(entity_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(row.map(|(entity_type, data)| {
        let preview = entity_to_embedding_content(&entity_type, &data);
        EntityData { entity_type, preview }
    }))
}
