//! Auth routes

use axum::{
    Router,
    routing::{get, post},
    extract::{State, Json},
    http::{header::SET_COOKIE, HeaderMap},
};
use chrono::Utc;
use core_auth::middleware::ExtractAuth;
use core_auth::password::hash_password;
use core_models::{CreateUser, UserInfo, UserRole, Tenant, TenantStatus, PlanTier};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::state::AppState;
use crate::error::ApiError;
use crate::seed::seed_new_tenant;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/register", post(register))
        .route("/register-tenant", post(register_tenant))
        .route("/check-subdomain", post(check_subdomain))
        .route("/me", get(me))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub tenant_subdomain: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserInfo,
    pub token: String,
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<LoginResponse>), ApiError> {
    // Resolve tenant
    let tenant = state.tenant_service
        .get_by_subdomain(&req.tenant_subdomain)
        .await?;

    // Authenticate user
    let user = state.user_service
        .authenticate(tenant.id, &req.email, &req.password)
        .await?;

    // Create session
    let (_session, token) = state.session_service
        .create_session(&user, None, None)
        .await?;

    // Set cookie
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        format!("session={}; HttpOnly; Path=/; Max-Age=604800", token)
            .parse()
            .unwrap(),
    );

    Ok((headers, Json(LoginResponse {
        user: user.into(),
        token,
    })))
}

async fn logout(
    State(state): State<Arc<AppState>>,
    auth: Option<ExtractAuth>,
) -> Result<(HeaderMap, Json<serde_json::Value>), ApiError> {
    // Delete session if authenticated
    if let Some(ExtractAuth(auth_context)) = auth {
        let _ = state.session_service
            .delete_session(auth_context.session_id)
            .await;
    }

    // Clear the session cookie
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        "session=; HttpOnly; Path=/; Max-Age=0".parse().unwrap(),
    );

    Ok((headers, Json(serde_json::json!({ "success": true }))))
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub tenant_subdomain: String,
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<UserInfo>, ApiError> {
    // Resolve tenant
    let tenant = state.tenant_service
        .get_by_subdomain(&req.tenant_subdomain)
        .await?;

    // Create user
    let user = state.user_service
        .create(CreateUser {
            tenant_id: tenant.id,
            email: req.email,
            name: req.name,
            password: req.password,
            role: UserRole::Member,
        })
        .await?;

    Ok(Json(user.into()))
}

async fn me(
    auth: ExtractAuth,
) -> Result<Json<UserInfo>, ApiError> {
    Ok(Json(auth.0.user))
}

// ============================================================================
// TENANT REGISTRATION
// ============================================================================

/// Reserved subdomains that cannot be used
const RESERVED_SUBDOMAINS: &[&str] = &[
    "www", "api", "app", "admin", "dashboard", "mail", "email",
    "support", "help", "docs", "blog", "status", "demo", "test",
    "staging", "dev", "development", "production", "cdn", "static",
    "assets", "media", "files", "images", "ftp", "ssh", "vpn",
];

