//! Rich Text Editor - Collaborative editor with CRDT support
//! 
//! Uses Yjs for real-time collaborative editing

use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use uuid::Uuid;

#[wasm_bindgen(module = "/public/js/yjs-editor.js")]
extern "C" {
    #[wasm_bindgen(js_name = initEditor)]
    fn init_editor(element_id: &str, initial_text: &str) -> JsValue;
    
    #[wasm_bindgen(js_name = getEditorUpdate)]
    fn get_editor_update(editor_id: &str) -> Vec<u8>;
    
    #[wasm_bindgen(js_name = applyEditorUpdate)]
    fn apply_editor_update(editor_id: &str, update: &[u8]);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorUpdate {
    pub entity_id: Uuid,
    pub field: String,
    pub update: Vec<u8>,
    pub state_vector: Vec<u8>,
}

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
    let editor_id_view = id.clone(); // For view
    let (is_syncing, set_is_syncing) = create_signal(false);
    let (last_update, set_last_update) = create_signal::<Option<Vec<u8>>>(None);
    
    // Clone for closures
    let initial_text_effect = initial_text.clone();
    let field_effect = field.clone();
    
    // Initialize editor on mount
    create_effect(move |_| {
        if collaborative {
            // Initialize Yjs editor
            init_editor(&editor_id, &initial_text_effect);
            
            // Start sync loop
            start_sync_loop(
                editor_id.clone(),
                entity_id,
                field_effect.clone(),
                set_is_syncing,
                set_last_update,
            );
        }
    });
    
    // Clone for footer closure
    let initial_text_len = initial_text.len();
    
    view! {
        <div class="rich-text-editor" data-editor-id=editor_id_view>
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
                    <div
                        class="sync-indicator"
                        class:syncing=move || is_syncing.get()
                    >
                        <Show
                            when=move || is_syncing.get()
                            fallback=|| view! {
                                <span class="sync-status">
                                    "✓ Synced"
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
                class="editor-content"
                contenteditable="true"
                placeholder="Start typing..."
            />
            
            <div class="editor-footer">
                <span class="char-count">
                    {move || format!("{} characters", initial_text_len)}
                </span>
            </div>
        </div>
    }
}

/// Start background sync loop for collaborative editing
fn start_sync_loop(
    editor_id: String,
    entity_id: Uuid,
    field: String,
    set_is_syncing: WriteSignal<bool>,
    set_last_update: WriteSignal<Option<Vec<u8>>>,
) {
    spawn_local(async move {
        loop {
            // Wait 500ms between syncs
            gloo_timers::future::TimeoutFuture::new(500).await;
            
            // Get local updates
            let update = get_editor_update(&editor_id);
            
            if !update.is_empty() {
                set_is_syncing.set(true);
                
                // Send to server
                match sync_with_server(entity_id, &field, update).await {
                    Ok(server_update) => {
                        if !server_update.is_empty() {
                            // Apply server updates
                            apply_editor_update(&editor_id, &server_update);
                        }
                        set_last_update.set(Some(server_update));
                    }
                    Err(e) => {
                        tracing::error!("Sync error: {}", e);
                    }
                }
                
                set_is_syncing.set(false);
            }
        }
    });
}

/// Sync with server
async fn sync_with_server(
    entity_id: Uuid,
    field: &str,
    update: Vec<u8>,
) -> Result<Vec<u8>, String> {
    // TODO: Actual API call to /api/v1/crdt/:entity_id/:field
    // For now, return empty (no server updates)
    Ok(Vec::new())
}

// Placeholder for gloo_timers
mod gloo_timers {
    pub mod future {
        pub struct TimeoutFuture(u32);
        
        impl TimeoutFuture {
            pub fn new(millis: u32) -> Self {
                Self(millis)
            }
        }
        
        impl std::future::Future for TimeoutFuture {
            type Output = ();
            
            fn poll(
                self: std::pin::Pin<&mut Self>,
                _cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output> {
                // Simplified - in real code use proper timer
                std::task::Poll::Ready(())
            }
        }
    }
}
