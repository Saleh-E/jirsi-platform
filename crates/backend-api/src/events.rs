//! Event Broadcasting Helpers
//!
//! Helper functions for broadcasting real-time events to WebSocket clients.

use uuid::Uuid;

use crate::routes::ws::{broadcast_event, WsChannels, WsEvent};
use crate::state::AppState;

/// Broadcast a new message event
pub fn emit_new_message(
    state: &AppState,
    tenant_id: Uuid,
    thread_id: Uuid,
    sender_name: String,
    preview: String,
) {
    let event = WsEvent::NewMessage {
        thread_id,
        sender_name,
        preview,
    };
    broadcast_event(&state.ws_channels, tenant_id, event);
}

/// Broadcast a lead assignment event
pub fn emit_lead_assigned(
    state: &AppState,
    tenant_id: Uuid,
    contact_id: Uuid,
    contact_name: String,
    assigned_by: String,
) {
    let event = WsEvent::LeadAssigned {
        contact_id,
        contact_name,
        assigned_by,
    };
    broadcast_event(&state.ws_channels, tenant_id, event);
}

/// Broadcast an interaction created event
pub fn emit_interaction_created(
    state: &AppState,
    tenant_id: Uuid,
    entity_type: String,
    entity_id: Uuid,
    interaction_type: String,
) {
    let event = WsEvent::InteractionCreated {
        entity_type,
        entity_id,
        interaction_type,
    };
    broadcast_event(&state.ws_channels, tenant_id, event);
}

/// Broadcast a webhook received event
pub fn emit_webhook_received(
    state: &AppState,
    tenant_id: Uuid,
    provider: String,
    message: String,
) {
    let event = WsEvent::WebhookReceived { provider, message };
    broadcast_event(&state.ws_channels, tenant_id, event);
}

/// Broadcast a generic notification
pub fn emit_notification(
    state: &AppState,
    tenant_id: Uuid,
    title: String,
    message: String,
    level: String,
) {
    let event = WsEvent::Notification {
        title,
        message,
        level,
    };
    broadcast_event(&state.ws_channels, tenant_id, event);
}
