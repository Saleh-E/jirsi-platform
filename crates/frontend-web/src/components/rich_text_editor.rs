//! Rich Text Editor - Collaborative editor with CRDT support
//! 
//! Uses Yrs (Yjs) for real-time collaborative editing via WebSocket

use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use uuid::Uuid;
use crate::offline::crdt::CrdtText;
use base64::{Engine as _, engine::general_purpose::STANDARD as Base64};

/// Editor update message for WebSocket sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorUpdate {
    pub entity_id: Uuid,
    pub field: String,
    pub update: String, // Base64-encoded Yjs update
    pub state_vector: String, // Base64-encoded state vector
}

/// Presence info for other users
#[derive(Debug, Clone)]
pub struct UserPresence {
    pub user_id: Uuid,
    pub user_name: String,
    pub color: String,
    pub cursor_position: Option<u32>,
}

/// Generate a random color for user presence
fn generate_user_color() -> String {
    let colors = [
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", 
        "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
        "#BB8FCE", "#85C1E9", "#F8B500", "#00CED1"
    ];
    let index = (js_sys::Math::random() * colors.len() as f64) as usize;
    colors[index % colors.len()].to_string()
}

/// Rich Text Editor Component with CRDT collaboration
#[component]
pub fn RichTextEditor(
    /// Unique ID for this editor instance
    #[prop(into)]
    id: String,
    
    /// Initial text content
    #[prop(default = String::new())]
    initial_text: String,
    
    /// Entity ID being edited
    entity_id: Uuid,
    
    /// Field name (e.g., "description", "notes")
    #[prop(into)]
    field: String,
    
    /// Callback when content changes
    #[prop(optional)]
    on_change: Option<Callback<String>>,
    
    /// Enable collaborative mode
    #[prop(default = true)]
    collaborative: bool,
) -> impl IntoView {
    let editor_id = id.clone();
    let document_id = format!("{}:{}", entity_id, field);
    
    // CRDT text document
    let crdt_text = store_value(CrdtText::new(entity_id, &field));
    
    // Initialize with content
    crdt_text.with_value(|t| t.set(&initial_text));
    
    // Reactive state
    let (content, set_content) = create_signal(initial_text.clone());
    let (is_syncing, set_is_syncing) = create_signal(false);
    let (is_synced, set_is_synced) = create_signal(true);
    let (other_users, set_other_users) = create_signal::<Vec<UserPresence>>(vec![]);
    let (char_count, set_char_count) = create_signal(initial_text.len());
    
    // User color for presence
    let user_color = store_value(generate_user_color());
    
    // Input reference
    let editor_ref = create_node_ref::<leptos::html::Div>();
    
    // Handle text input
    let field_for_input = field.clone();
    let document_id_for_input = document_id.clone();
    let on_input = move |ev: web_sys::Event| {
        if let Some(target) = ev.target() {
            if let Some(element) = target.dyn_ref::<web_sys::HtmlElement>() {
                let new_text = element.inner_text();
                
                // Update CRDT
                crdt_text.with_value(|t| t.set(&new_text));
                
                // Update local state
                set_content.set(new_text.clone());
                set_char_count.set(new_text.len());
                set_is_synced.set(false);
                
                // Trigger callback
                if let Some(cb) = on_change {
                    cb.call(new_text.clone());
                }
                
                // Send update via WebSocket if collaborative
                if collaborative {
                    set_is_syncing.set(true);
                    
                    let update = crdt_text.with_value(|t| t.get_update());
                    let update_b64 = Base64.encode(&update);
                    
                    // Send CRDT update (using window event for now)
                    let _ = send_document_update(&document_id_for_input, &update_b64);
                    
                    // Mark as synced after short delay (optimistic)
                    leptos::spawn_local(async move {
                        gloo_timers::future::TimeoutFuture::new(300).await;
                        set_is_syncing.set(false);
                        set_is_synced.set(true);
                    });
                }
            }
        }
    };
    
    // Selection/awareness handlers (simplified)
    let on_selection_change = move |_: web_sys::MouseEvent| {
        // In a full implementation, track cursor position and send awareness
    };
    
    let on_keyup = move |_: web_sys::KeyboardEvent| {
        // Track typing for awareness
    };
    
    // Set up effect to listen for incoming CRDT updates
    let document_id_for_effect = document_id.clone();
    create_effect(move |_| {
        if collaborative {
            // Listen for document updates via custom event
            listen_for_updates(&document_id_for_effect, move |update_bytes| {
                // Apply remote update to CRDT
                crdt_text.with_value(|t| {
                    if let Err(e) = t.apply_update(&update_bytes) {
                        gloo_console::error!("CRDT apply failed:", &e);
                    }
                });
                
                // Update content from CRDT
                let new_content = crdt_text.with_value(|t| t.get());
                set_content.set(new_content.clone());
                set_char_count.set(new_content.len());
                
                // Update editor DOM
                if let Some(elem) = editor_ref.get() {
                    elem.set_inner_text(&new_content);
                }
            });
        }
    });
    
    view! {
        <div class="rich-text-editor" data-editor-id=editor_id.clone()>
            <div class="editor-toolbar">
                <button class="toolbar-btn" data-action="bold" title="Bold (Ctrl+B)">
                    <strong>"B"</strong>
                </button>
                <button class="toolbar-btn" data-action="italic" title="Italic (Ctrl+I)">
                    <em>"I"</em>
                </button>
                <button class="toolbar-btn" data-action="underline" title="Underline (Ctrl+U)">
                    <u>"U"</u>
                </button>
                <div class="toolbar-divider" />
                <button class="toolbar-btn" data-action="heading1" title="Heading 1">
                    "H1"
                </button>
                <button class="toolbar-btn" data-action="heading2" title="Heading 2">
                    "H2"
                </button>
                <div class="toolbar-divider" />
                <button class="toolbar-btn" data-action="bullet-list" title="Bullet List">
                    "• List"
                </button>
                <button class="toolbar-btn" data-action="ordered-list" title="Numbered List">
                    "1. List"
                </button>
                
                <Show when=move || collaborative>
                    <div class="toolbar-spacer" />
                    
                    // Presence indicators
                    <div class="presence-indicators">
                        <For
                            each=move || other_users.get()
                            key=|user| user.user_id
                            children=|user| {
                                view! {
                                    <div
                                        class="presence-avatar"
                                        style=format!("background-color: {}", user.color)
                                        title=user.user_name.clone()
                                    >
                                        {user.user_name.chars().next().unwrap_or('?')}
                                    </div>
                                }
                            }
                        />
                    </div>
                    
                    // Sync status
                    <div class="sync-indicator" class:syncing=move || is_syncing.get()>
                        <Show
                            when=move || is_syncing.get()
                            fallback=move || view! {
                                <span class="sync-status synced">
                                    {move || if is_synced.get() { "✓ Synced" } else { "○ Unsaved" }}
                                </span>
                            }
                        >
                            <span class="sync-status syncing">
                                <div class="spinner-tiny" />
                                "Syncing..."
                            </span>
                        </Show>
                    </div>
                </Show>
            </div>
            
            <div
                id=id
                node_ref=editor_ref
                class="editor-content"
                contenteditable="true"
                placeholder="Start typing..."
                on:input=on_input
                on:mouseup=on_selection_change
                on:keyup=on_keyup
            >
                {initial_text}
            </div>
            
            <div class="editor-footer">
                <span class="char-count">
                    {move || format!("{} characters", char_count.get())}
                </span>
            </div>
        </div>
    }
}

