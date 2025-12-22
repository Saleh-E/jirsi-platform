use leptos::*;
use uuid::Uuid;
use crate::api::{fetch_entity_lookup, fetch_entity};
use crate::components::smart_select::{SMART_SELECT_STYLES}; // Reuse styles

#[component]
pub fn AsyncEntitySelect(
    #[prop(into)] entity_type: String,
    #[prop(into)] value: RwSignal<Option<Uuid>>,
    #[prop(optional, into)] class: String,
    #[prop(optional, default = String::from("Select..."))] placeholder: String,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    // State
    let (is_open, set_is_open) = create_signal(false);
    let (search_query, set_search_query) = create_signal(String::new());
    
    // Capture entity type for async closures (separate clones for separate moves)
    let entity_type_for_label = entity_type.clone();
    let entity_type_for_search = entity_type.clone();
    
    // Resource to resolve the label of the CURRENTLY selected value
    let selected_value_label = create_resource(
        move || value.get(),
        move |id| {
            let etype = entity_type_for_label.clone();
            async move {
                if let Some(uid) = id {
                    // Fetch full entity to get its name/title/label
                    match fetch_entity(&etype, &uid.to_string()).await {
                        Ok(data) => {
                           // Try common label fields
                           data.get("name")
                               .or_else(|| data.get("title"))
                               .or_else(|| data.get("label"))
                               .or_else(|| data.get("first_name")) // Contact hint
                               .and_then(|v| v.as_str())
                               .map(|s| s.to_string())
                               .or_else(|| {
                                   // Composite fallback for contact
                                   if let (Some(f), Some(l)) = (data.get("first_name"), data.get("last_name")) {
                                       Some(format!("{} {}", f.as_str().unwrap_or(""), l.as_str().unwrap_or("")))
                                   } else {
                                       Some(uid.to_string())
                                   }
                               })
                               .unwrap_or(uid.to_string())
                        },
                        Err(_) => uid.to_string()
                    }
                } else {
                    String::new()
                }
            }
        }
    );

    // Resource for search results (debouncing could be added here, or just let it fly)
    // We trigger this only when open or searching
    let search_results = create_resource(
        move || (is_open.get(), search_query.get()),
        move |(open, query)| {
            let etype = entity_type_for_search.clone();
            async move {
                if !open {
                    return vec![];
                }
                // Call generic lookup
                fetch_entity_lookup(&etype, Some(&query)).await.unwrap_or_else(|_| vec![])
            }
        }
    );

    let display_label = move || {
        if value.get().is_none() {
            placeholder.clone()
        } else {
            selected_value_label.get().unwrap_or(String::from("Loading..."))
        }
    };

    let handle_select = move |id_str: String, _label: String| {
        if let Ok(uid) = Uuid::parse_str(&id_str) {
             value.set(Some(uid));
             // Optimization: We could manually mutate selected_value_label resource here to avoid refetch
             // But simpler to just let it react to value change?
             // Actually, reactivity might be slow unless we optimistically update the label?
             // Let's just trust the resource reload for now.
             set_is_open.set(false);
             set_search_query.set(String::new());
        }
    };

    // Click outside handler
    let dropdown_ref = create_node_ref::<html::Div>();
    // Note: click outside logic usually requires window listener. 
    // Simplified: specific backdrop div.

    view! {
        <div class=format!("smart-select {}", class) node_ref=dropdown_ref>
            // Styles
            <style>{SMART_SELECT_STYLES}</style>

            // Trigger
            <button
                type="button"
                class="smart-select__trigger"
                disabled=disabled
                on:click=move |_| if !disabled { set_is_open.update(|v| *v = !*v) }
            >
                <span class="smart-select__value">{display_label}</span>
                <span class="smart-select__arrow">
                    {move || if is_open.get() { "▲" } else { "▼" }}
                </span>
            </button>

            // Dropdown
            {move || is_open.get().then(|| view! {
                 <div class="smart-select__dropdown">
                     // Search
                     <div class="smart-select__search">
                        <input
                            type="text"
                            class="smart-select__search-input"
                            placeholder="Search..."
                            prop:value=move || search_query.get()
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                            on:click=move |ev| ev.stop_propagation() // Prevent close
                            autofocus
                        />
                     </div>

                     // Results
                     <div class="smart-select__options">
                        <Suspense fallback=move || view! { <div class="smart-select__empty">"Loading..."</div> }>
                            {move || {
                                let results = search_results.get().unwrap_or(vec![]);
                                if results.is_empty() {
                                    view! { <div class="smart-select__empty">"No results"</div> }.into_view()
                                } else {
                                    results.into_iter().map(|item| {
                                        let id = item.id.clone();
                                        let label = item.label.clone();
                                        let is_sel = value.get().map(|v| v.to_string()) == Some(id.clone());
                                        
                                        view! {
                                            <div 
                                                class=format!("smart-select__option {}", if is_sel { "smart-select__option--selected" } else { "" })
                                                on:click=move |_| handle_select(item.id.clone(), item.label.clone())
                                            >
                                                <span class="smart-select__option-label">{label}</span>
                                            </div>
                                        }
                                    }).collect_view()
                                }
                            }}
                        </Suspense>
                     </div>
                 </div>
                 
                 // Backdrop
                 <div 
                    class="smart-select__backdrop" 
                    on:click=move |_| set_is_open.set(false)
                 />
            })}
        </div>
    }
}

/// Component to render the label of a linked entity (Async)
#[component]
pub fn AsyncEntityLabel(
    #[prop(into)] entity_type: String,
    #[prop(into)] id: String,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    // Capture entity_type for the resource fetcher
    let entity_type_clone = entity_type.clone();
    let id_clone = id.clone();

    // Resource to fetch label
    let label_resource = create_resource(
        move || id_clone.clone(),
        move |uid_str| {
            let etype = entity_type_clone.clone();
            async move {
                if uid_str.is_empty() { return String::new(); }
                
                match fetch_entity(&etype, &uid_str).await {
                    Ok(data) => {
                       data.get("name")
                           .or_else(|| data.get("title"))
                           .or_else(|| data.get("label"))
                           .or_else(|| data.get("first_name"))
                           .and_then(|v| v.as_str())
                           .map(|s| s.to_string())
                           .or_else(|| {
                               if let (Some(f), Some(l)) = (data.get("first_name"), data.get("last_name")) {
                                   Some(format!("{} {}", f.as_str().unwrap_or(""), l.as_str().unwrap_or("")))
                               } else {
                                   Some(uid_str.clone())
                               }
                           })
                           .unwrap_or(uid_str.clone())
                    },
                    Err(_) => uid_str
                }
            }
        }
    );
    
    // Capture id for view fallback safely
    let id_stored_fallback = store_value(id.clone());
    
    let display_content = move || {
        label_resource.get().unwrap_or_else(|| id_stored_fallback.with_value(|v| v.clone()))
    };
    
    view! {
        <span class=format!("async-entity-label {}", class)>
             <Suspense fallback=move || view! { <span class="loading-label">"..."</span> }>
                 {display_content}
             </Suspense>
        </span>
    }
}
