//! Application state

use core_auth::{session::SessionService, tenant::TenantService, user::UserService};
use core_metadata::MetadataService;
use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
    pub metadata: MetadataService,
    pub tenant_service: TenantService,
    pub user_service: UserService,
    pub session_service: SessionService,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            metadata: MetadataService::new(pool.clone()),
            tenant_service: TenantService::new(pool.clone()),
            user_service: UserService::new(pool.clone()),
            session_service: SessionService::new(pool.clone()),
            pool,
        }
    }
}
