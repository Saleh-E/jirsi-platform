//! Conflict Resolution Component
//!
//! Provides a modal UI for resolving version conflicts when syncing data.
//! Supports "Keep Mine", "Keep Theirs", and "Merge" options.

use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::Value as JsonValue;

/// Conflict data to be resolved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictData {
    /// Entity ID with conflict
    pub entity_id: Uuid,
    /// Entity type (e.g., "contact", "deal")
    pub entity_type: String,
    /// Field that has conflict (or "all" for full record)
    pub field: Option<String>,
    /// Local (client) version of the data
    pub local_data: JsonValue,
    /// Server version of the data
    pub server_data: JsonValue,
    /// Local version number
    pub local_version: u64,
    /// Server version number
    pub server_version: u64,
    /// Timestamp of local change
    pub local_updated_at: String,
    /// Timestamp of server change
    pub server_updated_at: String,
}

/// User's resolution choice
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResolutionChoice {
    /// Keep the local version
    KeepMine,
    /// Keep the server version
    KeepTheirs,
    /// Merge changes (field-level if possible)
    Merge,
    /// Cancel and decide later
    Cancel,
}

/// Conflict resolution result
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    pub entity_id: Uuid,
    pub choice: ResolutionChoice,
    pub merged_data: Option<JsonValue>,
}


