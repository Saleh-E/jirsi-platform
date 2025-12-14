//! Smart Select Component - Enhanced dropdown with search and inline creation
//! 
//! A custom dropdown component that supports:
//! - Search/filter options
//! - "+ Add New" inline creation button
//! - Keyboard navigation
//! - Custom styling
//! - Optional icons/images per option

use leptos::*;

/// A single option in the SmartSelect dropdown
#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    /// Optional icon URL or emoji
    pub icon: Option<String>,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
        }
    }
    
    pub fn with_icon(value: impl Into<String>, label: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: Some(icon.into()),
        }
    }
}

/// Smart Select - A searchable dropdown with optional inline creation
#[component]
pub fn SmartSelect(
    /// Available options to select from
    options: Vec<SelectOption>,
    /// Currently selected value (empty string = no selection)
    #[prop(optional, default = String::new())] value: String,
    /// Callback when selection changes
    #[prop(into)] on_change: Callback<String>,
    /// Enable search/filter input
    #[prop(optional, default = true)] allow_search: bool,
    /// Enable "+ Add New" button
    #[prop(optional, default = false)] allow_create: bool,
    /// Label for the create button (e.g., "+ Add New Property")
    #[prop(optional, default = String::from("+ Add New"))] create_label: String,
    /// Callback when "Add New" is clicked (opens modal)
    #[prop(optional, into)] on_create: Option<Callback<()>>,
    /// Callback with the typed text when creating inline (e.g., "+ Add 'Twitter Ads'")
    #[prop(optional, into)] on_create_value: Option<Callback<String>>,
    /// Placeholder text
    #[prop(optional, default = String::from("-- Select --"))] placeholder: String,
    /// Whether the field is disabled
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    // State
    let (is_open, set_is_open) = create_signal(false);
    let (search_query, set_search_query) = create_signal(String::new());
    let (focused_index, set_focused_index) = create_signal::<Option<usize>>(None);
    
    // Store options for filtering
    let options_stored = store_value(options.clone());
    let value_stored = store_value(if value.is_empty() { None } else { Some(value) });
    let placeholder_stored = store_value(placeholder);
    
    // Get display label for current value
    let display_label = move || {
        let current_value = value_stored.get_value();
        if let Some(val) = current_value {
            options_stored.get_value()
                .iter()
                .find(|o| o.value == val)
                .map(|o| o.label.clone())
                .unwrap_or(val)
        } else {
            placeholder_stored.get_value()
        }
    };
    
    // Filter options based on search query
    let filtered_options = move || {
        let query = search_query.get().to_lowercase();
        if query.is_empty() {
            options_stored.get_value()
        } else {
            options_stored.get_value()
                .into_iter()
                .filter(|o| o.label.to_lowercase().contains(&query))
                .collect()
        }
    };
    
    // Handle option selection
    let handle_select = move |value: String| {
        on_change.call(value);
        set_is_open.set(false);
        set_search_query.set(String::new());
        set_focused_index.set(None);
    };
    
    // Handle keyboard navigation
    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        let opts = filtered_options();
        let count = opts.len();
        
        match key.as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                if !is_open.get() {
                    set_is_open.set(true);
                } else {
                    set_focused_index.update(|idx| {
                        *idx = Some(match *idx {
                            Some(i) => (i + 1).min(count.saturating_sub(1)),
                            None => 0,
                        });
                    });
                }
            }
            "ArrowUp" => {
                ev.prevent_default();
                set_focused_index.update(|idx| {
                    *idx = Some(match *idx {
                        Some(i) => i.saturating_sub(1),
                        None => count.saturating_sub(1),
                    });
                });
            }
            "Enter" => {
                ev.prevent_default();
                if let Some(idx) = focused_index.get() {
                    let opts = filtered_options();
                    if let Some(opt) = opts.get(idx) {
                        handle_select(opt.value.clone());
                    }
                } else if !is_open.get() {
                    set_is_open.set(true);
                }
            }
            "Escape" => {
                set_is_open.set(false);
                set_search_query.set(String::new());
            }
            _ => {}
        }
    };
    
    // Handle create new
    let handle_create = move |_| {
        if let Some(ref callback) = on_create {
            callback.call(());
        }
        set_is_open.set(false);
    };
    
    // Click outside to close
    let dropdown_ref = create_node_ref::<html::Div>();
    
    // Compute class string (class: binding doesn't work with hyphens)
    let div_class = move || {
        let mut classes = vec!["smart-select".to_string()];
        if is_open.get() {
            classes.push("smart-select--open".to_string());
        }
        if disabled {
            classes.push("smart-select--disabled".to_string());
        }
        classes.join(" ")
    };
    
    view! {
        <div 
            class=div_class
            node_ref=dropdown_ref
        >
            // Trigger button
            <button
                type="button"
                class="smart-select__trigger"
                on:click=move |_| {
                    if !disabled {
                        set_is_open.update(|v| *v = !*v);
                    }
                }
                on:keydown=handle_keydown
                disabled=disabled
            >
                <span class="smart-select__value">{display_label}</span>
                <span class="smart-select__arrow">
                    {move || if is_open.get() { "▲" } else { "▼" }}
                </span>
            </button>
            
            // Dropdown menu
            {move || is_open.get().then(|| view! {
                <div class="smart-select__dropdown">
                    // Search input
                    {allow_search.then(|| view! {
                        <div class="smart-select__search">
                            <input
                                type="text"
                                class="smart-select__search-input"
                                placeholder="Search..."
                                prop:value=move || search_query.get()
                                on:input=move |ev| {
                                    set_search_query.set(event_target_value(&ev));
                                    set_focused_index.set(None);
                                }
                                on:keydown=handle_keydown
                            />
                        </div>
                    })}
                    
                    // Options list
                    <div class="smart-select__options">
                        {move || {
                            let opts = filtered_options();
                            if opts.is_empty() {
                                view! {
                                    <div class="smart-select__empty">
                                        "No options found"
                                    </div>
                                }.into_view()
                            } else {
                                opts.into_iter().enumerate().map(|(idx, opt)| {
                                    let opt_value_click = opt.value.clone();
                                    let opt_label = opt.label.clone();
                                    let opt_icon = opt.icon.clone();
                                    let is_selected = value_stored.get_value().as_ref() == Some(&opt.value);
                                    
                                    // Compute option class
                                    let option_class = move || {
                                        let mut classes = vec!["smart-select__option".to_string()];
                                        if is_selected {
                                            classes.push("smart-select__option--selected".to_string());
                                        }
                                        if focused_index.get() == Some(idx) {
                                            classes.push("smart-select__option--focused".to_string());
                                        }
                                        classes.join(" ")
                                    };
                                    
                                    view! {
                                        <div
                                            class=option_class
                                            on:click=move |_| {
                                                handle_select(opt_value_click.clone());
                                            }
                                            on:mouseenter=move |_| {
                                                set_focused_index.set(Some(idx));
                                            }
                                        >
                                            {opt_icon.map(|icon| view! {
                                                <span class="smart-select__option-icon">{icon}</span>
                                            })}
                                            <span class="smart-select__option-label">{opt_label}</span>
                                        </div>
                                    }
                                }).collect_view()
                            }
                        }}
                    </div>
                    
                    // Add New button (sticky footer)
                    {allow_create.then(|| {
                        let label = create_label.clone();
                        let on_create_clone = on_create.clone();
                        let on_create_value_clone = on_create_value.clone();
                        
                        view! {
                            <div class="smart-select__create">
                                // Show "+ Add 'typed text'" when user types a new value
                                {move || {
                                    let query = search_query.get();
                                    let trimmed = query.trim();
                                    let opts = options_stored.get_value();
                                    let exact_match = opts.iter().any(|o| 
                                        o.label.to_lowercase() == trimmed.to_lowercase()
                                    );
                                    
                                    if !trimmed.is_empty() && !exact_match && on_create_value_clone.is_some() {
                                        let typed_text = trimmed.to_string();
                                        let typed_text_display = typed_text.clone();
                                        let typed_text_click = typed_text.clone();
                                        let callback = on_create_value_clone.clone();
                                        view! {
                                            <button
                                                type="button"
                                                class="smart-select__create-btn smart-select__create-btn--inline"
                                                on:click=move |_| {
                                                    if let Some(ref cb) = callback {
                                                        cb.call(typed_text_click.clone());
                                                    }
                                                    set_is_open.set(false);
                                                    set_search_query.set(String::new());
                                                }
                                            >
                                                {format!("+ Add \"{}\"", typed_text_display)}
                                            </button>
                                        }.into_view()
                                    } else {
                                        // Show standard create button
                                        let label_inner = label.clone();
                                        view! {
                                            <button
                                                type="button"
                                                class="smart-select__create-btn"
                                                on:click=handle_create
                                            >
                                                {label_inner}
                                            </button>
                                        }.into_view()
                                    }
                                }}
                            </div>
                        }
                    })}
                </div>
            })}
            
            // Backdrop to catch clicks outside
            {move || is_open.get().then(|| view! {
                <div 
                    class="smart-select__backdrop"
                    on:click=move |_| {
                        set_is_open.set(false);
                        set_search_query.set(String::new());
                    }
                />
            })}
        </div>
    }
}

