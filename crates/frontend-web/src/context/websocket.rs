//! WebSocket Service - Real-time CRDT Collaboration
//!
//! Provides WebSocket connection for real-time collaborative editing.
//! Uses custom events to bridge between components.

use leptos::*;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::cell::RefCell;
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose::STANDARD as Base64};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// WebSocket event types (matches backend WsEvent)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WsEvent {
    /// Connection established
    Connected {
        user_id: Uuid,
        tenant_id: Uuid,
    },
    /// Subscribe to a document
    DocumentSubscribe {
        document_id: String,
    },
    /// Unsubscribe from a document
    DocumentUnsubscribe {
        document_id: String,
    },
    /// CRDT update
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
    /// Notification
    Notification {
        title: String,
        message: String,
        level: String,
    },
}

/// WebSocket connection state
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum WsConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

/// WebSocket service for real-time collaboration
#[derive(Clone)]
pub struct WebSocketService {
    // Connection state
    pub connected: RwSignal<bool>,
    pub connection_state: RwSignal<WsConnectionState>,
    
    // User info
    pub user_id: RwSignal<Option<Uuid>>,
    pub tenant_id: RwSignal<Option<Uuid>>,
    
    // Subscribed documents
    subscribed_documents: Rc<RefCell<Vec<String>>>,
    
    // WebSocket reference
    ws: Rc<RefCell<Option<web_sys::WebSocket>>>,
}

impl WebSocketService {
    /// Create a new WebSocket service
    pub fn new() -> Self {
        Self {
            connected: create_rw_signal(false),
            connection_state: create_rw_signal(WsConnectionState::Disconnected),
            user_id: create_rw_signal(None),
            tenant_id: create_rw_signal(None),
            subscribed_documents: Rc::new(RefCell::new(Vec::new())),
            ws: Rc::new(RefCell::new(None)),
        }
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> Signal<bool> {
        self.connected.into()
    }
    
    /// Get connection state
    pub fn get_connection_state(&self) -> Signal<WsConnectionState> {
        self.connection_state.into()
    }
    
    /// Connect to WebSocket server
    pub fn connect(&self, token: &str) {
        self.connection_state.set(WsConnectionState::Connecting);
        
        let ws_url = get_ws_url(token);
        
        if let Ok(websocket) = web_sys::WebSocket::new(&ws_url) {
            let connected = self.connected;
            let connection_state = self.connection_state;
            let user_id = self.user_id;
            let tenant_id = self.tenant_id;
            
            // On open
            {
                let connected = connected;
                let connection_state = connection_state;
                let onopen = Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
                    connected.set(true);
                    connection_state.set(WsConnectionState::Connected);
                    gloo_console::log!("WebSocket connected");
                });
                websocket.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                onopen.forget();
            }
            
            // On message
            {
                let user_id = user_id;
                let tenant_id = tenant_id;
                let onmessage = Closure::<dyn FnMut(_)>::new(move |e: web_sys::MessageEvent| {
                    if let Some(text) = e.data().as_string() {
                        if let Ok(event) = serde_json::from_str::<WsEvent>(&text) {
                            match event {
                                WsEvent::Connected { user_id: uid, tenant_id: tid } => {
                                    user_id.set(Some(uid));
                                    tenant_id.set(Some(tid));
                                    gloo_console::log!("Authenticated");
                                }
                                WsEvent::DocumentUpdate { document_id, update, .. } => {
                                    dispatch_document_update(&document_id, &update);
                                }
                                WsEvent::DocumentState { document_id, state, .. } => {
                                    dispatch_document_update(&document_id, &state);
                                }
                                WsEvent::AwarenessUpdate { 
                                    document_id, user_id, user_name, user_color, cursor_position, .. 
                                } => {
                                    dispatch_awareness_update(&document_id, user_id, &user_name, &user_color, cursor_position);
                                }
                                WsEvent::Notification { title, message, level } => {
                                    gloo_console::log!("Notification:", &title, &message, &level);
                                }
                                _ => {}
                            }
                        }
                    }
                });
                websocket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                onmessage.forget();
            }
            
            // On close
            {
                let connected = connected;
                let connection_state = connection_state;
                let onclose = Closure::<dyn FnMut(_)>::new(move |_: web_sys::CloseEvent| {
                    connected.set(false);
                    connection_state.set(WsConnectionState::Disconnected);
                    gloo_console::log!("WebSocket disconnected");
                });
                websocket.set_onclose(Some(onclose.as_ref().unchecked_ref()));
                onclose.forget();
            }
            
            // On error
            {
                let onerror = Closure::<dyn FnMut(_)>::new(move |e: web_sys::ErrorEvent| {
                    gloo_console::error!("WebSocket error:", e.message());
                });
                websocket.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                onerror.forget();
            }
            
            // Store WebSocket reference
            *self.ws.borrow_mut() = Some(websocket);
        } else {
            self.connection_state.set(WsConnectionState::Disconnected);
            gloo_console::error!("Failed to create WebSocket");
        }
    }
    
    /// Subscribe to a document for CRDT updates
    pub fn subscribe_document(&self, document_id: &str) {
        self.subscribed_documents.borrow_mut().push(document_id.to_string());
        self.send_message(&WsEvent::DocumentSubscribe {
            document_id: document_id.to_string(),
        });
    }
    
    /// Unsubscribe from a document
    pub fn unsubscribe_document(&self, document_id: &str) {
        self.subscribed_documents.borrow_mut().retain(|d| d != document_id);
        self.send_message(&WsEvent::DocumentUnsubscribe {
            document_id: document_id.to_string(),
        });
    }
    
    /// Send a CRDT update
    pub fn send_update(&self, document_id: &str, update: &[u8], user_id: Uuid) {
        self.send_message(&WsEvent::DocumentUpdate {
            document_id: document_id.to_string(),
            update: Base64.encode(update),
            user_id,
        });
    }
    
    /// Send awareness update
    pub fn send_awareness(&self, document_id: &str, user_id: Uuid, user_name: &str, user_color: &str, cursor_position: Option<u32>) {
        self.send_message(&WsEvent::AwarenessUpdate {
            document_id: document_id.to_string(),
            user_id,
            user_name: user_name.to_string(),
            user_color: user_color.to_string(),
            cursor_position,
            selection_start: None,
            selection_end: None,
        });
    }
    
    /// Send a message to WebSocket
    fn send_message(&self, event: &WsEvent) {
        if let Some(ws) = self.ws.borrow().as_ref() {
            if ws.ready_state() == web_sys::WebSocket::OPEN {
                if let Ok(json) = serde_json::to_string(event) {
                    let _ = ws.send_with_str(&json);
                }
            }
        }
    }
}

