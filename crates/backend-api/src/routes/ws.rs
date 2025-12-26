//! WebSocket Handler - Real-time CRDT Collaboration & Event Broadcasting
//!
//! Provides WebSocket endpoint for:
//! - Real-time CRDT document synchronization (Yjs protocol)
//! - Document rooms for targeted broadcasts
//! - Presence/awareness for live cursors
//! - General event broadcasting (notifications, etc.)

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use base64::{engine::general_purpose::STANDARD as Base64, Engine};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info, warn};
use uuid::Uuid;
use yrs::{Doc, GetString, ReadTxn, StateVector, Transact, Update};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;

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
    /// Interaction created
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
        level: String,
    },
    /// Connection established
    Connected {
        user_id: Uuid,
        tenant_id: Uuid,
    },
    
    // ============ CRDT Sync Events ============
    
    /// Subscribe to a document room
    DocumentSubscribe {
        document_id: String,
    },
    /// Unsubscribe from a document room
    DocumentUnsubscribe {
        document_id: String,
    },
    /// CRDT state vector request
    DocumentSyncRequest {
        document_id: String,
        state_vector: String,
    },
    /// CRDT update (Yjs binary, base64-encoded)
    DocumentUpdate {
        document_id: String,
        update: String,
        user_id: Uuid,
    },
    /// Full document state
    DocumentState {
        document_id: String,
        state: String,
        state_vector: String,
    },
    /// Awareness update
    AwarenessUpdate {
        document_id: String,
        user_id: Uuid,
        user_name: String,
        user_color: String,
        cursor_position: Option<u32>,
        selection_start: Option<u32>,
        selection_end: Option<u32>,
    },
    /// User left document
    AwarenessRemove {
        document_id: String,
        user_id: Uuid,
    },
}

/// Document room - tracks CRDT state
pub struct DocumentRoom {
    pub doc: Doc,
    pub users: HashSet<Uuid>,
}

impl DocumentRoom {
    pub fn new() -> Self {
        Self {
            doc: Doc::new(),
            users: HashSet::new(),
        }
    }
    
    pub fn get_state_base64(&self) -> String {
        let txn = self.doc.transact();
        let state = txn.encode_state_as_update_v1(&StateVector::default());
        Base64.encode(&state)
    }
    
    pub fn get_state_vector_base64(&self) -> String {
        let txn = self.doc.transact();
        Base64.encode(&txn.state_vector().encode_v1())
    }
    
    pub fn apply_update_base64(&mut self, update_b64: &str) -> Result<(), String> {
        let bytes = Base64.decode(update_b64)
            .map_err(|e| format!("Base64 decode error: {}", e))?;
        
        let update = Update::decode_v1(&bytes)
            .map_err(|e| format!("Yjs decode error: {:?}", e))?;
        
        let mut txn = self.doc.transact_mut();
        txn.apply_update(update);
        Ok(())
    }
    
    pub fn get_delta_base64(&self, state_vector_b64: &str) -> Result<String, String> {
        let sv_bytes = Base64.decode(state_vector_b64)
            .map_err(|e| format!("Base64 decode error: {}", e))?;
        
        let sv = StateVector::decode_v1(&sv_bytes)
            .map_err(|e| format!("State vector decode error: {:?}", e))?;
        
        let txn = self.doc.transact();
        let update = txn.encode_state_as_update_v1(&sv);
        Ok(Base64.encode(&update))
    }
}

impl Default for DocumentRoom {
    fn default() -> Self {
        Self::new()
    }
}

/// Document rooms manager
pub type DocumentRooms = Arc<DashMap<String, Arc<Mutex<DocumentRoom>>>>;

/// Create document rooms manager
pub fn create_document_rooms() -> DocumentRooms {
    Arc::new(DashMap::new())
}

/// WebSocket channel manager
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
    let auth_context = match state.session_service.validate_session(&query.token).await {
        Ok(ctx) => ctx,
        Err(e) => {
            warn!(error = %e, "WebSocket auth failed");
            return (axum::http::StatusCode::UNAUTHORIZED, "Invalid token").into_response();
        }
    };

    let user_id = auth_context.user.id;
    let user_name = auth_context.user.name.clone();
    let tenant_id = auth_context.tenant_id;

    info!(user_id = %user_id, tenant_id = %tenant_id, "WebSocket connection accepted");

    ws.on_upgrade(move |socket| handle_socket(socket, state, user_id, user_name, tenant_id))
}

