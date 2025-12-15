//! WebSocket Handler - Real-time event broadcasting
//!
//! Provides WebSocket endpoint for pushing real-time events to connected clients.
//! Tenant isolation is enforced via JWT token from query parameter.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// JWT token for authentication
    pub token: String,
}

/// WebSocket event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WsEvent {
    /// New message received
    NewMessage {
        thread_id: Uuid,
        sender_name: String,
        preview: String,
    },
    /// Lead assigned to user
    LeadAssigned {
        contact_id: Uuid,
        contact_name: String,
        assigned_by: String,
    },
    /// Interaction created (email, call, note)
    InteractionCreated {
        entity_type: String,
        entity_id: Uuid,
        interaction_type: String,
    },
    /// Webhook received
    WebhookReceived {
        provider: String,
        message: String,
    },
    /// Generic notification
    Notification {
        title: String,
        message: String,
        level: String, // info, success, warning, error
    },
    /// Connection established (sent on connect)
    Connected {
        user_id: Uuid,
        tenant_id: Uuid,
    },
}

/// WebSocket channel manager
/// Maps tenant_id to broadcast sender
pub type WsChannels = Arc<DashMap<Uuid, broadcast::Sender<WsEvent>>>;

/// Create a new WebSocket channel manager
pub fn create_ws_channels() -> WsChannels {
    Arc::new(DashMap::new())
}

/// Create WebSocket routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/ws", get(ws_handler))
}

/// WebSocket upgrade handler
async fn ws_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Verify token using session service
    let auth_context = match state.session_service.validate_session(&query.token).await {
        Ok(ctx) => ctx,
        Err(e) => {
            warn!(error = %e, "WebSocket auth failed");
            return (axum::http::StatusCode::UNAUTHORIZED, "Invalid token").into_response();
        }
    };

    info!(
        user_id = %auth_context.user.id,
        tenant_id = %auth_context.tenant_id,
        "WebSocket connection accepted"
    );

    let user_id = auth_context.user.id;
    let tenant_id = auth_context.tenant_id;

    ws.on_upgrade(move |socket| handle_socket(socket, state, user_id, tenant_id))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>, user_id: Uuid, tenant_id: Uuid) {
    // Get or create broadcast channel for this tenant
    let rx = {
        let channels = &state.ws_channels;
        
        let sender = channels.entry(tenant_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(100);
            tx
        });
        
        sender.subscribe()
    };

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Send connection confirmed event
    let connected_event = WsEvent::Connected { user_id, tenant_id };
    if let Ok(json) = serde_json::to_string(&connected_event) {
        let _ = ws_sender.send(Message::Text(json)).await;
    }

    // Spawn task to forward broadcast events to this client
    let mut rx = rx;
    let forward_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match serde_json::to_string(&event) {
                Ok(json) => {
                    if ws_sender.send(Message::Text(json)).await.is_err() {
                        break; // Client disconnected
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to serialize WebSocket event");
                }
            }
        }
    });

    // Handle incoming messages (ping/pong, client events)
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Ping(data)) => {
                info!(tenant_id = %tenant_id, "WebSocket ping received");
                // Pong is automatically sent by axum
                let _ = data;
            }
            Ok(Message::Close(_)) => {
                info!(tenant_id = %tenant_id, user_id = %user_id, "WebSocket closed by client");
                break;
            }
            Ok(Message::Text(text)) => {
                // Handle client messages (if any)
                info!(tenant_id = %tenant_id, "WebSocket message: {}", text);
            }
            Err(e) => {
                error!(tenant_id = %tenant_id, error = %e, "WebSocket error");
                break;
            }
            _ => {}
        }
    }

    // Cleanup
    forward_task.abort();
    info!(tenant_id = %tenant_id, user_id = %user_id, "WebSocket connection closed");
}

/// Broadcast an event to all clients of a tenant
pub fn broadcast_event(channels: &WsChannels, tenant_id: Uuid, event: WsEvent) {
    if let Some(sender) = channels.get(&tenant_id) {
        match sender.send(event) {
            Ok(count) => {
                info!(tenant_id = %tenant_id, receivers = count, "Event broadcast");
            }
            Err(_) => {
                // No receivers - channel is empty
            }
        }
    }
}