/// CSS styles for SmartSelect component
/// Include this in your main CSS file or inject dynamically
pub const SMART_SELECT_STYLES: &str = r#"
.smart-select {
    position: relative;
    display: inline-block;
    min-width: 200px;
    width: 100%;
}

.smart-select--disabled {
    opacity: 0.6;
    pointer-events: none;
}

.smart-select__trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 0.625rem 0.875rem;
    background: var(--bg-secondary, #1e1e2e);
    border: 1px solid var(--border-color, #3a3a4a);
    border-radius: 6px;
    color: var(--text-primary, #e0e0e0);
    cursor: pointer;
    transition: all 0.2s ease;
}

.smart-select__trigger:hover {
    border-color: var(--accent-color, #7c3aed);
    background: var(--bg-hover, #2a2a3a);
}

.smart-select--open .smart-select__trigger {
    border-color: var(--accent-color, #7c3aed);
    box-shadow: 0 0 0 3px rgba(124, 58, 237, 0.2);
}

.smart-select__value {
    flex: 1;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.smart-select__arrow {
    margin-left: 0.5rem;
    font-size: 0.75rem;
    color: var(--text-secondary, #888);
}

.smart-select__dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 4px;
    background: var(--bg-secondary, #1e1e2e);
    border: 1px solid var(--border-color, #3a3a4a);
    border-radius: 8px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.4);
    z-index: 1000;
    overflow: hidden;
    animation: slideDown 0.15s ease-out;
}

@keyframes slideDown {
    from {
        opacity: 0;
        transform: translateY(-8px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.smart-select__search {
    padding: 0.5rem;
    border-bottom: 1px solid var(--border-color, #3a3a4a);
}

.smart-select__search-input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    background: var(--bg-primary, #121218);
    border: 1px solid var(--border-color, #3a3a4a);
    border-radius: 4px;
    color: var(--text-primary, #e0e0e0);
    font-size: 0.875rem;
}

.smart-select__search-input:focus {
    outline: none;
    border-color: var(--accent-color, #7c3aed);
}

.smart-select__options {
    max-height: 200px;
    overflow-y: auto;
}

.smart-select__option {
    padding: 0.625rem 1rem;
    cursor: pointer;
    transition: background 0.15s ease;
}

.smart-select__option:hover,
.smart-select__option--focused {
    background: var(--bg-hover, #2a2a3a);
}

.smart-select__option--selected {
    background: var(--accent-color-soft, #7c3aed20);
    color: var(--accent-color, #7c3aed);
}

.smart-select__empty {
    padding: 1rem;
    text-align: center;
    color: var(--text-secondary, #888);
    font-style: italic;
}

.smart-select__create {
    border-top: 1px solid var(--border-color, #3a3a4a);
    padding: 0.5rem;
}

.smart-select__create-btn {
    width: 100%;
    padding: 0.625rem 1rem;
    background: var(--accent-color-soft, #7c3aed15);
    border: 1px dashed var(--accent-color, #7c3aed);
    border-radius: 6px;
    color: var(--accent-color, #7c3aed);
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s ease;
}

.smart-select__create-btn:hover {
    background: var(--accent-color-soft, #7c3aed25);
    transform: translateY(-1px);
}

.smart-select__backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 999;
}
"#;
