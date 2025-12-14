//! Tenant Resolution Middleware
//! 
//! Resolves tenant from subdomain for public endpoints.
//! Supports multiple resolution strategies:
//! 1. X-Tenant-Slug header (for API testing)
//! 2. Host header (production subdomain)
//! 3. tenant_slug query parameter (dev convenience)

use axum::{
    body::Body,
    extract::{Host, Query, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::sync::Arc;
use uuid::Uuid;

use crate::state::AppState;

/// Resolved tenant information available in request extensions
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResolvedTenant {
    pub id: Uuid,
    pub name: String,
    pub subdomain: String,
    pub settings: serde_json::Value,
}

impl ResolvedTenant {
    /// Get branding info from settings
    pub fn get_branding(&self) -> TenantBranding {
        let brand = self.settings.get("brand").cloned().unwrap_or_default();
        TenantBranding {
            logo_url: brand.get("logo_url").and_then(|v| v.as_str()).map(String::from),
            primary_color: brand.get("primary_color").and_then(|v| v.as_str()).map(String::from).unwrap_or_else(|| "#7c3aed".to_string()),
            secondary_color: brand.get("secondary_color").and_then(|v| v.as_str()).map(String::from).unwrap_or_else(|| "#6366f1".to_string()),
            listing_page_title: brand.get("listing_page_title").and_then(|v| v.as_str()).map(String::from),
            address: self.settings.get("address").and_then(|v| v.as_str()).map(String::from),
            phone: self.settings.get("phone").and_then(|v| v.as_str()).map(String::from),
            email: self.settings.get("email").and_then(|v| v.as_str()).map(String::from),
        }
    }
}

/// Public-safe tenant branding information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBranding {
    pub logo_url: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub listing_page_title: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// Query parameters for tenant resolution
#[derive(Debug, Deserialize)]
pub struct TenantQuery {
    pub tenant_slug: Option<String>,
}

/// Middleware to resolve tenant from various sources
/// Priority: X-Tenant-Slug header > Host subdomain > tenant_slug query param
pub async fn resolve_tenant(
    State(state): State<Arc<AppState>>,
    Host(host): Host,
    Query(query): Query<TenantQuery>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    // 1. Check X-Tenant-Slug header
    let slug = request
        .headers()
        .get("X-Tenant-Slug")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    
    // 2. Fall back to Host header subdomain
    let slug = slug.or_else(|| extract_subdomain(&host));
    
    // 3. Fall back to query parameter
    let slug = slug.or(query.tenant_slug);
    
    let slug = match slug {
        Some(s) if !s.is_empty() => s,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Unable to determine tenant. Provide X-Tenant-Slug header, use subdomain, or add ?tenant_slug= query param"
                }))
            ).into_response();
        }
    };
    
    // Query database for tenant
    match resolve_tenant_from_db(&state.pool, &slug).await {
        Ok(Some(tenant)) => {
            // Store tenant in request extensions
            request.extensions_mut().insert(tenant);
            next.run(request).await
        }
        Ok(None) => {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Tenant not found",
                    "subdomain": slug
                }))
            ).into_response()
        }
        Err(e) => {
            tracing::error!("Database error resolving tenant: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to resolve tenant"
                }))
            ).into_response()
        }
    }
}

/// Extract subdomain from host (e.g., "acme" from "acme.jirsi.com")
fn extract_subdomain(host: &str) -> Option<String> {
    // Remove port if present
    let host = host.split(':').next().unwrap_or(host);
    
    // Skip if localhost or IP address
    if host == "localhost" || host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }
    
    // Extract first part of subdomain
    let parts: Vec<&str> = host.split('.').collect();
    
    // Need at least subdomain.domain.tld
    if parts.len() >= 3 {
        let subdomain = parts[0];
        // Skip common non-tenant subdomains
        if subdomain != "www" && subdomain != "api" && subdomain != "app" {
            return Some(subdomain.to_string());
        }
    }
    
    None
}

/// Query database for tenant by subdomain
async fn resolve_tenant_from_db(pool: &PgPool, subdomain: &str) -> Result<Option<ResolvedTenant>, sqlx::Error> {
    let result = sqlx::query_as::<_, ResolvedTenant>(
        r#"
        SELECT id, name, subdomain, settings
        FROM tenants
        WHERE subdomain = $1 AND status IN ('active', 'trial')
        "#
    )
    .bind(subdomain)
    .fetch_optional(pool)
    .await?;
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_subdomain() {
        assert_eq!(extract_subdomain("acme.jirsi.com"), Some("acme".to_string()));
        assert_eq!(extract_subdomain("demo.example.com:3000"), Some("demo".to_string()));
        assert_eq!(extract_subdomain("www.jirsi.com"), None);
        assert_eq!(extract_subdomain("localhost"), None);
        assert_eq!(extract_subdomain("localhost:3000"), None);
        assert_eq!(extract_subdomain("127.0.0.1:3000"), None);
        assert_eq!(extract_subdomain("jirsi.com"), None);
    }
}