/// Send a CRDT update via the WebSocket
fn send_document_update(document_id: &str, update_b64: &str) -> Result<(), JsValue> {
    // Dispatch custom event for the WebSocket handler to pick up
    if let Some(window) = web_sys::window() {
        let detail = js_sys::Object::new();
        js_sys::Reflect::set(&detail, &"documentId".into(), &document_id.into())?;
        js_sys::Reflect::set(&detail, &"update".into(), &update_b64.into())?;
        
        let event = web_sys::CustomEvent::new_with_event_init_dict(
            "crdt-update",
            web_sys::CustomEventInit::new().detail(&detail)
        )?;
        window.dispatch_event(&event)?;
    }
    Ok(())
}

/// Listen for incoming CRDT updates
fn listen_for_updates<F>(document_id: &str, callback: F)
where
    F: Fn(Vec<u8>) + 'static
{
    let doc_id = document_id.to_string();
    
    if let Some(window) = web_sys::window() {
        let closure = Closure::wrap(Box::new(move |event: web_sys::CustomEvent| {
            if let Some(detail) = event.detail().dyn_ref::<js_sys::Object>() {
                let received_doc_id = js_sys::Reflect::get(detail, &"documentId".into())
                    .ok()
                    .and_then(|v| v.as_string());
                
                if received_doc_id.as_deref() == Some(&doc_id) {
                    if let Ok(update_b64) = js_sys::Reflect::get(detail, &"update".into()) {
                        if let Some(update_str) = update_b64.as_string() {
                            if let Ok(bytes) = Base64.decode(&update_str) {
                                callback(bytes);
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);
        
        let _ = window.add_event_listener_with_callback(
            "crdt-update-received",
            closure.as_ref().unchecked_ref()
        );
        
        // Keep the closure alive
        closure.forget();
    }
}

/// Simple text area with CRDT sync (non-rich version)
#[component]
pub fn CollaborativeTextArea(
    /// Unique ID
    #[prop(into)]
    id: String,
    
    /// Entity ID
    entity_id: Uuid,
    
    /// Field name
    #[prop(into)]
    field: String,
    
    /// Initial value
    #[prop(default = String::new())]
    value: String,
    
    /// Placeholder text
    #[prop(default = "Enter text...".to_string())]
    placeholder: String,
    
    /// Number of rows
    #[prop(default = 4)]
    rows: u32,
    
    /// On change callback
    #[prop(optional)]
    on_change: Option<Callback<String>>,
) -> impl IntoView {
    let (content, set_content) = create_signal(value);
    let crdt = store_value(CrdtText::new(entity_id, &field));
    
    // Init CRDT
    crdt.with_value(|t| t.set(&content.get_untracked()));
    
    let on_input = move |ev: web_sys::Event| {
        if let Some(target) = ev.target() {
            if let Some(textarea) = target.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                let new_value = textarea.value();
                crdt.with_value(|t| t.set(&new_value));
                set_content.set(new_value.clone());
                
                if let Some(cb) = on_change {
                    cb.call(new_value);
                }
            }
        }
    };
    
    view! {
        <textarea
            id=id
            class="collaborative-textarea"
            placeholder=placeholder
            rows=rows
            on:input=on_input
        >
            {move || content.get()}
        </textarea>
    }
}
