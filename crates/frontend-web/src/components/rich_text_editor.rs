//! Rich Text Editor - Collaborative editor with CRDT support
//! 
//! Uses Yrs (Yjs) for real-time collaborative editing via WebSocket

use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use uuid::Uuid;
use crate::offline::crdt::{CrdtText, AwarenessState};
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
    
    // Input reference for cursor tracking
    let editor_ref = create_node_ref::<leptos::html::Div>();
    
    // Clone field for different closures
    let field_for_input = field.clone();
    let field_for_selection = field.clone();
    let field_for_keyup = field.clone();
    let field_for_effect = field.clone();
    let field_for_awareness = field.clone();
    
    // Handle text input
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
                    cb.call(new_text);
                }
                
                // Schedule sync
                if collaborative {
                    set_is_syncing.set(true);
                    
                    let update = crdt_text.with_value(|t| t.get_update());
                    let update_b64 = Base64.encode(&update);
                    
                    // Send via WebSocket (would use leptos-use websocket hook)
                    send_crdt_update(entity_id, &field_for_input, &update_b64);
                    
                    // Mark as synced after short delay (optimistic)
                    spawn_local(async move {
                        gloo_timers::future::TimeoutFuture::new(300).await;
                        set_is_syncing.set(false);
                        set_is_synced.set(true);
                    });
                }
            }
        }
    };
    
    // Handle cursor position for awareness (simplified - actual implementation would use Selection API)
    let on_selection_change = move |_: web_sys::MouseEvent| {
        // For now, just track that user is active
        // Full cursor position tracking requires web-sys "Selection" feature
        if collaborative {
            // Send activity ping rather than exact cursor position
            send_awareness_update(entity_id, &field_for_selection, 0);
        }
    };
    
    // Separate handler for keyboard events
    let on_keyup_selection = move |_: web_sys::KeyboardEvent| {
        if collaborative {
            send_awareness_update(entity_id, &field_for_keyup, 0);
        }
    };

    
    // Set up WebSocket listener for incoming updates
    create_effect(move |_| {
        if collaborative {
            // Subscribe to document updates via WebSocket
            subscribe_to_document(entity_id, &field_for_effect, move |update_b64: String| {
                if let Ok(update_bytes) = Base64.decode(&update_b64) {
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
                }
            });
            
            // Subscribe to awareness updates
            subscribe_to_awareness(entity_id, &field_for_awareness, move |presence: UserPresence| {
                set_other_users.update(|users| {
                    // Update or add user
                    if let Some(existing) = users.iter_mut().find(|u| u.user_id == presence.user_id) {
                        *existing = presence;
                    } else {
                        users.push(presence);
                    }
                });
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
                    
                    // Presence indicators - show other users
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
                on:keyup=on_keyup_selection
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

// ============ WebSocket Integration Stubs ============
// These would be implemented using leptos-use's use_websocket hook

/// Send CRDT update via WebSocket
fn send_crdt_update(_entity_id: Uuid, _field: &str, _update_b64: &str) {
    // TODO: Use global WebSocket context to send:
    // WsEvent::DocumentUpdate { document_id, update, user_id }
    gloo_console::log!("CRDT update ready to send");
}

/// Send awareness update (cursor position)
fn send_awareness_update(_entity_id: Uuid, _field: &str, _cursor_pos: u32) {
    // TODO: Use global WebSocket context to send:
    // WsEvent::AwarenessUpdate { document_id, cursor_position, ... }
}

/// Subscribe to document updates
fn subscribe_to_document<F>(_entity_id: Uuid, _field: &str, _on_update: F)
where
    F: Fn(String) + 'static,
{
    // TODO: Register callback for incoming DocumentUpdate events
}

/// Subscribe to awareness updates
fn subscribe_to_awareness<F>(_entity_id: Uuid, _field: &str, _on_presence: F)
where
    F: Fn(UserPresence) + 'static,
{
    // TODO: Register callback for incoming AwarenessUpdate events
}

// ============ Timer for debouncing ============

mod gloo_timers {
    pub mod future {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};
        use std::cell::Cell;
        use std::rc::Rc;

        pub struct TimeoutFuture {
            millis: u32,
            started: bool,
            completed: Rc<Cell<bool>>,
        }

        impl TimeoutFuture {
            pub fn new(millis: u32) -> Self {
                Self {
                    millis,
                    started: false,
                    completed: Rc::new(Cell::new(false)),
                }
            }
        }

        impl Future for TimeoutFuture {
            type Output = ();

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                if self.completed.get() {
                    return Poll::Ready(());
                }

                if !self.started {
                    self.started = true;
                    let completed = self.completed.clone();
                    let waker = cx.waker().clone();
                    
                    let closure = Closure::once(Box::new(move || {
                        completed.set(true);
                        waker.wake();
                    }) as Box<dyn FnOnce()>);

                    if let Some(window) = web_sys::window() {
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            closure.as_ref().unchecked_ref(),
                            self.millis as i32,
                        );
                    }
                    
                    closure.forget();
                }

                Poll::Pending
            }
        }
    }
}