#[derive(Debug, Deserialize)]
pub struct RegisterTenantRequest {
    pub company_name: String,
    pub subdomain: String,
    pub admin_email: String,
    pub admin_password: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterTenantResponse {
    pub success: bool,
    pub tenant_id: Uuid,
    pub subdomain: String,
    pub admin_email: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CheckSubdomainRequest {
    pub subdomain: String,
}

#[derive(Debug, Serialize)]
pub struct CheckSubdomainResponse {
    pub available: bool,
    pub subdomain: String,
    pub message: Option<String>,
}

/// Check if a subdomain is available
async fn check_subdomain(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CheckSubdomainRequest>,
) -> Result<Json<CheckSubdomainResponse>, ApiError> {
    let subdomain = sanitize_subdomain(&req.subdomain);
    
    // Check if reserved
    if RESERVED_SUBDOMAINS.contains(&subdomain.as_str()) {
        return Ok(Json(CheckSubdomainResponse {
            available: false,
            subdomain,
            message: Some("This subdomain is reserved".to_string()),
        }));
    }
    
    // Check format
    if !is_valid_subdomain(&subdomain) {
        return Ok(Json(CheckSubdomainResponse {
            available: false,
            subdomain,
            message: Some("Subdomain must be 3-50 characters, lowercase letters, numbers, and hyphens only".to_string()),
        }));
    }
    
    // Check availability in database
    let available = state.tenant_service
        .subdomain_available(&subdomain)
        .await
        .unwrap_or(false);
    
    Ok(Json(CheckSubdomainResponse {
        available,
        subdomain,
        message: if available { None } else { Some("This subdomain is already taken".to_string()) },
    }))
}

/// Register a new tenant with admin user and seed data
async fn register_tenant(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterTenantRequest>,
) -> Result<Json<RegisterTenantResponse>, ApiError> {
    use sqlx::Row;
    
    let subdomain = sanitize_subdomain(&req.subdomain);
    
    // Validate subdomain format
    if !is_valid_subdomain(&subdomain) {
        return Err(ApiError::BadRequest(
            "Subdomain must be 3-50 characters, lowercase letters, numbers, and hyphens only".to_string()
        ));
    }
    
    // Check reserved subdomains
    if RESERVED_SUBDOMAINS.contains(&subdomain.as_str()) {
        return Err(ApiError::BadRequest(
            "This subdomain is reserved and cannot be used".to_string()
        ));
    }
    
    // Check subdomain availability
    let available = state.tenant_service
        .subdomain_available(&subdomain)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;
    
    if !available {
        return Err(ApiError::Conflict(
            "This subdomain is already taken".to_string()
        ));
    }
    
    // Validate email format (basic check)
    if !req.admin_email.contains('@') || !req.admin_email.contains('.') {
        return Err(ApiError::BadRequest("Invalid email format".to_string()));
    }
    
    // Validate password strength
    if req.admin_password.len() < 8 {
        return Err(ApiError::BadRequest(
            "Password must be at least 8 characters".to_string()
        ));
    }
    
    // Start a transaction for atomic tenant creation
    let mut tx = state.pool.begin().await
        .map_err(|e| ApiError::Internal(format!("Failed to start transaction: {}", e)))?;
    
    let now = Utc::now();
    let tenant_id = Uuid::new_v4();
    
    // Create Tenant record
    let default_settings = serde_json::json!({
        "branding": {
            "primary_color": "#7c3aed",
            "secondary_color": "#6366f1"
        },
        "hero": {
            "headline": "Welcome to Your New Platform",
            "subtext": "Manage your business with ease"
        }
    });
    
    sqlx::query(
        r#"
        INSERT INTO tenants (id, name, subdomain, custom_domain, plan, status, settings, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#
    )
    .bind(tenant_id)
    .bind(&req.company_name)
    .bind(&subdomain)
    .bind(None::<String>)
    .bind(serde_json::to_string(&PlanTier::Free).unwrap_or_else(|_| "free".to_string()))
    .bind(serde_json::to_string(&TenantStatus::Trial).unwrap_or_else(|_| "trial".to_string()))
    .bind(&default_settings)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to create tenant: {}", e)))?;
    
    // Create Admin User
    let admin_id = Uuid::new_v4();
    let password_hash = hash_password(&req.admin_password)
        .map_err(|e| ApiError::Internal(format!("Failed to hash password: {}", e)))?;
    
    sqlx::query(
        r#"
        INSERT INTO users (id, tenant_id, email, name, password_hash, role, status, avatar_url, preferences, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#
    )
    .bind(admin_id)
    .bind(tenant_id)
    .bind(&req.admin_email)
    .bind(&req.company_name) // Use company name as admin name initially
    .bind(&password_hash)
    .bind("admin")
    .bind("active")
    .bind(None::<String>)
    .bind(serde_json::json!({}))
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to create admin user: {}", e)))?;
    
    // Commit the transaction before seeding
    tx.commit().await
        .map_err(|e| ApiError::Internal(format!("Failed to commit transaction: {}", e)))?;
    
    // Seed the tenant data (entity types, fields, views, workflows)
    // This runs in its own transaction
    seed_new_tenant(tenant_id, &state.pool)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to seed tenant data: {}", e)))?;
    
    Ok(Json(RegisterTenantResponse {
        success: true,
        tenant_id,
        subdomain: subdomain.clone(),
        admin_email: req.admin_email.clone(),
        message: format!(
            "Tenant '{}' created successfully! You can now login at {}.jirsi.com",
            req.company_name, subdomain
        ),
    }))
}

/// Sanitize subdomain: lowercase, trim, remove invalid chars
fn sanitize_subdomain(subdomain: &str) -> String {
    subdomain
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

/// Validate subdomain format
fn is_valid_subdomain(subdomain: &str) -> bool {
    let len = subdomain.len();
    
    // Length check: 3-50 characters
    if len < 3 || len > 50 {
        return false;
    }
    
    // Must start and end with alphanumeric
    let chars: Vec<char> = subdomain.chars().collect();
    if !chars[0].is_alphanumeric() || !chars[len - 1].is_alphanumeric() {
        return false;
    }
    
    // Only lowercase letters, numbers, and hyphens
    subdomain.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

