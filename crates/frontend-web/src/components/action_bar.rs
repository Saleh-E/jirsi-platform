//! Action Bar Component - Filter, Sort, Columns, Export
//!
//! Provides a toolbar above list views with actions to manipulate data display

use leptos::*;
use crate::api::FieldDef;

/// Sort direction
#[derive(Clone, Debug, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Active filter
#[derive(Clone, Debug)]
pub struct ActiveFilter {
    pub field: String,
    pub operator: String,
    pub value: String,
}

/// Action Bar props
#[derive(Clone)]
pub struct ActionBarState {
    pub filters: Vec<ActiveFilter>,
    pub sort_field: Option<String>,
    pub sort_direction: SortDirection,
    pub visible_columns: Vec<String>,
    pub density: String, // "compact" | "comfortable" | "spacious"
}

impl Default for ActionBarState {
    fn default() -> Self {
        Self {
            filters: vec![],
            sort_field: None,
            sort_direction: SortDirection::Asc,
            visible_columns: vec![],
            density: "comfortable".to_string(),
        }
    }
}

/// Action Bar component with Filter, Sort, Columns, Export
#[component]
pub fn ActionBar(
    /// Available fields for filtering/sorting
    fields: Vec<FieldDef>,
    /// Current state
    #[prop(into)]
    state: RwSignal<ActionBarState>,
    /// Callback when state changes
    #[prop(optional, into)]
    on_change: Option<Callback<ActionBarState>>,
) -> impl IntoView {
    let (show_filters, set_show_filters) = create_signal(false);
    let (show_columns, set_show_columns) = create_signal(false);
    let (show_sort, set_show_sort) = create_signal(false);
    
    // Filter operators based on field type
    let _get_operators = |field_type: &str| -> Vec<(&'static str, &'static str)> {
        match field_type {
            "text" | "string" | "email" => vec![
                ("contains", "Contains"),
                ("equals", "Equals"),
                ("starts_with", "Starts with"),
                ("ends_with", "Ends with"),
            ],
            "number" | "integer" | "currency" => vec![
                ("equals", "="),
                ("not_equals", "≠"),
                ("greater_than", ">"),
                ("less_than", "<"),
                ("greater_or_equal", "≥"),
                ("less_or_equal", "≤"),
            ],
            "date" | "datetime" => vec![
                ("equals", "Is"),
                ("before", "Before"),
                ("after", "After"),
                ("between", "Between"),
            ],
            "select" | "status" => vec![
                ("equals", "Is"),
                ("not_equals", "Is not"),
            ],
            _ => vec![
                ("contains", "Contains"),
                ("equals", "Equals"),
            ],
        }
    };
    
    // Add filter
    let add_filter = move |field: String, operator: String, value: String| {
        state.update(|s| {
            s.filters.push(ActiveFilter { field, operator, value });
        });
        if let Some(cb) = on_change {
            cb.call(state.get());
        }
    };
    
    // Remove filter
    let remove_filter = move |index: usize| {
        state.update(|s| {
            if index < s.filters.len() {
                s.filters.remove(index);
            }
        });
        if let Some(cb) = on_change {
            cb.call(state.get());
        }
    };
    
    // Set sort
    let set_sort = move |field: String| {
        state.update(|s| {
            if s.sort_field.as_ref() == Some(&field) {
                // Toggle direction
                s.sort_direction = match s.sort_direction {
                    SortDirection::Asc => SortDirection::Desc,
                    SortDirection::Desc => SortDirection::Asc,
                };
            } else {
                s.sort_field = Some(field);
                s.sort_direction = SortDirection::Asc;
            }
        });
        if let Some(cb) = on_change {
            cb.call(state.get());
        }
    };
    
    // Toggle column visibility
    let toggle_column = move |field: String| {
        state.update(|s| {
            if s.visible_columns.contains(&field) {
                s.visible_columns.retain(|c| c != &field);
            } else {
                s.visible_columns.push(field);
            }
        });
        if let Some(cb) = on_change {
            cb.call(state.get());
        }
    };
    
    // Set density
    let set_density = move |density: String| {
        state.update(|s| {
            s.density = density;
        });
        if let Some(cb) = on_change {
            cb.call(state.get());
        }
    };
    
    // Export handler
    let export_csv = move |_| {
        // TODO: Implement CSV export
        web_sys::window()
            .and_then(|w| w.alert_with_message("Export to CSV - Coming soon!").ok());
    };
    
    let fields_for_filter = fields.clone();
    let fields_for_sort = fields.clone();
    let fields_for_columns = fields.clone();
    
    view! {
        <div class="action-bar">
            // Filter button with active count
            <div class="action-bar-group">
                <button 
                    class="action-btn"
                    class:active=move || !state.get().filters.is_empty()
                    on:click=move |_| set_show_filters.update(|v| *v = !*v)
                >
                    <span class="icon">"🔍"</span>
                    <span class="label">"Filter"</span>
                    {move || {
                        let count = state.get().filters.len();
                        (count > 0).then(|| view! {
                            <span class="badge">{count}</span>
                        })
                    }}
                </button>
                
                // Filter dropdown
                {move || show_filters.get().then(|| {
                    let fields = fields_for_filter.clone();
                    view! {
                        <div class="action-dropdown filter-dropdown">
                            <div class="dropdown-header">
                                <h4>"Filters"</h4>
                                <button class="btn-close" on:click=move |_| set_show_filters.set(false)>"×"</button>
                            </div>
                            <div class="dropdown-body">
                                // Active filters
                                <div class="active-filters">
                                    {move || {
                                        state.get().filters.iter().enumerate().map(|(i, f)| {
                                            let idx = i;
                                            view! {
                                                <div class="filter-chip">
                                                    <span>{f.field.clone()} " " {f.operator.clone()} " " {f.value.clone()}</span>
                                                    <button class="chip-remove" on:click=move |_| remove_filter(idx)>"×"</button>
                                                </div>
                                            }
                                        }).collect_view()
                                    }}
                                </div>
                                
                                // Add new filter
                                <div class="add-filter">
                                    <select id="filter-field" class="filter-select">
                                        <option value="">"Select field..."</option>
                                        {fields.iter().map(|f| {
                                            view! { <option value=f.name.clone()>{f.label.clone()}</option> }
                                        }).collect_view()}
                                    </select>
                                    <select id="filter-operator" class="filter-select">
                                        <option value="contains">"Contains"</option>
                                        <option value="equals">"Equals"</option>
                                    </select>
                                    <input type="text" id="filter-value" class="filter-input" placeholder="Value..." />
                                    <button class="btn btn-sm btn-primary" on:click=move |_| {
                                        // Get values from inputs
                                        if let Some(window) = web_sys::window() {
                                            if let Some(doc) = window.document() {
                                                let field = doc.get_element_by_id("filter-field")
                                                    .and_then(|el| el.dyn_into::<web_sys::HtmlSelectElement>().ok())
                                                    .map(|el| el.value())
                                                    .unwrap_or_default();
                                                let op = doc.get_element_by_id("filter-operator")
                                                    .and_then(|el| el.dyn_into::<web_sys::HtmlSelectElement>().ok())
                                                    .map(|el| el.value())
                                                    .unwrap_or_default();
                                                let val = doc.get_element_by_id("filter-value")
                                                    .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
                                                    .map(|el| el.value())
                                                    .unwrap_or_default();
                                                if !field.is_empty() && !val.is_empty() {
                                                    add_filter(field, op, val);
                                                }
                                            }
                                        }
                                    }>"Add"</button>
                                </div>
                            </div>
                        </div>
                    }
                })}
            </div>
            
            // Sort button
            <div class="action-bar-group">
                <button 
                    class="action-btn"
                    class:active=move || state.get().sort_field.is_some()
                    on:click=move |_| set_show_sort.update(|v| *v = !*v)
                >
                    <span class="icon">"↕"</span>
                    <span class="label">"Sort"</span>
                </button>
                
                {move || show_sort.get().then(|| {
                    let fields = fields_for_sort.clone();
                    let current = state.get().sort_field.clone();
                    let dir = state.get().sort_direction.clone();
                    view! {
                        <div class="action-dropdown sort-dropdown">
                            <div class="dropdown-header">
                                <h4>"Sort by"</h4>
                                <button class="btn-close" on:click=move |_| set_show_sort.set(false)>"×"</button>
                            </div>
                            <div class="dropdown-body">
                                {fields.iter().map(|f| {
                                    let field_name = f.name.clone();
                                    let is_current = current.as_ref() == Some(&field_name);
                                    let direction = if is_current {
                                        match dir {
                                            SortDirection::Asc => " ↑",
                                            SortDirection::Desc => " ↓",
                                        }
                                    } else { "" };
                                    let fn_clone = field_name.clone();
                                    view! {
                                        <button 
                                            class="sort-option"
                                            class:active=is_current
                                            on:click=move |_| {
                                                set_sort(fn_clone.clone());
                                                set_show_sort.set(false);
                                            }
                                        >
                                            {f.label.clone()} {direction}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    }
                })}
            </div>
            
            // Columns button
            <div class="action-bar-group">
                <button 
                    class="action-btn"
                    on:click=move |_| set_show_columns.update(|v| *v = !*v)
                >
                    <span class="icon">"⊞"</span>
                    <span class="label">"Columns"</span>
                </button>
                
                {move || show_columns.get().then(|| {
                    let fields = fields_for_columns.clone();
                    let visible = state.get().visible_columns.clone();
                    view! {
                        <div class="action-dropdown columns-dropdown">
                            <div class="dropdown-header">
                                <h4>"Visible Columns"</h4>
                                <button class="btn-close" on:click=move |_| set_show_columns.set(false)>"×"</button>
                            </div>
                            <div class="dropdown-body">
                                {fields.iter().map(|f| {
                                    let field_name = f.name.clone();
                                    let is_visible = visible.contains(&field_name) || visible.is_empty();
                                    let fn_clone = field_name.clone();
                                    view! {
                                        <label class="column-option">
                                            <input 
                                                type="checkbox" 
                                                checked=is_visible
                                                on:change=move |_| toggle_column(fn_clone.clone())
                                            />
                                            {f.label.clone()}
                                        </label>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    }
                })}
            </div>
            
            // Density toggle
            <div class="action-bar-group density-group">
                <button 
                    class="action-btn density-btn"
                    class:active=move || state.get().density == "compact"
                    on:click=move |_| set_density("compact".to_string())
                    title="Compact"
                >
                    <span class="density-icon">"≡"</span>
                </button>
                <button 
                    class="action-btn density-btn"
                    class:active=move || state.get().density == "comfortable"
                    on:click=move |_| set_density("comfortable".to_string())
                    title="Comfortable"
                >
                    <span class="density-icon">"☰"</span>
                </button>
            </div>
            
            // Spacer
            <div class="action-bar-spacer"></div>
            
            // Export button
            <button class="action-btn" on:click=export_csv>
                <span class="icon">"📥"</span>
                <span class="label">"Export"</span>
            </button>
        </div>
    }
}

use wasm_bindgen::JsCast;
