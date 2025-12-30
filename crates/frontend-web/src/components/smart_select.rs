//! Smart Select Component - Enhanced dropdown with search and inline creation
//! 
//! A custom dropdown component that supports:
//! - Search/filter options
//! - "+ Add New" inline creation button
//! - Keyboard navigation
//! - Custom styling
//! - Optional icons/images per option
//! - Multi-select with chips
//! - Async option loading with debounce
//! - Mobile: Uses BottomSheet instead of dropdown

use leptos::*;
use crate::context::mobile::use_mobile;

/// Icon type for SelectOption - can be emoji/text or image URL
#[derive(Clone, Debug, PartialEq)]
pub enum IconType {
    /// Text/emoji icon (e.g., "🔥")
    Text(String),
    /// Image URL
    Image(String),
}

/// A single option in the SmartSelect dropdown
#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    /// Optional icon - can be emoji or image URL
    pub icon: Option<IconType>,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
        }
    }
    
    /// Create option with text/emoji icon
    pub fn with_icon(value: impl Into<String>, label: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: Some(IconType::Text(icon.into())),
        }
    }
    
    /// Create option with image URL icon
    pub fn with_image(value: impl Into<String>, label: impl Into<String>, image_url: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: Some(IconType::Image(image_url.into())),
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
    /// Callback when delete button is clicked on an option (value to delete)
    #[prop(optional, into)] on_delete_option: Option<Callback<String>>,
    /// Placeholder text
    #[prop(optional, default = String::from("-- Select --"))] placeholder: String,
    /// Whether the field is disabled
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    // Mobile detection
    let mobile_ctx = use_mobile();
    let is_mobile = move || mobile_ctx.is_mobile.get();
    
    // State
    let (is_open, set_is_open) = create_signal(false);
    let (search_query, set_search_query) = create_signal(String::new());
    let (focused_index, set_focused_index) = create_signal::<Option<usize>>(None);
    let (create_mode, set_create_mode) = create_signal(false); // True when showing create input
    let (local_options, set_local_options) = create_signal(options.clone()); // Local copy to add new options
    
    // Store options for filtering (use local_options which can grow)
    let options_stored = store_value(options.clone());
    // Local signal for selected value - tracks what's currently selected
    let (selected_value, set_selected_value) = create_signal(if value.is_empty() { None } else { Some(value) });
    let placeholder_stored = store_value(placeholder);
    let create_label_stored = store_value(create_label);
    let on_create_stored = store_value(on_create);
    let on_create_value_stored = store_value(on_create_value.clone());
    let on_delete_option_stored = store_value(on_delete_option.clone());
    
    // Get display label for current value - uses reactive selected_value signal
    let display_label = move || {
        let current_value = selected_value.get();
        if let Some(val) = current_value {
            // Also check local_options for newly added values
            local_options.get()
                .iter()
                .find(|o| o.value == val)
                .map(|o| o.label.clone())
                .or_else(|| {
                    options_stored.get_value()
                        .iter()
                        .find(|o| o.value == val)
                        .map(|o| o.label.clone())
                })
                .unwrap_or(val)
        } else {
            placeholder_stored.get_value()
        }
    };
    
    // Filter options based on search query - uses local_options to include newly added values
    let filtered_options = move || {
        let query = search_query.get().to_lowercase();
        if query.is_empty() {
            local_options.get()
        } else {
            local_options.get()
                .into_iter()
                .filter(|o| o.label.to_lowercase().contains(&query))
                .collect()
        }
    };
    
    // Handle option selection - update local selected value and notify parent
    let handle_select = move |value: String| {
        set_selected_value.set(Some(value.clone())); // Update local display
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
    
    // Handle create new - uses on_create if provided, otherwise enables create mode
    let handle_create = move |_| {
        let on_create_opt = on_create_stored.get_value();
        let on_create_value_opt = on_create_value_stored.get_value();
        
        if let Some(ref callback) = on_create_opt {
            // Use the on_create callback (opens external modal)
            callback.call(());
            set_is_open.set(false);
            set_create_mode.set(false);
        } else if on_create_value_opt.is_some() {
            // Enable create mode - shows dedicated input field
            set_create_mode.set(true);
            set_search_query.set(String::new()); // Clear search for typing new value
            // Keep dropdown open for user to type
        } else {
            set_is_open.set(false);
        }
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
            
            // Dropdown menu - Desktop only
            {move || (!is_mobile() && is_open.get()).then(|| view! {
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
                                    let opt_value_delete = opt.value.clone();
                                    let opt_label = opt.label.clone();
                                    let opt_icon = opt.icon.clone();
                                    let is_selected = selected_value.get().as_ref() == Some(&opt.value);
                                    let can_delete = on_delete_option.is_some();
                                    let on_delete = on_delete_option_stored.get_value();
                                    
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
                                            {opt_icon.map(|icon| match icon {
                                                IconType::Text(text) => view! {
                                                    <span class="smart-select__option-icon">{text}</span>
                                                }.into_view(),
                                                IconType::Image(url) => view! {
                                                    <img class="smart-select__option-img" src=url alt="" />
                                                }.into_view(),
                                            })}
                                            <span class="smart-select__option-label">{opt_label}</span>
                                            {can_delete.then(|| {
                                                let del_value = opt_value_delete.clone();
                                                let on_del = on_delete.clone();
                                                view! {
                                                    <button
                                                        type="button"
                                                        class="smart-select__option-delete"
                                                        title="Delete option"
                                                        on:click=move |ev| {
                                                            ev.stop_propagation();
                                                            if let Some(ref cb) = on_del {
                                                                // Remove from local options immediately
                                                                set_local_options.update(|opts| {
                                                                    opts.retain(|o| o.value != del_value);
                                                                });
                                                                cb.call(del_value.clone());
                                                            }
                                                        }
                                                    >
                                                        "×"
                                                    </button>
                                                }
                                            })}
                                        </div>
                                    }
                                }).collect_view()
                            }
                        }}
                    </div>
                    
                    // Add New button (sticky footer)
                    {allow_create.then(|| {
                        let label = create_label_stored.get_value();
                        let _on_create_clone = on_create_stored.get_value();
                        let on_create_value_clone = on_create_value.clone();
                        
                        view! {
                            <div class="smart-select__create">
                                // Show "Create: [input]" when in create mode, or "+ Add 'typed text'" when user types
                                {move || {
                                    let query = search_query.get();
                                    let trimmed = query.trim();
                                    let opts = local_options.get(); // Use local_options for persistence
                                    let exact_match = opts.iter().any(|o| 
                                        o.label.to_lowercase() == trimmed.to_lowercase()
                                    );
                                    let in_create_mode = create_mode.get();
                                    
                                    // Show add button when typing new value OR when in create mode
                                    if (!trimmed.is_empty() && !exact_match && on_create_value_clone.is_some()) || (in_create_mode && on_create_value_clone.is_some()) {
                                        let typed_text = trimmed.to_string();
                                        let typed_text_display = typed_text.clone();
                                        let typed_text_click = typed_text.clone();
                                        let callback = on_create_value_clone.clone();
                                        
                                        // Don't show button if empty and in create mode (user needs to type first)
                                        if typed_text.is_empty() && in_create_mode {
                                            view! {
                                                <div class="smart-select__create-hint">
                                                    "Type a new value above, then click Add"
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <button
                                                    type="button"
                                                    class="smart-select__create-btn smart-select__create-btn--inline"
                                                    on:click=move |_| {
                                                        if let Some(ref cb) = callback {
                                                            // Add the new value to local_options for persistence
                                                            let new_option = SelectOption::new(typed_text_click.clone(), typed_text_click.clone());
                                                            set_local_options.update(|opts| {
                                                                opts.push(new_option);
                                                            });
                                                            
                                                            // Update selected value to show new option
                                                            set_selected_value.set(Some(typed_text_click.clone()));
                                                            cb.call(typed_text_click.clone());
                                                        }
                                                        set_is_open.set(false);
                                                        set_search_query.set(String::new());
                                                        set_create_mode.set(false);
                                                    }
                                                >
                                                    {format!("+ Add \"{}\"", typed_text_display)}
                                                </button>
                                            }.into_view()
                                        }
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
            
            // Backdrop to catch clicks outside - Desktop only
            {move || (!is_mobile() && is_open.get()).then(|| view! {
                <div 
                    class="smart-select__backdrop"
                    on:click=move |_| {
                        set_is_open.set(false);
                        set_search_query.set(String::new());
                    }
                />
            })}
            
            // Mobile: BottomSheet
            {move || {
                let is_mob = is_mobile();
                let open = is_open.get();
                
                if is_mob && open {
                    let opts = options_stored.get_value();
                    let items: Vec<(String, String)> = opts.iter()
                        .map(|o| (o.value.clone(), o.label.clone()))
                        .collect();
                    
                    let title_val = placeholder_stored.get_value();
                    let create_label_val = create_label_stored.get_value();
                    
                    // Render with or without "Add New" based on allow_create
                    if allow_create {
                        if let Some(create_callback) = on_create_stored.get_value() {
                            let cb = create_callback.clone();
                            view! {
                                <crate::components::bottom_sheet::SearchableBottomSheet
                                    is_open=is_open
                                    on_close=move |_: ()| {
                                        set_is_open.set(false);
                                        set_search_query.set(String::new());
                                    }
                                    title=title_val
                                    items=items
                                    on_select=move |val: String| handle_select(val)
                                    show_add_new=true
                                    add_new_label=create_label_val
                                    on_add_new=cb
                                />
                            }.into_view()
                        } else {
                            view! {
                                <crate::components::bottom_sheet::SearchableBottomSheet
                                    is_open=is_open
                                    on_close=move |_: ()| {
                                        set_is_open.set(false);
                                        set_search_query.set(String::new());
                                    }
                                    title=title_val
                                    items=items
                                    on_select=move |val: String| handle_select(val)
                                    show_add_new=true
                                    add_new_label=create_label_val
                                />
                            }.into_view()
                        }
                    } else {
                        view! {
                            <crate::components::bottom_sheet::SearchableBottomSheet
                                is_open=is_open
                                on_close=move |_: ()| {
                                    set_is_open.set(false);
                                    set_search_query.set(String::new());
                                }
                                title=title_val
                                items=items
                                on_select=move |val: String| handle_select(val)
                            />
                        }.into_view()
                    }
                } else {
                    view! {}.into_view()
                }
            }}
        </div>
    }
}

/// CSS styles for SmartSelect component
/// Include this in your main CSS file or inject dynamically
pub const SMART_SELECT_STYLES: &str = r#"
/* ============================================
   PREMIUM SMART SELECT - World-Class Design
   ============================================ */

.smart-select {
    position: relative;
    display: inline-block;
    min-width: 200px;
    width: 100%;
}

.smart-select--disabled {
    opacity: 0.5;
    pointer-events: none;
}

/* Premium Trigger Button */
.smart-select__trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 0.875rem 1rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.75rem;
    color: var(--text-primary, #f0f0f0);
    cursor: pointer;
    font-size: 0.9375rem;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.smart-select__trigger:hover {
    border-color: rgba(99, 102, 241, 0.4);
    background: rgba(255, 255, 255, 0.05);
    transform: translateY(-1px);
}

.smart-select--open .smart-select__trigger {
    border-color: #6366f1;
    background: rgba(99, 102, 241, 0.08);
    box-shadow: 
        0 0 0 3px rgba(99, 102, 241, 0.15),
        0 0 20px rgba(99, 102, 241, 0.2);
}

.smart-select__value {
    flex: 1;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
}

.smart-select__arrow {
    margin-left: 0.75rem;
    font-size: 0.625rem;
    color: rgba(148, 163, 184, 0.7);
    transition: transform 0.3s ease;
}

.smart-select--open .smart-select__arrow {
    transform: rotate(180deg);
    color: #6366f1;
}

/* Premium Dropdown Panel */
.smart-select__dropdown {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    right: 0;
    background: linear-gradient(145deg, rgba(20, 20, 40, 0.98), rgba(15, 15, 30, 0.95));
    backdrop-filter: blur(20px) saturate(150%);
    -webkit-backdrop-filter: blur(20px) saturate(150%);
    border: 1px solid rgba(99, 102, 241, 0.25);
    border-radius: 1rem;
    box-shadow:
        0 0 0 1px rgba(255, 255, 255, 0.05),
        0 20px 50px -10px rgba(0, 0, 0, 0.5),
        0 0 40px rgba(99, 102, 241, 0.15);
    z-index: 9999;
    overflow: hidden;
    animation: dropdownSlide 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes dropdownSlide {
    0% {
        opacity: 0;
        transform: translateY(-12px) scale(0.95);
    }
    100% {
        opacity: 1;
        transform: translateY(0) scale(1);
    }
}

/* Glowing top border */
.smart-select__dropdown::before {
    content: '';
    position: absolute;
    top: 0;
    left: 15%;
    right: 15%;
    height: 1px;
    background: linear-gradient(90deg, transparent, #6366f1, #8b5cf6, #6366f1, transparent);
    filter: blur(0.5px);
}

/* Premium Search Input */
.smart-select__search {
    padding: 0.75rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(0, 0, 0, 0.2);
}

.smart-select__search-input {
    width: 100%;
    padding: 0.75rem 1rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 0.625rem;
    color: var(--text-primary, #f0f0f0);
    font-size: 0.875rem;
    transition: all 0.3s ease;
}

.smart-select__search-input::placeholder {
    color: rgba(148, 163, 184, 0.5);
}

.smart-select__search-input:focus {
    outline: none;
    border-color: rgba(99, 102, 241, 0.5);
    background: rgba(99, 102, 241, 0.05);
    box-shadow: 
        0 0 0 3px rgba(99, 102, 241, 0.12),
        inset 0 0 10px rgba(99, 102, 241, 0.05);
}

/* Options List */
.smart-select__options {
    max-height: 280px;
    overflow-y: auto;
    padding: 0.5rem;
    scrollbar-width: thin;
    scrollbar-color: rgba(99, 102, 241, 0.3) transparent;
}

.smart-select__options::-webkit-scrollbar {
    width: 5px;
}

.smart-select__options::-webkit-scrollbar-track {
    background: transparent;
}

.smart-select__options::-webkit-scrollbar-thumb {
    background: linear-gradient(180deg, #6366f1, #8b5cf6);
    border-radius: 3px;
}

/* Premium Option Item */
.smart-select__option {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    margin: 0.25rem 0;
    border-radius: 0.625rem;
    cursor: pointer;
    transition: all 0.2s ease;
    position: relative;
}

.smart-select__option-icon {
    font-size: 1.125rem;
    min-width: 24px;
    text-align: center;
}

.smart-select__option-label {
    flex: 1;
    font-size: 0.9375rem;
    font-weight: 500;
}

.smart-select__option:hover,
.smart-select__option--focused {
    background: linear-gradient(135deg, rgba(99, 102, 241, 0.12), rgba(139, 92, 246, 0.08));
}

.smart-select__option--selected {
    background: linear-gradient(135deg, rgba(99, 102, 241, 0.2), rgba(139, 92, 246, 0.15));
    color: #a5b4fc;
    box-shadow: inset 0 0 0 1px rgba(99, 102, 241, 0.3);
}

/* Checkmark for selected */
.smart-select__option--selected::after {
    content: '✓';
    font-size: 0.875rem;
    color: #22c55e;
    margin-left: auto;
    font-weight: 600;
    text-shadow: 0 0 8px rgba(34, 197, 94, 0.5);
}

/* Premium Delete Button - Always Visible */
.smart-select__option-delete {
    display: flex !important;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    margin-left: 0.5rem;
    padding: 0;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 50%;
    color: #f87171;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.25s ease;
    opacity: 0.7;
}

.smart-select__option:hover .smart-select__option-delete {
    opacity: 1;
}

.smart-select__option-delete:hover {
    background: linear-gradient(135deg, #ef4444, #dc2626);
    border-color: #ef4444;
    color: white;
    transform: scale(1.1);
    box-shadow: 0 0 15px rgba(239, 68, 68, 0.4);
}

/* Empty State */
.smart-select__empty {
    padding: 2rem 1rem;
    text-align: center;
    color: rgba(148, 163, 184, 0.7);
    font-style: italic;
    font-size: 0.875rem;
}

/* Premium Create Section */
.smart-select__create {
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    padding: 0.75rem;
    background: rgba(0, 0, 0, 0.15);
}

.smart-select__create-btn {
    width: 100%;
    padding: 0.875rem 1.25rem;
    background: linear-gradient(135deg, rgba(99, 102, 241, 0.15), rgba(139, 92, 246, 0.1));
    border: 1px dashed rgba(99, 102, 241, 0.5);
    border-radius: 0.75rem;
    color: #a5b4fc;
    cursor: pointer;
    font-weight: 600;
    font-size: 0.9375rem;
    transition: all 0.3s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
}

.smart-select__create-btn::before {
    content: '✨';
    font-size: 1rem;
}

.smart-select__create-btn:hover {
    background: linear-gradient(135deg, rgba(99, 102, 241, 0.25), rgba(139, 92, 246, 0.2));
    border-style: solid;
    border-color: #6366f1;
    transform: translateY(-2px);
    box-shadow: 
        0 8px 25px rgba(99, 102, 241, 0.3),
        0 0 20px rgba(139, 92, 246, 0.2);
}

.smart-select__create-btn--inline {
    background: linear-gradient(135deg, rgba(34, 197, 94, 0.15), rgba(34, 197, 94, 0.08));
    border-color: rgba(34, 197, 94, 0.5);
    color: #86efac;
}

.smart-select__create-btn--inline::before {
    content: '+';
    font-size: 1.25rem;
    font-weight: 700;
}

.smart-select__create-btn--inline:hover {
    background: linear-gradient(135deg, rgba(34, 197, 94, 0.25), rgba(34, 197, 94, 0.15));
    border-color: #22c55e;
    box-shadow: 
        0 8px 25px rgba(34, 197, 94, 0.25),
        0 0 20px rgba(34, 197, 94, 0.15);
}

.smart-select__create-hint {
    text-align: center;
    color: rgba(148, 163, 184, 0.6);
    font-size: 0.8125rem;
    padding: 0.75rem;
    font-style: italic;
}

/* Backdrop */
.smart-select__backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 9998;
    background: transparent;
}

/* Image icons */
.smart-select__option-img {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    object-fit: cover;
    border: 1px solid rgba(255, 255, 255, 0.1);
}

/* ============================================
   PREMIUM MULTI-SELECT CHIPS
   ============================================ */

.multi-select__chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    padding: 0.5rem 0;
}

.multi-select__chip {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.75rem;
    background: linear-gradient(135deg, rgba(99, 102, 241, 0.15), rgba(139, 92, 246, 0.1));
    border: 1px solid rgba(99, 102, 241, 0.4);
    border-radius: 100px;
    font-size: 0.8125rem;
    font-weight: 500;
    color: #a5b4fc;
    animation: chipIn 0.25s ease;
}

@keyframes chipIn {
    from {
        opacity: 0;
        transform: scale(0.8);
    }
    to {
        opacity: 1;
        transform: scale(1);
    }
}

.multi-select__chip-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    padding: 0;
    margin-left: 0.25rem;
    border: none;
    background: rgba(239, 68, 68, 0.2);
    color: #f87171;
    border-radius: 50%;
    cursor: pointer;
    font-size: 0.75rem;
    font-weight: 700;
    transition: all 0.2s ease;
}

.multi-select__chip-remove:hover {
    background: #ef4444;
    color: white;
    transform: scale(1.1);
}

.multi-select__add-btn {
    padding: 0.375rem 0.875rem;
    background: transparent;
    border: 1px dashed rgba(99, 102, 241, 0.4);
    border-radius: 100px;
    color: rgba(165, 180, 252, 0.8);
    cursor: pointer;
    font-size: 0.8125rem;
    font-weight: 500;
    transition: all 0.25s ease;
}

.multi-select__add-btn:hover {
    border-color: #6366f1;
    border-style: solid;
    color: #a5b4fc;
    background: rgba(99, 102, 241, 0.1);
    box-shadow: 0 0 15px rgba(99, 102, 241, 0.2);
}

/* ============================================
   LIGHT MODE OVERRIDES
   ============================================ */

html.light .smart-select__trigger {
    background: rgba(0, 0, 0, 0.02);
    border-color: rgba(0, 0, 0, 0.1);
    color: #1e293b;
}

html.light .smart-select__trigger:hover {
    background: rgba(99, 102, 241, 0.05);
    border-color: rgba(99, 102, 241, 0.4);
}

html.light .smart-select--open .smart-select__trigger {
    background: rgba(99, 102, 241, 0.08);
}

html.light .smart-select__dropdown {
    background: rgba(255, 255, 255, 0.98);
    border-color: rgba(99, 102, 241, 0.2);
    box-shadow:
        0 20px 50px -10px rgba(0, 0, 0, 0.15),
        0 0 40px rgba(99, 102, 241, 0.1);
}

html.light .smart-select__search-input {
    background: rgba(0, 0, 0, 0.02);
    border-color: rgba(0, 0, 0, 0.1);
    color: #1e293b;
}

html.light .smart-select__option-label {
    color: #1e293b;
}

html.light .smart-select__option:hover,
html.light .smart-select__option--focused {
    background: rgba(99, 102, 241, 0.08);
}

html.light .smart-select__option--selected {
    background: rgba(99, 102, 241, 0.12);
    color: #4f46e5;
}

html.light .smart-select__create-btn {
    color: #4f46e5;
}

html.light .multi-select__chip {
    background: rgba(99, 102, 241, 0.1);
    color: #4f46e5;
}
"#;

/// Multi Select - A searchable dropdown for selecting multiple values with chips
#[component]
pub fn MultiSelect(
    /// Available options to select from
    options: Vec<SelectOption>,
    /// Currently selected values
    #[prop(optional, default = Vec::new())] values: Vec<String>,
    /// Callback when selection changes
    #[prop(into)] on_change: Callback<Vec<String>>,
    /// Enable search/filter input
    #[prop(optional, default = true)] allow_search: bool,
    /// Enable "+ Add New" button
    #[prop(optional, default = false)] allow_create: bool,
    /// Label for the create button
    #[prop(optional, default = String::from("+ Add New"))] create_label: String,
    /// Callback when "Add New" is clicked
    #[prop(optional, into)] on_create: Option<Callback<()>>,
    /// Callback with typed text for inline creation
    #[prop(optional, into)] on_create_value: Option<Callback<String>>,
    /// Placeholder text
    #[prop(optional, default = String::from("Select options..."))] placeholder: String,
    /// Whether the field is disabled
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    // State
    let (is_open, set_is_open) = create_signal(false);
    let (search_query, set_search_query) = create_signal(String::new());
    let (selected_values, set_selected_values) = create_signal(values.clone());
    
    // Store options
    let options_stored = store_value(options.clone());
    
    // Get labels for selected values
    let selected_labels = move || {
        let vals = selected_values.get();
        let opts = options_stored.get_value();
        vals.iter().filter_map(|v| {
            opts.iter().find(|o| &o.value == v).map(|o| (v.clone(), o.label.clone()))
        }).collect::<Vec<_>>()
    };
    
    // Filter available options (exclude already selected)
    let available_options = move || {
        let query = search_query.get().to_lowercase();
        let selected = selected_values.get();
        options_stored.get_value()
            .into_iter()
            .filter(|o| !selected.contains(&o.value))
            .filter(|o| query.is_empty() || o.label.to_lowercase().contains(&query))
            .collect::<Vec<_>>()
    };
    
    // Add a value
    let add_value = move |val: String| {
        set_selected_values.update(|vals| {
            if !vals.contains(&val) {
                vals.push(val);
            }
        });
        on_change.call(selected_values.get());
        set_search_query.set(String::new());
    };
    
    // Remove a value
    let remove_value = move |val: String| {
        set_selected_values.update(|vals| {
            vals.retain(|v| v != &val);
        });
        on_change.call(selected_values.get());
    };
    
    // Handle create
    let on_create_clone = on_create.clone();
    let handle_create = move |_: web_sys::MouseEvent| {
        if let Some(ref callback) = on_create_clone {
            callback.call(());
        }
        set_is_open.set(false);
    };
    
    view! {
        <div class={move || {
            let mut classes = vec!["smart-select".to_string(), "multi-select".to_string()];
            if is_open.get() {
                classes.push("smart-select--open".to_string());
            }
            if disabled {
                classes.push("smart-select--disabled".to_string());
            }
            classes.join(" ")
        }}>
            // Chips display
            <div class="multi-select__chips">
                {move || selected_labels().into_iter().map(|(val, label)| {
                    let val_remove = val.clone();
                    view! {
                        <span class="multi-select__chip">
                            {label}
                            <button
                                type="button"
                                class="multi-select__chip-remove"
                                on:click=move |_| remove_value(val_remove.clone())
                            >
                                "×"
                            </button>
                        </span>
                    }
                }).collect_view()}
                
                // Add button to open dropdown
                <button
                    type="button"
                    class="multi-select__add-btn"
                    on:click=move |_| {
                        if !disabled {
                            set_is_open.set(true);
                        }
                    }
                    disabled=disabled
                >
                    {move || if selected_values.get().is_empty() { placeholder.clone() } else { "+ Add".to_string() }}
                </button>
            </div>
            
            // Dropdown
            {move || is_open.get().then(|| {
                let on_create_value_clone = on_create_value.clone();
                view! {
                    <div class="smart-select__dropdown">
                        // Search input
                        {allow_search.then(|| view! {
                            <div class="smart-select__search-wrapper">
                                <input
                                    type="text"
                                    class="smart-select__search"
                                    placeholder="Search..."
                                    prop:value=move || search_query.get()
                                    on:input=move |ev| set_search_query.set(event_target_value(&ev))
                                />
                            </div>
                        })}
                        
                        // Options
                        <div class="smart-select__options">
                            {move || {
                                let opts = available_options();
                                if opts.is_empty() {
                                    view! {
                                        <div class="smart-select__empty">
                                            "No more options"
                                        </div>
                                    }.into_view()
                                } else {
                                    opts.into_iter().map(|opt| {
                                        let opt_value = opt.value.clone();
                                        let opt_label = opt.label.clone();
                                        let opt_icon = opt.icon.clone();
                                        view! {
                                            <div
                                                class="smart-select__option"
                                                on:click=move |_| {
                                                    add_value(opt_value.clone());
                                                }
                                            >
                                                {opt_icon.map(|icon| match icon {
                                                    IconType::Text(text) => view! {
                                                        <span class="smart-select__option-icon">{text}</span>
                                                    }.into_view(),
                                                    IconType::Image(url) => view! {
                                                        <img class="smart-select__option-img" src=url alt="" />
                                                    }.into_view(),
                                                })}
                                                <span class="smart-select__option-label">{opt_label}</span>
                                            </div>
                                        }
                                    }).collect_view()
                                }
                            }}
                        </div>
                        
                        // Create button
                        {allow_create.then(|| {
                            let label = create_label.clone();
                            let on_create_value_inner = on_create_value_clone.clone();
                            view! {
                                <div class="smart-select__create">
                                    {move || {
                                        let query = search_query.get();
                                        let trimmed = query.trim();
                                        let opts = options_stored.get_value();
                                        let exact_match = opts.iter().any(|o| 
                                            o.label.to_lowercase() == trimmed.to_lowercase()
                                        );
                                        
                                        if !trimmed.is_empty() && !exact_match && on_create_value_inner.is_some() {
                                            let typed_text = trimmed.to_string();
                                            let typed_text_click = typed_text.clone();
                                            let callback = on_create_value_inner.clone();
                                            view! {
                                                <button
                                                    type="button"
                                                    class="smart-select__create-btn"
                                                    on:click=move |_| {
                                                        if let Some(ref cb) = callback {
                                                            cb.call(typed_text_click.clone());
                                                        }
                                                        set_is_open.set(false);
                                                        set_search_query.set(String::new());
                                                    }
                                                >
                                                    {format!("+ Add \"{}\"", typed_text)}
                                                </button>
                                            }.into_view()
                                        } else {
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
                }
            })}
            
            // Backdrop
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

