//! View Switcher Component - Toggle between Table/Kanban/Calendar/Map views
//! Reads available views from ViewDef API and allows switching
//! Persists selected view to localStorage per entity type

use leptos::*;
use crate::api::{fetch_views, ViewDef};

#[component]
pub fn ViewSwitcher(
    #[prop(into)] entity_type: Signal<String>,
    #[prop(into)] on_view_change: Callback<ViewDef>,
) -> impl IntoView {
    // State
    let (views, set_views) = create_signal::<Vec<ViewDef>>(Vec::new());
    let (active_view, set_active_view) = create_signal::<Option<String>>(None);
    let (loading, set_loading) = create_signal(true);
    
    // Fetch available views for this entity type
    let entity_for_effect = entity_type.clone();
    let on_view_change_effect = on_view_change.clone();
    create_effect(move |_| {
        let et = entity_for_effect.get();
        if et.is_empty() { return; }
        
        let et_storage = et.clone();
        let callback = on_view_change_effect.clone();
        
        spawn_local(async move {
            set_loading.set(true);
            
            match fetch_views(&et).await {
                Ok(view_list) => {
                    // If empty, create default fallback views
                    let final_views = if view_list.is_empty() {
                        create_default_views()
                    } else {
                        view_list
                    };
                    
                    // Try to restore saved view from localStorage
                    let saved_view_id = get_saved_view(&et_storage);
                    let selected_view = if let Some(ref saved_id) = saved_view_id {
                        final_views.iter().find(|v| &v.id == saved_id)
                    } else {
                        None
                    };
                    
                    // Use saved view, or default view, or first view
                    let view_to_use = selected_view
                        .or_else(|| final_views.iter().find(|v| v.is_default))
                        .or_else(|| final_views.first());
                    
                    if let Some(view) = view_to_use {
                        set_active_view.set(Some(view.id.clone()));
                        callback.call(view.clone());
                    } else {
                        // Reset active view if none found (prevents stale view type)
                         set_active_view.set(None);
                    }
                    
                    set_views.set(final_views);
                    set_loading.set(false);
                }
                Err(_) => {
                    // On error, use default fallback views
                    let default_views = create_default_views();
                    if let Some(view) = default_views.first() {
                        set_active_view.set(Some(view.id.clone()));
                        callback.call(view.clone());
                    }
                    set_views.set(default_views);
                    set_loading.set(false);
                }
            }
        });
    });
    
    // Get view type icon
    let get_view_icon = |view_type: &str| -> &'static str {
        match view_type {
            "table" => "📋",
            "kanban" => "📊",
            "calendar" => "📅",
            "map" => "🗺️",
            "board" => "📌",
            "timeline" => "📈",
            "gallery" => "🖼️",
            _ => "📄",
        }
    };
    
    view! {
        <div class="view-switcher">
            {move || {
                let current_entity = entity_type.get();
                if loading.get() {
                    view! { <div class="view-switcher-loading">"Loading views..."</div> }.into_view()
                } else {
                    view! {
                        <div class="view-tabs">
                            <For
                                each=move || views.get()
                                key=|v| v.id.clone()
                                children=move |view_def| {
                                    let view_id = view_def.id.clone();
                                    let view_id_click = view_def.id.clone();
                                    let view_id_save = view_def.id.clone();
                                    let view_type = view_def.view_type.clone();
                                    let view_label = view_def.label.clone();
                                    let view_def_click = view_def.clone();
                                    let is_active = move || active_view.get() == Some(view_id.clone());
                                    let icon = get_view_icon(&view_type);
                                    let et_for_save = current_entity.clone();
                                    
                                    view! {
                                        <button
                                            class=move || format!("view-tab {}", if is_active() { "active" } else { "" })
                                            on:click=move |_| {
                                                set_active_view.set(Some(view_id_click.clone()));
                                                on_view_change.call(view_def_click.clone());
                                                // Persist selection to localStorage
                                                save_view(&et_for_save, &view_id_save);
                                            }
                                        >
                                            <span class="view-icon">{icon}</span>
                                            <span class="view-label">{view_label.clone()}</span>
                                        </button>
                                    }
                                }
                            />
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}

/// Get saved view ID from localStorage for an entity type
fn get_saved_view(entity_type: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let key = format!("viewswitcher_{}", entity_type);
    storage.get_item(&key).ok()?
}

/// Save view ID to localStorage for an entity type
fn save_view(entity_type: &str, view_id: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let key = format!("viewswitcher_{}", entity_type);
            let _ = storage.set_item(&key, view_id);
        }
    }
}

/// Create default fallback views when API returns empty
fn create_default_views() -> Vec<ViewDef> {
    vec![
        ViewDef {
            id: "default_table".to_string(),
            entity_type_id: "".to_string(),
            name: "table".to_string(),
            label: "Table".to_string(),
            view_type: "table".to_string(),
            is_default: true,
            is_system: true,
            created_by: None,
            columns: vec![],
            filters: serde_json::json!({}),
            sort: serde_json::json!({}),
            settings: serde_json::json!({}),
        },
        ViewDef {
            id: "default_kanban".to_string(),
            entity_type_id: "".to_string(),
            name: "kanban".to_string(),
            label: "Kanban".to_string(),
            view_type: "kanban".to_string(),
            is_default: false,
            is_system: true,
            created_by: None,
            columns: vec![],
            filters: serde_json::json!({}),
            sort: serde_json::json!({}),
            settings: serde_json::json!({"group_by_field": "status"}),
        },
        ViewDef {
            id: "default_calendar".to_string(),
            entity_type_id: "".to_string(),
            name: "calendar".to_string(),
            label: "Calendar".to_string(),
            view_type: "calendar".to_string(),
            is_default: false,
            is_system: true,
            created_by: None,
            columns: vec![],
            filters: serde_json::json!({}),
            sort: serde_json::json!({}),
            settings: serde_json::json!({"date_field": "created_at"}),
        },
        ViewDef {
            id: "default_map".to_string(),
            entity_type_id: "".to_string(),
            name: "map".to_string(),
            label: "Map".to_string(),
            view_type: "map".to_string(),
            is_default: false,
            is_system: true,
            created_by: None,
            columns: vec![],
            filters: serde_json::json!({}),
            sort: serde_json::json!({}),
            settings: serde_json::json!({}),
        },
    ]
}

