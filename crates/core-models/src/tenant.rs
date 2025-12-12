//! Tenant model - Multi-tenant isolation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of a tenant account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantStatus {
    /// Active and operational
    Active,
    /// Trial period
    Trial,
    /// Temporarily suspended (e.g., payment issues)
    Suspended,
    /// Cancelled/deleted
    Cancelled,
}

impl Default for TenantStatus {
    fn default() -> Self {
        Self::Trial
    }
}

/// Subscription plan tier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanTier {
    Free,
    Starter,
    Professional,
    Enterprise,
}

impl Default for PlanTier {
    fn default() -> Self {
        Self::Free
    }
}

/// A tenant represents a single organization/company using the platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    /// Display name of the organization
    pub name: String,
    /// Subdomain (e.g., "acme" for acme.platform.com)
    pub subdomain: String,
    /// Optional custom domain (e.g., "crm.acme.com")
    pub custom_domain: Option<String>,
    /// Current subscription plan
    pub plan: PlanTier,
    /// Account status
    pub status: TenantStatus,
    /// Settings stored as JSON
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tenant {
    pub fn new(name: String, subdomain: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            subdomain,
            custom_domain: None,
            plan: PlanTier::default(),
            status: TenantStatus::default(),
            settings: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, TenantStatus::Active | TenantStatus::Trial)
    }
}

/// Lightweight tenant context passed through requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub subdomain: String,
    pub plan: PlanTier,
}
