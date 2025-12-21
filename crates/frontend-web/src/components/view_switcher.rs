//! View Switcher Component - Toggle between Table/Kanban/Calendar/Map views
//! Reads available views from ViewDef API and allows switching
//! Persists selected view to localStorage per entity type

use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use crate::api::{fetch_views, ViewColumn, ViewDef};

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
                    // Try to restore saved view from localStorage
                    let saved_view_id = get_saved_view(&et_storage);
                    let selected_view = if let Some(ref saved_id) = saved_view_id {
                        view_list.iter().find(|v| &v.id == saved_id)
                    } else {
                        None
                    };
                    
                    // Use saved view, or default view, or first view
                    let view_to_use = selected_view
                        .or_else(|| view_list.iter().find(|v| v.is_default))
                        .or_else(|| view_list.first());
                    
                    if let Some(view) = view_to_use {
                        set_active_view.set(Some(view.id.clone()));
                        callback.call(view.clone());
                    } else {
                        // Reset active view if none found (prevents stale view type)
                         set_active_view.set(None);
                    }
                    
                    set_views.set(view_list);
                    set_loading.set(false);
                }
                Err(_) => {
                    // On error, clear views to prevent stale data
                    set_views.set(Vec::new());
                    set_active_view.set(None);
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
