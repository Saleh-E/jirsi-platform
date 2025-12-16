//! Bottom Sheet Component - Mobile-friendly slide-up panel
//!
//! Used for dropdowns, selects, and modals on mobile devices.

use leptos::*;

/// Bottom Sheet - slides up from bottom on mobile
#[component]
pub fn BottomSheet(
    /// Whether the sheet is open
    is_open: ReadSignal<bool>,
    /// Callback when sheet should close
    #[prop(into)] on_close: Callback<()>,
    /// Optional title for the sheet header
    #[prop(optional)] title: Option<String>,
    /// Content to render inside the sheet
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <Show when=move || is_open.get()>
            <div class="bottom-sheet-overlay" on:click={
                let on_close = on_close.clone();
                move |_| on_close.call(())
            }>
                <div 
                    class="bottom-sheet" 
                    on:click=|ev| ev.stop_propagation()
                >
                    // Handle bar for drag gesture (visual only for now)
                    <div class="bottom-sheet-handle">
                        <div class="handle-bar"></div>
                    </div>
                    
                    // Header with optional title
                    {title.clone().map(|t| {
                        let on_close = on_close.clone();
                        view! {
                            <div class="bottom-sheet-header">
                                <h3 class="bottom-sheet-title">{t}</h3>
                                <button 
                                    class="bottom-sheet-close" 
                                    on:click=move |_| on_close.call(())
                                >
                                    "✕"
                                </button>
                            </div>
                        }
                    })}
                    
                    // Content area
                    <div class="bottom-sheet-content">
                        {children()}
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// SearchableBottomSheet - Bottom sheet with built-in search
#[component]
pub fn SearchableBottomSheet(
    /// Whether the sheet is open
    is_open: ReadSignal<bool>,
    /// Callback when sheet should close
    #[prop(into)] on_close: Callback<()>,
    /// Title for the sheet
    title: String,
    /// Search placeholder
    #[prop(optional, default = "Search...".to_string())] search_placeholder: String,
    /// Items to display
    items: Vec<(String, String)>, // (value, label)
    /// Callback when item is selected
    #[prop(into)] on_select: Callback<String>,
    /// Show "Add New" button
    #[prop(optional, default = false)] show_add_new: bool,
    /// Label for "Add New" button
    #[prop(optional)] add_new_label: Option<String>,
    /// Callback when "Add New" is clicked
    #[prop(optional)] on_add_new: Option<Callback<()>>,
) -> impl IntoView {
    let (search_query, set_search_query) = create_signal(String::new());
    let items_stored = store_value(items);
    
    // Filter items based on search
    let filtered_items = move || {
        let query = search_query.get().to_lowercase();
        let items = items_stored.get_value();
        if query.is_empty() {
            items
        } else {
            items.into_iter()
                .filter(|(_, label)| label.to_lowercase().contains(&query))
                .collect::<Vec<_>>()
        }
    };
    
    view! {
        <Show when=move || is_open.get()>
            <div class="bottom-sheet-overlay" on:click={
                let on_close = on_close.clone();
                move |_| on_close.call(())
            }>
                <div 
                    class="bottom-sheet bottom-sheet-searchable" 
                    on:click=|ev| ev.stop_propagation()
                >
                    // Handle bar
                    <div class="bottom-sheet-handle">
                        <div class="handle-bar"></div>
                    </div>
                    
                    // Header
                    <div class="bottom-sheet-header">
                        <h3 class="bottom-sheet-title">{title.clone()}</h3>
                        <button 
                            class="bottom-sheet-close" 
                            on:click={
                                let on_close = on_close.clone();
                                move |_| on_close.call(())
                            }
                        >
                            "✕"
                        </button>
                    </div>
                    
                    // Search input
                    <div class="bottom-sheet-search">
                        <input
                            type="text"
                            class="bottom-sheet-search-input"
                            placeholder=search_placeholder.clone()
                            on:input=move |ev| {
                                set_search_query.set(event_target_value(&ev));
                            }
                        />
                    </div>
                    
                    // Items list
                    <div class="bottom-sheet-items">
                        <For
                            each=filtered_items
                            key=|(value, _)| value.clone()
                            children={
                                let on_select = on_select.clone();
                                let on_close = on_close.clone();
                                move |(value, label)| {
                                    let value_click = value.clone();
                                    let on_select = on_select.clone();
                                    let on_close = on_close.clone();
                                    view! {
                                        <button 
                                            class="bottom-sheet-item"
                                            on:click=move |_| {
                                                on_select.call(value_click.clone());
                                                on_close.call(());
                                            }
                                        >
                                            {label}
                                        </button>
                                    }
                                }
                            }
                        />
                        
                        // Empty state
                        {move || filtered_items().is_empty().then(|| view! {
                            <div class="bottom-sheet-empty">
                                "No results found"
                            </div>
                        })}
                    </div>
                    
                    // Add New footer (sticky)
                    {show_add_new.then(|| {
                        let label = add_new_label.clone().unwrap_or_else(|| "+ Add New".to_string());
                        let on_add = on_add_new.clone();
                        let on_close = on_close.clone();
                        view! {
                            <div class="bottom-sheet-footer">
                                <button 
                                    class="bottom-sheet-add-new"
                                    on:click=move |_| {
                                        if let Some(ref callback) = on_add {
                                            callback.call(());
                                        }
                                        on_close.call(());
                                    }
                                >
                                    {label}
                                </button>
                            </div>
                        }
                    })}
                </div>
            </div>
        </Show>
    }
}
