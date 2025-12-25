//! Application state

use core_auth::{session::SessionService, tenant::TenantService, user::UserService};
use core_metadata::MetadataService;
use sqlx::PgPool;

use crate::routes::ws::{create_ws_channels, WsChannels};

use core_node_engine::{ai::AiService, EventPublisher, GraphExecutor, repository::NodeGraphRepository};
use std::sync::Arc;
use crate::ai::service::create_ai_service;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub metadata: MetadataService,
    pub tenant_service: TenantService,
    pub user_service: UserService,
    pub session_service: SessionService,
    pub ws_channels: WsChannels,
    pub ai_service: Arc<dyn AiService>,
    pub event_publisher: EventPublisher,
    pub graph_executor: Arc<GraphExecutor>,
    pub graph_repo: NodeGraphRepository,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let ai_service = create_ai_service();
        
        // Init workflow engine components
        let event_publisher = EventPublisher::new(pool.clone());
        let graph_repo = NodeGraphRepository::new(pool.clone());
        let graph_executor = Arc::new(GraphExecutor::new(pool.clone()).with_ai_service(ai_service.clone()));

        Self {
            metadata: MetadataService::new(pool.clone()),
            tenant_service: TenantService::new(pool.clone()),
            user_service: UserService::new(pool.clone()),
            session_service: SessionService::new(pool.clone()),
            ws_channels: create_ws_channels(),
            ai_service,
            event_publisher,
            graph_executor,
            graph_repo,
            pool,
        }
    }
}
