//! WebSocket Client - Real-time connection from frontend

use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use uuid::Uuid;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = WebSocket)]
    type WsClient;
    
    #[wasm_bindgen(constructor, js_class = "WebSocket")]
    fn new(url: &str) -> WsClient;
    
    #[wasm_bindgen(method, setter)]
    fn set_onopen(this: &WsClient, callback: &Closure<dyn FnMut(JsValue)>);
    
    #[wasm_bindgen(method, setter)]
    fn set_onmessage(this: &WsClient, callback: &Closure<dyn FnMut(JsValue)>);
    
    #[wasm_bindgen(method, setter)]
    fn set_onclose(this: &WsClient, callback: &Closure<dyn FnMut(JsValue)>);
    
    #[wasm_bindgen(method, setter)]
    fn set_onerror(this: &WsClient, callback: &Closure<dyn FnMut(JsValue)>);
    
    #[wasm_bindgen(method)]
    fn send(this: &WsClient, data: &str);
    
    #[wasm_bindgen(method)]
    fn close(this: &WsClient);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    EntityUpdated {
        entity_id: Uuid,
        entity_type: String,
        field_values: serde_json::Value,
        updated_by: Uuid,
    },
    EntityCreated {
        entity_id: Uuid,
        entity_type: String,
        field_values: serde_json::Value,
        created_by: Uuid,
    },
    EntityDeleted {
        entity_id: Uuid,
        entity_type: String,
        deleted_by: Uuid,
    },
    Presence {
        user_id: Uuid,
        status: PresenceStatus,
        location: Option<String>,
    },
    Cursor {
        user_id: Uuid,
        entity_id: Uuid,
        field: String,
        position: CursorPosition,
    },
    CrdtUpdate {
        entity_id: Uuid,
        field: String,
        update: Vec<u8>,
    },
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

/// WebSocket connection hook
pub fn use_websocket(
    tenant_id: Uuid,
    user_id: Uuid,
    on_message: impl Fn(WsMessage) + 'static,
) -> (ReadSignal<bool>, WriteSignal<WsMessage>) {
    let (is_connected, set_is_connected) = create_signal(false);
    let (outgoing, set_outgoing) = create_signal(WsMessage::Ping);
    
    // Create WebSocket connection
    create_effect(move |_| {
        let ws_url = format!("ws://localhost:8080/ws/{}/{}", tenant_id, user_id);
        let ws = WsClient::new(&ws_url);
        
        // On open
        let on_open = Closure::wrap(Box::new(move |_: JsValue| {
            set_is_connected.set(true);
            tracing::info!("WebSocket connected");
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(&on_open);
        on_open.forget();
        
        // On message
        let on_msg_cb = on_message.clone();
        let on_msg = Closure::wrap(Box::new(move |e: JsValue| {
            if let Ok(msg_event) = e.dyn_into::<web_sys::MessageEvent>() {
                if let Ok(text) = msg_event.data().dyn_into::<js_sys::JsString>() {
                    let text: String = text.into();
                    if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                        on_msg_cb(msg);
                    }
                }
            }
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onmessage(&on_msg);
        on_msg.forget();
        
        // On close
        let on_close = Closure::wrap(Box::new(move |_: JsValue| {
            set_is_connected.set(false);
            tracing::warn!("WebSocket disconnected");
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onclose(&on_close);
        on_close.forget();
        
        // On error
        let on_error = Closure::wrap(Box::new(move |e: JsValue| {
            tracing::error!("WebSocket error: {:?}", e);
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onerror(&on_error);
        on_error.forget();
        
        // Send outgoing messages
        create_effect(move |_| {
            let msg = outgoing.get();
            if let Ok(json) = serde_json::to_string(&msg) {
                ws.send(&json);
            }
        });
        
        // Cleanup on unmount
        on_cleanup(move || {
            ws.close();
        });
    });
    
    (is_connected, set_outgoing)
}

/// Presence indicator component
#[component]
pub fn PresenceIndicator(
    tenant_id: Uuid,
    user_id: Uuid,
) -> impl IntoView {
    let (online_users, set_online_users) = create_signal(Vec::<Uuid>::new());
    
    let (_is_connected, send_message) = use_websocket(
        tenant_id,
        user_id,
        move |msg| {
            if let WsMessage::Presence { user_id, status, .. } = msg {
                match status {
                    PresenceStatus::Online => {
                        set_online_users.update(|users| {
                            if !users.contains(&user_id) {
                                users.push(user_id);
                            }
                        });
                    }
                    PresenceStatus::Offline => {
                        set_online_users.update(|users| {
                            users.retain(|u| *u != user_id);
                        });
                    }
                    _ => {}
                }
            }
        },
    );
    
    view! {
        <div class="presence-indicator">
            <div class="presence-count">
                {move || online_users.get().len()}
                " online"
            </div>
            <div class="presence-dots">
                <For
                    each=move || online_users.get()
                    key=|user| *user
                    children=move |user| {
                        view! {
                            <div
                                class="presence-dot"
                                title=format!("User {}", user)
                            />
                        }
                    }
                />
            </div>
        </div>
    }
}
