//! Embedding Service - Vector embeddings for RAG
//!
//! Provides:
//! - EmbeddingService trait for embedding generation
//! - OpenAI embeddings integration
//! - Entity embedding storage and retrieval

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Dimension of embedding vectors (OpenAI ada-002)
pub const EMBEDDING_DIM: usize = 1536;

/// Embedding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResult {
    pub embedding: Vec<f32>,
    pub model: String,
    pub tokens_used: u32,
}

/// Entity with embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityWithSimilarity {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub similarity: f32,
    pub content_preview: Option<String>,
}

/// Errors that can occur during embedding operations
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Rate limited")]
    RateLimited,
}

/// Trait for embedding generation services
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Generate embedding for a single text
    async fn embed(&self, text: &str) -> Result<EmbeddingResult, EmbeddingError>;
    
    /// Generate embeddings for multiple texts (batch)
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<EmbeddingResult>, EmbeddingError>;
    
    /// Get the model name
    fn model_name(&self) -> &str;
}

/// OpenAI Embeddings Provider
pub struct OpenAIEmbeddingProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAIEmbeddingProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "text-embedding-ada-002".to_string(),
            client: reqwest::Client::new(),
        }
    }
    
    pub fn with_model(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl EmbeddingService for OpenAIEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<EmbeddingResult, EmbeddingError> {
        if text.trim().is_empty() {
            return Err(EmbeddingError::InvalidInput("Empty text".to_string()));
        }
        
        let response = self.client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": self.model,
                "input": text
            }))
            .send()
            .await
            .map_err(|e| EmbeddingError::ApiError(e.to_string()))?;
        
        if response.status() == 429 {
            return Err(EmbeddingError::RateLimited);
        }
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::ApiError(format!("HTTP error: {}", error_text)));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| EmbeddingError::ApiError(e.to_string()))?;
        
        let embedding = json["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| EmbeddingError::ApiError("Invalid response format".to_string()))?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();
        
        let tokens_used = json["usage"]["total_tokens"]
            .as_u64()
            .unwrap_or(0) as u32;
        
        Ok(EmbeddingResult {
            embedding,
            model: self.model.clone(),
            tokens_used,
        })
    }
    
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<EmbeddingResult>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        
        let response = self.client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": self.model,
                "input": texts
            }))
            .send()
            .await
            .map_err(|e| EmbeddingError::ApiError(e.to_string()))?;
        
        if response.status() == 429 {
            return Err(EmbeddingError::RateLimited);
        }
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::ApiError(format!("HTTP error: {}", error_text)));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| EmbeddingError::ApiError(e.to_string()))?;
        
        let data = json["data"].as_array()
            .ok_or_else(|| EmbeddingError::ApiError("Invalid response format".to_string()))?;
        
        let tokens_used = json["usage"]["total_tokens"]
            .as_u64()
            .unwrap_or(0) as u32;
        
        let results = data.iter()
            .map(|item| {
                let embedding = item["embedding"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect();
                
                EmbeddingResult {
                    embedding,
                    model: self.model.clone(),
                    tokens_used: tokens_used / data.len() as u32,
                }
            })
            .collect();
        
        Ok(results)
    }
    
    fn model_name(&self) -> &str {
        &self.model
    }
}

/// Store an entity embedding in the database
pub async fn store_entity_embedding(
    pool: &PgPool,
    tenant_id: Uuid,
    entity_id: Uuid,
    entity_type: &str,
    embedding: &[f32],
    content_preview: Option<&str>,
    model_name: &str,
) -> Result<(), EmbeddingError> {
    // Create content hash
    let content_hash = format!("{:x}", md5::compute(content_preview.unwrap_or("")));
    
    // Convert embedding to pgvector format
    let embedding_str = format!(
        "[{}]",
        embedding.iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    
    sqlx::query(
        r#"
        INSERT INTO entity_embeddings (tenant_id, entity_id, entity_type, embedding, content_hash, content_preview, model_name)
        VALUES ($1, $2, $3, $4::vector, $5, $6, $7)
        ON CONFLICT (tenant_id, entity_id) DO UPDATE SET
            embedding = EXCLUDED.embedding,
            content_hash = EXCLUDED.content_hash,
            content_preview = EXCLUDED.content_preview,
            model_name = EXCLUDED.model_name,
            updated_at = NOW()
        "#
    )
    .bind(tenant_id)
    .bind(entity_id)
    .bind(entity_type)
    .bind(&embedding_str)
    .bind(&content_hash)
    .bind(content_preview)
    .bind(model_name)
    .execute(pool)
    .await?;
    
    Ok(())
}

/// Find similar entities using vector search
pub async fn find_similar_entities(
    pool: &PgPool,
    tenant_id: Uuid,
    query_embedding: &[f32],
    entity_type: Option<&str>,
    limit: i32,
    threshold: f32,
) -> Result<Vec<EntityWithSimilarity>, EmbeddingError> {
    let embedding_str = format!(
        "[{}]",
        query_embedding.iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    
    let rows = sqlx::query_as::<_, (Uuid, String, f32, Option<String>)>(
        r#"
        SELECT 
            entity_id,
            entity_type,
            1 - (embedding <=> $1::vector) as similarity,
            content_preview
        FROM entity_embeddings
        WHERE tenant_id = $2
          AND ($3::text IS NULL OR entity_type = $3)
          AND 1 - (embedding <=> $1::vector) >= $4
        ORDER BY embedding <=> $1::vector
        LIMIT $5
        "#
    )
    .bind(&embedding_str)
    .bind(tenant_id)
    .bind(entity_type)
    .bind(threshold)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    
    Ok(rows.into_iter()
        .map(|(entity_id, entity_type, similarity, content_preview)| {
            EntityWithSimilarity {
                entity_id,
                entity_type,
                similarity,
                content_preview,
            }
        })
        .collect())
}

/// Generate text content from entity data for embedding
pub fn entity_to_embedding_content(entity_type: &str, data: &serde_json::Value) -> String {
    match entity_type {
        "contact" => {
            let name = data.get("name")
                .or_else(|| data.get("first_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let email = data.get("email").and_then(|v| v.as_str()).unwrap_or("");
            let company = data.get("company").and_then(|v| v.as_str()).unwrap_or("");
            let notes = data.get("notes").and_then(|v| v.as_str()).unwrap_or("");
            
            format!("Contact: {} Email: {} Company: {} Notes: {}", name, email, company, notes)
        }
        "property" => {
            let title = data.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let address = data.get("address").and_then(|v| v.as_str()).unwrap_or("");
            let city = data.get("city").and_then(|v| v.as_str()).unwrap_or("");
            let property_type = data.get("property_type").and_then(|v| v.as_str()).unwrap_or("");
            let bedrooms = data.get("bedrooms").and_then(|v| v.as_i64()).unwrap_or(0);
            let price = data.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let description = data.get("description").and_then(|v| v.as_str()).unwrap_or("");
            
            format!(
                "Property: {} Address: {} {} Type: {} Bedrooms: {} Price: ${:.0} Description: {}",
                title, address, city, property_type, bedrooms, price, description
            )
        }
        "deal" => {
            let title = data.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let value = data.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let stage = data.get("stage").and_then(|v| v.as_str()).unwrap_or("");
            let notes = data.get("notes").and_then(|v| v.as_str()).unwrap_or("");
            
            format!("Deal: {} Value: ${:.0} Stage: {} Notes: {}", title, value, stage, notes)
        }
        _ => {
            // Generic fallback
            serde_json::to_string_pretty(data).unwrap_or_default()
        }
    }
}
