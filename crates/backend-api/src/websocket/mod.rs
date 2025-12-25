//! WebSocket Server - Real-time Updates
//! 
//! Broadcasts events to connected clients in real-time

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State, Path,
    },
    response::Response,
    Router,
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;
use futures::{sink::SinkExt, stream::StreamExt};

use crate::state::AppState;

/// Connected client info
#[derive(Debug, Clone)]
pub struct ConnectedClient {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub tx: tokio::sync::mpsc::UnboundedSender<Message>,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Entity updated
    EntityUpdated {
        entity_id: Uuid,
        entity_type: String,
        field_values: serde_json::Value,
        updated_by: Uuid,
    },
    
    /// Entity created
    EntityCreated {
        entity_id: Uuid,
        entity_type: String,
        field_values: serde_json::Value,
        created_by: Uuid,
    },
    
    /// Entity deleted
    EntityDeleted {
        entity_id: Uuid,
        entity_type: String,
        deleted_by: Uuid,
    },
    
    /// User presence update
    Presence {
        user_id: Uuid,
        status: PresenceStatus,
        location: Option<String>,
    },
    
    /// Cursor position (for collaborative editing)
    Cursor {
        user_id: Uuid,
        entity_id: Uuid,
        field: String,
        position: CursorPosition,
    },
    
    /// CRDT update
    CrdtUpdate {
        entity_id: Uuid,
        field: String,
        update: Vec<u8>,
    },
    
    /// Ping/Pong for keep-alive
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceStatus {
    Online,
    Away,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: u32,
    pub column: u32,
}

/// WebSocket connection manager
pub struct WsManager {
    clients: Arc<RwLock<HashMap<Uuid, ConnectedClient>>>,
}

impl WsManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add a new client
    pub async fn add_client(&self, client_id: Uuid, client: ConnectedClient) {
        self.clients.write().await.insert(client_id, client);
    }
    
    /// Remove a client
    pub async fn remove_client(&self, client_id: &Uuid) {
        self.clients.write().await.remove(client_id);
    }
    
    /// Broadcast message to all clients in tenant
    pub async fn broadcast_to_tenant(&self, tenant_id: Uuid, message: WsMessage) {
        let clients = self.clients.read().await;
        
        let msg_json = serde_json::to_string(&message).unwrap();
        let ws_msg = Message::Text(msg_json);
        
        for client in clients.values() {
            if client.tenant_id == tenant_id {
                let _ = client.tx.send(ws_msg.clone());
            }
        }
    }
    
    /// Send message to specific client
    pub async fn send_to_client(&self, client_id: &Uuid, message: WsMessage) {
        let clients = self.clients.read().await;
        
        if let Some(client) = clients.get(client_id) {
            let msg_json = serde_json::to_string(&message).unwrap();
            let ws_msg = Message::Text(msg_json);
            let _ = client.tx.send(ws_msg);
        }
    }
    
    /// Get online users count for tenant
    pub async fn get_online_count(&self, tenant_id: Uuid) -> usize {
        let clients = self.clients.read().await;
        clients.values()
            .filter(|c| c.tenant_id == tenant_id)
            .count()
    }
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(ws_manager): State<Arc<WsManager>>,
    Path((tenant_id, user_id)): Path<(Uuid, Uuid)>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, ws_manager, tenant_id, user_id))
}

/// Handle WebSocket connection
async fn handle_socket(
    socket: WebSocket,
    ws_manager: Arc<WsManager>,
    tenant_id: Uuid,
    user_id: Uuid,
) {
    let (mut sender, mut receiver) = socket.split();
    
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    
    let client_id = Uuid::new_v4();
    let client = ConnectedClient {
        user_id,
        tenant_id,
        tx,
    };
    
    // Add client
    ws_manager.add_client(client_id, client).await;
    
    // Send presence update
    ws_manager.broadcast_to_tenant(
        tenant_id,
        WsMessage::Presence {
            user_id,
            status: PresenceStatus::Online,
            location: None,
        },
    ).await;
    
    // Spawn task to send messages to client
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });
    
    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    handle_message(ws_msg, &ws_manager, tenant_id, user_id).await;
                }
            }
            Message::Close(_) => {
                break;
            }
            Message::Ping(_) => {
                // axum handles pong automatically
            }
            _ => {}
        }
    }
    
    // Client disconnected
    ws_manager.remove_client(&client_id).await;
    
    // Send offline presence
    ws_manager.broadcast_to_tenant(
        tenant_id,
        WsMessage::Presence {
            user_id,
            status: PresenceStatus::Offline,
            location: None,
        },
    ).await;
    
    send_task.abort();
}

/// Handle incoming WebSocket message
async fn handle_message(
    msg: WsMessage,
    ws_manager: &Arc<WsManager>,
    tenant_id: Uuid,
    user_id: Uuid,
) {
    match msg {
        WsMessage::Presence { location, .. } => {
            // Broadcast presence to others
            ws_manager.broadcast_to_tenant(
                tenant_id,
                WsMessage::Presence {
                    user_id,
                    status: PresenceStatus::Online,
                    location,
                },
            ).await;
        }
        
        WsMessage::Cursor { entity_id, field, position, user_id: _ } => {
            // Broadcast cursor position to others
            ws_manager.broadcast_to_tenant(
                tenant_id,
                WsMessage::Cursor {
                    user_id,
                    entity_id,
                    field,
                    position,
                },
            ).await;
        }
        
        WsMessage::CrdtUpdate { entity_id, field, update } => {
            // Broadcast CRDT update
            ws_manager.broadcast_to_tenant(
                tenant_id,
                WsMessage::CrdtUpdate {
                    entity_id,
                    field,
                    update,
                },
            ).await;
        }
        
        WsMessage::Ping => {
            // Respond with pong (handled by client)
        }
        
        _ => {
            // Other messages are server->client only
        }
    }
}

/// WebSocket routes
pub fn ws_routes(ws_manager: Arc<WsManager>) -> Router<AppState> {
    Router::new()
        .route("/ws/:tenant_id/:user_id", get(ws_handler))
        .with_state(ws_manager)
}
