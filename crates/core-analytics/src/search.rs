//! Search service

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// A search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub title: String,
    pub subtitle: Option<String>,
    pub highlights: Vec<String>,
    pub score: f32,
}

/// Search service for full-text search across entities
pub struct SearchService {
    pool: PgPool,
}

impl SearchService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Global search across all searchable entities
    pub async fn search(
        &self,
        tenant_id: Uuid,
        query: &str,
        _entity_types: Option<Vec<String>>,
        limit: i32,
    ) -> Result<Vec<SearchResult>, sqlx::Error> {
        use sqlx::Row;
        
        // For v1, use simple ILIKE search
        // TODO: Upgrade to Postgres FTS or ParadeDB in v2
        
        let search_pattern = format!("%{}%", query);
        
        // Search contacts
        let rows = sqlx::query(
            r#"
            SELECT 
                id,
                name as title,
                email as subtitle
            FROM contacts
            WHERE tenant_id = $1 
              AND (name ILIKE $2 OR email ILIKE $2)
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(&search_pattern)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let results: Vec<SearchResult> = rows
            .iter()
            .map(|row| SearchResult {
                entity_type: "contact".to_string(),
                entity_id: row.try_get("id").unwrap_or_default(),
                title: row.try_get("title").unwrap_or_default(),
                subtitle: row.try_get("subtitle").ok(),
                highlights: vec![],
                score: 1.0,
            })
            .collect();

        Ok(results)
    }
}
