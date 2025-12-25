//! JsonLogic Rule Builder - Simplified visual interface
//!
//! Features:
//! - Visual condition builder
//! - AND/OR grouping
//! - Field/operator/value selection
//! - Live JSON preview

use leptos::*;
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Clone, PartialEq)]
enum LogicOperator {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Not all operators implemented yet
enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
}

impl ConditionOperator {
    fn to_jsonlogic(&self) -> &str {
        match self {
            Self::Equals => "==",
            Self::NotEquals => "!=",
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::Contains => "in",
        }
    }
}

#[derive(Debug, Clone)]
struct Condition {
    field: String,
    operator: ConditionOperator,
    value: String,
}

/// Simplified JsonLogic Rule Builder
#[component]
pub fn RuleBuilder(
    /// Callback when rule changes
    on_change: Callback<JsonValue>,
    /// Available fields for selection
    #[prop(default = vec!["name".to_string(), "status".to_string(), "amount".to_string()])]
    available_fields: Vec<String>,
) -> impl IntoView {
    let (conditions, set_conditions) = create_signal(Vec::<Condition>::new());
    let (logic_op, set_logic_op) = create_signal(LogicOperator::And);
    let (show_json, set_show_json) = create_signal(false);
    
    let update_json = move || {
        let conds = conditions.get();
        if conds.is_empty() {
            on_change.call(json!(null));
            return;
        }
        
        let json_conditions: Vec<JsonValue> = conds.iter().map(|c| {
            json!({
                c.operator.to_jsonlogic(): [
                    { "var": &c.field },
                    &c.value
                ]
            })
        }).collect();
        
        let result = match logic_op.get() {
            LogicOperator::And => json!({ "and": json_conditions }),
            LogicOperator::Or => json!({ "or": json_conditions }),
        };
        
        on_change.call(result);
    };
    
    let fields_for_add = available_fields.clone();
    let fields_for_render = available_fields.clone();
    
    let add_condition = move |_| {
        set_conditions.update(|cs| {
            cs.push(Condition {
                field: fields_for_add.first().cloned().unwrap_or_default(),
                operator: ConditionOperator::Equals,
                value: String::new(),
            });
        });
        update_json();
    };
    
    let remove_condition = move |idx: usize| {
        set_conditions.update(|cs| {
            cs.remove(idx);
        });
        update_json();
    };
    
    let toggle_logic = move |_| {
        set_logic_op.update(|op| {
            *op = match op {
                LogicOperator::And => LogicOperator::Or,
                LogicOperator::Or => LogicOperator::And,
            };
        });
        update_json();
    };

    view! {
        <div class="rule-builder bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-lg p-4">
            <div class="flex items-center justify-between mb-4">
                <h3 class="font-semibold text-gray-900 dark:text-white">
                    "Rule Builder"
                </h3>
                <button
                    type="button"
                    class="text-sm text-blue-600 hover:text-blue-700"
                    on:click=move |_| set_show_json.update(|s| *s = !*s)
                >
                    {move || if show_json.get() { "Hide JSON" } else { "Show JSON" }}
                </button>
            </div>
            
            {move || show_json.get().then(|| {
                let json_output = if conditions.get().is_empty() {
                    "{}".to_string()
                } else {
                    let conds = conditions.get();
                    let json_conditions: Vec<JsonValue> = conds.iter().map(|c| {
                        json!({
                            c.operator.to_jsonlogic(): [
                                { "var": &c.field },
                                &c.value
                            ]
                        })
                    }).collect();
                    
                    let result = match logic_op.get() {
                        LogicOperator::And => json!({ "and": json_conditions }),
                        LogicOperator::Or => json!({ "or": json_conditions }),
                    };
                    
                    serde_json::to_string_pretty(&result).unwrap_or_default()
                };
                
                view! {
                    <pre class="p-3 bg-gray-50 dark:bg-gray-900 rounded text-xs font-mono mb-4 overflow-x-auto">
                        {json_output}
                    </pre>
                }
            })}
            
            <div class="space-y-3">
                <div class="flex items-center gap-2">
                    <span class="text-sm font-medium text-gray-700 dark:text-gray-300">"IF"</span>
                    <button
                        type="button"
                        class="px-3 py-1 text-sm font-medium rounded border bg-blue-100 text-blue-700 border-blue-300"
                        on:click=toggle_logic
                    >
                        {move || match logic_op.get() {
                            LogicOperator::And => "ALL",
                            LogicOperator::Or => "ANY",
                        }}
                    </button>
                    <span class="text-sm text-gray-600 dark:text-gray-400">
                        "of the following conditions are met:"
                    </span>
                </div>
                
                {move || {
                    let conds = conditions.get();
                    if conds.is_empty() {
                        view! {
                            <div class="text-sm text-gray-500 py-4">
                                "No conditions yet. Click below to add one."
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <div class="space-y-2">
                                {conds.into_iter().enumerate().map(|(idx, cond)| {
                                    let fields = fields_for_render.clone();
                                    view! {
                                        <div class="flex items-center gap-2">
                                            <select class="form-select text-sm flex-1">
                                                {fields.iter().map(|f| {
                                                    view! {
                                                        <option selected={f == &cond.field}>{f}</option>
                                                    }
                                                }).collect_view()}
                                            </select>
                                            
                                            <select class="form-select text-sm flex-1">
                                                <option selected={matches!(cond.operator, ConditionOperator::Equals)}>"equals"</option>
                                                <option selected={matches!(cond.operator, ConditionOperator::NotEquals)}>"not equals"</option>
                                                <option selected={matches!(cond.operator, ConditionOperator::GreaterThan)}>"greater than"</option>
                                                <option selected={matches!(cond.operator, ConditionOperator::LessThan)}>"less than"</option>
                                                <option selected={matches!(cond.operator, ConditionOperator::Contains)}>"contains"</option>
                                            </select>
                                            
                                            <input
                                                type="text"
                                                class="form-input text-sm flex-1"
                                                placeholder="Value"
                                                value=&cond.value
                                            />
                                            
                                            <button
                                                type="button"
                                                class="p-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
                                                on:click=move |_| remove_condition(idx)
                                            >
                                                "✕"
                                            </button>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_view()
                    }
                }}
                
                <button
                    type="button"
                    class="text-sm text-blue-600 hover:text-blue-700"
                    on:click=add_condition
                >
                    "+ Add Condition"
                </button>
            </div>
        </div>
    }
}
