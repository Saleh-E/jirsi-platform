//! Tenant management

use core_models::{Tenant, TenantContext, TenantStatus, PlanTier};
use sqlx::PgPool;
use uuid::Uuid;

use crate::AuthError;

/// Tenant service for multi-tenant management
#[derive(Clone)]
pub struct TenantService {
    pool: PgPool,
}

impl TenantService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get tenant by subdomain
    pub async fn get_by_subdomain(&self, subdomain: &str) -> Result<Tenant, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, name, subdomain, custom_domain,
                plan, status, settings,
                created_at, updated_at
            FROM tenants
            WHERE subdomain = $1
            "#,
        )
        .bind(subdomain)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AuthError::TenantNotFound)?;

        tenant_from_row(&row)
    }

    /// Get tenant by custom domain
    pub async fn get_by_domain(&self, domain: &str) -> Result<Tenant, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, name, subdomain, custom_domain,
                plan, status, settings,
                created_at, updated_at
            FROM tenants
            WHERE custom_domain = $1
            "#,
        )
        .bind(domain)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AuthError::TenantNotFound)?;

        tenant_from_row(&row)
    }

    /// Get tenant by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Tenant, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, name, subdomain, custom_domain,
                plan, status, settings,
                created_at, updated_at
            FROM tenants
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AuthError::TenantNotFound)?;

        tenant_from_row(&row)
    }

    /// Resolve tenant from host header
    /// 
    /// Supports: subdomain.platform.com or custom domains
    pub async fn resolve_from_host(
        &self,
        host: &str,
        platform_domain: &str,
    ) -> Result<TenantContext, AuthError> {
        // Check if it's a subdomain of the platform
        let tenant = if host.ends_with(platform_domain) {
            // Extract subdomain
            let subdomain = host
                .strip_suffix(platform_domain)
                .and_then(|s| s.strip_suffix('.'))
                .ok_or(AuthError::TenantNotFound)?;
            
            self.get_by_subdomain(subdomain).await?
        } else {
            // Try as custom domain
            self.get_by_domain(host).await?
        };

        // Check tenant is active
        if !tenant.is_active() {
            return Err(AuthError::TenantNotActive);
        }

        Ok(TenantContext {
            tenant_id: tenant.id,
            subdomain: tenant.subdomain,
            plan: tenant.plan,
        })
    }

    /// Create a new tenant
    pub async fn create(&self, tenant: &Tenant) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            INSERT INTO tenants (id, name, subdomain, custom_domain, plan, status, settings, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(tenant.id)
        .bind(&tenant.name)
        .bind(&tenant.subdomain)
        .bind(&tenant.custom_domain)
        .bind(serde_json::to_string(&tenant.plan).unwrap_or_default())
        .bind(serde_json::to_string(&tenant.status).unwrap_or_default())
        .bind(&tenant.settings)
        .bind(tenant.created_at)
        .bind(tenant.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if subdomain is available
    pub async fn subdomain_available(&self, subdomain: &str) -> Result<bool, AuthError> {
        let row = sqlx::query(
            r#"SELECT COUNT(*) as count FROM tenants WHERE subdomain = $1"#,
        )
        .bind(subdomain)
        .fetch_one(&self.pool)
        .await?;

        use sqlx::Row;
        let count: i64 = row.try_get("count").unwrap_or(0);
        Ok(count == 0)
    }
}

fn tenant_from_row(row: &sqlx::postgres::PgRow) -> Result<Tenant, AuthError> {
    use sqlx::Row;
    
    let plan_str: String = row.try_get("plan").unwrap_or_default();
    let status_str: String = row.try_get("status").unwrap_or_default();
    
    Ok(Tenant {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        subdomain: row.try_get("subdomain")?,
        custom_domain: row.try_get("custom_domain")?,
        plan: serde_json::from_str(&format!("\"{}\"", plan_str)).unwrap_or(PlanTier::Free),
        status: serde_json::from_str(&format!("\"{}\"", status_str)).unwrap_or(TenantStatus::Trial),
        settings: row.try_get("settings")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
