//! Session management

use chrono::{Duration, Utc};
use core_models::{Session, User, UserInfo, UserRole, AuthContext};
use sqlx::PgPool;
use uuid::Uuid;

use crate::AuthError;

/// Session service for managing user sessions
pub struct SessionService {
    pool: PgPool,
    /// Session duration (default: 7 days)
    session_duration: Duration,
}

impl SessionService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            session_duration: Duration::days(7),
        }
    }

    pub fn with_duration(pool: PgPool, duration: Duration) -> Self {
        Self {
            pool,
            session_duration: duration,
        }
    }

    /// Create a new session for a user
    pub async fn create_session(
        &self,
        user: &User,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<(Session, String), AuthError> {
        // Generate a random token
        let token = Uuid::new_v4().to_string();
        let token_hash = sha256_hash(&token);
        let now = Utc::now();
        let expires_at = now + self.session_duration;

        let session = Session {
            id: Uuid::new_v4(),
            user_id: user.id,
            tenant_id: user.tenant_id,
            token_hash: token_hash.clone(),
            user_agent: user_agent.clone(),
            ip_address: ip_address.clone(),
            expires_at,
            created_at: now,
        };

        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, tenant_id, token_hash, user_agent, ip_address, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(session.id)
        .bind(session.user_id)
        .bind(session.tenant_id)
        .bind(&session.token_hash)
        .bind(&session.user_agent)
        .bind(&session.ip_address)
        .bind(session.expires_at)
        .bind(session.created_at)
        .execute(&self.pool)
        .await?;

        Ok((session, token))
    }

    /// Validate a session token and return the auth context
    pub async fn validate_session(&self, token: &str) -> Result<AuthContext, AuthError> {
        let token_hash = sha256_hash(token);
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            SELECT 
                s.id as session_id,
                s.tenant_id,
                u.id as user_id,
                u.email,
                u.name,
                u.role,
                u.avatar_url
            FROM sessions s
            JOIN users u ON s.user_id = u.id
            WHERE s.token_hash = $1 AND s.expires_at > $2
            "#,
        )
        .bind(&token_hash)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AuthError::SessionNotFound)?;

        use sqlx::Row;
        let role_str: String = row.try_get("role").unwrap_or_default();
        
        Ok(AuthContext {
            user: UserInfo {
                id: row.try_get("user_id")?,
                tenant_id: row.try_get("tenant_id")?,
                email: row.try_get("email")?,
                name: row.try_get("name")?,
                role: serde_json::from_str(&format!("\"{}\"", role_str)).unwrap_or(UserRole::Member),
                avatar_url: row.try_get("avatar_url")?,
            },
            tenant_id: row.try_get("tenant_id")?,
            session_id: row.try_get("session_id")?,
        })
    }

    /// Extend a session's expiration
    pub async fn extend_session(&self, session_id: Uuid) -> Result<(), AuthError> {
        let expires_at = Utc::now() + self.session_duration;

        sqlx::query(
            r#"UPDATE sessions SET expires_at = $1 WHERE id = $2"#,
        )
        .bind(expires_at)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete a session (logout)
    pub async fn delete_session(&self, session_id: Uuid) -> Result<(), AuthError> {
        sqlx::query(
            r#"DELETE FROM sessions WHERE id = $1"#,
        )
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete all sessions for a user (logout everywhere)
    pub async fn delete_user_sessions(&self, user_id: Uuid) -> Result<(), AuthError> {
        sqlx::query(
            r#"DELETE FROM sessions WHERE user_id = $1"#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self) -> Result<u64, AuthError> {
        let result = sqlx::query(
            r#"DELETE FROM sessions WHERE expires_at < $1"#,
        )
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

/// Simple SHA256 hash for tokens
fn sha256_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // Note: In production, use a proper SHA256 implementation
    // This is a placeholder for development
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
