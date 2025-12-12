//! User management

use chrono::Utc;
use core_models::{CreateUser, User, UserInfo, UserRole, UserStatus};
use sqlx::PgPool;
use uuid::Uuid;

use crate::password::{hash_password, verify_password, validate_password_strength};
use crate::AuthError;

/// User service for user management
pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get user by email within a tenant
    pub async fn get_by_email(&self, tenant_id: Uuid, email: &str) -> Result<User, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, email, name, password_hash,
                role, status,
                avatar_url, preferences,
                last_login_at, created_at, updated_at
            FROM users
            WHERE tenant_id = $1 AND email = $2
            "#,
        )
        .bind(tenant_id)
        .bind(email)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AuthError::UserNotFound)?;

        user_from_row(&row)
    }

    /// Get user by ID
    pub async fn get_by_id(&self, tenant_id: Uuid, user_id: Uuid) -> Result<User, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, email, name, password_hash,
                role, status,
                avatar_url, preferences,
                last_login_at, created_at, updated_at
            FROM users
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AuthError::UserNotFound)?;

        user_from_row(&row)
    }
    
    /// Create a new user
    pub async fn create(&self, input: CreateUser) -> Result<User, AuthError> {
        // Validate password strength
        validate_password_strength(&input.password)?;

        // Check for existing user
        let existing_row = sqlx::query(
            r#"SELECT COUNT(*) as count FROM users WHERE tenant_id = $1 AND email = $2"#,
        )
        .bind(input.tenant_id)
        .bind(&input.email)
        .fetch_one(&self.pool)
        .await?;

        use sqlx::Row;
        let existing: i64 = existing_row.try_get("count").unwrap_or(0);

        if existing > 0 {
            return Err(AuthError::UserExists(input.email));
        }

        // Hash password
        let password_hash = hash_password(&input.password)?;
        let now = Utc::now();

        let user = User {
            id: Uuid::new_v4(),
            tenant_id: input.tenant_id,
            email: input.email,
            name: input.name,
            password_hash,
            role: input.role,
            status: UserStatus::Active,
            avatar_url: None,
            preferences: serde_json::json!({}),
            last_login_at: None,
            created_at: now,
            updated_at: now,
        };

        sqlx::query(
            r#"
            INSERT INTO users (id, tenant_id, email, name, password_hash, role, status, avatar_url, preferences, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(user.id)
        .bind(user.tenant_id)
        .bind(&user.email)
        .bind(&user.name)
        .bind(&user.password_hash)
        .bind(serde_json::to_string(&user.role).unwrap_or_default())
        .bind(serde_json::to_string(&user.status).unwrap_or_default())
        .bind(&user.avatar_url)
        .bind(&user.preferences)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(user)
    }

    /// Authenticate user with email and password
    pub async fn authenticate(
        &self,
        tenant_id: Uuid,
        email: &str,
        password: &str,
    ) -> Result<User, AuthError> {
        let user = self.get_by_email(tenant_id, email).await?;

        // Check account status
        if user.status == UserStatus::Disabled {
            return Err(AuthError::AccountDisabled);
        }

        // Verify password
        if !verify_password(password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Update last login
        sqlx::query(
            r#"UPDATE users SET last_login_at = $1 WHERE id = $2"#,
        )
        .bind(Utc::now())
        .bind(user.id)
        .execute(&self.pool)
        .await?;

        Ok(user)
    }

    /// List all users in a tenant
    pub async fn list(&self, tenant_id: Uuid) -> Result<Vec<UserInfo>, AuthError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, email, name,
                role, avatar_url
            FROM users
            WHERE tenant_id = $1
            ORDER BY name
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(user_info_from_row).collect()
    }

    /// Update user role
    pub async fn update_role(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        role: UserRole,
    ) -> Result<(), AuthError> {
        sqlx::query(
            r#"UPDATE users SET role = $1, updated_at = $2 WHERE tenant_id = $3 AND id = $4"#,
        )
        .bind(serde_json::to_string(&role).unwrap_or_default())
        .bind(Utc::now())
        .bind(tenant_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Disable a user
    pub async fn disable(&self, tenant_id: Uuid, user_id: Uuid) -> Result<(), AuthError> {
        sqlx::query(
            r#"UPDATE users SET status = $1, updated_at = $2 WHERE tenant_id = $3 AND id = $4"#,
        )
        .bind(serde_json::to_string(&UserStatus::Disabled).unwrap_or_default())
        .bind(Utc::now())
        .bind(tenant_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

fn user_from_row(row: &sqlx::postgres::PgRow) -> Result<User, AuthError> {
    use sqlx::Row;
    
    let role_str: String = row.try_get("role").unwrap_or_default();
    let status_str: String = row.try_get("status").unwrap_or_default();
    
    Ok(User {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        email: row.try_get("email")?,
        name: row.try_get("name")?,
        password_hash: row.try_get("password_hash")?,
        role: serde_json::from_str(&format!("\"{}\"", role_str)).unwrap_or(UserRole::Member),
        status: serde_json::from_str(&format!("\"{}\"", status_str)).unwrap_or(UserStatus::Active),
        avatar_url: row.try_get("avatar_url")?,
        preferences: row.try_get("preferences")?,
        last_login_at: row.try_get("last_login_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn user_info_from_row(row: &sqlx::postgres::PgRow) -> Result<UserInfo, AuthError> {
    use sqlx::Row;
    
    let role_str: String = row.try_get("role").unwrap_or_default();
    
    Ok(UserInfo {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        email: row.try_get("email")?,
        name: row.try_get("name")?,
        role: serde_json::from_str(&format!("\"{}\"", role_str)).unwrap_or(UserRole::Member),
        avatar_url: row.try_get("avatar_url")?,
    })
}