/// Handle individual WebSocket connection
async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    user_id: Uuid,
    user_name: String,
    tenant_id: Uuid
) {
    // Get or create broadcast channel
    let rx = {
        let sender = state.ws_channels.entry(tenant_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(100);
            tx
        });
        sender.subscribe()
    };

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let user_documents: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    // Send connection confirmed
    let connected_event = WsEvent::Connected { user_id, tenant_id };
    if let Ok(json) = serde_json::to_string(&connected_event) {
        let _ = ws_sender.send(Message::Text(json)).await;
    }

    let user_docs_for_broadcast = Arc::clone(&user_documents);
    
    // Forward broadcast events
    let mut rx = rx;
    let forward_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let should_send = match &event {
                WsEvent::DocumentUpdate { document_id, user_id: sender_id, .. } => {
                    if *sender_id == user_id {
                        false
                    } else {
                        let docs = user_docs_for_broadcast.lock().await;
                        docs.contains(document_id)
                    }
                }
                WsEvent::AwarenessUpdate { document_id, user_id: sender_id, .. } => {
                    if *sender_id == user_id {
                        false
                    } else {
                        let docs = user_docs_for_broadcast.lock().await;
                        docs.contains(document_id)
                    }
                }
                WsEvent::AwarenessRemove { document_id, .. } => {
                    let docs = user_docs_for_broadcast.lock().await;
                    docs.contains(document_id)
                }
                WsEvent::DocumentState { .. } => false,
                _ => true,
            };
            
            if should_send {
                if let Ok(json) = serde_json::to_string(&event) {
                    if ws_sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Close(_)) => break,
            Ok(Message::Text(text)) => {
                if let Ok(event) = serde_json::from_str::<WsEvent>(&text) {
                    handle_client_event(&state, tenant_id, user_id, &user_name, event, &user_documents).await;
                }
            }
            Ok(Message::Binary(data)) => {
                // Binary CRDT update format: [doc_id_len: u8][doc_id][update]
                if data.len() > 1 {
                    let doc_id_len = data[0] as usize;
                    if data.len() > 1 + doc_id_len {
                        if let Ok(doc_id) = String::from_utf8(data[1..1+doc_id_len].to_vec()) {
                            let update_b64 = Base64.encode(&data[1+doc_id_len..]);
                            let event = WsEvent::DocumentUpdate {
                                document_id: doc_id,
                                update: update_b64,
                                user_id,
                            };
                            handle_client_event(&state, tenant_id, user_id, &user_name, event, &user_documents).await;
                        }
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "WebSocket error");
                break;
            }
            _ => {}
        }
    }

    // Cleanup
    let docs = user_documents.lock().await;
    for doc_id in docs.iter() {
        if let Some(rooms) = state.document_rooms.as_ref() {
            if let Some(room) = rooms.get(doc_id) {
                let mut room = room.lock().await;
                room.users.remove(&user_id);
            }
        }
        broadcast_event(&state.ws_channels, tenant_id, WsEvent::AwarenessRemove {
            document_id: doc_id.clone(),
            user_id,
        });
    }
    
    forward_task.abort();
    info!(user_id = %user_id, "WebSocket connection closed");
}

/// Broadcast an event
pub fn broadcast_event(channels: &WsChannels, tenant_id: Uuid, event: WsEvent) {
    if let Some(sender) = channels.get(&tenant_id) {
        let _ = sender.send(event);
    }
}

/// Handle client events
async fn handle_client_event(
    state: &Arc<AppState>,
    tenant_id: Uuid,
    user_id: Uuid,
    _user_name: &str,
    event: WsEvent,
    user_documents: &Arc<Mutex<HashSet<String>>>
) {
    match event {
        WsEvent::DocumentSubscribe { document_id } => {
            info!(document_id = %document_id, "Client subscribed");
            
            user_documents.lock().await.insert(document_id.clone());
            
            if let Some(rooms) = state.document_rooms.as_ref() {
                let room = rooms.entry(document_id.clone()).or_insert_with(|| {
                    Arc::new(Mutex::new(DocumentRoom::new()))
                });
                
                let mut room = room.lock().await;
                room.users.insert(user_id);
                
                let state_event = WsEvent::DocumentState {
                    document_id,
                    state: room.get_state_base64(),
                    state_vector: room.get_state_vector_base64(),
                };
                broadcast_event(&state.ws_channels, tenant_id, state_event);
            }
        }
        
        WsEvent::DocumentUnsubscribe { document_id } => {
            info!(document_id = %document_id, "Client unsubscribed");
            
            user_documents.lock().await.remove(&document_id);
            
            if let Some(rooms) = state.document_rooms.as_ref() {
                if let Some(room) = rooms.get(&document_id) {
                    room.lock().await.users.remove(&user_id);
                }
            }
            
            broadcast_event(&state.ws_channels, tenant_id, WsEvent::AwarenessRemove {
                document_id,
                user_id,
            });
        }
        
        WsEvent::DocumentUpdate { document_id, update, user_id: sender_id } => {
            info!(document_id = %document_id, "CRDT update received");
            
            if let Some(rooms) = state.document_rooms.as_ref() {
                if let Some(room) = rooms.get(&document_id) {
                    if let Err(e) = room.lock().await.apply_update_base64(&update) {
                        error!(error = %e, "Failed to apply CRDT update");
                    }
                }
            }
            
            broadcast_event(&state.ws_channels, tenant_id, WsEvent::DocumentUpdate {
                document_id,
                update,
                user_id: sender_id,
            });
        }
        
        WsEvent::DocumentSyncRequest { document_id, state_vector } => {
            if let Some(rooms) = state.document_rooms.as_ref() {
                if let Some(room) = rooms.get(&document_id) {
                    if let Ok(delta) = room.lock().await.get_delta_base64(&state_vector) {
                        broadcast_event(&state.ws_channels, tenant_id, WsEvent::DocumentUpdate {
                            document_id,
                            update: delta,
                            user_id,
                        });
                    }
                }
            }
        }
        
        WsEvent::AwarenessUpdate { document_id, user_id, user_name, user_color, cursor_position, selection_start, selection_end } => {
            broadcast_event(&state.ws_channels, tenant_id, WsEvent::AwarenessUpdate {
                document_id,
                user_id,
                user_name,
                user_color,
                cursor_position,
                selection_start,
                selection_end,
            });
        }
        
        _ => {}
    }
}
