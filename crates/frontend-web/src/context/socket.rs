//! WebSocket Context - Real-time event connection manager
//!
//! Provides global WebSocket connection with auto-reconnect for real-time events.

use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket, CloseEvent};
use uuid::Uuid;

/// WebSocket event types (mirrors backend WsEvent)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WsEvent {
    NewMessage {
        thread_id: Uuid,
        sender_name: String,
        preview: String,
    },
    LeadAssigned {
        contact_id: Uuid,
        contact_name: String,
        assigned_by: String,
    },
    InteractionCreated {
        entity_type: String,
        entity_id: Uuid,
        interaction_type: String,
    },
    WebhookReceived {
        provider: String,
        message: String,
    },
    Notification {
        title: String,
        message: String,
        level: String,
    },
    Connected {
        user_id: Uuid,
        tenant_id: Uuid,
    },
}

/// Socket connection state
#[derive(Debug, Clone, PartialEq)]
pub enum SocketState {
    Connecting,
    Connected,
    Disconnected,
    Error(String),
}

/// Global socket context
#[derive(Clone)]
pub struct SocketContext {
    pub state: ReadSignal<SocketState>,
    pub events: ReadSignal<Vec<WsEvent>>,
    pub last_event: ReadSignal<Option<WsEvent>>,
}

/// Create and provide WebSocket context
#[component]
pub fn SocketProvider(children: Children) -> impl IntoView {
    let (state, set_state) = create_signal(SocketState::Disconnected);
    let (events, set_events) = create_signal::<Vec<WsEvent>>(Vec::new());
    let (last_event, set_last_event) = create_signal::<Option<WsEvent>>(None);

    // Get auth token from localStorage
    let get_token = || {
        web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .and_then(|s| s.get_item("session_token").ok())
            .flatten()
    };

    // Connect to WebSocket
    let connect = move || {
        if let Some(token) = get_token() {
            let window = web_sys::window().expect("no window");
            let location = window.location();
            let protocol = location.protocol().unwrap_or_else(|_| "http:".to_string());
            let host = location.host().unwrap_or_else(|_| "localhost:8080".to_string());
            
            let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
            let url = format!("{}//{}/ws?token={}", ws_protocol, host, token);

            match WebSocket::new(&url) {
                Ok(ws) => {
                    set_state.set(SocketState::Connecting);

                    // On open
                    let set_state_open = set_state;
                    let onopen = Closure::wrap(Box::new(move |_: JsValue| {
                        set_state_open.set(SocketState::Connected);
                        web_sys::console::log_1(&"WebSocket connected".into());
                    }) as Box<dyn Fn(JsValue)>);
                    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                    onopen.forget();

                    // On message
                    let set_events_msg = set_events;
                    let set_last_event_msg = set_last_event;
                    let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
                        if let Some(text) = e.data().as_string() {
                            if let Ok(event) = serde_json::from_str::<WsEvent>(&text) {
                                set_last_event_msg.set(Some(event.clone()));
                                set_events_msg.update(|v| {
                                    v.push(event);
                                    // Keep only last 100 events
                                    if v.len() > 100 {
                                        v.remove(0);
                                    }
                                });
                            }
                        }
                    }) as Box<dyn Fn(MessageEvent)>);
                    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                    onmessage.forget();

                    // On close - reconnect after delay
                    let set_state_close = set_state;
                    let onclose = Closure::wrap(Box::new(move |_: CloseEvent| {
                        set_state_close.set(SocketState::Disconnected);
                        web_sys::console::log_1(&"WebSocket disconnected".into());
                    }) as Box<dyn Fn(CloseEvent)>);
                    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
                    onclose.forget();

                    // On error
                    let set_state_err = set_state;
                    let onerror = Closure::wrap(Box::new(move |_: JsValue| {
                        set_state_err.set(SocketState::Error("Connection error".to_string()));
                    }) as Box<dyn Fn(JsValue)>);
                    ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                    onerror.forget();
                }
                Err(e) => {
                    set_state.set(SocketState::Error(format!("{:?}", e)));
                }
            }
        }
    };

    // Connect on mount if token exists
    create_effect(move |_| {
        if get_token().is_some() {
            connect();
        }
    });

    // Provide context
    let context = SocketContext {
        state,
        events,
        last_event,
    };
    provide_context(context.clone());

    children()
}

/// Hook to access socket context
pub fn use_socket() -> SocketContext {
    use_context::<SocketContext>().expect("SocketProvider not found in tree")
}

