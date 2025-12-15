//! Tenant management routes

use axum::{
    Router,
    routing::{get, patch},
    extract::{State, Json, Query},
};
use chrono::Utc;
use core_auth::middleware::ExtractAuth;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::state::AppState;
use crate::error::ApiError;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/settings", get(get_settings))
        .route("/settings", patch(update_settings))
        .route("/branding", get(get_branding))
}

// ============================================================================
// TENANT SETTINGS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantBranding {
    #[serde(default)]
    pub logo_url: Option<String>,
    #[serde(default)]
    pub favicon_url: Option<String>,
    #[serde(default = "default_primary_color")]
    pub primary_color: String,
    #[serde(default = "default_secondary_color")]
    pub secondary_color: String,
}

fn default_primary_color() -> String { "#7c3aed".to_string() }
fn default_secondary_color() -> String { "#6366f1".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantHero {
    #[serde(default)]
    pub headline: Option<String>,
    #[serde(default)]
    pub subtext: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantContact {
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantSettings {
    #[serde(default)]
    pub branding: TenantBranding,
    #[serde(default)]
    pub hero: TenantHero,
    #[serde(default)]
    pub contact: TenantContact,
}

#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub subdomain: String,
    pub settings: TenantSettings,
}

#[derive(Debug, Deserialize)]
pub struct TenantQuery {
    pub tenant_id: Option<Uuid>,
}

/// Get current tenant settings
async fn get_settings(
    State(state): State<Arc<AppState>>,
    auth: ExtractAuth,
) -> Result<Json<SettingsResponse>, ApiError> {
    use sqlx::Row;
    
    let tenant_id = auth.0.user.tenant_id;
    
    let row = sqlx::query(
        "SELECT id, name, subdomain, settings FROM tenants WHERE id = $1"
    )
    .bind(tenant_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound("Tenant not found".to_string()))?;
    
    let settings_json: serde_json::Value = row.try_get("settings").unwrap_or(serde_json::json!({}));
    let settings: TenantSettings = serde_json::from_value(settings_json).unwrap_or_default();
    
    Ok(Json(SettingsResponse {
        tenant_id,
        tenant_name: row.try_get("name").unwrap_or_default(),
        subdomain: row.try_get("subdomain").unwrap_or_default(),
        settings,
    }))
}

/// Update tenant settings (partial update)
async fn update_settings(
    State(state): State<Arc<AppState>>,
    auth: ExtractAuth,
    Json(new_settings): Json<TenantSettings>,
) -> Result<Json<SettingsResponse>, ApiError> {
    use sqlx::Row;
    
    let tenant_id = auth.0.user.tenant_id;
    let now = Utc::now();
    
    // Get current settings
    let row = sqlx::query(
        "SELECT id, name, subdomain, settings FROM tenants WHERE id = $1"
    )
    .bind(tenant_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound("Tenant not found".to_string()))?;
    
    let tenant_name: String = row.try_get("name").unwrap_or_default();
    let subdomain: String = row.try_get("subdomain").unwrap_or_default();
    
    // Merge with existing settings (new values override)
    let current_json: serde_json::Value = row.try_get("settings").unwrap_or(serde_json::json!({}));
    let mut current: TenantSettings = serde_json::from_value(current_json).unwrap_or_default();
    
    // Update branding
    if new_settings.branding.logo_url.is_some() {
        current.branding.logo_url = new_settings.branding.logo_url;
    }
    if new_settings.branding.favicon_url.is_some() {
        current.branding.favicon_url = new_settings.branding.favicon_url;
    }
    if new_settings.branding.primary_color != default_primary_color() {
        current.branding.primary_color = new_settings.branding.primary_color;
    }
    if new_settings.branding.secondary_color != default_secondary_color() {
        current.branding.secondary_color = new_settings.branding.secondary_color;
    }
    
    // Update hero
    if new_settings.hero.headline.is_some() {
        current.hero.headline = new_settings.hero.headline;
    }
    if new_settings.hero.subtext.is_some() {
        current.hero.subtext = new_settings.hero.subtext;
    }
    if new_settings.hero.image_url.is_some() {
        current.hero.image_url = new_settings.hero.image_url;
    }
    
    // Update contact
    if new_settings.contact.phone.is_some() {
        current.contact.phone = new_settings.contact.phone;
    }
    if new_settings.contact.email.is_some() {
        current.contact.email = new_settings.contact.email;
    }
    if new_settings.contact.address.is_some() {
        current.contact.address = new_settings.contact.address;
    }
    
    // Save updated settings
    let settings_json = serde_json::to_value(&current)
        .map_err(|e| ApiError::Internal(format!("Failed to serialize settings: {}", e)))?;
    
    sqlx::query(
        "UPDATE tenants SET settings = $1, updated_at = $2 WHERE id = $3"
    )
    .bind(&settings_json)
    .bind(now)
    .bind(tenant_id)
    .execute(&state.pool)
    .await?;
    
    Ok(Json(SettingsResponse {
        tenant_id,
        tenant_name,
        subdomain,
        settings: current,
    }))
}

// ============================================================================
// PUBLIC BRANDING (no auth required)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct BrandingQuery {
    pub tenant_slug: Option<String>,
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct BrandingResponse {
    pub id: Uuid,
    pub name: String,
    pub logo_url: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub hero_headline: Option<String>,
    pub hero_subtext: Option<String>,
    pub hero_image_url: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// Get tenant branding for public pages (no auth required)
async fn get_branding(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BrandingQuery>,
) -> Result<Json<BrandingResponse>, ApiError> {
    use sqlx::Row;
    
    let row = if let Some(slug) = query.tenant_slug {
        sqlx::query(
            "SELECT id, name, settings FROM tenants WHERE subdomain = $1"
        )
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await?
    } else if let Some(id) = query.tenant_id {
        sqlx::query(
            "SELECT id, name, settings FROM tenants WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
    } else {
        return Err(ApiError::BadRequest("Either tenant_slug or tenant_id is required".to_string()));
    };
    
    let row = row.ok_or(ApiError::NotFound("Tenant not found".to_string()))?;
    
    let tenant_id: Uuid = row.try_get("id").unwrap_or_default();
    let tenant_name: String = row.try_get("name").unwrap_or_default();
    let settings_json: serde_json::Value = row.try_get("settings").unwrap_or(serde_json::json!({}));
    let settings: TenantSettings = serde_json::from_value(settings_json).unwrap_or_default();
    
    Ok(Json(BrandingResponse {
        id: tenant_id,
        name: tenant_name,
        logo_url: settings.branding.logo_url,
        primary_color: settings.branding.primary_color,
        secondary_color: settings.branding.secondary_color,
        hero_headline: settings.hero.headline,
        hero_subtext: settings.hero.subtext,
        hero_image_url: settings.hero.image_url,
        address: settings.contact.address,
        phone: settings.contact.phone,
        email: settings.contact.email,
    }))
}
