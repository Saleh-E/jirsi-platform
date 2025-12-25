//! Component Playground - Interactive showcase of Jirsi UI components
//!
//! Demonstrates:
//! - SmartField with all field types
//! - Different contexts (CreateForm, EditForm, ListView, etc.)
//! - Theme switching
//! - AsyncSelect with large datasets

use leptos::*;
use serde_json::json;
use uuid::Uuid;

use core_models::field::{FieldDef, FieldType, FieldContext, SelectChoice};
use crate::components::smart_field::SmartField;
use crate::context::theme_context::{use_theme, ThemeMode};

#[component]
pub fn ComponentPlayground() -> impl IntoView {
    let theme_ctx = use_theme();
    
    // Sample field definitions for each type
    let sample_fields = create_sample_fields();
    
    // Current context selector
    let (current_context, set_current_context) = create_signal(FieldContext::CreateForm);
    
    // Field values (for form demo)
    let field_values = create_rw_signal(std::collections::HashMap::new());
    
    view! {
        <div class="component-playground min-h-screen bg-gray-50 dark:bg-gray-900">
            <div class="container mx-auto px-4 py-8">
                // Header
                <div class="mb-8">
                    <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-2">
                        "Component Playground"
                    </h1>
                    <p class="text-gray-600 dark:text-gray-400">
                        "Interactive showcase of Jirsi's metadata-driven UI components"
                    </p>
                </div>
                
                // Controls
                <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6 mb-8">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        // Theme Switcher
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                "Theme"
                            </label>
                            <select
                                class="form-select w-full"
                                on:change=move |ev| {
                                    let mode = match event_target_value(&ev).as_str() {
                                        "light" => ThemeMode::Light,
                                        "dark" => ThemeMode::Dark,
                                        _ => ThemeMode::System,
                                    };
                                    theme_ctx.0.update(|theme| theme.mode = mode);
                                }
                            >
                                <option value="system">"System"</option>
                                <option value="light">"Light"</option>
                                <option value="dark">"Dark"</option>
                            </select>
                        </div>
                        
                        // Context Switcher
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                "Field Context"
                            </label>
                            <select
                                class="form-select w-full"
                                on:change=move |ev| {
                                    let ctx = match event_target_value(&ev).as_str() {
                                        "create_form" => FieldContext::CreateForm,
                                        "edit_form" => FieldContext::EditForm,
                                        "list_view" => FieldContext::ListView,
                                        "detail_view" => FieldContext::DetailView,
                                        "kanban_card" => FieldContext::KanbanCard,
                                        "filter_builder" => FieldContext::FilterBuilder,
                                        _ => FieldContext::InlineEdit,
                                    };
                                    set_current_context.set(ctx);
                                }
                            >
                                <option value="create_form">"Create Form"</option>
                                <option value="edit_form">"Edit Form"</option>
                                <option value="list_view">"List View"</option>
                                <option value="detail_view">"Detail View"</option>
                                <option value="kanban_card">"Kanban Card"</option>
                                <option value="filter_builder">"Filter Builder"</option>
                                <option value="inline_edit">"Inline Edit"</option>
                            </select>
                        </div>
                    </div>
                </div>
                
                // Field Showcase
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    {sample_fields.into_iter().map(|(category, fields)| {
                        view! {
                            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
                                <h2 class="text-2xl font-semibold text-gray-900 dark:text-white mb-4">
                                    {category}
                                </h2>
                                <div class="space-y-4">
                                    {fields.into_iter().map(|(field, default_value)| {
                                        let field_id = field.id.clone();
                                        let value_signal = create_memo(move |_| {
                                            field_values.with(|values| {
                                                values.get(&field_id)
                                                    .cloned()
                                                    .unwrap_or_else(|| default_value.clone())
                                            })
                                        });
                                        
                                        view! {
                                            <div class="border-b border-gray-200 dark:border-gray-700 pb-4 last:border-0">
                                                <SmartField
                                                    field=field.clone()
                                                    value=Signal::from(value_signal)
                                                    context=current_context.get()
                                                    on_change=Callback::new(move |new_val| {
                                                        field_values.update(|values| {
                                                            values.insert(field_id.clone(), new_val);
                                                        });
                                                    })
                                                />
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
                
                // JSON Output (for debugging)
                <div class="mt-8 bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
                    <h2 class="text-2xl font-semibold text-gray-900 dark:text-white mb-4">
                        "Field Values (JSON)"
                    </h2>
                    <pre class="bg-gray-100 dark:bg-gray-900 p-4 rounded-md overflow-auto text-sm">
                        <code>
                            {move || {
                                let values = field_values.get();
                                serde_json::to_string_pretty(&values)
                                    .unwrap_or_else(|_| "{}".to_string())
                            }}
                        </code>
                    </pre>
                </div>
            </div>
        </div>
    }
}

/// Create sample field definitions for each category
fn create_sample_fields() -> Vec<(&'static str, Vec<(FieldDef, serde_json::Value)>)> {
    use uuid::Uuid;
    
    let tenant_id = Uuid::new_v4();
    let entity_type_id = Uuid::new_v4();
    
    vec![
        ("Basic Fields", vec![
            (create_field(tenant_id, entity_type_id, "text_field", "Full Name", FieldType::Text), json!("John Doe")),
            (create_field(tenant_id, entity_type_id, "email_field", "Email Address", FieldType::Email), json!("john@example.com")),
            (create_field(tenant_id, entity_type_id, "phone_field", "Phone Number", FieldType::Phone), json!("+1234567890")),
            (create_field(tenant_id, entity_type_id, "textarea_field", "Description", FieldType::TextArea), json!("This is a sample description...")),
        ]),
        
        ("Number & Money", vec![
            (create_field(tenant_id, entity_type_id, "number_field", "Quantity", FieldType::Number { decimals: Some(0) }), json!(42)),
            (create_field(tenant_id, entity_type_id, "decimal_field", "Rate", FieldType::Number { decimals: Some(2) }), json!(3.14)),
            (create_field(tenant_id, entity_type_id, "money_field", "Price", FieldType::Money { currency_code: Some("USD".to_string()) }), json!(99.99)),
        ]),
        
        ("Date & Boolean", vec![
            (create_field(tenant_id, entity_type_id, "date_field", "Birth Date", FieldType::Date), json!("1990-01-15")),
            (create_field(tenant_id, entity_type_id, "datetime_field", "Created At", FieldType::DateTime), json!("2024-12-24T21:00:00")),
            (create_field(tenant_id, entity_type_id, "boolean_field", "Active", FieldType::Boolean), json!(true)),
        ]),
        
        ("Advanced Fields", vec![
            (create_field(tenant_id, entity_type_id, "color_field", "Brand Color", FieldType::ColorPicker), json!("#3B82F6")),
            (create_dropdown_field(tenant_id, entity_type_id, "status_field", "Status"), json!("active")),
            (create_select_field(tenant_id, entity_type_id, "priority_field", "Priority"), json!("high")),
        ]),
    ]
}

fn create_field(tenant_id: Uuid, entity_type_id: Uuid, name: &str, label: &str, field_type: FieldType) -> FieldDef {
    FieldDef {
        id: Uuid::new_v4(),
        tenant_id,
        entity_type_id,
        name: name.to_string(),
        label: label.to_string(),
        field_type,
        is_required: false,
        is_unique: false,
        show_in_list: true,
        show_in_card: true,
        is_searchable: true,
        is_filterable: true,
        is_sortable: true,
        is_readonly: false,
        default_value: None,
        placeholder: Some(format!("Enter {}", label.to_lowercase())),
        help_text: Some(format!("Sample help text for {}", label)),
        validation: None,
        options: None,
        ui_hints: None,
        context_hints: None,
        sort_order: 0,
        group: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

fn create_dropdown_field(tenant_id: Uuid, entity_type_id: Uuid, name: &str, label: &str) -> FieldDef {
    let mut field = create_field(tenant_id, entity_type_id, name, label, FieldType::Dropdown {
        options: vec![
            SelectChoice { 
                value: "active".to_string(), 
                label: "Active".to_string(), 
                color: Some("#10B981".to_string()), 
                icon: None,
                is_default: false,
                sort_order: 0,
            },
            SelectChoice { 
                value: "pending".to_string(), 
                label: "Pending".to_string(), 
                color: Some("#F59E0B".to_string()), 
                icon: None,
                is_default: false,
                sort_order: 1,
            },
            SelectChoice { 
                value: "inactive".to_string(), 
                label: "Inactive".to_string(), 
                color: Some("#EF4444".to_string()), 
                icon: None,
                is_default: false,
                sort_order: 2,
            },
        ],
        allow_create: true,
    });
    field.placeholder = Some("Select or create status".to_string());
    field
}

fn create_select_field(tenant_id: Uuid, entity_type_id: Uuid, name: &str, label: &str) -> FieldDef {
    create_field(tenant_id, entity_type_id, name, label, FieldType::Select {
        options: vec!["low".to_string(), "medium".to_string(), "high".to_string(), "urgent".to_string()],
    })
}
