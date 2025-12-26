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
    
    // ============ CRDT Sync Events ============
    
    /// Subscribe to a document room (for CRDT sync)
    DocumentSubscribe {
        document_id: String,  // format: "entity_id:field_name"
    },
    /// Unsubscribe from a document room
    DocumentUnsubscribe {
        document_id: String,
    },
    /// CRDT state vector request (for delta sync)
    DocumentSyncRequest {
        document_id: String,
        /// Base64-encoded state vector
        state_vector: String,
    },
    /// CRDT update (Yjs binary, base64-encoded)
    DocumentUpdate {
        document_id: String,
        /// Base64-encoded Yjs update
        update: String,
        /// User who made the change
        user_id: Uuid,
    },
    /// Full document state (for initial sync)
    DocumentState {
        document_id: String,
        /// Base64-encoded full state
        state: String,
    },
    /// Awareness update (cursor position, selection, user info)
    AwarenessUpdate {
        document_id: String,
        user_id: Uuid,
        user_name: String,
        user_color: String,
        cursor_position: Option<u32>,
        selection_start: Option<u32>,
        selection_end: Option<u32>,
    },
    /// User left document (remove their cursor)
    AwarenessRemove {
        document_id: String,
        user_id: Uuid,
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
                // Parse and handle client events
                match serde_json::from_str::<WsEvent>(&text) {
                    Ok(event) => {
                        handle_client_event(&state, tenant_id, user_id, event).await;
                    }
                    Err(e) => {
                        warn!(tenant_id = %tenant_id, error = %e, "Failed to parse WebSocket message");
                    }
                }
            }
            Ok(Message::Binary(data)) => {
                // Handle binary CRDT updates directly (more efficient)
                info!(tenant_id = %tenant_id, size = data.len(), "Binary CRDT update received");
                // TODO: Parse binary format: [doc_id_len(u8)][doc_id][update]
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

/// Handle events sent by clients (CRDT updates, subscriptions, etc.)
async fn handle_client_event(state: &Arc<AppState>, tenant_id: Uuid, user_id: Uuid, event: WsEvent) {
    match event {
        WsEvent::DocumentSubscribe { document_id } => {
            info!(
                tenant_id = %tenant_id,
                user_id = %user_id,
                document_id = %document_id,
                "Client subscribed to document"
            );
            // Track subscription in state (for targeted broadcasts)
            // In production, maintain a map of document_id -> Vec<user_id>
        }
        
        WsEvent::DocumentUnsubscribe { document_id } => {
            info!(
                tenant_id = %tenant_id,
                user_id = %user_id,
                document_id = %document_id,
                "Client unsubscribed from document"
            );
            // Broadcast awareness remove to other clients
            let remove_event = WsEvent::AwarenessRemove {
                document_id,
                user_id,
            };
            broadcast_event(&state.ws_channels, tenant_id, remove_event);
        }
        
        WsEvent::DocumentUpdate { document_id, update, user_id: sender_id } => {
            info!(
                tenant_id = %tenant_id,
                document_id = %document_id,
                update_size = update.len(),
                "CRDT update received - broadcasting to peers"
            );
            
            // Broadcast to all clients in the same tenant
            // In production, filter to only clients subscribed to this document
            let broadcast_event_data = WsEvent::DocumentUpdate {
                document_id,
                update,
                user_id: sender_id,
            };
            broadcast_event(&state.ws_channels, tenant_id, broadcast_event_data);
        }
        
        WsEvent::AwarenessUpdate { 
            document_id, 
            user_id: awareness_user_id,
            user_name,
            user_color,
            cursor_position,
            selection_start,
            selection_end,
        } => {
            // Broadcast awareness to all clients viewing the same document
            let awareness_event = WsEvent::AwarenessUpdate {
                document_id,
                user_id: awareness_user_id,
                user_name,
                user_color,
                cursor_position,
                selection_start,
                selection_end,
            };
            broadcast_event(&state.ws_channels, tenant_id, awareness_event);
        }
        
        WsEvent::DocumentSyncRequest { document_id, state_vector } => {
            info!(
                tenant_id = %tenant_id,
                document_id = %document_id,
                "Sync request received - delta sync not yet implemented server-side"
            );
            // In production: Load document from DB, compute delta, send DocumentState
            let _ = state_vector; // Silence unused warning
        }
        
        // Other events don't require server-side handling
        _ => {}
    }
}

