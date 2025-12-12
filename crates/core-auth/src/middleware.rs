//! Auth middleware for Axum

use axum::{
    async_trait,
    extract::{FromRequestParts, Request, State},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::Response,
};
use core_models::{AuthContext, TenantContext};
use std::sync::Arc;

use crate::session::SessionService;
use crate::tenant::TenantService;

/// Extract tenant context from request
/// 
/// Uses the Host header to determine the tenant
pub async fn tenant_middleware(
    State(tenant_service): State<Arc<TenantService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get host header
    let host = request
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Platform domain (would come from config in production)
    let platform_domain = "saas.local"; // TODO: from config

    // Resolve tenant
    let tenant_context = tenant_service
        .resolve_from_host(host, platform_domain)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Add tenant context to request extensions
    request.extensions_mut().insert(tenant_context);

    Ok(next.run(request).await)
}

/// Extract auth context from session cookie
pub async fn auth_middleware(
    State(session_service): State<Arc<SessionService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get session token from cookie
    let token = request
        .headers()
        .get("cookie")
        .and_then(|c| c.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find_map(|cookie| {
                    let cookie = cookie.trim();
                    if cookie.starts_with("session=") {
                        Some(cookie.strip_prefix("session=").unwrap().to_string())
                    } else {
                        None
                    }
                })
        });

    if let Some(token) = token {
        // Validate session
        if let Ok(auth_context) = session_service.validate_session(&token).await {
            request.extensions_mut().insert(auth_context);
        }
    }

    Ok(next.run(request).await)
}

/// Require authentication - returns 401 if no valid session
pub async fn require_auth(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check for auth context
    if request.extensions().get::<AuthContext>().is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}

/// Extractor for TenantContext
#[derive(Debug, Clone)]
pub struct ExtractTenant(pub TenantContext);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractTenant
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<TenantContext>()
            .cloned()
            .map(ExtractTenant)
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// Extractor for AuthContext
#[derive(Debug, Clone)]
pub struct ExtractAuth(pub AuthContext);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractAuth
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .map(ExtractAuth)
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}
