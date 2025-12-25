//! Entity Hover Card - Preview related entity on hover
//!
//! Displays key entity information in a floating card.

use leptos::*;
use gloo_net::http::Request;
use serde::Deserialize;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Deserialize)]
struct EntitySummary {
    id: String,
    display_name: String,
    description: Option<String>,
    fields: Option<JsonValue>,
}

#[component]
pub fn EntityHoverCard(
    /// Entity ID to load
    entity_id: String,
    /// Entity type
    entity_type: String,
    /// Position (x, y) for the card
    #[prop(into)] position: Signal<(f64, f64)>,
) -> impl IntoView {
    let entity_type_clone = entity_type.clone();
    let entity_type_for_render = entity_type.clone();
    let (entity, set_entity) = create_signal(None::<EntitySummary>);
    let (is_loading, set_is_loading) = create_signal(true);
    
    // Load entity summary on mount
    create_effect(move |_| {
        let id = entity_id.clone();
        let typ = entity_type_clone.clone();
        
        spawn_local(async move {
            match load_entity_summary(&typ, &id).await {
                Ok(summary) => set_entity.set(Some(summary)),
                Err(_) => set_entity.set(None),
            }
            set_is_loading.set(false);
        });
    });

    view! {
        <div
            class="absolute z-50 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-lg shadow-xl p-4 w-80"
            style:left=move || format!("{}px", position.get().0)
            style:top=move || format!("{}px", position.get().1)
        >
            {move || if is_loading.get() {
                view! {
                    <div class="text-center text-gray-500">
                        "Loading..."
                    </div>
                }.into_view()
            } else if let Some(data) = entity.get() {
                view! {
                    <div>
                        <h3 class="font-semibold text-lg text-gray-900 dark:text-white mb-2">
                            {&data.display_name}
                        </h3>
                        {data.description.as_ref().map(|desc| view! {
                            <p class="text-sm text-gray-600 dark:text-gray-400 mb-3">
                                {desc}
                            </p>
                        })}
                        <div class="space-y-1">
                            <div class="text-xs text-gray-500">
                                "ID: " <span class="font-mono">{&data.id}</span>
                            </div>
                            <div class="text-xs text-gray-500">
                                "Type: " {&entity_type_for_render}
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! {
                    <div class="text-center text-red-500">
                        "Failed to load"
                    </div>
                }.into_view()
            }}
        </div>
    }
}

/// Load entity summary from API
async fn load_entity_summary(entity_type: &str, entity_id: &str) -> Result<EntitySummary, String> {
    let url = format!("/api/v1/entities/{}/{}/summary", entity_type, entity_id);
    
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {:?}", e))?;
    
    if !response.ok() {
        return Err(format!("Server error: {}", response.status()));
    }
    
    response.json().await
        .map_err(|e| format!("Invalid response: {:?}", e))
}
