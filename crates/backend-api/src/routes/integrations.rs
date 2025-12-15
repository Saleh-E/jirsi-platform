//! Integration Settings API
//!
//! Endpoints for managing provider configurations (Twilio, Facebook, etc.)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::state::AppState;

/// Query parameters
#[derive(Debug, Deserialize)]
pub struct IntegrationQuery {
    pub tenant_id: Uuid,
}

/// Provider status response
#[derive(Debug, Serialize)]
pub struct ProviderStatusResponse {
    pub provider: String,
    pub display_name: String,
    pub icon: String,
    pub is_configured: bool,
    pub is_enabled: bool,
    pub webhook_url: Option<String>,
    pub last_webhook_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Request to save integration credentials
#[derive(Debug, Deserialize)]
pub struct SaveIntegrationRequest {
    pub credentials: serde_json::Value,
}

/// Create integration routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/integrations", get(list_integrations))
        .route("/api/v1/integrations/:provider", post(save_integration))
        .route("/api/v1/integrations/:provider", delete(delete_integration))
        .route("/api/v1/integrations/:provider/toggle", post(toggle_integration))
}

/// List all provider statuses
async fn list_integrations(
    State(state): State<Arc<AppState>>,
    Query(query): Query<IntegrationQuery>,
) -> impl IntoResponse {
    let tenant_id = query.tenant_id;

    // Get all configured integrations
    let integrations: Result<Vec<(String, bool, Option<String>, Option<chrono::DateTime<chrono::Utc>>)>, _> = 
        sqlx::query_as(
            r#"
            SELECT provider, is_enabled, webhook_url, last_webhook_at
            FROM integrations
            WHERE tenant_id = $1
            "#
        )
        .bind(tenant_id)
        .fetch_all(&state.pool)
        .await;

    let integrations = match integrations {
        Ok(rows) => rows,
        Err(e) => {
            error!(error = %e, "Failed to fetch integrations");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to fetch integrations"
            })));
        }
    };

    // Build status list for all providers
    let all_providers = vec![
        ("twilio", "Twilio (SMS/Voice)", "📱"),
        ("facebook", "Facebook Lead Ads", "📘"),
        ("whatsapp", "WhatsApp Business", "💬"),
        ("email", "Email (SMTP)", "📧"),
    ];

    let base_url = std::env::var("WEBHOOK_BASE_URL")
        .unwrap_or_else(|_| "https://api.jirsi.com".to_string());

    let statuses: Vec<ProviderStatusResponse> = all_providers.iter().map(|(provider, name, icon)| {
        let config = integrations.iter().find(|(p, _, _, _)| p == *provider);
        
        ProviderStatusResponse {
            provider: provider.to_string(),
            display_name: name.to_string(),
            icon: icon.to_string(),
            is_configured: config.is_some(),
            is_enabled: config.map(|(_, e, _, _)| *e).unwrap_or(false),
            webhook_url: config
                .and_then(|(_, _, url, _)| url.clone())
                .or_else(|| Some(format!("{}/webhooks/{}/{}", base_url, tenant_id, provider))),
            last_webhook_at: config.and_then(|(_, _, _, ts)| *ts),
        }
    }).collect();

    (StatusCode::OK, Json(json!({ "integrations": statuses })))
}

/// Save integration credentials
async fn save_integration(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    Query(query): Query<IntegrationQuery>,
    Json(request): Json<SaveIntegrationRequest>,
) -> impl IntoResponse {
    let tenant_id = query.tenant_id;

    // Validate provider
    let valid_providers = vec!["twilio", "facebook", "whatsapp", "email"];
    if !valid_providers.contains(&provider.as_str()) {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "error": "Invalid provider"
        })));
    }

    info!(tenant_id = %tenant_id, provider = %provider, "Saving integration");

    // TODO: Encrypt credentials before saving
    // For now, store as JSON (should encrypt in production!)
    let credentials_json = serde_json::to_vec(&request.credentials)
        .unwrap_or_default();

    let base_url = std::env::var("WEBHOOK_BASE_URL")
        .unwrap_or_else(|_| "https://api.jirsi.com".to_string());
    let webhook_url = format!("{}/webhooks/{}/{}", base_url, tenant_id, provider);
    
    // Generate webhook secret
    let webhook_secret: String = (0..32)
        .map(|_| format!("{:02x}", rand::random::<u8>()))
        .collect();

    let result = sqlx::query(
        r#"
        INSERT INTO integrations (tenant_id, provider, is_enabled, credentials_encrypted, webhook_secret, webhook_url)
        VALUES ($1, $2, true, $3, $4, $5)
        ON CONFLICT (tenant_id, provider) DO UPDATE SET
            credentials_encrypted = EXCLUDED.credentials_encrypted,
            is_enabled = true,
            updated_at = NOW()
        "#
    )
    .bind(tenant_id)
    .bind(&provider)
    .bind(&credentials_json)
    .bind(&webhook_secret)
    .bind(&webhook_url)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            info!(tenant_id = %tenant_id, provider = %provider, "Integration saved");
            (StatusCode::OK, Json(json!({
                "success": true,
                "webhook_url": webhook_url,
                "message": "Integration configured successfully"
            })))
        }
        Err(e) => {
            error!(error = %e, "Failed to save integration");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to save integration"
            })))
        }
    }
}

/// Toggle integration enabled status
async fn toggle_integration(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    Query(query): Query<IntegrationQuery>,
) -> impl IntoResponse {
    let tenant_id = query.tenant_id;

    let result: Result<Option<(bool,)>, _> = sqlx::query_as(
        r#"
        UPDATE integrations
        SET is_enabled = NOT is_enabled, updated_at = NOW()
        WHERE tenant_id = $1 AND provider = $2
        RETURNING is_enabled
        "#
    )
    .bind(tenant_id)
    .bind(&provider)
    .fetch_optional(&state.pool)
    .await;

    match result.ok().flatten() {
        Some((enabled,)) => {
            info!(tenant_id = %tenant_id, provider = %provider, enabled = enabled, "Integration toggled");
            (StatusCode::OK, Json(json!({
                "success": true,
                "is_enabled": enabled
            })))
        }
        None => {
            (StatusCode::NOT_FOUND, Json(json!({
                "error": "Integration not found"
            })))
        }
    }
}

/// Delete/disconnect integration
async fn delete_integration(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    Query(query): Query<IntegrationQuery>,
) -> impl IntoResponse {
    let tenant_id = query.tenant_id;

    let result = sqlx::query(
        "DELETE FROM integrations WHERE tenant_id = $1 AND provider = $2"
    )
    .bind(tenant_id)
    .bind(&provider)
    .execute(&state.pool)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                info!(tenant_id = %tenant_id, provider = %provider, "Integration deleted");
                (StatusCode::OK, Json(json!({
                    "success": true,
                    "message": "Integration disconnected"
                })))
            } else {
                (StatusCode::NOT_FOUND, Json(json!({
                    "error": "Integration not found"
                })))
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to delete integration");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to delete integration"
            })))
        }
    }
}