impl Default for WebSocketService {
    fn default() -> Self {
        Self::new()
    }
}

/// Get WebSocket URL based on current location
fn get_ws_url(token: &str) -> String {
    let location = web_sys::window()
        .and_then(|w| w.location().href().ok())
        .unwrap_or_else(|| "http://localhost:8080".to_string());
    
    let protocol = if location.starts_with("https") { "wss" } else { "ws" };
    let host = web_sys::window()
        .and_then(|w| w.location().host().ok())
        .unwrap_or_else(|| "localhost:8080".to_string());
    
    // In development, backend runs on different port
    let api_host = if host.contains("localhost") || host.contains("127.0.0.1") {
        "localhost:8080".to_string()
    } else {
        host
    };
    
    format!("{}://{}/ws?token={}", protocol, api_host, token)
}

/// Dispatch document update event
fn dispatch_document_update(document_id: &str, update_b64: &str) {
    if let Some(window) = web_sys::window() {
        let detail = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&detail, &"documentId".into(), &document_id.into());
        let _ = js_sys::Reflect::set(&detail, &"update".into(), &update_b64.into());
        
        if let Ok(event) = web_sys::CustomEvent::new_with_event_init_dict(
            "crdt-update-received",
            web_sys::CustomEventInit::new().detail(&detail)
        ) {
            let _ = window.dispatch_event(&event);
        }
    }
}

/// Dispatch awareness update event
fn dispatch_awareness_update(document_id: &str, user_id: Uuid, user_name: &str, user_color: &str, cursor_position: Option<u32>) {
    if let Some(window) = web_sys::window() {
        let detail = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&detail, &"documentId".into(), &document_id.into());
        let _ = js_sys::Reflect::set(&detail, &"userId".into(), &user_id.to_string().into());
        let _ = js_sys::Reflect::set(&detail, &"userName".into(), &user_name.into());
        let _ = js_sys::Reflect::set(&detail, &"userColor".into(), &user_color.into());
        let pos_val: JsValue = cursor_position.map(|p| JsValue::from(p)).unwrap_or(JsValue::NULL);
        let _ = js_sys::Reflect::set(&detail, &"cursorPosition".into(), &pos_val);
        
        if let Ok(event) = web_sys::CustomEvent::new_with_event_init_dict(
            "awareness-update",
            web_sys::CustomEventInit::new().detail(&detail)
        ) {
            let _ = window.dispatch_event(&event);
        }
    }
}

/// Create WebSocket context provider
#[component]
pub fn WebSocketProvider(children: Children) -> impl IntoView {
    let service = WebSocketService::new();
    provide_context(service);
    
    children()
}

/// Hook to use WebSocket service
pub fn use_websocket() -> WebSocketService {
    use_context::<WebSocketService>()
        .expect("WebSocketProvider must be present")
}

/// Hook for collaborative document editing
pub fn use_collaborative_document(
    entity_id: Uuid,
    field: &str,
) -> (Signal<String>, Callback<String>) {
    let ws = use_websocket();
    let document_id = format!("{}:{}", entity_id, field);
    
    // Local content state
    let (content, set_content) = create_signal(String::new());
    
    // Subscribe on mount
    let doc_id_clone = document_id.clone();
    let ws_for_subscribe = ws.clone();
    create_effect(move |_| {
        ws_for_subscribe.subscribe_document(&doc_id_clone);
    });
    
    // Create update callback
    let doc_id_for_update = document_id;
    let ws_for_update = ws;
    let on_change = Callback::new(move |new_content: String| {
        set_content.set(new_content.clone());
        ws_for_update.send_update(&doc_id_for_update, new_content.as_bytes(), Uuid::nil());
    });
    
    (content.into(), on_change)
}
