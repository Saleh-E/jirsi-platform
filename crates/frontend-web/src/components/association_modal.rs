//! Association Modal - Inline entity creation component
//!
//! Allows users to create new related entities without leaving the current form.
//! Auto-selects the created entity upon successful creation.

use leptos::*;
use gloo_net::http::Request;
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use crate::api::{get_api_base, TENANT_ID};

#[component]
pub fn AssociationModal(
    /// Entity type to create (e.g., "Contact", "Deal")
    entity_type: String,
    /// Callback when entity is created (returns entity ID)
    on_created: Callback<String>,
    /// Callback to close modal
    on_close: Callback<()>,
) -> impl IntoView {
    let entity_type_clone = entity_type.clone();
    let entity_type_for_render = entity_type.clone();
    let (is_saving, set_is_saving) = create_signal(false);
    let (error_message, set_error_message) = create_signal(None::<String>);
    let (form_data, set_form_data) = create_signal(HashMap::<String, JsonValue>::new());

    // TODO: Load entity schema from API to render dynamic form
   // For now, use simple name field
    
    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        
        set_is_saving.set(true);
        set_error_message.set(None);
        
        let entity_type_for_api = entity_type_clone.clone();
        let data = form_data.get();
        
        spawn_local(async move {
            match create_entity(&entity_type_for_api, data).await {
                Ok(entity_id) => {
                    on_created.call(entity_id);
                }
                Err(err) => {
                    set_error_message.set(Some(err));
                    set_is_saving.set(false);
                }
            }
        });
    };
    
    let update_field = move |field: &str, value: JsonValue| {
        set_form_data.update(|data| {
            data.insert(field.to_string(), value);
        });
    };

    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-2xl mx-4">
                // Header
                <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                    <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
                        "Create New " {&entity_type_for_render}
                    </h2>
                    <button
                        type="button"
                        class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                        on:click=move |_| on_close.call(())
                    >
                        "✕"
                    </button>
                </div>
                
                // Body
                <form on:submit=handle_submit>
                    <div class="px-6 py-4 space-y-4">
                        {move || error_message.get().map(|err| view! {
                            <div class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md text-red-800 dark:text-red-300">
                                {err}
                            </div>
                        })}
                        
                        // Simple name field (TODO: Load actual schema)
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Name"
                            </label>
                            <input
                                type="text"
                                class="form-input w-full"
                                required
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    update_field("name", json!(value));
                                }
                            />
                        </div>
                        
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Description"
                            </label>
                            <textarea
                                class="form-input w-full"
                                rows="3"
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    update_field("description", json!(value));
                                }
                            ></textarea>
                        </div>
                    </div>
                    
                    // Footer
                    <div class="ui-modal-footer">
                        <button
                            type="button"
                            class="ui-btn ui-btn-ghost"
                            on:click=move |_| on_close.call(())
                        >
                            "Cancel"
                        </button>
                        <button
                            type="submit"
                            class="ui-btn ui-btn-primary"
                            disabled=move || is_saving.get()
                        >
                            {move || if is_saving.get() { "Creating..." } else { "Create" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}

/// Create entity via API
async fn create_entity(entity_type: &str, data: HashMap<String, JsonValue>) -> Result<String, String> {
    let url = format!("{}/entities/{}", get_api_base(), entity_type);
    
    let response = Request::post(&url)
        .header("X-Tenant-Id", TENANT_ID)
        .header("X-Tenant-Slug", "demo")
        .json(&json!({"fields": data}))
        .map_err(|e| format!("Request error: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {:?}", e))?;
    
    if !response.ok() {
        return Err(format!("Server error: {}", response.status()));
    }
    
    let result: JsonValue = response.json().await
        .map_err(|e| format!("Invalid response: {:?}", e))?;
    
    result["id"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No ID returned".to_string())
}
