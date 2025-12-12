//! User model - Authentication and user management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User role within a tenant
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// Full access to everything
    Admin,
    /// Can manage most things except billing/settings
    Manager,
    /// Regular user with standard permissions
    Member,
    /// Limited read-only access
    Viewer,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Member
    }
}

/// User account status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    /// Invitation sent, not yet accepted
    Pending,
    /// Active user
    Active,
    /// Temporarily disabled
    Disabled,
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
    /// User preferences stored as JSON
    pub preferences: serde_json::Value,
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
