//! Filter Builder Component
//! 
//! Provides a universal filter builder popover for entity list views.
//! Supports field/operator/value filtering with visual filter chips.

use leptos::*;
use crate::api::FieldDef;

/// A single filter condition
#[derive(Clone, Debug, PartialEq)]
pub struct FilterCondition {
    pub id: u32,
    pub field: String,
    pub field_label: String,
    pub operator: String,
    pub value: String,
}

/// Get operators for a field type
fn get_operators_for_type(field_type: &str) -> Vec<(&'static str, &'static str)> {
    match field_type {
        "text" | "email" | "phone" | "url" => vec![
            ("contains", "contains"),
            ("equals", "equals"),
            ("starts_with", "starts with"),
            ("ends_with", "ends with"),
            ("is_empty", "is empty"),
            ("is_not_empty", "is not empty"),
        ],
        "number" | "currency" | "percent" => vec![
            ("equals", "equals"),
            ("not_equals", "not equals"),
            ("gt", "greater than"),
            ("gte", "greater or equal"),
            ("lt", "less than"),
            ("lte", "less or equal"),
        ],
        "date" | "datetime" => vec![
            ("equals", "is"),
            ("before", "is before"),
            ("after", "is after"),
            ("between", "is between"),
            ("is_empty", "is empty"),
        ],
        "select" | "status" => vec![
            ("equals", "is"),
            ("not_equals", "is not"),
            ("is_any", "is any of"),
        ],
        "boolean" | "checkbox" => vec![
            ("is_true", "is true"),
            ("is_false", "is false"),
        ],
        "lookup" => vec![
            ("equals", "is"),
            ("not_equals", "is not"),
            ("is_empty", "is empty"),
        ],
        _ => vec![
            ("contains", "contains"),
            ("equals", "equals"),
        ],
    }
}

