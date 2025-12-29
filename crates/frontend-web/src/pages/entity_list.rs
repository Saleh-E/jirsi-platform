//! Cinematic Entity List - Ultimate Edition
//! Integrates EditableTable, Kanban, ViewSwitcher, CreateModal, and FilterBuilder
//! All wrapped in Obsidian Glass Styles

use leptos::*;
use leptos_router::*;
use crate::api::{
    fetch_field_defs, fetch_entity_list, FieldDef, ViewDef
};
use crate::components::editable_table::EditableTable;
use crate::components::view_switcher::ViewSwitcher;
use crate::components::create_modal::{CreateModal, CreatedRecord};
use crate::components::kanban::{KanbanView, KanbanConfig};
use crate::components::filter_builder::{FilterBuilder, FilterChipBar, FilterCondition};

#[component]
pub fn EntityListPage() -> impl IntoView {
    let params = use_params_map();
    let entity_type = move || params.with(|p| p.get("entity").cloned().unwrap_or_default());
    
    // Signals
    let (fields, set_fields) = create_signal(Vec::<FieldDef>::new());
    
    // Data Management
    // raw_data holds the complete fetched list
    let raw_data = create_rw_signal(Vec::<serde_json::Value>::new());
    // data holds the filtered list passed to views
    let data = create_rw_signal(Vec::<serde_json::Value>::new());
    
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal(Option::<String>::None);
    
    // View State
    let (active_view, set_active_view) = create_signal::<Option<ViewDef>>(None);
    let (show_create, set_show_create) = create_signal(false);
    
    // Filter State
    let (filters, set_filters) = create_signal(Vec::<FilterCondition>::new());
    let show_filter_popover = create_rw_signal(false);
    
    // Derived View Type
    let current_view_type = move || {
        active_view.get()
            .map(|v| v.view_type)
            .unwrap_or_else(|| "table".to_string())
    };
    
    // Filter Logic Effect
    create_effect(move |_| {
        let all_records = raw_data.get();
        let current_filters = filters.get();
        
        if current_filters.is_empty() {
            data.set(all_records);
        } else {
            let filtered: Vec<serde_json::Value> = all_records.into_iter().filter(|record| {
                current_filters.iter().all(|cond| {
                    let field_val = record.get(&cond.field)
                        .and_then(|v| v.as_str())
                        .or_else(|| record.get(&cond.field).map(|v| if v.is_null() { "" } else { "value" })) // simple check for non-string
                        .unwrap_or("")
                        .to_string();
                        
                    let target_val = &cond.value;
                    
                    match cond.operator.as_str() {
                        "contains" => field_val.to_lowercase().contains(&target_val.to_lowercase()),
                        "equals" => field_val.to_lowercase() == target_val.to_lowercase(),
                        "not_equals" => field_val.to_lowercase() != target_val.to_lowercase(),
                        "starts_with" => field_val.to_lowercase().starts_with(&target_val.to_lowercase()),
                        "ends_with" => field_val.to_lowercase().ends_with(&target_val.to_lowercase()),
                        "is_empty" => field_val.trim().is_empty(),
                        "is_not_empty" => !field_val.trim().is_empty(),
                        // Basic comparison for numbers/dates as strings (simplified)
                        "gt" => field_val > *target_val,
                        "lt" => field_val < *target_val,
                        _ => true,
                    }
                })
            }).collect();
            
            data.set(filtered);
        }
    });

    // Load Data
    let entity_for_load = entity_type.clone();
    create_effect(move |_| {
        let etype = entity_for_load();
        if etype.is_empty() { return; }
        
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            
            // 1. Fetch Field Defs
            if let Ok(f) = fetch_field_defs(&etype).await {
                set_fields.set(f);
            }
            
            // 2. Fetch Entity Data for Table
            if current_view_type() == "table" {
                match fetch_entity_list(&etype).await {
                    Ok(response) => {
                        raw_data.set(response.data);
                        // Filter effect will trigger and populate 'data'
                    },
                    Err(e) => set_error.set(Some(e)),
                }
            }
            
            set_loading.set(false);
        });
    });
    
    let navigate = use_navigate();
    let nav_store = store_value(navigate);
    
    // Handle new record creation
    let handle_created = move |_record: CreatedRecord| {
        set_show_create.set(false);
        // Reload data
        let etype = entity_type();
        spawn_local(async move {
             if let Ok(response) = fetch_entity_list(&etype).await {
                raw_data.set(response.data);
            }
        });
    };
    
    view! {
        <div class="h-full flex flex-col p-8 overflow-hidden animate-fade-in text-white relative">
            // Filter Popover
            <FilterBuilder 
                fields=fields.into()
                show_popover=show_filter_popover
                on_add_filter=Callback::new(move |cond: FilterCondition| {
                    set_filters.update(|f| f.push(cond));
                })
            />

            // Header
            <header class="flex flex-col gap-4 mb-6">
                <div class="flex items-center justify-between">
                    <div class="flex flex-col gap-2">
                        <h1 class="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-white to-zinc-400 tracking-tight capitalize">
                            {entity_type}
                        </h1>
                        
                         <div class="">
                            <ViewSwitcher 
                                entity_type=Signal::derive(move || entity_type())
                                on_view_change=move |v| {
                                    set_active_view.set(Some(v));
                                }
                            />
                        </div>
                    </div>
                    
                    <div class="flex gap-3 items-center">
                        <button 
                            class="ui-btn ui-btn-secondary"
                            on:click=move |_| show_filter_popover.set(true)
                        >
                             <i class="fa-solid fa-filter"></i> "Filter"
                             {move || if !filters.get().is_empty() {
                                 view! { <span class="ml-1 ui-badge ui-badge-info">{filters.get().len()}</span> }.into_view()
                             } else {
                                 view! {}.into_view()
                             }}
                        </button>
                        <button 
                            class="ui-btn ui-btn-primary"
                            on:click=move |_| set_show_create.set(true)
                        >
                            <i class="fa-solid fa-plus"></i> "New " {move || entity_type().to_uppercase()}
                        </button>
                    </div>
                </div>
                
                // Active Chips
                <FilterChipBar 
                    filters=filters.into()
                    on_remove=Callback::new(move |id| {
                         set_filters.update(|list| list.retain(|f| f.id != id));
                    })
                    on_clear_all=Callback::new(move |_| {
                        set_filters.set(Vec::new());
                    })
                />
            </header>
            
            // Content
            <div class="flex-1 overflow-hidden flex flex-col relative"> 
                 {move || loading.get().then(|| view! {
                    <div class="absolute inset-0 flex items-center justify-center z-10 bg-black/50 backdrop-blur-sm">
                        <div class="text-zinc-500 animate-pulse flex flex-col items-center">
                            <span class="text-2xl mb-4">"⚡"</span>
                            "Syncing Neural Data..."
                        </div>
                    </div>
                })}
                
                {move || match current_view_type().as_str() {
                    "kanban" => {
                        let settings = active_view.get().map(|v| v.settings).unwrap_or(serde_json::json!({}));
                        let group_by = settings.get("group_by_field").and_then(|s| s.as_str()).unwrap_or("status").to_string();
                        
                        view! {
                            <KanbanView 
                                entity_type=entity_type()
                                config=KanbanConfig {
                                    group_by_field: group_by,
                                    card_title_field: "name".to_string(), 
                                    card_subtitle_field: None,
                                    card_fields: vec![],
                                    column_order: None
                                }
                                field_options=vec![] 
                            />
                        }.into_view()
                    },
                    _ => {
                        // Table View (Default)
                        view! {
                             <div class="h-full overflow-hidden">
                                <EditableTable
                                    entity_type={entity_type()}
                                    columns={fields.get().into_iter().filter(|f| f.show_in_list).collect()}
                                    data={data}
                                    density="comfortable".to_string()
                                    on_row_click={move |id| {
                                        nav_store.get_value()(&format!("/app/crm/entity/{}/{}", entity_type(), id), Default::default());
                                    }}
                                />
                             </div>
                        }.into_view()
                    }
                }}
            </div>
            
            // Create Modal
            {move || show_create.get().then(|| view! {
                <CreateModal
                    entity_type=entity_type()
                    entity_label=entity_type() 
                    on_close=move |_| set_show_create.set(false)
                    on_created=handle_created
                />
            })}
        </div>
    }
}
