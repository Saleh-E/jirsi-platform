//! Audit Timeline Component
//!
//! Displays the complete history of changes to an entity using event sourcing.
//! Features:
//! - Timeline visualization of all changes
//! - User attribution for each change
//! - Field-level diff display
//! - Restore to previous version functionality
//! - Export as CSV/PDF

use leptos::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;

/// Audit event representing a single change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event ID
    pub id: Uuid,
    /// Entity ID this event belongs to
    pub entity_id: Uuid,
    /// Entity type (e.g., "contact", "deal")
    pub entity_type: String,
    /// Event type (e.g., "Created", "FieldUpdated", "Deleted")
    pub event_type: String,
    /// User who made the change
    pub user_id: Uuid,
    /// User name for display
    pub user_name: String,
    /// When the change occurred
    pub occurred_at: DateTime<Utc>,
    /// Version number after this change
    pub version: u64,
    /// Changed fields (field_name -> {old_value, new_value})
    pub changes: JsonValue,
}

/// Field change for display
#[derive(Debug, Clone)]
pub struct FieldChange {
    pub field_name: String,
    pub old_value: String,
    pub new_value: String,
}

/// Audit Timeline Component
#[component]
pub fn AuditTimeline(
    /// Entity ID to show audit trail for
    entity_id: Uuid,
    /// Entity type
    entity_type: String,
    /// Optional callback when restore is clicked
    #[prop(optional)]
    on_restore: Option<Callback<u64>>,
) -> impl IntoView {
    let (events, set_events) = create_signal::<Vec<AuditEvent>>(vec![]);
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (selected_event, set_selected_event) = create_signal::<Option<AuditEvent>>(None);
    let (filter_user, set_filter_user) = create_signal::<Option<Uuid>>(None);
    
    // Load audit events on mount
    let entity_id_clone = entity_id;
    let entity_type_clone = entity_type.clone();
    create_effect(move |_| {
        let entity_id = entity_id_clone;
        let entity_type = entity_type_clone.clone();
        
        spawn_local(async move {
            set_loading.set(true);
            
            match fetch_audit_events(entity_id, &entity_type).await {
                Ok(data) => {
                    set_events.set(data);
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            
            set_loading.set(false);
        });
    });
    
    // Filter events by user
    let filtered_events = move || {
        let all = events.get();
        match filter_user.get() {
            Some(user_id) => all.into_iter().filter(|e| e.user_id == user_id).collect(),
            None => all,
        }
    };
    
    // Get unique users for filter dropdown
    let unique_users = move || {
        let all = events.get();
        let mut users: Vec<(Uuid, String)> = all.iter()
            .map(|e| (e.user_id, e.user_name.clone()))
            .collect();
        users.sort_by(|a, b| a.1.cmp(&b.1));
        users.dedup_by(|a, b| a.0 == b.0);
        users
    };
    
    // Export handler
    let export_csv = move |_| {
        let data = events.get();
        let csv = generate_csv(&data);
        download_file("audit_history.csv", &csv, "text/csv");
    };
    
    view! {
        <div class="audit-timeline">
            <div class="audit-header">
                <h3>
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="10"/>
                        <polyline points="12 6 12 12 16 14"/>
                    </svg>
                    "Activity History"
                </h3>
                
                <div class="audit-controls">
                    // User filter
                    <select 
                        class="filter-select"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            if value.is_empty() {
                                set_filter_user.set(None);
                            } else if let Ok(uid) = Uuid::parse_str(&value) {
                                set_filter_user.set(Some(uid));
                            }
                        }
                    >
                        <option value="">"All Users"</option>
                        {move || unique_users().into_iter().map(|(id, name)| {
                            view! {
                                <option value={id.to_string()}>{name}</option>
                            }
                        }).collect_view()}
                    </select>
                    
                    // Export button
                    <button class="export-btn" on:click=export_csv>
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                            <polyline points="7 10 12 15 17 10"/>
                            <line x1="12" y1="15" x2="12" y2="3"/>
                        </svg>
                        "Export"
                    </button>
                </div>
            </div>
            
            <Show when=move || loading.get()>
                <div class="audit-loading">
                    <div class="spinner"></div>
                    "Loading history..."
                </div>
            </Show>
            
            <Show when=move || error.get().is_some()>
                <div class="audit-error">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>
            
            <Show when=move || !loading.get() && error.get().is_none()>
                <div class="audit-list">
                    {move || {
                        let events = filtered_events();
                        if events.is_empty() {
                            view! {
                                <div class="audit-empty">
                                    "No activity recorded yet"
                                </div>
                            }.into_view()
                        } else {
                            events.into_iter().map(|event| {
                                let event_clone = event.clone();
                                let event_for_restore = event.clone();
                                let on_restore_clone = on_restore.clone();
                                
                                view! {
                                    <div class={format!("audit-item {}", event_type_class(&event.event_type))}>
                                        <div class="audit-icon">
                                            {event_icon(&event.event_type)}
                                        </div>
                                        
                                        <div class="audit-content">
                                            <div class="audit-meta">
                                                <span class="audit-user">{event.user_name.clone()}</span>
                                                <span class="audit-action">{event_action_text(&event.event_type)}</span>
                                                <span class="audit-time">{format_relative_time(event.occurred_at)}</span>
                                            </div>
                                            
                                            <div class="audit-changes">
                                                {render_changes(&event.changes)}
                                            </div>
                                            
                                            <div class="audit-version">
                                                "v"{event.version}
                                            </div>
                                        </div>
                                        
                                        <div class="audit-actions">
                                            <button 
                                                class="view-btn"
                                                title="View details"
                                                on:click=move |_| set_selected_event.set(Some(event_clone.clone()))
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                                                    <circle cx="12" cy="12" r="3"/>
                                                </svg>
                                            </button>
                                            
                                            {move || {
                                                if on_restore_clone.is_some() {
                                                    let version = event_for_restore.version;
                                                    let callback = on_restore_clone.clone();
                                                    view! {
                                                        <button 
                                                            class="restore-btn"
                                                            title="Restore to this version"
                                                            on:click=move |_| {
                                                                if let Some(cb) = callback.as_ref() {
                                                                    cb.call(version);
                                                                }
                                                            }
                                                        >
                                                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
                                                                <path d="M3 3v5h5"/>
                                                            </svg>
                                                        </button>
                                                    }.into_view()
                                                } else {
                                                    view! { <span></span> }.into_view()
                                                }
                                            }}
                                        </div>
                                    </div>
                                }
                            }).collect_view()
                        }
                    }}
                </div>
            </Show>
            
            // Detail modal
            <Show when=move || selected_event.get().is_some()>
                {move || {
                    let event = selected_event.get().unwrap();
                    view! {
                        <div class="audit-modal-overlay" on:click=move |_| set_selected_event.set(None)>
                            <div class="audit-modal" on:click=|e| e.stop_propagation()>
                                <div class="modal-header">
                                    <h4>"Event Details"</h4>
                                    <button class="close-btn" on:click=move |_| set_selected_event.set(None)>
                                        "×"
                                    </button>
                                </div>
                                <div class="modal-body">
                                    <div class="detail-row">
                                        <span class="label">"Event ID:"</span>
                                        <span class="value mono">{event.id.to_string()}</span>
                                    </div>
                                    <div class="detail-row">
                                        <span class="label">"Type:"</span>
                                        <span class="value">{event.event_type.clone()}</span>
                                    </div>
                                    <div class="detail-row">
                                        <span class="label">"User:"</span>
                                        <span class="value">{event.user_name.clone()}</span>
                                    </div>
                                    <div class="detail-row">
                                        <span class="label">"Time:"</span>
                                        <span class="value">{event.occurred_at.to_rfc3339()}</span>
                                    </div>
                                    <div class="detail-row">
                                        <span class="label">"Version:"</span>
                                        <span class="value">{event.version}</span>
                                    </div>
                                    <div class="detail-section">
                                        <span class="label">"Changes:"</span>
                                        <pre class="changes-json">{format_json(&event.changes)}</pre>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                }}
            </Show>
        </div>
    }
}

/// Fetch audit events from API
async fn fetch_audit_events(entity_id: Uuid, entity_type: &str) -> Result<Vec<AuditEvent>, String> {
    use gloo_net::http::Request;
    use crate::api::get_api_base;
    
    let url = format!("{}/entities/{}/{}/audit", get_api_base(), entity_type, entity_id);
    
    let response = Request::get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !response.ok() {
        return Err(format!("HTTP {}", response.status()));
    }
    
    let events: Vec<AuditEvent> = response.json().await.map_err(|e| e.to_string())?;
    Ok(events)
}

/// Get CSS class for event type
fn event_type_class(event_type: &str) -> &'static str {
    match event_type {
        s if s.contains("Created") => "event-created",
        s if s.contains("Deleted") => "event-deleted",
        s if s.contains("Updated") || s.contains("Changed") => "event-updated",
        _ => "event-default",
    }
}

/// Get icon for event type
fn event_icon(event_type: &str) -> &'static str {
    match event_type {
        s if s.contains("Created") => "➕",
        s if s.contains("Deleted") => "🗑️",
        s if s.contains("Updated") || s.contains("Changed") => "✏️",
        s if s.contains("Assigned") => "👤",
        s if s.contains("Stage") => "📊",
        _ => "📝",
    }
}

