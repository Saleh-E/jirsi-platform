//! Filter Builder Component
//! 
//! Provides a universal filter builder popover for entity list views.
//! Uses Custom CustomSelect to avoid native white dropdowns.

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

/// Custom Select Component to replace native <select>
#[component]
fn CustomSelect(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] options: Signal<Vec<(String, String)>>, // (value, label)
    #[prop(into)] on_change: Callback<String>,
    #[prop(optional)] placeholder: &'static str
) -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let placeholder = if placeholder.is_empty() { "Select..." } else { placeholder };
    
    // Determine label
    let label = move || {
        let val = value.get();
        if val.is_empty() { return placeholder.to_string(); }
        options.get().into_iter().find(|(v, _)| *v == val)
             .map(|(_, l)| l).unwrap_or(val)
    };

    view! {
        <div class="custom-select relative w-full">
            <button
                class="field-input w-full text-left flex justify-between items-center bg-white/5 border border-white/10 rounded-xl px-4 py-3 hover:bg-white/10 transition-colors"
                on:click=move |e| {
                    e.stop_propagation();
                    set_is_open.update(|v| *v = !*v);
                }
            >
                <span class={move || if value.get().is_empty() { "text-zinc-500" } else { "font-medium" }}>
                    {label}
                </span>
                <i class="fa-solid fa-chevron-down text-zinc-500 text-xs transition-transform duration-300" 
                   class:rotate-180=is_open></i>
            </button>
            
            <Show when=move || is_open.get()>
                // Backdrop (click outside)
                <div class="fixed inset-0 z-40 cursor-default" on:click=move |_| set_is_open.set(false)></div>
                
                // Menu
                <div class="absolute top-full left-0 right-0 mt-2 dropdown-menu border border-white/10 rounded-xl shadow-[0_10px_40px_rgba(0,0,0,0.5)] z-50 max-h-60 overflow-y-auto custom-scrollbar animate-in fade-in zoom-in-95 duration-200">
                     {move || options.get().into_iter().map(|(val, lbl)| {
                         let val_clone = val.clone();
                         let is_selected = val == value.get();
                         view! {
                             <div 
                                class=format!("px-4 py-3 cursor-pointer transition-colors text-sm flex items-center justify-between {}", 
                                    if is_selected { "ui-dropdown-item active" } else { "ui-dropdown-item" }
                                )
                                on:click=move |e| {
                                    e.stop_propagation();
                                    on_change.call(val_clone.clone());
                                    set_is_open.set(false);
                                }
                             >
                                 <span>{lbl}</span>
                                 <Show when=move || is_selected>
                                    <i class="fa-solid fa-check text-xs"></i>
                                 </Show>
                             </div>
                         }
                     }).collect_view()}
                </div>
            </Show>
        </div>
    }
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
        "date" | "datetime" | "date_time" => vec![
            ("equals", "is exactly"),
            ("before", "is before"),
            ("after", "is after"),
            ("between", "is between"),
            ("last_7_days", "last 7 days"),
            ("last_30_days", "last 30 days"),
            ("this_week", "this week"),
            ("this_month", "this month"),
            ("last_month", "last month"),
            ("this_year", "this year"),
            ("is_empty", "is empty"),
            ("is_not_empty", "is set"),
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
    fields: Signal<Vec<FieldDef>>,
    on_add_filter: Callback<FilterCondition>,
    show_popover: RwSignal<bool>,
) -> impl IntoView {
    let (selected_field, set_selected_field) = create_signal(String::new());
    let (selected_operator, set_selected_operator) = create_signal(String::new());
    let (filter_value, set_filter_value) = create_signal(String::new());
    
    // Derived: Selected Field Def
    let selected_field_def = move || {
        let field_name = selected_field.get();
        fields.get().into_iter().find(|f| f.name == field_name)
    };
    
    // Derived: Field Options (for CustomSelect)
    let field_list_options = move || {
        fields.get().into_iter().map(|f| (f.name, f.label)).collect::<Vec<_>>()
    };
    
    // Derived: Operator Options
    let operator_options = move || {
        selected_field_def()
            .map(|f| get_operators_for_type(&f.get_field_type())
                .iter().map(|(v, l)| (v.to_string(), l.to_string())).collect::<Vec<_>>()
            )
            .unwrap_or_default()
    };
    
    // Derived: Needs Value Input?
    let needs_value_input = move || {
        let op = selected_operator.get();
        !matches!(op.as_str(), "is_empty" | "is_not_empty" | "is_true" | "is_false")
    };
    
    // Derived: Value Options (if select field)
    let value_options = move || {
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
    
    // Can apply?
    let can_apply = move || {
        let field = selected_field.get();
        let operator = selected_operator.get();
        let value = filter_value.get();
        
        !field.is_empty() && !operator.is_empty() && 
        (!needs_value_input() || !value.is_empty())
    };
    
    let filter_counter = store_value(0u32);
    
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
        
        // Reset
        set_selected_field.set(String::new());
        set_selected_operator.set(String::new());
        set_filter_value.set(String::new());
        show_popover.set(false);
    };
    
    let cancel = move |_| {
        set_selected_field.set(String::new());
        set_selected_operator.set(String::new());
        set_filter_value.set(String::new());
        show_popover.set(false);
    };

    view! {
        <Show when=move || show_popover.get()>
            <div class="filter-popover-backdrop animate-fade-in" on:click=cancel></div>
            <div class="filter-popover animate-scale-in">
                <div class="filter-popover-header">
                    <h4>"Add Filter"</h4>
                    <button class="close-btn hover:text-white transition-colors" on:click=cancel>"×"</button>
                </div>
                
                <div class="filter-popover-body space-y-4">
                    // Field Selector
                    <div class="filter-field-group">
                        <label>"Field"</label>
                        <CustomSelect 
                            value=selected_field 
                            options=Signal::derive(field_list_options)
                            on_change=Callback::new(move |v| {
                                set_selected_field.set(v);
                                set_selected_operator.set(String::new());
                                set_filter_value.set(String::new());
                            })
                            placeholder="Select field..."
                        />
                    </div>
                    
                    // Operator Selector
                    <Show when=move || !selected_field.get().is_empty()>
                        <div class="filter-field-group animate-slide-in-up">
                            <label>"Condition"</label>
                            <CustomSelect 
                                value=selected_operator 
                                options=Signal::derive(operator_options)
                                on_change=Callback::new(move |v| set_selected_operator.set(v))
                                placeholder="Select condition..."
                            />
                        </div>
                    </Show>
                    
                    // Value Input
                    <Show when=move || !selected_operator.get().is_empty() && needs_value_input()>
                        <div class="filter-field-group animate-slide-in-up">
                            <label>"Value"</label>
                            {move || {
                                if is_select_field() {
                                    view! {
                                        <CustomSelect 
                                            value=filter_value 
                                            options=Signal::derive(value_options)
                                            on_change=Callback::new(move |v| set_filter_value.set(v))
                                            placeholder="Select value..."
                                        />
                                    }.into_view()
                                } else {
                                    view! {
                                        <input 
                                            type="text"
                                            class="field-input w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 focus:border-violet-500 focus:bg-white/10 transition-all outline-none"
                                            placeholder="Enter value..."
                                            on:input=move |ev| set_filter_value.set(event_target_value(&ev))
                                            prop:value=filter_value
                                        />
                                    }.into_view()
                                }
                            }}
                        </div>
                    </Show>
                </div>
                
                <div class="filter-popover-footer">
                    <button class="ui-btn ui-btn-ghost" on:click=cancel>"Cancel"</button>
                    <button 
                        class="ui-btn ui-btn-primary"
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
    filter: FilterCondition,
    on_remove: Callback<u32>,
) -> impl IntoView {
    let filter_id = filter.id;
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
        <div class="flex items-center gap-2 bg-violet-500/10 border border-violet-500/20 rounded-full px-3 py-1 text-xs text-violet-300 animate-scale-in">
            <span>{display_text}</span>
            <button 
                class="hover:text-white transition-colors"
                on:click=move |_| on_remove.call(filter_id)
            >
                "×"
            </button>
        </div>
    }
}

/// Filter Chip Bar
#[component]
pub fn FilterChipBar(
    filters: Signal<Vec<FilterCondition>>,
    on_remove: Callback<u32>,
    on_clear_all: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || !filters.get().is_empty()>
            <div class="flex flex-wrap gap-2 items-center mb-4 animate-fade-in">
                {move || filters.get().into_iter().map(|f| {
                    let on_remove = on_remove.clone();
                    view! {
                        <FilterChip filter=f on_remove=on_remove />
                    }
                }).collect_view()}
                
                <button 
                    class="text-xs text-zinc-500 hover:text-white underline transition-colors ml-2"
                    on:click=move |_| on_clear_all.call(())
                >
                    "Clear all"
                </button>
            </div>
        </Show>
    }
}
