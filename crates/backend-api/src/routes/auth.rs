//! Auth routes

use axum::{
    Router,
    routing::{get, post},
    extract::{State, Json},
    http::{header::SET_COOKIE, HeaderMap},
};
use core_auth::middleware::ExtractAuth;
use core_models::{CreateUser, UserInfo, UserRole};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::AppState;
use crate::error::ApiError;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/register", post(register))
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