/// Get action text for event type
fn event_action_text(event_type: &str) -> &'static str {
    match event_type {
        s if s.contains("Created") => "created this record",
        s if s.contains("Deleted") => "deleted this record",
        s if s.contains("Updated") => "updated fields",
        s if s.contains("StageChanged") => "changed the stage",
        s if s.contains("Assigned") => "assigned",
        s if s.contains("ValueAdded") => "added value",
        _ => "made changes",
    }
}

/// Format relative time
fn format_relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(dt);
    
    if diff.num_minutes() < 1 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{} min ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{} hours ago", diff.num_hours())
    } else if diff.num_days() < 7 {
        format!("{} days ago", diff.num_days())
    } else {
        dt.format("%b %d, %Y").to_string()
    }
}

/// Render field changes
fn render_changes(changes: &JsonValue) -> impl IntoView {
    match changes {
        JsonValue::Object(obj) => {
            let items: Vec<_> = obj.iter().map(|(field, change)| {
                let old_val = change.get("old").map(|v| format_value(v)).unwrap_or_else(|| "—".to_string());
                let new_val = change.get("new").map(|v| format_value(v)).unwrap_or_else(|| "—".to_string());
                
                view! {
                    <div class="change-item">
                        <span class="field-name">{field.clone()}</span>
                        <span class="old-value">{old_val}</span>
                        <span class="arrow">"→"</span>
                        <span class="new-value">{new_val}</span>
                    </div>
                }
            }).collect();
            
            view! {
                <div class="changes-list">
                    {items}
                </div>
            }.into_view()
        }
        _ => view! { <span></span> }.into_view(),
    }
}

