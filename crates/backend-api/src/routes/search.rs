//! Unified Search API
//!
//! Provides a single endpoint to search across all entity types:
//! - Contacts
//! - Deals
//! - Properties
//! - Companies
//! - Tasks

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::AppState;

/// Search query parameters
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    /// Search query string
    pub q: String,
    /// Tenant ID (required for multi-tenancy)
    pub tenant_id: Uuid,
    /// Optional limit (default 20)
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Optional entity type filter (contact, deal, property, etc.)
    pub entity_type: Option<String>,
}

fn default_limit() -> i64 {
    20
}

/// Search result item
#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub id: Uuid,
    pub entity_type: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: String,
    pub url: String,
}

/// Search response
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResultItem>,
    pub total: usize,
    pub query: String,
}

/// Build search routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/search", get(unified_search))
}

/// Unified search endpoint - searches across all entity types
async fn unified_search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, (axum::http::StatusCode, String)> {
    let query = params.q.to_lowercase();
    let tenant_id = params.tenant_id;
    let limit = params.limit;
    
    if query.len() < 2 {
        return Ok(Json(SearchResponse {
            results: vec![],
            total: 0,
            query: params.q,
        }));
    }
    
    let mut results = Vec::new();
    
    // Search contacts
    if params.entity_type.is_none() || params.entity_type.as_deref() == Some("contact") {
        let contacts = search_contacts(&state.pool, tenant_id, &query, limit).await?;
        results.extend(contacts);
    }
    
    // Search deals
    if params.entity_type.is_none() || params.entity_type.as_deref() == Some("deal") {
        let deals = search_deals(&state.pool, tenant_id, &query, limit).await?;
        results.extend(deals);
    }
    
    // Search properties
    if params.entity_type.is_none() || params.entity_type.as_deref() == Some("property") {
        let properties = search_properties(&state.pool, tenant_id, &query, limit).await?;
        results.extend(properties);
    }
    
    // Search companies
    if params.entity_type.is_none() || params.entity_type.as_deref() == Some("company") {
        let companies = search_companies(&state.pool, tenant_id, &query, limit).await?;
        results.extend(companies);
    }
    
    let total = results.len();
    
    // Sort by relevance (exact matches first, then partial)
    results.sort_by(|a, b| {
        let a_exact = a.title.to_lowercase() == query;
        let b_exact = b.title.to_lowercase() == query;
        b_exact.cmp(&a_exact)
    });
    
    // Limit total results
    results.truncate(limit as usize);
    
    Ok(Json(SearchResponse {
        results,
        total,
        query: params.q,
    }))
}

/// Search contacts by name, email, or phone
async fn search_contacts(
    pool: &PgPool,
    tenant_id: Uuid,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResultItem>, (axum::http::StatusCode, String)> {
    let search_pattern = format!("%{}%", query);
    
    let rows = sqlx::query!(
        r#"
        SELECT id, data
        FROM entities
        WHERE tenant_id = $1
          AND entity_code = 'contact'
          AND (
            LOWER(data->>'name') LIKE LOWER($2)
            OR LOWER(data->>'first_name') LIKE LOWER($2)
            OR LOWER(data->>'last_name') LIKE LOWER($2)
            OR LOWER(data->>'email') LIKE LOWER($2)
            OR data->>'phone' LIKE $2
          )
        ORDER BY updated_at DESC
        LIMIT $3
        "#,
        tenant_id,
        search_pattern,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(rows
        .into_iter()
        .map(|row| {
            let data = row.data;
            let name = data
                .get("name")
                .or_else(|| data.get("first_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed")
                .to_string();
            let email = data
                .get("email")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            SearchResultItem {
                id: row.id,
                entity_type: "contact".to_string(),
                title: name,
                subtitle: email,
                icon: "👤".to_string(),
                url: format!("/app/crm/entity/contact/{}", row.id),
            }
        })
        .collect())
}

/// Search deals by title
async fn search_deals(
    pool: &PgPool,
    tenant_id: Uuid,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResultItem>, (axum::http::StatusCode, String)> {
    let search_pattern = format!("%{}%", query);
    
    let rows = sqlx::query!(
        r#"
        SELECT id, data
        FROM entities
        WHERE tenant_id = $1
          AND entity_code = 'deal'
          AND (
            LOWER(data->>'title') LIKE LOWER($2)
            OR LOWER(data->>'name') LIKE LOWER($2)
          )
        ORDER BY updated_at DESC
        LIMIT $3
        "#,
        tenant_id,
        search_pattern,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(rows
        .into_iter()
        .map(|row| {
            let data = row.data;
            let title = data
                .get("title")
                .or_else(|| data.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed Deal")
                .to_string();
            let value = data
                .get("value")
                .and_then(|v| v.as_f64())
                .map(|v| format!("${:.0}", v));
            
            SearchResultItem {
                id: row.id,
                entity_type: "deal".to_string(),
                title,
                subtitle: value,
                icon: "💰".to_string(),
                url: format!("/app/crm/entity/deal/{}", row.id),
            }
        })
        .collect())
}

/// Search properties by title or address
async fn search_properties(
    pool: &PgPool,
    tenant_id: Uuid,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResultItem>, (axum::http::StatusCode, String)> {
    let search_pattern = format!("%{}%", query);
    
    let rows = sqlx::query!(
        r#"
        SELECT id, data
        FROM entities
        WHERE tenant_id = $1
          AND entity_code = 'property'
          AND (
            LOWER(data->>'title') LIKE LOWER($2)
            OR LOWER(data->>'address') LIKE LOWER($2)
            OR LOWER(data->>'city') LIKE LOWER($2)
          )
        ORDER BY updated_at DESC
        LIMIT $3
        "#,
        tenant_id,
        search_pattern,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(rows
        .into_iter()
        .map(|row| {
            let data = row.data;
            let title = data
                .get("title")
                .or_else(|| data.get("address"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed Property")
                .to_string();
            let price = data
                .get("price")
                .and_then(|v| v.as_f64())
                .map(|v| format!("${:.0}", v));
            
            SearchResultItem {
                id: row.id,
                entity_type: "property".to_string(),
                title,
                subtitle: price,
                icon: "🏠".to_string(),
                url: format!("/app/realestate/entity/property/{}", row.id),
            }
        })
        .collect())
}

/// Search companies by name
async fn search_companies(
    pool: &PgPool,
    tenant_id: Uuid,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResultItem>, (axum::http::StatusCode, String)> {
    let search_pattern = format!("%{}%", query);
    
    let rows = sqlx::query!(
        r#"
        SELECT id, data
        FROM entities
        WHERE tenant_id = $1
          AND entity_code = 'company'
          AND LOWER(data->>'name') LIKE LOWER($2)
        ORDER BY updated_at DESC
        LIMIT $3
        "#,
        tenant_id,
        search_pattern,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(rows
        .into_iter()
        .map(|row| {
            let data = row.data;
            let name = data
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed Company")
                .to_string();
            let industry = data
                .get("industry")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            SearchResultItem {
                id: row.id,
                entity_type: "company".to_string(),
                title: name,
                subtitle: industry,
                icon: "🏢".to_string(),
                url: format!("/app/crm/entity/company/{}", row.id),
            }
        })
        .collect())
}
