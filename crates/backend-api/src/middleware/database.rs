use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use std::sync::Arc;

use crate::state::AppState;
use crate::middleware::tenant::ResolvedTenant;
use crate::error::ApiError;

/// Robust RLS Connection Extractor
/// 
/// Acquires a database connection and automatically sets the `app.current_tenant`
/// environment variable based on the resolved tenant. This ensures RLS is
/// always active for the lifespan of this connection.
pub struct RlsConn(pub PoolConnection<Postgres>);

#[async_trait]
impl<S> FromRequestParts<S> for RlsConn
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::from_ref(state);
        
        // 1. Get resolved tenant from extensions (must be set by resolve_tenant middleware)
        let tenant = parts
            .extensions
            .get::<ResolvedTenant>()
            .ok_or_else(|| ApiError::Internal("Tenant not resolved. Ensure resolve_tenant middleware is applied.".to_string()))?;

        // 2. Acquire connection
        let mut conn = app_state.pool.acquire().await
            .map_err(|e| ApiError::Database(e))?;

        // 3. Set RLS context
        sqlx::query("SELECT set_config('app.current_tenant', $1::text, false)")
            .bind(tenant.id)
            .execute(&mut *conn)
            .await
            .map_err(|e| ApiError::Database(e))?;

        Ok(RlsConn(conn))
    }
}

// Implement Deref/DerefMut for seamless access to the underlying connection
impl std::ops::Deref for RlsConn {
    type Target = PoolConnection<Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RlsConn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