/// Format JSON value for display
fn format_value(v: &JsonValue) -> String {
    match v {
        JsonValue::Null => "—".to_string(),
        JsonValue::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => {
            if s.len() > 50 {
                format!("{}...", &s[..50])
            } else {
                s.clone()
            }
        }
        _ => v.to_string(),
    }
}

/// Format JSON for display
fn format_json(v: &JsonValue) -> String {
    serde_json::to_string_pretty(v).unwrap_or_else(|_| "{}".to_string())
}

/// Generate CSV from events
fn generate_csv(events: &[AuditEvent]) -> String {
    let mut csv = String::from("Event ID,Type,User,Time,Version,Changes\n");
    
    for event in events {
        csv.push_str(&format!(
            "{},{},{},{},{},\"{}\"\n",
            event.id,
            event.event_type,
            event.user_name,
            event.occurred_at.to_rfc3339(),
            event.version,
            format_json(&event.changes).replace("\"", "\"\"")
        ));
    }
    
    csv
}

/// Download file helper - browser download via data URL
fn download_file(filename: &str, content: &str, _mime_type: &str) {
    use wasm_bindgen::JsCast;
    use base64::Engine;
    
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            // Create data URL
            let encoded = base64::engine::general_purpose::STANDARD.encode(content.as_bytes());
            let data_url = format!("data:text/csv;base64,{}", encoded);
            
            if let Ok(a) = document.create_element("a") {
                let _ = a.set_attribute("href", &data_url);
                let _ = a.set_attribute("download", filename);
                if let Some(body) = document.body() {
                    let _ = body.append_child(&a);
                    if let Some(el) = a.dyn_ref::<web_sys::HtmlElement>() {
                        el.click();
                    }
                    let _ = body.remove_child(&a);
                }
            }
        }
    }
}
