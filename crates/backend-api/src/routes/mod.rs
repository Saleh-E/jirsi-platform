//! API routes

use axum::Router;
use std::sync::Arc;

use crate::state::AppState;

pub mod auth;
pub mod entities;
pub mod metadata;
pub mod associations;
pub mod interactions;
pub mod tasks;
pub mod views;
pub mod properties;
pub mod workflows;
pub mod workflow_graph;
pub mod public;
pub mod analytics;
pub mod inbox;
pub mod tenant;
pub mod webhooks;
pub mod integrations;
pub mod ws;

/// Build all API routes
pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Auth routes (public - login, register, logout, register-tenant)
        .nest("/auth", auth::routes())
        // Tenant settings routes
        .nest("/tenant", tenant::routes())
        // Metadata routes (authentication enforced via extractors in handlers)
        .nest("/metadata", metadata::routes())
        // Entity CRUD routes (authentication enforced via extractors in handlers)
        .nest("/entities", entities::routes())
        // Association routes (linking records together)
        .nest("/associations", associations::routes())
        // Interactions routes (timeline/activities)
        .nest("/interactions", interactions::routes())
        // Tasks routes (entity-linked tasks)
        .nest("/tasks", tasks::routes())
        // Views routes (saved user views)
        .nest("/views", views::routes())
        // Properties routes (Phase 3 - real estate)
        .merge(properties::router())
        // Analytics routes (dashboard)
        .nest("/analytics", analytics::routes())
        // Inbox routes (unified messaging)
        .nest("/inbox", inbox::routes())
        // Workflow graph routes (visual editor)
        .nest("/workflows", workflow_graph::routes())
        // Integration settings routes
        .merge(integrations::routes())
}


