//! Integration service for managing provider configurations

use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::encryption::{encrypt_json, decrypt_json, generate_webhook_secret};
use crate::models::{IntegrationConfig, Provider, ProviderCredentials, ProviderStatus};

/// Master encryption key - in production, load from environment/secrets manager
/// This should be 32 bytes for AES-256
fn get_encryption_key() -> [u8; 32] {
    // TODO: Load from environment variable INTEGRATION_ENCRYPTION_KEY
    // For now, use a fixed key (CHANGE IN PRODUCTION!)
    let key_hex = std::env::var("INTEGRATION_ENCRYPTION_KEY")
        .unwrap_or_else(|_| "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string());
    
    let mut key = [0u8; 32];
    if let Ok(bytes) = hex::decode(&key_hex) {
        if bytes.len() >= 32 {
            key.copy_from_slice(&bytes[..32]);
        }
    }
    key
}

/// Service for managing integrations
pub struct IntegrationService;

impl IntegrationService {
    /// Get the status of all providers for a tenant
    pub async fn get_all_statuses(
        pool: &PgPool,
        tenant_id: Uuid,
        base_webhook_url: &str,
    ) -> Result<Vec<ProviderStatus>, String> {
        let all_providers = vec![
            Provider::Twilio,
            Provider::Facebook,
            Provider::WhatsApp,
            Provider::Email,
        ];

        let configs: Vec<(String, bool, Option<String>)> = sqlx::query_as(
            r#"
            SELECT provider, is_enabled, webhook_url
            FROM integrations
            WHERE tenant_id = $1
            "#
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

        let statuses = all_providers.into_iter().map(|provider| {
            let config = configs.iter().find(|(p, _, _)| p == provider.as_str());
            
            ProviderStatus {
                provider,
                display_name: provider.display_name().to_string(),
                icon: provider.icon().to_string(),
                is_configured: config.is_some(),
                is_enabled: config.map(|(_, enabled, _)| *enabled).unwrap_or(false),
                webhook_url: config
                    .and_then(|(_, _, url)| url.clone())
                    .or_else(|| Some(format!("{}/webhooks/{}/{}", base_webhook_url, tenant_id, provider.as_str()))),
                last_webhook_at: None,
            }
        }).collect();

        Ok(statuses)
    }

    /// Get active provider configuration with decrypted credentials
    pub async fn get_active_provider<T: serde::de::DeserializeOwned>(
        pool: &PgPool,
        tenant_id: Uuid,
        provider: Provider,
    ) -> Result<Option<T>, String> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            r#"
            SELECT credentials_encrypted
            FROM integrations
            WHERE tenant_id = $1 
              AND provider = $2
              AND is_enabled = true
              AND credentials_encrypted IS NOT NULL
            "#
        )
        .bind(tenant_id)
        .bind(provider.as_str())
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

        match row {
            Some((encrypted,)) => {
                let key = get_encryption_key();
                let credentials: T = decrypt_json(&encrypted, &key)?;
                Ok(Some(credentials))
            }
            None => Ok(None),
        }
    }

    /// Get webhook secret for signature validation
    pub async fn get_webhook_secret(
        pool: &PgPool,
        tenant_id: Uuid,
        provider: Provider,
    ) -> Result<Option<String>, String> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT webhook_secret
            FROM integrations
            WHERE tenant_id = $1 AND provider = $2
            "#
        )
        .bind(tenant_id)
        .bind(provider.as_str())
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

        Ok(row.map(|(secret,)| secret))
    }

    /// Save or update integration credentials
    pub async fn save_integration(
        pool: &PgPool,
        tenant_id: Uuid,
        credentials: ProviderCredentials,
        base_webhook_url: &str,
    ) -> Result<IntegrationConfig, String> {
        let provider = credentials.provider();
        let key = get_encryption_key();
        
        // Encrypt credentials
        let encrypted = encrypt_json(&credentials, &key)?;
        
        // Generate webhook secret if new
        let webhook_secret = generate_webhook_secret();
        let webhook_url = format!("{}/webhooks/{}/{}", base_webhook_url, tenant_id, provider.as_str());
        
        let id = Uuid::new_v4();
        
        // Upsert integration
        let row: (Uuid, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
            r#"
            INSERT INTO integrations (id, tenant_id, provider, is_enabled, credentials_encrypted, webhook_secret, webhook_url)
            VALUES ($1, $2, $3, true, $4, $5, $6)
            ON CONFLICT (tenant_id, provider) DO UPDATE SET
                credentials_encrypted = EXCLUDED.credentials_encrypted,
                webhook_secret = COALESCE(integrations.webhook_secret, EXCLUDED.webhook_secret),
                is_enabled = true,
                updated_at = NOW()
            RETURNING id, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(tenant_id)
        .bind(provider.as_str())
        .bind(&encrypted)
        .bind(&webhook_secret)
        .bind(&webhook_url)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Insert failed: {}", e))?;

        info!(tenant_id = %tenant_id, provider = ?provider, "Integration saved");

        Ok(IntegrationConfig {
            id: row.0,
            tenant_id,
            provider,
            is_enabled: true,
            credentials_encrypted: Some(encrypted),
            webhook_secret,
            webhook_url,
            created_at: row.1,
            updated_at: row.2,
        })
    }

    /// Toggle integration enabled status
    pub async fn toggle_integration(
        pool: &PgPool,
        tenant_id: Uuid,
        provider: Provider,
    ) -> Result<bool, String> {
        let row: Option<(bool,)> = sqlx::query_as(
            r#"
            UPDATE integrations
            SET is_enabled = NOT is_enabled, updated_at = NOW()
            WHERE tenant_id = $1 AND provider = $2
            RETURNING is_enabled
            "#
        )
        .bind(tenant_id)
        .bind(provider.as_str())
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Update failed: {}", e))?;

        match row {
            Some((enabled,)) => {
                info!(tenant_id = %tenant_id, provider = ?provider, enabled = enabled, "Integration toggled");
                Ok(enabled)
            }
            None => Err("Integration not found".to_string()),
        }
    }

    /// Delete/disconnect an integration
    pub async fn delete_integration(
        pool: &PgPool,
        tenant_id: Uuid,
        provider: Provider,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM integrations
            WHERE tenant_id = $1 AND provider = $2
            "#
        )
        .bind(tenant_id)
        .bind(provider.as_str())
        .execute(pool)
        .await
        .map_err(|e| format!("Delete failed: {}", e))?;

        warn!(tenant_id = %tenant_id, provider = ?provider, "Integration deleted");
        Ok(())
    }
}