/// Filter Builder Popover Component
#[component]
pub fn FilterBuilder(
    /// Available fields from metadata
    fields: Signal<Vec<FieldDef>>,
    /// Callback when filter is added
    on_add_filter: Callback<FilterCondition>,
    /// Signal to control popover visibility
    show_popover: RwSignal<bool>,
) -> impl IntoView {
    let (selected_field, set_selected_field) = create_signal(String::new());
    let (selected_operator, set_selected_operator) = create_signal(String::new());
    let (filter_value, set_filter_value) = create_signal(String::new());
    
    // Get the currently selected field definition
    let selected_field_def = move || {
        let field_name = selected_field.get();
        fields.get().into_iter().find(|f| f.name == field_name)
    };
    
    // Get operators for the selected field
    let available_operators = move || {
        selected_field_def()
            .map(|f| get_operators_for_type(&f.get_field_type()))
            .unwrap_or_default()
    };
    
    // Check if value input is needed (some operators like is_empty don't need value)
    let needs_value_input = move || {
        let op = selected_operator.get();
        !matches!(op.as_str(), "is_empty" | "is_not_empty" | "is_true" | "is_false")
    };
    
    // Get options if field is a select type
    let field_options = move || {
        selected_field_def()
            .map(|f| f.get_options())
            .unwrap_or_default()
    };
    
    // Check if field is select type
    let is_select_field = move || {
        selected_field_def()
            .map(|f| {
                let ft = f.get_field_type();
                ft == "select" || ft == "status"
            })
            .unwrap_or(false)
    };
    
    // Can apply filter?
    let can_apply = move || {
        let field = selected_field.get();
        let operator = selected_operator.get();
        let value = filter_value.get();
        
        !field.is_empty() && !operator.is_empty() && 
        (!needs_value_input() || !value.is_empty())
    };
    
    // Counter for unique filter IDs
    let filter_counter = store_value(0u32);
    
    // Apply filter
    let apply_filter = move |_| {
        if !can_apply() { return; }
        
        let field_def = selected_field_def();
        let label = field_def.map(|f| f.label.clone()).unwrap_or_else(|| selected_field.get());
        
        filter_counter.update_value(|c| *c += 1);
        
        let condition = FilterCondition {
            id: filter_counter.get_value(),
            field: selected_field.get(),
            field_label: label,
            operator: selected_operator.get(),
            value: filter_value.get(),
        };
        
        on_add_filter.call(condition);
        
        // Reset form
        set_selected_field.set(String::new());
        set_selected_operator.set(String::new());
        set_filter_value.set(String::new());
        show_popover.set(false);
    };
    
    // Cancel
    let cancel = move |_| {
        set_selected_field.set(String::new());
        set_selected_operator.set(String::new());
        set_filter_value.set(String::new());
        show_popover.set(false);
    };

    view! {
        <Show when=move || show_popover.get()>
            <div class="filter-popover-backdrop" on:click=cancel></div>
            <div class="filter-popover">
                <div class="filter-popover-header">
                    <h4>"Add Filter"</h4>
                    <button class="close-btn" on:click=cancel>"×"</button>
                </div>
                
                <div class="filter-popover-body">
                    // Field selector
                    <div class="filter-field-group">
                        <label>"Field"</label>
                        <select 
                            class="filter-select"
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                set_selected_field.set(value);
                                set_selected_operator.set(String::new());
                                set_filter_value.set(String::new());
                            }
                            prop:value=selected_field
                        >
                            <option value="">"Select field..."</option>
                            {move || fields.get().into_iter().map(|f| {
                                let name = f.name.clone();
                                let label = f.label.clone();
                                view! {
                                    <option value=name>{label}</option>
                                }
                            }).collect_view()}
                        </select>
                    </div>
                    
                    // Operator selector (shown after field is selected)
                    <Show when=move || !selected_field.get().is_empty()>
                        <div class="filter-field-group">
                            <label>"Condition"</label>
                            <select 
                                class="filter-select"
                                on:change=move |ev| {
                                    set_selected_operator.set(event_target_value(&ev));
                                }
                                prop:value=selected_operator
                            >
                                <option value="">"Select condition..."</option>
                                {move || available_operators().into_iter().map(|(value, label)| {
                                    view! {
                                        <option value=value>{label}</option>
                                    }
                                }).collect_view()}
                            </select>
                        </div>
                    </Show>
                    
                    // Value input (shown after operator is selected, if needed)
                    <Show when=move || !selected_operator.get().is_empty() && needs_value_input()>
                        <div class="filter-field-group">
                            <label>"Value"</label>
                            {move || {
                                if is_select_field() {
                                    // Dropdown for select fields
                                    view! {
                                        <select 
                                            class="filter-select"
                                            on:change=move |ev| {
                                                set_filter_value.set(event_target_value(&ev));
                                            }
                                            prop:value=filter_value
                                        >
                                            <option value="">"Select value..."</option>
                                            {move || field_options().into_iter().map(|(value, label)| {
                                                view! {
                                                    <option value=value>{label}</option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    }.into_view()
                                } else {
                                    // Text input for other fields
                                    view! {
                                        <input 
                                            type="text"
                                            class="filter-input"
                                            placeholder="Enter value..."
                                            on:input=move |ev| {
                                                set_filter_value.set(event_target_value(&ev));
                                            }
                                            prop:value=filter_value
                                        />
                                    }.into_view()
                                }
                            }}
                        </div>
                    </Show>
                </div>
                
                <div class="filter-popover-footer">
                    <button class="btn btn-secondary" on:click=cancel>"Cancel"</button>
                    <button 
                        class="btn btn-primary"
                        disabled=move || !can_apply()
                        on:click=apply_filter
                    >
                        "Apply Filter"
                    </button>
                </div>
            </div>
        </Show>
    }
}

/// Single Filter Chip Component
#[component]
pub fn FilterChip(
    /// The filter condition to display
    filter: FilterCondition,
    /// Callback when chip is removed
    on_remove: Callback<u32>,
) -> impl IntoView {
    let filter_id = filter.id;
    let _display_label = format!("{}: {}", filter.field_label, filter.value);
    
    let operator_label = match filter.operator.as_str() {
        "contains" => "contains",
        "equals" => "is",
        "not_equals" => "is not",
        "gt" => ">",
        "gte" => "≥",
        "lt" => "<",
        "lte" => "≤",
        "before" => "before",
        "after" => "after",
        "is_empty" => "is empty",
        "is_not_empty" => "is not empty",
        "is_true" => "is true",
        "is_false" => "is false",
        _ => "=",
    };
    
    let display_text = if filter.value.is_empty() {
        format!("{} {}", filter.field_label, operator_label)
    } else {
        format!("{} {} {}", filter.field_label, operator_label, filter.value)
    };
    
    view! {
        <div class="filter-chip">
            <span class="chip-text">{display_text}</span>
            <button 
                class="chip-remove"
                on:click=move |_| on_remove.call(filter_id)
                title="Remove filter"
            >
                "×"
            </button>
        </div>
    }
}

/// Filter Chip Bar - displays all active filters
#[component]
pub fn FilterChipBar(
    /// Active filters
    filters: Signal<Vec<FilterCondition>>,
    /// Callback to remove a filter
    on_remove: Callback<u32>,
    /// Callback to clear all filters
    on_clear_all: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || !filters.get().is_empty()>
            <div class="filter-chip-bar">
                <div class="filter-chips">
                    {move || filters.get().into_iter().map(|f| {
                        let on_remove = on_remove.clone();
                        view! {
                            <FilterChip filter=f on_remove=on_remove />
                        }
                    }).collect_view()}
                </div>
                <button 
                    class="clear-filters-btn"
                    on:click=move |_| on_clear_all.call(())
                >
                    "Clear all"
                </button>
            </div>
        </Show>
    }
}
