//! Generic Entity Detail Page - Metadata-driven
//! 
//! This component displays entity details using FieldDefs metadata.
//! Includes tabs: Details, Related (Task 9), Timeline (Task 10)

use leptos::*;
use leptos_router::*;
use crate::api::{
    fetch_field_defs, fetch_entity, FieldDef,
};

/// Tab options for the detail page
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Details,
    Related,
    Timeline,
}

/// Main entity detail page component
#[component]
pub fn EntityDetailPage() -> impl IntoView {
    let params = use_params_map();
    
    let entity_type = move || params.with(|p| p.get("entity").cloned().unwrap_or_default());
    let record_id = move || params.with(|p| p.get("id").cloned().unwrap_or_default());
    
    // Signals
    let (fields, set_fields) = create_signal(Vec::<FieldDef>::new());
    let (record, set_record) = create_signal(serde_json::Value::Null);
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal(Option::<String>::None);
    let (active_tab, set_active_tab) = create_signal(DetailTab::Details);
    
    // Load data
    let entity_for_load = entity_type.clone();
    let id_for_load = record_id.clone();
    create_effect(move |_| {
        let etype = entity_for_load();
        let id = id_for_load();
        if etype.is_empty() || id.is_empty() { return; }
        
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            
            // Fetch field definitions
            match fetch_field_defs(&etype).await {
                Ok(f) => set_fields.set(f),
                Err(e) => logging::log!("Failed to fetch fields: {}", e),
            }
            
            // Fetch entity record
            match fetch_entity(&etype, &id).await {
                Ok(r) => set_record.set(r),
                Err(e) => set_error.set(Some(e)),
            }
            
            set_loading.set(false);
        });
    });
    
    // Get display title (first_name + last_name or name field)
    let record_title = move || {
        let r = record.get();
        // Try first_name + last_name (for contacts)
        let first = r.get("first_name").and_then(|v| v.as_str()).unwrap_or("");
        let last = r.get("last_name").and_then(|v| v.as_str()).unwrap_or("");
        if !first.is_empty() || !last.is_empty() {
            return format!("{} {}", first, last).trim().to_string();
        }
        // Try name (for companies, deals)
        if let Some(name) = r.get("name").and_then(|v| v.as_str()) {
            return name.to_string();
        }
        // Try title (for tasks, properties)
        if let Some(title) = r.get("title").and_then(|v| v.as_str()) {
            return title.to_string();
        }
        "Record".to_string()
    };
    
    // Entity label for display
    let entity_label = move || {
        match entity_type().as_str() {
            "contact" => "Contact",
            "company" => "Company",
            "deal" => "Deal",
            "task" => "Task",
            "property" => "Property",
            _ => "Record"
        }.to_string()
    };
    
    view! {
        <div class="entity-detail-page">
            <header class="page-header">
                <a href=move || format!("/app/crm/entity/{}", entity_type()) class="back-link">
                    "← Back to " {entity_label}
                </a>
                {move || (!loading.get()).then(|| view! {
                    <h1>{record_title}</h1>
                })}
            </header>
            
            // Error display
            {move || error.get().map(|e| view! {
                <div class="error-banner">{e}</div>
            })}
            
            // Loading state
            {move || loading.get().then(|| view! {
                <div class="loading">"Loading..."</div>
            })}
            
            // Main content with tabs
            {move || (!loading.get()).then(|| view! {
                <div class="page-content">
                    // Tab navigation
                    <div class="detail-tabs">
                        <button 
                            class=move || if active_tab.get() == DetailTab::Details { "tab active" } else { "tab" }
                            on:click=move |_| set_active_tab.set(DetailTab::Details)
                        >
                            "Details"
                        </button>
                        <button 
                            class=move || if active_tab.get() == DetailTab::Related { "tab active" } else { "tab" }
                            on:click=move |_| set_active_tab.set(DetailTab::Related)
                        >
                            "Related"
                        </button>
                        <button 
                            class=move || if active_tab.get() == DetailTab::Timeline { "tab active" } else { "tab" }
                            on:click=move |_| set_active_tab.set(DetailTab::Timeline)
                        >
                            "Timeline"
                        </button>
                    </div>
                    
                    // Tab content
                    <div class="detail-content">
                        {move || match active_tab.get() {
                            DetailTab::Details => view! {
                                <DetailsTab fields=fields.get() record=record.get() />
                            }.into_view(),
                            DetailTab::Related => view! {
                                <RelatedTab />
                            }.into_view(),
                            DetailTab::Timeline => view! {
                                <TimelineTab />
                            }.into_view(),
                        }}
                    </div>
                </div>
            })}
        </div>
    }
}

/// Details tab - renders all fields from metadata
#[component]
fn DetailsTab(fields: Vec<FieldDef>, record: serde_json::Value) -> impl IntoView {
    view! {
        <div class="details-tab">
            <div class="field-grid">
                {fields.into_iter().map(|field| {
                    let value = record.get(&field.name)
                        .map(|v| format_field_display(v, &field.field_type))
                        .unwrap_or_else(|| "—".to_string());
                    
                    view! {
                        <div class="field-item">
                            <label class="field-label">{field.label}</label>
                            <div class="field-value">{value}</div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

/// Related tab - placeholder for Task 9
#[component]
fn RelatedTab() -> impl IntoView {
    view! {
        <div class="related-tab">
            <p class="placeholder">"Related records (associations) will be displayed here."</p>
            <p class="placeholder-hint">"Task 9: Link contacts to companies, deals, etc."</p>
        </div>
    }
}

/// Timeline tab - placeholder for Task 10
#[component]
fn TimelineTab() -> impl IntoView {
    view! {
        <div class="timeline-tab">
            <p class="placeholder">"Activity timeline (interactions) will be displayed here."</p>
            <p class="placeholder-hint">"Task 10: Show calls, emails, notes, etc."</p>
        </div>
    }
}

/// Format a field value for display based on field type
fn format_field_display(value: &serde_json::Value, field_type: &str) -> String {
    match value {
        serde_json::Value::Null => "—".to_string(),
        serde_json::Value::String(s) if s.is_empty() => "—".to_string(),
        serde_json::Value::String(s) => {
            match field_type {
                "email" => format!("📧 {}", s),
                "phone" => format!("📞 {}", s),
                "url" => format!("🔗 {}", s),
                _ => s.clone(),
            }
        }
        serde_json::Value::Number(n) => {
            if field_type == "currency" || field_type == "money" {
                format!("${}", n)
            } else {
                n.to_string()
            }
        }
        serde_json::Value::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
        other => other.to_string(),
    }
}
