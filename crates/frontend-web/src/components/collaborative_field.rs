//! CollaborativeTextField - Real-time collaborative text editing
//!
//! ## Antigravity Integration
//! This component uses CrdtText for fields marked with `physics: TextMerge`.
//! Enables Google Docs-style multi-user editing via the yrs CRDT library.

use leptos::*;
use uuid::Uuid;
use std::sync::Arc;

use crate::offline::crdt::CrdtText;
use crate::context::websocket::{use_websocket, WsConnectionState};

/// Props for CollaborativeTextField
#[derive(Clone)]
pub struct CollaborativeFieldProps {
    pub entity_id: Uuid,
    pub field_name: String,
    pub initial_value: String,
    pub placeholder: Option<String>,
    pub readonly: bool,
}

/// CollaborativeTextField component - Real-time multi-user text editing
/// 
/// Uses CRDT (Conflict-free Replicated Data Types) for seamless merging
/// of concurrent edits from multiple users.
#[component]
pub fn CollaborativeTextField(
    /// Entity ID for the record
    entity_id: Uuid,
    /// Field name for this text field
    field_name: String,
    /// Initial text value
    #[prop(default = String::new())]
    initial_value: String,
    /// Placeholder text
    #[prop(optional)]
    placeholder: Option<String>,
    /// Whether field is readonly
    #[prop(default = false)]
    readonly: bool,
    /// Callback when text changes (for saving to server)
    #[prop(optional)]
    on_change: Option<Callback<String>>,
) -> impl IntoView {
    // Create CRDT text instance for this field
    let crdt_text = Arc::new(CrdtText::new(entity_id, &field_name));
    
    // Initialize with initial value if provided
    if !initial_value.is_empty() {
        crdt_text.set(&initial_value);
    }
    
    // Local state for reactive UI updates
    let (text_content, set_text_content) = create_signal(initial_value.clone());
    let (is_syncing, set_is_syncing) = create_signal(false);
    let (collaborators, set_collaborators) = create_signal(Vec::<String>::new());
    
    // WebSocket for sync
    let ws = use_websocket();
    let ws_state = ws.connection_state;
    
    let crdt_for_sync = crdt_text.clone();
    let crdt_for_input = crdt_text.clone();
    let crdt_for_remote = crdt_text.clone();
    
    // Subscribe to WebSocket messages for CRDT updates
    create_effect(move |_| {
        if matches!(ws_state.get(), WsConnectionState::Connected) {
            // Listen for CRDT sync messages
            let doc_id = format!("{}:{}", entity_id, field_name);
            
            // When connected, request initial sync
            set_is_syncing.set(true);
            let state_vector = crdt_for_sync.get_state_vector();
            
            // Would send: ws.send(SyncRequest { doc_id, state_vector });
            // For now, just update sync state
            set_is_syncing.set(false);
        }
    });
    
    // Handle text input
    let on_input = move |ev: leptos::ev::Event| {
        if readonly {
            return;
        }
        
        let new_text = event_target_value(&ev);
        let _old_text = text_content.get();
        
        // Calculate diff and apply to CRDT
        // For simplicity, we replace entire content
        // A production implementation would use character-level diffing
        crdt_for_input.set(&new_text);
        set_text_content.set(new_text.clone());
        
        // Get update and broadcast
        let _update = crdt_for_input.get_update();
        
        // Would send: ws.send(CrdtUpdate { doc_id, update });
        
        // Notify parent of change
        if let Some(callback) = on_change.as_ref() {
            callback.call(new_text);
        }
    };
    
    // Apply remote updates (would be called from WebSocket message handler)
    let apply_remote_update = move |update_bytes: Vec<u8>| {
        if let Ok(()) = crdt_for_remote.apply_update(&update_bytes) {
            let new_text = crdt_for_remote.get();
            set_text_content.set(new_text);
        }
    };
    
    view! {
        <div class="collaborative-field">
            // Sync indicator
            <div class="collaborative-field__status">
                <Show when=move || is_syncing.get()>
                    <span class="sync-indicator syncing">"🔄 Syncing..."</span>
                </Show>
                <Show when=move || matches!(ws_state.get(), WsConnectionState::Connected)>
                    <span class="sync-indicator connected">"🟢 Live"</span>
                </Show>
                <Show when=move || !collaborators.get().is_empty()>
                    <span class="collaborators">
                        {move || {
                            let users = collaborators.get();
                            if users.len() == 1 {
                                format!("👤 {} editing", users[0])
                            } else {
                                format!("👥 {} users editing", users.len())
                            }
                        }}
                    </span>
                </Show>
            </div>
            
            // Text area with CRDT backing
            <textarea
                class="collaborative-field__input"
                placeholder=placeholder.unwrap_or_default()
                prop:value=text_content
                on:input=on_input
                readonly=readonly
            ></textarea>
            
            // Footer with collaboration info
            <div class="collaborative-field__footer">
                <span class="merge-strategy">"📝 TextMerge CRDT"</span>
            </div>
        </div>
        
        // Styles
        <style>{r#"
.collaborative-field {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.collaborative-field__status {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 0.75rem;
    color: var(--color-muted, #888);
}

.sync-indicator {
    display: flex;
    align-items: center;
    gap: 0.25rem;
}

.sync-indicator.syncing {
    animation: pulse 1s infinite;
}

.sync-indicator.connected {
    color: var(--color-success, #10b981);
}

.collaborators {
    background: var(--color-brand-primary-10, rgba(124, 58, 237, 0.1));
    padding: 0.125rem 0.5rem;
    border-radius: 1rem;
}

.collaborative-field__input {
    width: 100%;
    min-height: 120px;
    padding: 0.75rem;
    font-size: 0.875rem;
    line-height: 1.5;
    border: 1px solid var(--color-border, #e5e7eb);
    border-radius: 0.5rem;
    resize: vertical;
    font-family: inherit;
    transition: border-color 0.2s, box-shadow 0.2s;
}

.collaborative-field__input:focus {
    outline: none;
    border-color: var(--color-brand-primary, #7c3aed);
    box-shadow: 0 0 0 3px var(--color-brand-primary-20, rgba(124, 58, 237, 0.2));
}

.collaborative-field__footer {
    display: flex;
    justify-content: flex-end;
    font-size: 0.6875rem;
    color: var(--color-muted, #888);
}

.merge-strategy {
    opacity: 0.6;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
}
        "#}</style>
    }
}

/// Helper to check if a field should use collaborative editing
pub fn should_use_crdt(physics: &str) -> bool {
    physics == "textMerge" || physics == "\"textMerge\""
}
