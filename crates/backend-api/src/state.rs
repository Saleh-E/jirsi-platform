//! Application state

use core_auth::{session::SessionService, tenant::TenantService, user::UserService};
use core_metadata::MetadataService;
use sqlx::PgPool;

use crate::routes::ws::{create_ws_channels, WsChannels};

use core_node_engine::ai::AiService;
use std::sync::Arc;
use crate::ai::service::create_ai_service;

pub struct AppState {
    pub pool: PgPool,
    pub metadata: MetadataService,
    pub tenant_service: TenantService,
    pub user_service: UserService,
    pub session_service: SessionService,
    pub ws_channels: WsChannels,
    pub ai_service: Arc<dyn AiService>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            metadata: MetadataService::new(pool.clone()),
            tenant_service: TenantService::new(pool.clone()),
            user_service: UserService::new(pool.clone()),
            session_service: SessionService::new(pool.clone()),
            ws_channels: create_ws_channels(),
            ai_service: create_ai_service(),
            pool,
        }
    }
}
