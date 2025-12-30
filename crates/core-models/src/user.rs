//! User model - Authentication and Persona management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Functional Persona - Determines the "Lens" through which the user sees the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// System Super Admin
    Admin,
    /// Organization Manager
    Manager,
    /// Standard Internal User
    Member,
    /// Real Estate Agent (Can manage listings, contracts)
    Agent,
    /// Broker (Manages multiple agents)
    Broker,
    /// Landlord (Property Owner View)
    Landlord,
    /// Tenant/Buyer (Portal View)
    Tenant,
    /// External Vendor (Contractor, Photographer)
    Vendor,
    /// Read-only access
    Viewer,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Member
    }
}

impl UserRole {
    /// Returns the entities this role can access
    pub fn accessible_entities(&self) -> Vec<&'static str> {
        match self {
            Self::Admin | Self::Manager => vec!["*"], // All entities
            Self::Agent => vec!["contact", "property", "deal", "contract", "viewing", "task", "requirement"],
            Self::Broker => vec!["contact", "property", "deal", "contract", "agent", "report", "commission"],
            Self::Landlord => vec!["property", "contract", "tenant", "maintenance_request", "payment"],
            Self::Tenant => vec!["contract", "payment", "maintenance_request", "viewing"],
            Self::Vendor => vec!["task", "property", "work_order"],
            Self::Member => vec!["contact", "deal", "task"],
            Self::Viewer => vec![], // Read-only, determined by view permissions
        }
    }
    
    /// UI sidebar sections visible to this role
    pub fn sidebar_sections(&self) -> Vec<&'static str> {
        match self {
            Self::Admin => vec!["dashboard", "crm", "real_estate", "reports", "settings", "marketplace"],
            Self::Manager => vec!["dashboard", "crm", "real_estate", "reports"],
            Self::Agent => vec!["dashboard", "clients", "properties", "deals", "calendar", "dialer"],
            Self::Broker => vec!["dashboard", "agents", "properties", "deals", "commissions", "reports"],
            Self::Landlord => vec!["dashboard", "my_properties", "contracts", "financials", "maintenance"],
            Self::Tenant => vec!["my_home", "payments", "requests", "documents"],
            Self::Vendor => vec!["work_orders", "schedule", "invoices"],
            Self::Member => vec!["dashboard", "contacts", "tasks"],
            Self::Viewer => vec!["dashboard"],
        }
    }
    
    /// Check if this role has dashboard access
    pub fn has_dashboard(&self) -> bool {
        !matches!(self, Self::Viewer)
    }
    
    /// Check if this role can access the AI dialer
    pub fn has_dialer(&self) -> bool {
        matches!(self, Self::Admin | Self::Agent | Self::Broker)
    }
    
    /// Check if this role can manage payments
    pub fn can_manage_payments(&self) -> bool {
        matches!(self, Self::Admin | Self::Manager | Self::Landlord | Self::Broker)
    }
}

/// User account status with Marketplace context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    /// Invitation sent, not yet accepted
    Pending,
    /// Active user
    Active,
    /// Temporarily disabled
    Disabled,
    /// Users created via Marketplace apps (e.g., "CRM Plugin User")
    AppManaged(String),
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// A user within a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub name: String,
    /// Argon2 hashed password
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub status: UserStatus,
    /// Avatar URL
    pub avatar_url: Option<String>,
    /// Extended preferences for Marketplace App settings
    pub preferences: serde_json::Value,
    /// Verification level for Agents/Landlords (KYC)
    /// 0 = Unverified, 1 = Email Verified, 2 = ID Verified, 3 = Fully Verified
    pub verification_level: i32,
    /// Phone number for SMS/WhatsApp
    pub phone: Option<String>,
    /// Stripe Connect account ID (for receiving payments)
    pub stripe_account_id: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User data for creating a new user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub tenant_id: Uuid,
    pub email: String,
    pub name: String,
    pub password: String,
    pub role: UserRole,
    pub phone: Option<String>,
}

/// User data for authentication responses (no sensitive fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub avatar_url: Option<String>,
    pub verification_level: i32,
    pub phone: Option<String>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            tenant_id: user.tenant_id,
            email: user.email,
            name: user.name,
            role: user.role,
            avatar_url: user.avatar_url,
            verification_level: user.verification_level,
            phone: user.phone,
        }
    }
}

/// Team within a tenant for organizing users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User membership in a team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTeam {
    pub user_id: Uuid,
    pub team_id: Uuid,
    pub is_leader: bool,
    pub joined_at: DateTime<Utc>,
}

/// Session for authenticated users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub token_hash: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Current authenticated context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user: UserInfo,
    pub tenant_id: Uuid,
    pub session_id: Uuid,
}
