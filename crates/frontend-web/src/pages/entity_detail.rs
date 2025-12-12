//! Generic Entity Detail Page - Metadata-driven
//! 
//! This component displays entity details using FieldDefs metadata.
//! Includes tabs: Details, Related (Task 9), Timeline (Task 10)
//! Includes inline editing support.

use leptos::*;
use leptos_router::*;
use crate::api::{
    fetch_field_defs, fetch_entity, update_entity, fetch_associations, FieldDef, Association,
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
    let (is_editing, set_is_editing) = create_signal(false);
    let (saving, set_saving) = create_signal(false);
    
    // Load data
    let entity_for_load = entity_type.clone();
    let id_for_load = record_id.clone();
    let reload_record = create_rw_signal(0u32); // Trigger for reloading
    
    create_effect(move |_| {
        let etype = entity_for_load();
        let id = id_for_load();
        let _ = reload_record.get(); // Subscribe to reload trigger
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
                <div class="header-left">
                    <a href=move || format!("/app/crm/entity/{}", entity_type()) class="back-link">
                        "← Back to " {entity_label}
                    </a>
                    {move || (!loading.get()).then(|| view! {
                        <h1>{record_title}</h1>
                    })}
                </div>
                <div class="header-actions">
                    {move || (!loading.get() && !is_editing.get()).then(|| view! {
                        <button class="btn btn-primary" on:click=move |_| set_is_editing.set(true)>
                            "Edit"
                        </button>
                    })}
                </div>
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
                            DetailTab::Details => {
                                if is_editing.get() {
                                    view! {
                                        <EditForm 
                                            fields=fields.get()
                                            record=record.get()
                                            entity_type=entity_type()
                                            record_id=record_id()
                                            saving=saving
                                            set_saving=set_saving
                                            set_is_editing=set_is_editing
                                            set_error=set_error
                                            reload_trigger=reload_record
                                        />
                                    }.into_view()
                                } else {
                                    view! {
                                        <DetailsTab fields=fields.get() record=record.get() />
                                    }.into_view()
                                }
                            },
                            DetailTab::Related => view! {
                                <RelatedTab entity_type=entity_type() record_id=record_id() />
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

/// Details tab - renders all fields from metadata (read-only view)
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

/// Edit form component - metadata-driven form for editing
#[component]
fn EditForm(
    fields: Vec<FieldDef>,
    record: serde_json::Value,
    entity_type: String,
    record_id: String,
    saving: ReadSignal<bool>,
    set_saving: WriteSignal<bool>,
    set_is_editing: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
    reload_trigger: RwSignal<u32>,
) -> impl IntoView {
    // Initialize form data from existing record
    let initial_data = record.as_object().cloned().unwrap_or_default();
    let (form_data, set_form_data) = create_signal(initial_data);
    
    // Handle form input change
    let update_form_field = move |field_name: String, value: String| {
        set_form_data.update(|map| {
            map.insert(field_name, serde_json::Value::String(value));
        });
    };
    
    // Handle save
    let etype = entity_type.clone();
    let rid = record_id.clone();
    let on_save = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let etype_clone = etype.clone();
        let rid_clone = rid.clone();
        let body = serde_json::Value::Object(form_data.get());
        
        spawn_local(async move {
            set_saving.set(true);
            match update_entity(&etype_clone, &rid_clone, body).await {
                Ok(_) => {
                    set_is_editing.set(false);
                    reload_trigger.update(|n| *n += 1); // Trigger data reload
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_saving.set(false);
        });
    };
    
    view! {
        <div class="edit-form">
            <form on:submit=on_save>
                <div class="field-grid">
                    {fields.into_iter().filter(|f| !f.is_readonly).map(|field| {
                        let field_name = field.name.clone();
                        let field_label = field.label.clone();
                        let field_type = field.field_type.clone();
                        let is_required = field.is_required;
                        let placeholder = field.placeholder.clone().unwrap_or_default();
                        let options = field.options.clone();
                        
                        // Get initial value from record
                        let initial_value = form_data.get()
                            .get(&field_name)
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        
                        view! {
                            <div class="field-item">
                                <label class="field-label">
                                    {field_label}
                                    {is_required.then(|| " *")}
                                </label>
                                {render_edit_input(
                                    field_name, 
                                    field_type, 
                                    is_required, 
                                    placeholder, 
                                    options,
                                    initial_value,
                                    update_form_field.clone()
                                )}
                            </div>
                        }
                    }).collect_view()}
                </div>
                
                <div class="form-actions">
                    <button 
                        type="button" 
                        class="btn" 
                        on:click=move |_| set_is_editing.set(false)
                        disabled=move || saving.get()
                    >
                        "Cancel"
                    </button>
                    <button 
                        type="submit" 
                        class="btn btn-primary"
                        disabled=move || saving.get()
                    >
                        {move || if saving.get() { "Saving..." } else { "Save Changes" }}
                    </button>
                </div>
            </form>
        </div>
    }
}

/// Render an editable input field with initial value
fn render_edit_input(
    field_name: String,
    field_type: String, 
    is_required: bool,
    placeholder: String,
    options: Option<serde_json::Value>,
    initial_value: String,
    on_change: impl Fn(String, String) + Clone + 'static,
) -> impl IntoView {
    let name = field_name.clone();
    let on_input = move |ev: web_sys::Event| {
        let target = event_target::<web_sys::HtmlInputElement>(&ev);
        on_change(name.clone(), target.value());
    };
    
    match field_type.as_str() {
        "email" => view! {
            <input 
                type="email" 
                value=initial_value
                placeholder=placeholder 
                required=is_required 
                on:input=on_input
                class="form-input"
            />
        }.into_view(),
        "phone" => view! {
            <input 
                type="tel" 
                value=initial_value
                placeholder=placeholder 
                required=is_required
                on:input=on_input
                class="form-input"
            />
        }.into_view(),
        "number" | "integer" | "currency" => view! {
            <input 
                type="number" 
                value=initial_value
                placeholder=placeholder 
                required=is_required
                on:input=on_input
                class="form-input"
            />
        }.into_view(),
        "textarea" | "longtext" => view! {
            <textarea 
                placeholder=placeholder 
                required=is_required
                on:input=on_input
                class="form-input"
            >{initial_value}</textarea>
        }.into_view(),
        "date" => view! {
            <input 
                type="date" 
                value=initial_value
                required=is_required
                on:input=on_input
                class="form-input"
            />
        }.into_view(),
        "select" => {
            let select_options: Vec<(String, String)> = options
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default()
                .into_iter()
                .filter_map(|opt| {
                    let value = opt.get("value")?.as_str()?.to_string();
                    let label = opt.get("label")?.as_str()?.to_string();
                    Some((value, label))
                })
                .collect();
            
            view! {
                <select required=is_required on:change=on_input class="form-input">
                    <option value="">-- Select --</option>
                    {select_options.into_iter().map(|(value, label)| {
                        let selected = value == initial_value;
                        view! { <option value=value selected=selected>{label}</option> }
                    }).collect_view()}
                </select>
            }.into_view()
        },
        _ => view! {
            <input 
                type="text" 
                value=initial_value
                placeholder=placeholder 
                required=is_required
                on:input=on_input
                class="form-input"
            />
        }.into_view(),
    }
}

/// Related tab - displays associations (linked records)
#[component]
fn RelatedTab(entity_type: String, record_id: String) -> impl IntoView {
    let (associations, set_associations) = create_signal(Vec::<Association>::new());
    let (loading, set_loading) = create_signal(true);
    let (show_link_modal, set_show_link_modal) = create_signal(false);
    let reload_trigger = create_rw_signal(0u32);
    
    // Fetch associations on mount and when reload_trigger changes
    let etype = entity_type.clone();
    let rid = record_id.clone();
    create_effect(move |_| {
        let etype = etype.clone();
        let rid = rid.clone();
        let _ = reload_trigger.get(); // Subscribe to reload
        spawn_local(async move {
            set_loading.set(true);
            match fetch_associations(&etype, &rid).await {
                Ok(assocs) => set_associations.set(assocs),
                Err(e) => logging::log!("Failed to fetch associations: {}", e),
            }
            set_loading.set(false);
        });
    });
    
    // Entity type for linking
    let source_entity = entity_type.clone();
    let source_id = record_id.clone();
    
    view! {
        <div class="related-tab">
            <div class="related-header">
                <h3>"Related Records"</h3>
                <button class="btn btn-secondary" on:click=move |_| set_show_link_modal.set(true)>
                    "+ Link Record"
                </button>
            </div>
            
            {move || loading.get().then(|| view! {
                <p class="loading">"Loading associations..."</p>
            })}
            
            {move || (!loading.get()).then(|| {
                let assocs = associations.get();
                if assocs.is_empty() {
                    view! {
                        <div class="empty-state">
                            <p>"No related records yet."</p>
                            <p class="hint">"Click '+ Link Record' to connect this record to another."</p>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="associations-list">
                            {assocs.into_iter().map(|assoc| {
                                view! {
                                    <div class="association-item">
                                        <span class="target-id">{assoc.target_id.clone()}</span>
                                        {assoc.role.map(|r| view! { <span class="role">{r}</span> })}
                                        {assoc.is_primary.then(|| view! { <span class="badge">"Primary"</span> })}
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_view()
                }
            })}
            
            // Link Modal
            {move || show_link_modal.get().then(|| {
                view! {
                    <LinkRecordModal 
                        source_entity=source_entity.clone()
                        source_id=source_id.clone()
                        set_show_modal=set_show_link_modal
                        reload_trigger=reload_trigger
                    />
                }
            })}
        </div>
    }
}

/// Modal for linking records
#[component]
fn LinkRecordModal(
    source_entity: String,
    source_id: String,
    set_show_modal: WriteSignal<bool>,
    reload_trigger: RwSignal<u32>,
) -> impl IntoView {
    // Available entity types to link to
    let linkable_entities = vec![
        ("contact", "Contact"),
        ("company", "Company"),
        ("deal", "Deal"),
    ];
    
    let (selected_entity, set_selected_entity) = create_signal(String::new());
    let (search_results, set_search_results) = create_signal(Vec::<serde_json::Value>::new());
    let (searching, set_searching) = create_signal(false);
    let (linking, set_linking) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);
    
    // Search records when entity type changes
    let handle_entity_change = move |ev: web_sys::Event| {
        let target = event_target::<web_sys::HtmlSelectElement>(&ev);
        let entity = target.value();
        set_selected_entity.set(entity.clone());
        set_search_results.set(vec![]);
        
        if entity.is_empty() { return; }
        
        spawn_local(async move {
            set_searching.set(true);
            match crate::api::fetch_entity_list(&entity).await {
                Ok(response) => set_search_results.set(response.data),
                Err(e) => set_error.set(Some(e)),
            }
            set_searching.set(false);
        });
    };
    
    // Link record
    let src_entity = source_entity.clone();
    let src_id = source_id.clone();
    let link_record = move |target_id: String| {
        let src_entity = src_entity.clone();
        let src_id = src_id.clone();
        let target_entity = selected_entity.get();
        
        spawn_local(async move {
            set_linking.set(true);
            
            // Get association def for this entity pair
            match crate::api::fetch_association_defs(&src_entity).await {
                Ok(defs) => {
                    // Find matching def
                    if let Some(def) = defs.iter().find(|d| d.target_entity == target_entity) {
                        match crate::api::create_association(&def.id, &src_id, &target_id).await {
                            Ok(_) => {
                                set_show_modal.set(false);
                                reload_trigger.update(|n| *n += 1);
                            }
                            Err(e) => set_error.set(Some(e)),
                        }
                    } else {
                        set_error.set(Some(format!("No association definition found for {} -> {}", src_entity, target_entity)));
                    }
                }
                Err(e) => set_error.set(Some(e)),
            }
            
            set_linking.set(false);
        });
    };
    
    view! {
        <div class="modal-overlay" on:click=move |_| set_show_modal.set(false)>
            <div class="modal link-modal" on:click=move |ev| ev.stop_propagation()>
                <h2>"Link Record"</h2>
                
                {move || error.get().map(|e| view! {
                    <div class="error-banner">{e}</div>
                })}
                
                <div class="form-group">
                    <label>"Select Entity Type"</label>
                    <select class="form-input" on:change=handle_entity_change>
                        <option value="">"-- Choose type --"</option>
                        {linkable_entities.iter().map(|(value, label)| {
                            view! { <option value=*value>{*label}</option> }
                        }).collect_view()}
                    </select>
                </div>
                
                {move || searching.get().then(|| view! {
                    <p class="loading">"Searching..."</p>
                })}
                
                {move || {
                    let results = search_results.get();
                    (!results.is_empty() && !searching.get()).then(|| {
                        view! {
                            <div class="search-results">
                                <label>"Select a record to link:"</label>
                                <div class="results-list">
                                    {results.into_iter().map(|record| {
                                        let id = record.get("id")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or_default()
                                            .to_string();
                                        let display_name = get_record_display_name(&record);
                                        let id_for_click = id.clone();
                                        let link_fn = link_record.clone();
                                        
                                        view! {
                                            <div class="result-item" on:click=move |_| link_fn(id_for_click.clone())>
                                                <span class="record-name">{display_name}</span>
                                                <span class="record-id">{id.clone()}</span>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        }
                    })
                }}
                
                {move || linking.get().then(|| view! {
                    <p class="linking">"Linking..."</p>
                })}
                
                <div class="form-actions">
                    <button 
                        class="btn" 
                        on:click=move |_| set_show_modal.set(false)
                        disabled=move || linking.get()
                    >
                        "Cancel"
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Get display name from a record
fn get_record_display_name(record: &serde_json::Value) -> String {
    // Try first_name + last_name
    let first = record.get("first_name").and_then(|v| v.as_str()).unwrap_or("");
    let last = record.get("last_name").and_then(|v| v.as_str()).unwrap_or("");
    if !first.is_empty() || !last.is_empty() {
        return format!("{} {}", first, last).trim().to_string();
    }
    // Try name
    if let Some(name) = record.get("name").and_then(|v| v.as_str()) {
        return name.to_string();
    }
    // Try title
    if let Some(title) = record.get("title").and_then(|v| v.as_str()) {
        return title.to_string();
    }
    "Unnamed".to_string()
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