/// Conflict Resolver modal component
#[component]
pub fn ConflictResolver(
    /// The conflict to resolve
    conflict: ConflictData,
    /// Callback when resolved
    on_resolve: Callback<ResolutionResult>,
    /// Callback when cancelled
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (selected_choice, set_selected_choice) = create_signal::<Option<ResolutionChoice>>(None);
    let (show_details, set_show_details) = create_signal(false);
    
    let entity_id = conflict.entity_id;
    let conflict_for_merge = conflict.clone();
    
    // Handler for keeping local version
    let conflict_for_mine = conflict.clone();
    let on_resolve_mine = on_resolve.clone();
    let keep_mine = move |_| {
        set_selected_choice.set(Some(ResolutionChoice::KeepMine));
        on_resolve_mine.call(ResolutionResult {
            entity_id,
            choice: ResolutionChoice::KeepMine,
            merged_data: Some(conflict_for_mine.local_data.clone()),
        });
    };
    
    // Handler for keeping server version
    let conflict_for_theirs = conflict.clone();
    let on_resolve_theirs = on_resolve.clone();
    let keep_theirs = move |_| {
        set_selected_choice.set(Some(ResolutionChoice::KeepTheirs));
        on_resolve_theirs.call(ResolutionResult {
            entity_id,
            choice: ResolutionChoice::KeepTheirs,
            merged_data: Some(conflict_for_theirs.server_data.clone()),
        });
    };
    
    // Handler for merge (auto-merge if possible)
    let on_resolve_merge = on_resolve.clone();
    let try_merge = move |_| {
        set_selected_choice.set(Some(ResolutionChoice::Merge));
        
        // Attempt automatic field-level merge
        let merged = merge_json_objects(
            &conflict_for_merge.local_data, 
            &conflict_for_merge.server_data
        );
        
        on_resolve_merge.call(ResolutionResult {
            entity_id,
            choice: ResolutionChoice::Merge,
            merged_data: Some(merged),
        });
    };
    
    // Cancel handler
    let handle_cancel = move |_| {
        on_cancel.call(());
    };
    
    view! {
        <div class="conflict-overlay">
            <div class="conflict-modal">
                <div class="conflict-header">
                    <div class="conflict-icon">
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
                            <line x1="12" y1="9" x2="12" y2="13"/>
                            <line x1="12" y1="17" x2="12.01" y2="17"/>
                        </svg>
                    </div>
                    <h2>"Sync Conflict Detected"</h2>
                </div>
                
                <div class="conflict-body">
                    <p class="conflict-description">
                        "This record was modified on another device while you were offline. 
                        Choose how to resolve the conflict:"
                    </p>
                    
                    <div class="conflict-info">
                        <div class="info-row">
                            <span class="label">"Entity:"</span>
                            <span class="value">{conflict.entity_type.clone()}</span>
                        </div>
                        <div class="info-row">
                            <span class="label">"Your version:"</span>
                            <span class="value">"v"{conflict.local_version}</span>
                        </div>
                        <div class="info-row">
                            <span class="label">"Server version:"</span>
                            <span class="value">"v"{conflict.server_version}</span>
                        </div>
                    </div>
                    
                    // Toggle diff view
                    <button 
                        class="toggle-details"
                        on:click=move |_| set_show_details.update(|v| *v = !*v)
                    >
                        {move || if show_details.get() { "▼ Hide differences" } else { "▶ Show differences" }}
                    </button>
                    
                    <Show when=move || show_details.get()>
                        <div class="conflict-diff">
                            <div class="diff-column local">
                                <h4>"Your Version"</h4>
                                <pre>{format_json(&conflict.local_data)}</pre>
                            </div>
                            <div class="diff-column server">
                                <h4>"Server Version"</h4>
                                <pre>{format_json(&conflict.server_data)}</pre>
                            </div>
                        </div>
                    </Show>
                </div>
                
                <div class="conflict-actions">
                    <button 
                        class="btn btn-primary"
                        on:click=keep_mine
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M20 6L9 17l-5-5"/>
                        </svg>
                        "Keep Mine"
                    </button>
                    <button 
                        class="btn btn-secondary"
                        on:click=keep_theirs
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
                            <path d="M21 3v5h-5"/>
                            <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
                        </svg>
                        "Keep Theirs"
                    </button>
                    <button 
                        class="btn btn-success"
                        on:click=try_merge
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="18" cy="18" r="3"/>
                            <circle cx="6" cy="6" r="3"/>
                            <path d="M6 21V9a9 9 0 0 0 9 9"/>
                        </svg>
                        "Merge Both"
                    </button>
                    <button 
                        class="btn btn-ghost"
                        on:click=handle_cancel
                    >
                        "Decide Later"
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Format JSON for display
fn format_json(value: &JsonValue) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
}

/// Field difference type
#[derive(Debug, Clone, PartialEq)]
pub enum DiffType {
    /// Field unchanged
    Same,
    /// Field modified
    Changed,
    /// Field added (exists in one, not other)
    Added,
    /// Field removed (exists in one, not other)
    Removed,
}

/// A single field diff
#[derive(Debug, Clone)]
pub struct FieldDiff {
    pub field: String,
    pub diff_type: DiffType,
    pub local_value: Option<String>,
    pub server_value: Option<String>,
}

/// Calculate field-level differences between local and server data
pub fn calculate_field_diffs(local: &JsonValue, server: &JsonValue) -> Vec<FieldDiff> {
    let mut diffs = Vec::new();

    let local_obj = local.as_object();
    let server_obj = server.as_object();

    match (local_obj, server_obj) {
        (Some(local_map), Some(server_map)) => {
            // Check all fields in local
            for (key, local_val) in local_map {
                // Skip internal fields
                if key == "id" || key == "tenant_id" || key == "created_at" || key == "updated_at" {
                    continue;
                }

                let local_str = format_value(local_val);
                
                if let Some(server_val) = server_map.get(key) {
                    let server_str = format_value(server_val);
                    
                    let diff_type = if local_str == server_str {
                        DiffType::Same
                    } else {
                        DiffType::Changed
                    };
                    
                    diffs.push(FieldDiff {
                        field: key.clone(),
                        diff_type,
                        local_value: Some(local_str),
                        server_value: Some(server_str),
                    });
                } else {
                    // Field only in local
                    diffs.push(FieldDiff {
                        field: key.clone(),
                        diff_type: DiffType::Added,
                        local_value: Some(local_str),
                        server_value: None,
                    });
                }
            }

            // Check for fields only in server
            for (key, server_val) in server_map {
                if key == "id" || key == "tenant_id" || key == "created_at" || key == "updated_at" {
                    continue;
                }
                
                if !local_map.contains_key(key) {
                    diffs.push(FieldDiff {
                        field: key.clone(),
                        diff_type: DiffType::Removed,
                        local_value: None,
                        server_value: Some(format_value(server_val)),
                    });
                }
            }
        }
        _ => {}
    }

    // Sort: changed first, then added/removed, then same
    diffs.sort_by(|a, b| {
        let ord_a = match a.diff_type {
            DiffType::Changed => 0,
            DiffType::Added => 1,
            DiffType::Removed => 1,
            DiffType::Same => 2,
        };
        let ord_b = match b.diff_type {
            DiffType::Changed => 0,
            DiffType::Added => 1,
            DiffType::Removed => 1,
            DiffType::Same => 2,
        };
        ord_a.cmp(&ord_b).then_with(|| a.field.cmp(&b.field))
    });

    diffs
}

/// Format a JSON value as a string for display
fn format_value(value: &JsonValue) -> String {
    match value {
        JsonValue::String(s) => s.clone(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Null => "null".to_string(),
        JsonValue::Array(arr) => format!("[{} items]", arr.len()),
        JsonValue::Object(_) => "[object]".to_string(),
    }
}

/// Visual diff display component
#[component]
pub fn VisualDiff(
    local: JsonValue,
    server: JsonValue,
) -> impl IntoView {
    let diffs = calculate_field_diffs(&local, &server);
    let has_changes = diffs.iter().any(|d| d.diff_type != DiffType::Same);

    view! {
        <div class="visual-diff">
            {if !has_changes {
                view! { <p class="no-changes">"No field-level changes detected."</p> }.into_view()
            } else {
                view! {
                    <table class="diff-table">
                        <thead>
                            <tr>
                                <th>"Field"</th>
                                <th class="local-col">"Your Version"</th>
                                <th class="server-col">"Server Version"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {diffs.into_iter().map(|diff| {
                                let row_class = match diff.diff_type {
                                    DiffType::Changed => "diff-changed",
                                    DiffType::Added => "diff-added",
                                    DiffType::Removed => "diff-removed",
                                    DiffType::Same => "diff-same",
                                };
                                
                                let indicator = match diff.diff_type {
                                    DiffType::Changed => "⚠️",
                                    DiffType::Added => "➕",
                                    DiffType::Removed => "➖",
                                    DiffType::Same => "✓",
                                };

                                view! {
                                    <tr class=row_class>
                                        <td class="field-name">
                                            <span class="indicator">{indicator}</span>
                                            {diff.field}
                                        </td>
                                        <td class="local-value">
                                            {diff.local_value.unwrap_or_else(|| "—".to_string())}
                                        </td>
                                        <td class="server-value">
                                            {diff.server_value.unwrap_or_else(|| "—".to_string())}
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                }.into_view()
            }}
        </div>
    }
}

/// Merge two JSON objects, preferring local values for conflicting fields
/// but keeping server values for fields not in local
fn merge_json_objects(local: &JsonValue, server: &JsonValue) -> JsonValue {
    match (local, server) {
        (JsonValue::Object(local_obj), JsonValue::Object(server_obj)) => {
            let mut merged = server_obj.clone();
            
            // Overwrite with local values where present
            for (key, value) in local_obj {
                // Skip internal fields
                if key == "id" || key == "tenant_id" || key == "created_at" {
                    continue;
                }
                // Prefer local value
                merged.insert(key.clone(), value.clone());
            }
            
            JsonValue::Object(merged)
        }
        // If not objects, prefer local
        _ => local.clone(),
    }
}

/// ConflictQueue component - shows pending conflicts
#[component]
pub fn ConflictQueue(
    /// List of conflicts to resolve
    conflicts: RwSignal<Vec<ConflictData>>,
    /// Callback when a conflict is resolved
    on_resolved: Callback<ResolutionResult>,
) -> impl IntoView {
    view! {
        <Show when=move || !conflicts.get().is_empty()>
            <div class="conflict-queue-banner">
                <div class="banner-content">
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="10"/>
                        <line x1="12" y1="8" x2="12" y2="12"/>
                        <line x1="12" y1="16" x2="12.01" y2="16"/>
                    </svg>
                    <span>
                        {move || {
                            let count = conflicts.get().len();
                            format!("{} conflict{} need{} resolution", 
                                count, 
                                if count == 1 { "" } else { "s" },
                                if count == 1 { "s" } else { "" }
                            )
                        }}
                    </span>
                    <button class="resolve-btn" on:click=move |_| {
                        // TODO: Open conflict resolver for first conflict
                    }>
                        "Resolve Now"
                    </button>
                </div>
            </div>
        </Show>
    }
}
