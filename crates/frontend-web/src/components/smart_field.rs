//! SmartField - Polymorphic field component that renders based on FieldType + FieldContext
//!
//! The Golden Rule: Never hardcode <input> or <select> - always use SmartField

use leptos::*;
use serde_json::Value as JsonValue;
use wasm_bindgen::JsCast;

use core_models::field::{FieldDef, FieldType, FieldContext};
use crate::components::async_select::{AsyncSelect, SelectOption};

/// SmartField - Intelligently renders fields based on type and context
#[component]
pub fn SmartField(
    field: FieldDef,
    #[prop(into)] value: Signal<JsonValue>,
    context: FieldContext,
    #[prop(optional)] on_change: Option<Callback<JsonValue>>,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let field_id = create_rw_signal(format!("field-{}", field.id));
    let is_readonly = field.is_readonly || disabled;
    
    view! {
        <div 
            class="smart-field"
            data-field-type={format!("{:?}", field.field_type)}
            data-context={format!("{:?}", context)}
        >
            {move || render_field_by_context(
                &field,
                value.get(),
                context,
                field_id.get(),
                is_readonly,
                on_change
            )}
        </div>
    }
}

fn render_field_by_context(
    field: &FieldDef,
    value: JsonValue,
    context: FieldContext,
    field_id: String,
    is_readonly: bool,
    on_change: Option<Callback<JsonValue>>,
) -> View {
    match context {
        FieldContext::ListView => render_list_cell(field, &value),
        FieldContext::DetailView => render_detail_display(field, &value),
        FieldContext::KanbanCard => render_kanban_cell(field, &value),
        FieldContext::CreateForm | FieldContext::EditForm | FieldContext::InlineEdit => {
            render_form_input(field, value, field_id, is_readonly, on_change)
        },
        FieldContext::FilterBuilder => render_filter_input(field, value, on_change),
    }
}

// ==================
// List View Rendering
// ==================

fn render_list_cell(field: &FieldDef, value: &JsonValue) -> View {
    match &field.field_type {
        FieldType::Dropdown { .. } | FieldType::Select { .. } => {
            let val_str = value.as_str().unwrap_or("").to_string();
            view! {
                <span class="badge status-badge">{val_str}</span>
            }.into_view()
        },
        FieldType::Association {target_entity, ..} | FieldType::Link {target_entity} => {
            let val_str = value.as_str().unwrap_or("").to_string();
            let target = target_entity.clone();
            view! {
                <a href={format!("/app/crm/entity/{}/{}", target, val_str.clone())} class="entity-link">
                    {val_str}
                </a>
            }.into_view()
        },
        FieldType::Boolean => {
            let is_true = value.as_bool().unwrap_or(false);
            view! {
                <span class={if is_true { "icon-check text-green-600" } else { "icon-x text-gray-400" }}>
                    {if is_true { "✓" } else { "✗" }}
                </span>
            }.into_view()
        },
        FieldType::ColorPicker => {
            let color = value.as_str().unwrap_or("#000000").to_string();
            view! {
                <div class="flex items-center gap-2">
                    <div class="w-6 h-6 rounded border" style:background-color=color.clone()></div>
                    <span>{color}</span>
                </div>
            }.into_view()
        },
        _ => {
            // Default: plain text
            let display_value = format_display_value(field, value);
            view! {
                <span>{display_value}</span>
            }.into_view()
        }
    }
}

// ==================
// Detail View Rendering
// ==================

fn render_detail_display(field: &FieldDef, value: &JsonValue) -> View {
    view! {
        <div class="field-detail">
            <label class="field-label">{&field.label}</label>
            <div class="field-value">
                {render_list_cell(field, value)}
            </div>
        </div>
    }.into_view()
}

// ==================
// Kanban Card Rendering
// ==================

fn render_kanban_cell(field: &FieldDef, value: &JsonValue) -> View {
    let display_value = format_display_value(field, value);
    view! {
        <div class="kanban-field">
            <span class="field-label-compact">{&field.label}:</span>
            <span class="field-value-compact">{display_value}</span>
        </div>
    }.into_view()
}

// ==================
// Form Input Rendering
// ==================

fn render_form_input(
    field: &FieldDef,
    value: JsonValue,
    field_id: String,
    is_readonly: bool,
    on_change: Option<Callback<JsonValue>>,
) -> View {
    let show_label = field.ui_hints.as_ref().map(|h| !h.hide_label).unwrap_or(true);
    let field_clone = field.clone();
    let field_id_for_label = field_id.clone();
    
    view! {
        <div class="form-field">
            {if show_label {
                view! {
                    <label 
                        for=field_id_for_label
                        class="form-label"
                        class:required=field_clone.is_required
                    >
                        {&field_clone.label}
                    </label>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
            
            {render_input_control(field, value, field_id, is_readonly, on_change)}
            
            {field.help_text.as_ref().map(|help| view! {
                <span class="form-help-text">{help}</span>
            })}
        </div>
    }.into_view()
}

fn render_input_control(
    field: &FieldDef,
    value: JsonValue,
    field_id: String,
    is_readonly: bool,
    on_change: Option<Callback<JsonValue>>,
) -> View {
    let val_str = value.as_str().unwrap_or("").to_string();
    
    match &field.field_type {
        FieldType::Text | FieldType::Email | FieldType::Phone | FieldType::Url => {
            let input_type = match &field.field_type {
                FieldType::Email => "email",
                FieldType::Phone => "tel",
                FieldType::Url => "url",
                _ => "text",
            };
            
            view! {
                <input
                    type=input_type
                    id=field_id
                    class="form-input"
                    value=val_str
                    placeholder=field.placeholder.clone().unwrap_or_default()
                    required=field.is_required
                    disabled=is_readonly
                    on:input=move |ev| {
                        if let Some(cb) = on_change {
                            let new_val = event_target_value(&ev);
                            cb.call(JsonValue::String(new_val));
                        }
                    }
                />
            }.into_view()
        },
        FieldType::TextArea => {
            view! {
                <textarea
                    id=field_id
                    class="form-textarea"
                    placeholder=field.placeholder.clone().unwrap_or_default()
                    required=field.is_required
                    disabled=is_readonly
                    on:input=move |ev| {
                        if let Some(cb) = on_change {
                            let new_val = event_target_value(&ev);
                            cb.call(JsonValue::String(new_val));
                        }
                    }
                >
                    {val_str}
                </textarea>
            }.into_view()
        },
        FieldType::Number { decimals } => {
            let step = if decimals.unwrap_or(0) > 0 { "0.01" } else { "1" };
            view! {
                <input
                    type="number"
                    id=field_id
                    class="form-input"
                    value=val_str
                    required=field.is_required
                    disabled=is_readonly
                    step=step
                    on:input=move |ev| {
                        if let Some(cb) = on_change {
                            let new_val = event_target_value(&ev);
                            if let Ok(num) = new_val.parse::<f64>() {
                                cb.call(JsonValue::Number(serde_json::Number::from_f64(num).unwrap()));
                            }
                        }
                    }
                />
            }.into_view()
        },
        FieldType::Money { currency_code: _ } => {
            view! {
                <input
                    type="number"
                    id=field_id
                    class="form-input"
                    value=val_str
                    required=field.is_required
                    disabled=is_readonly
                    step="0.01"
                    on:input=move |ev| {
                        if let Some(cb) = on_change {
                            let new_val = event_target_value(&ev);
                            if let Ok(num) = new_val.parse::<f64>() {
                                 cb.call(JsonValue::Number(serde_json::Number::from_f64(num).unwrap()));
                            }
                        }
                    }
                />
            }.into_view()
        },
        FieldType::Boolean => {
            let checked = value.as_bool().unwrap_or(false);
            view! {
                <input
                    type="checkbox"
                    id=field_id
                    class="form-checkbox"
                    checked=checked
                    disabled=is_readonly
                    on:change=move |ev| {
                        if let Some(cb) = on_change {
                            let checked = event_target_checked(&ev);
                            cb.call(JsonValue::Bool(checked));
                        }
                    }
                />
            }.into_view()
        },
        FieldType::Date => {
            view! {
                <input
                    type="date"
                    id=field_id
                    class="form-input"
                    value=val_str
                    required=field.is_required
                    disabled=is_readonly
                    on:input=move |ev| {
                        if let Some(cb) = on_change {
                            let new_val = event_target_value(&ev);
                            cb.call(JsonValue::String(new_val));
                        }
                    }
                />
            }.into_view()
        },
        FieldType::DateTime => {
            view! {
                <input
                    type="datetime-local"
                    id=field_id
                    class="form-input"
                    value=val_str
                    required=field.is_required
                    disabled=is_readonly
                    on:input=move |ev| {
                        if let Some(cb) = on_change {
                            let new_val = event_target_value(&ev);
                            cb.call(JsonValue::String(new_val));
                        }
                    }
                />
            }.into_view()
        },
        FieldType::ColorPicker => {
            view! {
                <input
                    type="color"
                    id=field_id
                    class="form-color-picker"
                    value=val_str
                    disabled=is_readonly
                    on:input=move |ev| {
                        if let Some(cb) = on_change {
                            let new_val = event_target_value(&ev);
                            cb.call(JsonValue::String(new_val));
                        }
                    }
                />
            }.into_view()
        },
        FieldType::Dropdown { options, allow_create } => {
            // Convert SelectChoice to SelectOption
            let select_options: Vec<SelectOption> = options.iter().map(|choice| {
                SelectOption {
                    value: choice.value.clone(),
                    label: choice.label.clone(),
                    description: None,
                    color: choice.color.clone(),
                    icon: choice.icon.clone(),
                }
            }).collect();
            
            let options_signal = create_rw_signal(select_options);
            let selected_value = create_rw_signal(value.as_str().map(|s| s.to_string()));
            let allow_inline_create = *allow_create;
            
            view! {
                <AsyncSelect
                    value=Signal::from(selected_value)
                    on_change=Callback::new(move |new_val: Option<String>| {
                        if let Some(cb) = on_change {
                            cb.call(new_val.map(JsonValue::String).unwrap_or(JsonValue::Null));
                        }
                    })
                    options=Signal::from(options_signal)
                    placeholder=field.placeholder.clone().unwrap_or_default()
                    allow_create=allow_inline_create
                    on_create={
                        if allow_inline_create {
                            Callback::new(move |new_label: String| {
                                // Add new option to list
                                options_signal.update(|opts| {
                                    opts.push(SelectOption::new(new_label.clone(), new_label.clone()));
                                });
                                selected_value.set(Some(new_label.clone()));
                                if let Some(cb) = on_change {
                                    cb.call(JsonValue::String(new_label));
                                }
                            })
                        } else {
                            Callback::new(|_: String| {})
                        }
                    }
                    disabled=is_readonly
                    required=field.is_required
                />
            }.into_view()
        },
        FieldType::Select { options } => {
            // Legacy Select field (fallback to basic select element)
            view! {
                <select
                    id=field_id
                    class="form-select"
                    disabled=is_readonly
                    required=field.is_required
                    on:change=move |ev| {
                        if let Some(cb) = on_change {
                            let new_val = event_target_value(&ev);
                            cb.call(JsonValue::String(new_val));
                        }
                    }
                >
                    <option value="" selected={val_str.is_empty()}>
                        "-- Select --"
                    </option>
                    {options.iter().map(|opt| {
                        let opt_val = opt.clone();
                        let is_selected = val_str == opt_val;
                        view! {
                            <option value=opt_val.clone() selected=is_selected>
                                {opt_val}
                            </option>
                        }
                    }).collect_view()}
                </select>
            }.into_view()
        },
        FieldType::MultiSelect { options } => {
            // Multi-select with checkboxes
            let selected_values = create_rw_signal(
                if let Some(arr) = value.as_array() {
                    arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>()
                } else {
                    vec![]
                }
            );
            
            view! {
                <div class="multi-select-field space-y-2 max-h-64 overflow-y-auto border border-gray-300 dark:border-gray-700 rounded p-3">
                    {options.iter().map(|choice_str| {
                        let choice_value = choice_str.clone();
                        let choice_label = choice_str.clone();
                        let choice_value_checked = choice_value.clone();
                        let choice_value_change = choice_value.clone();
                        
                        view! {
                            <label class="flex items-center gap-2 p-2 hover:bg-gray-50 dark:hover:bg-gray-800 rounded cursor-pointer">
                                <input
                                    type="checkbox"
                                    class="form-checkbox"
                                    checked=move || selected_values.get().contains(&choice_value_checked)
                                    disabled=is_readonly
                                    on:change=move |ev| {
                                        let checked = event_target_checked(&ev);
                                        selected_values.update(|vals| {
                                            if checked {
                                                if !vals.contains(&choice_value_change) {
                                                    vals.push(choice_value_change.clone());
                                                }
                                            } else {
                                                vals.retain(|v| v != &choice_value_change);
                                            }
                                        });
                                        if let Some(cb) = on_change {
                                            let json_array: Vec<JsonValue> = selected_values.get().into_iter()
                                                .map(JsonValue::String)
                                                .collect();
                                            cb.call(JsonValue::Array(json_array));
                                        }
                                    }
                                />
                                <span>{choice_label}</span>
                            </label>
                        }
                    }).collect_view()}
                </div>
            }.into_view()
        },
        FieldType::MultiLink { target_entity } => {
            // Multi-link field (multiple entity associations)
            let target = target_entity.clone();
            let selected_ids = create_rw_signal(
                if let Some(arr) = value.as_array() {
                    arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>()
                } else {
                    vec![]
                }
            );
            
            // Mock options - in real implementation, these would be loaded from API
            let mock_options = vec![
                SelectOption::new("1", format!("{} 1", target)),
                SelectOption::new("2", format!("{} 2", target)),
                SelectOption::new("3", format!("{} 3", target)),
            ];
            let options_signal = create_rw_signal(mock_options);
            
            view! {
                <div class="multi-link-field space-y-2">
                    <div class="flex flex-wrap gap-2 mb-2 min-h-[40px] p-2 border border-gray-300 dark:border-gray-700 rounded">
                        {{
                        let target_for_display = target.clone();
                        move || selected_ids.get().into_iter().map(|id| {
                            let id_clone = id.clone();
                            let target_clone = target_for_display.clone();
                            view! {
                                <span class="inline-flex items-center gap-1 px-3 py-1 bg-purple-100 dark:bg-purple-900 text-purple-800 dark:text-purple-200 rounded-full text-sm">
                                    {format!("{} {}", target_clone, id.clone())}
                                    {if !is_readonly {
                                        view! {
                                            <button
                                                type="button"
                                                class="hover:text-purple-600 ml-1 font-bold"
                                                on:click=move |_| {
                                                    selected_ids.update(|ids| ids.retain(|item| item != &id_clone));
                                                    if let Some(cb) = on_change {
                                                        let json_array: Vec<JsonValue> = selected_ids.get().into_iter()
                                                            .map(JsonValue::String)
                                                            .collect();
                                                        cb.call(JsonValue::Array(json_array));
                                                    }
                                                }
                                            >
                                                "\u{00d7}"
                                            </button>
                                        }.into_view()
                                    } else {
                                        view! {}.into_view()
                                    }}
                                </span>
                            }
                        }).collect_view()}
                        }
                    </div>
                    {if !is_readonly {
                        let target_for_select = target.clone();
                        view! {
                            <AsyncSelect
                                value=Signal::from(create_rw_signal(None::<String>))
                                on_change=Callback::new(move |new_val: Option<String>| {
                                    if let Some(id) = new_val {
                                        selected_ids.update(|ids| {
                                            if !ids.contains(&id) {
                                                ids.push(id);
                                            }
                                        });
                                        if let Some(cb) = on_change {
                                            let json_array: Vec<JsonValue> = selected_ids.get().into_iter()
                                                .map(JsonValue::String)
                                                .collect();
                                            cb.call(JsonValue::Array(json_array));
                                        }
                                    }
                                })
                                options=Signal::from(options_signal)
                                placeholder=format!("Select {}...", target_for_select)
                                allow_create=false
                            />
                            <small class="text-xs text-gray-500 mt-1 block">
                                "API integration for async entity lookup needed"
                            </small>
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }}
                </div>
            }.into_view()
        },
        FieldType::TagList => {
            // Tag list with inline creation
            let tags = create_rw_signal(
                if let Some(arr) = value.as_array() {
                    arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>()
                } else {
                    vec![]
                }
            );
            
            view! {
                <div class="tag-list-field">
                    <div class="flex flex-wrap gap-2 mb-2 min-h-[40px] p-2 border border-gray-300 dark:border-gray-700 rounded">
                        {move || tags.get().into_iter().map(|tag| {
                            let tag_clone = tag.clone();
                            view! {
                                <span class="inline-flex items-center gap-1 px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-sm">
                                    {tag.clone()}
                                    {if !is_readonly {
                                        view! {
                                            <button
                                                type="button"
                                                class="hover:text-blue-600 ml-1 font-bold"
                                                on:click=move |_| {
                                                    tags.update(|t| t.retain(|item| item != &tag_clone));
                                                    if let Some(cb) = on_change {
                                                        let json_array: Vec<JsonValue> = tags.get().into_iter()
                                                            .map(JsonValue::String)
                                                            .collect();
                                                        cb.call(JsonValue::Array(json_array));
                                                    }
                                                }
                                            >
                                                "\u{00d7}"
                                            </button>
                                        }.into_view()
                                    } else {
                                        view! {}.into_view()
                                    }}
                                </span>
                            }
                        }).collect_view()}
                    </div>
                    {if !is_readonly {
                        view! {
                            <input
                                type="text"
                                class="form-input"
                                placeholder="Type and press Enter to add tag..."
                                on:keydown=move |ev| {
                                    if ev.key() == "Enter" {
                                        ev.prevent_default();
                                        let target = ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
                                        let input_value = target.value();
                                        if !input_value.trim().is_empty() {
                                            tags.update(|t| t.push(input_value.trim().to_string()));
                                            if let Some(cb) = on_change {
                                                let json_array: Vec<JsonValue> = tags.get().into_iter()
                                                    .map(JsonValue::String)
                                                    .collect();
                                                cb.call(JsonValue::Array(json_array));
                                            }
                                            target.set_value("");
                                        }
                                    }
                                }
                            />
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }}
                </div>
            }.into_view()
        },
        FieldType::RichText => {
            // Enhanced Rich text editor
            view! {
                <div class="richtext-field">
                    <textarea
                        id=field_id
                        class="form-textarea min-h-[200px] font-mono"
                        placeholder={field.placeholder.clone().unwrap_or_else(|| "Enter rich text content...".to_string())}
                        disabled=is_readonly
                        on:input=move |ev| {
                            if let Some(cb) = on_change {
                                let new_val = event_target_value(&ev);
                                cb.call(JsonValue::String(new_val));
                            }
                        }
                    >
                        {val_str}
                    </textarea>
                    <div class="flex gap-2 mt-2 text-xs text-gray-500">
                        <span>"Supports: Markdown, HTML"</span>
                        <span>"|"</span>
                        <span>"Full WYSIWYG editor coming soon"</span>
                    </div>
                </div>
            }.into_view()
        },
        FieldType::Image => {
            // Image upload with preview
            let image_url = create_rw_signal(val_str.clone());
            
            view! {
                <div class="image-upload-field space-y-2">
                    {move || if !image_url.get().is_empty() {
                        view! {
                            <div class="relative inline-block">
                                <img src=image_url.get() alt="Uploaded image" class="max-w-xs max-h-64 rounded border shadow-sm"/>
                                {if !is_readonly {
                                    view! {
                                        <button
                                            type="button"
                                            class="absolute top-2 right-2 bg-red-500 text-white rounded-full w-6 h-6 flex items-center justify-center hover:bg-red-600"
                                            on:click=move |_| {
                                                image_url.set(String::new());
                                                if let Some(cb) = on_change {
                                                    cb.call(JsonValue::String(String::new()));
                                                }
                                            }
                                        >
                                            "\u{00d7}"
                                        </button>
                                    }.into_view()
                                } else {
                                    view! {}.into_view()
                                }}
                            </div>
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }}
                    {if !is_readonly {
                        view! {
                            <div>
                                <input
                                    type="file"
                                    id=field_id
                                    class="form-input"
                                    accept="image/*"
                                    on:change=move |ev| {
                                        // TODO: Upload to server and get URL
                                        // For now, create a local object URL for preview
                                        if let Some(input) = ev.target() {
                                            let input = input.unchecked_into::<web_sys::HtmlInputElement>();
                                            if let Some(files) = input.files() {
                                                if let Some(file) = files.get(0) {
                                                    // Create object URL for preview
                                                    let url = web_sys::Url::create_object_url_with_blob(&file).unwrap_or_default();
                                                    image_url.set(url.clone());
                                                   if let Some(cb) = on_change {
                                                        // TODO: Actually upload file and get permanent URL
                                                        cb.call(JsonValue::String(url));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                />
                                <small class="text-xs text-gray-500 mt-1 block">
                                    "Supported: JPG, PNG, GIF, WebP | Upload API integration needed"
                                </small>
                            </div>
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }}
                </div>
            }.into_view()
        },
        FieldType::Attachment => {
            // File attachment with multiple files
            let attachments = create_rw_signal(
                if let Some(arr) = value.as_array() {
                    arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>()
                } else if !val_str.is_empty() {
                    vec![val_str.clone()]
                } else {
                    vec![]
                }
            );
            
            view! {
                <div class="attachment-field space-y-2">
                    {move || if !attachments.get().is_empty() {
                        view! {
                            <div class="space-y-1 p-2 bg-gray-50 dark:bg-gray-800 rounded">
                                {attachments.get().into_iter().map(|file_url| {
                                    let file_name = file_url.split('/').last().unwrap_or(&file_url).to_string();
                                    view! {
                                        <div class="flex items-center justify-between gap-2 p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded">
                                            <div class="flex items-center gap-2 flex-1 min-w-0">
                                                <span class="text-xl">"\u{1f4ce}"</span>
                                                <a href=file_url.clone() class="text-blue-600 hover:underline truncate" target="_blank">
                                                    {file_name}
                                                </a>
                                            </div>
                                            {if !is_readonly {
                                                view! {
                                                    <button
                                                        type="button"
                                                        class="text-red-500 hover:text-red-700 text-sm"
                                                        on:click=move |_| {
                                                            let url_to_remove = file_url.clone();
                                                            attachments.update(|atts| atts.retain(|f| f != &url_to_remove));
                                                            if let Some(cb) = on_change {
                                                                let json_array: Vec<JsonValue> = attachments.get().into_iter()
                                                                    .map(JsonValue::String)
                                                                    .collect();
                                                                cb.call(JsonValue::Array(json_array));
                                                            }
                                                        }
                                                    >
                                                        "Remove"
                                                    </button>
                                                }.into_view()
                                            } else {
                                                view! {}.into_view()
                                            }}
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <div class="text-sm text-gray-500 italic">"No attachments"</div>
                        }.into_view()
                    }}
                    {if !is_readonly {
                        view! {
                            <div>
                                <input
                                    type="file"
                                    id=field_id
                                    class="form-input"
                                    multiple=true
                                    on:change=move |ev| {
                                        // TODO: Upload files to server
                                        if let Some(input) = ev.target() {
                                            let input = input.unchecked_into::<web_sys::HtmlInputElement>();
                                            if let Some(files) = input.files() {
                                                let mut new_files = Vec::new();
                                                for i in 0..files.length() {
                                                    if let Some(file) = files.get(i) {
                                                        // TODO: Upload and get permanent URL
                                                        let file_name = file.name();
                                                        new_files.push(format!("/uploads/{}", file_name));
                                                    }
                                                }
                                                attachments.update(|atts| atts.extend(new_files));
                                                if let Some(cb) = on_change {
                                                    let json_array: Vec<JsonValue> = attachments.get().into_iter()
                                                        .map(JsonValue::String)
                                                        .collect();
                                                    cb.call(JsonValue::Array(json_array));
                                                }
                                            }
                                        }
                                    }
                                />
                                <small class="text-xs text-gray-500 mt-1 block">
                                    "Multiple files supported | Upload API integration needed"
                                </small>
                            </div>
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }}
                </div>
            }.into_view()
        },
        FieldType::Progress { max_value } => {
            // Progress bar (0-100 or custom max)
            let current = value.as_i64().unwrap_or(0) as i32;
            let max = *max_value;
            let percentage = if max > 0 { (current * 100) / max } else { 0 };
            
            view! {
                <div class="progress-field">
                    <div class="flex justify-between text-sm mb-1">
                        <span>{current} " / " {max}</span>
                        <span class="font-semibold">{percentage}"%"</span>
                    </div>
                    <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3 overflow-hidden">
                        <div 
                            class="h-full rounded-full transition-all"
                            class:bg-blue-600={percentage <= 100}
                            class:bg-green-600={percentage > 100}
                            style:width={format!("{}%", percentage.min(100))}
                        ></div>
                    </div>
                    {if !is_readonly {
                        view! {
                            <input
                                type="number"
                                class="form-input mt-2"
                                value=current
                                min="0"
                                max=max
                                on:input=move |ev| {
                                    if let Some(cb) = on_change {
                                        let new_val = event_target_value(&ev);
                                        if let Ok(num) = new_val.parse::<i64>() {
                                            cb.call(JsonValue::Number(serde_json::Number::from(num)));
                                        }
                                    }
                                }
                            />
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }}
                </div>
            }.into_view()
        },
        FieldType::Rating { max_stars } => {
            // Star rating system
            let current_rating = value.as_i64().unwrap_or(0) as u8;
            let max = *max_stars;
            
            view! {
                <div class="rating-field flex gap-1">
                    {(1..=max).map(|star| {
                        let is_filled = star <= current_rating;
                        view! {
                            <button
                                type="button"
                                class="text-2xl transition-colors"
                                class:text-yellow-400=is_filled
                                class:text-gray-300=!is_filled
                                class:hover:text-yellow-300=!is_readonly
                                disabled=is_readonly
                                on:click=move |_| {
                                    if let Some(cb) = on_change {
                                        cb.call(JsonValue::Number(serde_json::Number::from(star)));
                                    }
                                }
                            >
                                {if is_filled { "\u{2605}" } else { "\u{2606}" }}
                            </button>
                        }
                    }).collect_view()}
                    <span class="ml-2 text-sm text-gray-600 dark:text-gray-400">
                        {current_rating} " / " {max}
                    </span>
                </div>
            }.into_view()
        },
        FieldType::Location { show_map: _ } => {
            // Location/Address field
            view! {
                <div class="location-field space-y-2">
                    <input
                        type="text"
                        class="form-input"
                        value=val_str
                        placeholder="Enter address..."
                        disabled=is_readonly
                        on:input=move |ev| {
                            if let Some(cb) = on_change {
                                let new_val = event_target_value(&ev);
                                cb.call(JsonValue::String(new_val));
                            }
                        }
                    />
                    <small class="text-xs text-gray-500">
                        "Map integration and geocoding coming soon"
                    </small>
                </div>
            }.into_view()
        },
        FieldType::JsonLogic { .. } => {
            // JsonLogic formula field (visual rule builder)
            view! {
                <div class="jsonlogic-field p-4 border border-dashed border-yellow-300 dark:border-yellow-700 rounded bg-yellow-50 dark:bg-yellow-900/20">
                    <p class="text-sm text-yellow-800 dark:text-yellow-200 mb-2 font-semibold">
                        "JsonLogic Rule Builder"
                    </p>
                    <textarea
                        class="form-textarea font-mono text-xs"
                        placeholder="{\"==\": [{\"var\": \"status\"}, \"active\"]}"
                        disabled=is_readonly
                    >
                        {val_str}
                    </textarea>
                    <small class="text-xs text-gray-600 dark:text-gray-400 mt-1 block">
                        "Visual rule builder coming soon"
                    </small>
                </div>
            }.into_view()
        },
        FieldType::Signature => {
            // Digital signature field
            view! {
                <div class="signature-field p-4 border border-dashed border-purple-300 dark:border-purple-700 rounded bg-purple-50 dark:bg-purple-900/20">
                    {if !val_str.is_empty() {
                        view! {
                            <div class="mb-2">
                                <img src=val_str.clone() alt="Signature" class="max-w-xs border rounded"/>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <div class="text-center py-8 text-gray-400">
                                "No signature captured"
                            </div>
                        }.into_view()
                    }}
                    <button
                        type="button"
                        class="btn btn-secondary text-sm"
                        disabled=is_readonly
                    >
                        "Capture Signature"
                    </button>
                    <small class="text-xs text-gray-600 dark:text-gray-400 mt-2 block">
                        "Canvas signature capture coming soon"
                    </small>
                </div>
            }.into_view()
        },
        FieldType::Association { target_entity, display_field, allow_inline_create } => {
            // Association field with AsyncSelect for entity lookup
            let target = target_entity.clone();
            let display = display_field.clone();
            let allow_create = *allow_inline_create;
            
            let selected_id = create_rw_signal(value.as_str().map(|s| s.to_string()));
            
            // Mock entity options - in real implementation, these would be loaded from API
            let mock_options = vec![
                SelectOption {
                    value: "1".to_string(),
                    label: format!("{} Record 1", target),
                    description: Some(format!("Sample {} #1", target)),
                    color: None,
                    icon: None,
                },
                SelectOption {
                    value: "2".to_string(),
                    label: format!("{} Record 2", target),
                    description: Some(format!("Sample {} #2", target)),
                    color: None,
                    icon: None,
                },
                SelectOption {
                    value: "3".to_string(),
                    label: format!("{} Record 3", target),
                    description: Some(format!("Sample {} #3", target)),
                    color: None,
                    icon: None,
                },
            ];
            let options_signal = create_rw_signal(mock_options);
            
            view! {
                <div class="association-field space-y-2">
                    <AsyncSelect
                        value=Signal::from(selected_id)
                        on_change=Callback::new(move |new_val: Option<String>| {
                            selected_id.set(new_val.clone());
                            if let Some(cb) = on_change {
                                cb.call(new_val.map(JsonValue::String).unwrap_or(JsonValue::Null));
                            }
                        })
                        options=Signal::from(options_signal)
                        placeholder=format!("Select {}...", target)
                        allow_create=allow_create
                        on_create={
                            if allow_create {
                                let target_for_create = target.clone();
                                Callback::new(move |new_label: String| {
                                    // Simulate creating new entity
                                    let new_id = format!("new_{}", new_label.to_lowercase().replace(' ', "_"));
                                    options_signal.update(|opts| {
                                        opts.push(SelectOption {
                                            value: new_id.clone(),
                                            label: new_label.clone(),
                                            description: Some(format!("Newly created {}", target_for_create)),
                                            color: Some("#10B981".to_string()),
                                            icon: None,
                                        });
                                    });
                                    selected_id.set(Some(new_id.clone()));
                                    if let Some(cb) = on_change {
                                        cb.call(JsonValue::String(new_id));
                                    }
                                })
                            } else {
                                Callback::new(|_: String| {})
                            }
                        }
                        disabled=is_readonly
                        required=field.is_required
                    />
                    <small class="text-xs text-gray-500">
                        "API endpoint: GET /api/v1/entities/" {target} " | Field: " {display}
                    </small>
                </div>
            }.into_view()
        },
        // Fallback for other field types
        _ => {
            view! {
                <input
                    type="text"
                    id=field_id
                    class="form-input"
                    value=val_str
                    placeholder="Field type not yet implemented"
                    disabled=true
                />
            }.into_view()
        }
    }
}

// ==================
// Filter Builder Rendering
// ==================

fn render_filter_input(
    _field: &FieldDef,
    _value: JsonValue,
    _on_change: Option<Callback<JsonValue>>,
) -> View {
    // TODO: Implement filter builder inputs
    view! {
        <div class="filter-input">
            <span>"Filter builder coming soon"</span>
        </div>
    }.into_view()
}

// ==================
// Utility Functions
// ==================

fn format_display_value(field: &FieldDef, value: &JsonValue) -> String {
    match &field.field_type {
        FieldType::Money { currency_code } => {
            let amount = value.as_f64().unwrap_or(0.0);
            let currency = currency_code.as_deref().unwrap_or("USD");
            format!("{} {:.2}", currency, amount)
        },
        FieldType::Date | FieldType::DateTime => {
            value.as_str().unwrap_or("").to_string()
        },
        FieldType::Boolean => {
            if value.as_bool().unwrap_or(false) { "Yes" } else { "No" }.to_string()
        },
        _ => {
            if let Some(s) = value.as_str() {
                s.to_string()
            } else if let Some(i) = value.as_i64() {
                i.to_string()
            } else if let Some(f) = value.as_f64() {
                f.to_string()
            } else {
                "".to_string()
            }
        }
    }
}

// ==================
// Field Validation Module
// ==================

use regex::Regex;
use std::sync::OnceLock;

/// Validation result with optional error message
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub error_message: Option<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self { is_valid: true, error_message: None }
    }
    
    pub fn invalid(message: impl Into<String>) -> Self {
        Self { is_valid: false, error_message: Some(message.into()) }
    }
}

// Compiled regex patterns (cached for performance)
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
static PHONE_REGEX: OnceLock<Regex> = OnceLock::new();
static URL_REGEX: OnceLock<Regex> = OnceLock::new();

fn email_regex() -> &'static Regex {
    EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
    })
}

fn phone_regex() -> &'static Regex {
    PHONE_REGEX.get_or_init(|| {
        // Matches: +1234567890, (123) 456-7890, 123-456-7890, 123.456.7890, 1234567890
        Regex::new(r"^[\+]?[(]?[0-9]{1,4}[)]?[-\s\.]?[0-9]{1,4}[-\s\.]?[0-9]{1,9}$").unwrap()
    })
}

fn url_regex() -> &'static Regex {
    URL_REGEX.get_or_init(|| {
        Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap()
    })
}

/// Validate a field value based on its type and requirements
pub fn validate_field(field: &FieldDef, value: &JsonValue) -> ValidationResult {
    // Required check
    if field.is_required && is_empty_value(value) {
        return ValidationResult::invalid(format!("{} is required", field.label));
    }
    
    // Skip type validation if empty and not required
    if is_empty_value(value) {
        return ValidationResult::valid();
    }
    
    // Type-specific validation
    match &field.field_type {
        FieldType::Email => {
            let email = value.as_str().unwrap_or("");
            if !email_regex().is_match(email) {
                return ValidationResult::invalid("Please enter a valid email address");
            }
        }
        FieldType::Phone => {
            let phone = value.as_str().unwrap_or("");
            // Remove common formatting
            let cleaned: String = phone.chars().filter(|c| c.is_numeric() || *c == '+').collect();
            if cleaned.len() < 7 || cleaned.len() > 15 {
                return ValidationResult::invalid("Phone number should be 7-15 digits");
            }
            if !phone_regex().is_match(phone) {
                return ValidationResult::invalid("Please enter a valid phone number");
            }
        }
        FieldType::Url => {
            let url = value.as_str().unwrap_or("");
            if !url_regex().is_match(url) {
                return ValidationResult::invalid("Please enter a valid URL (starting with http:// or https://)");
            }
        }
        FieldType::Number { decimals } => {
            if let Some(num_str) = value.as_str() {
                if num_str.parse::<f64>().is_err() {
                    return ValidationResult::invalid("Please enter a valid number");
                }
            } else if value.as_f64().is_none() && value.as_i64().is_none() {
                return ValidationResult::invalid("Please enter a valid number");
            }
        }
        FieldType::Money { .. } => {
            if let Some(num_str) = value.as_str() {
                if num_str.parse::<f64>().is_err() {
                    return ValidationResult::invalid("Please enter a valid amount");
                }
            } else if value.as_f64().is_none() && value.as_i64().is_none() {
                return ValidationResult::invalid("Please enter a valid amount");
            }
        }
        FieldType::Date => {
            let date_str = value.as_str().unwrap_or("");
            // Basic date validation (YYYY-MM-DD)
            if !date_str.is_empty() {
                let parts: Vec<&str> = date_str.split('-').collect();
                if parts.len() != 3 {
                    return ValidationResult::invalid("Date should be in YYYY-MM-DD format");
                }
            }
        }
        FieldType::DateTime => {
            let datetime_str = value.as_str().unwrap_or("");
            if !datetime_str.is_empty() && datetime_str.len() < 10 {
                return ValidationResult::invalid("Invalid date/time format");
            }
        }
        _ => {}
    }
    
    ValidationResult::valid()
}

/// Check if a JSON value is considered empty
fn is_empty_value(value: &JsonValue) -> bool {
    match value {
        JsonValue::Null => true,
        JsonValue::String(s) => s.trim().is_empty(),
        JsonValue::Array(arr) => arr.is_empty(),
        JsonValue::Object(obj) => obj.is_empty(),
        _ => false,
    }
}

/// Validation state component - displays inline error messages
#[component]
pub fn ValidationMessage(
    #[prop(into)] result: Signal<ValidationResult>,
) -> impl IntoView {
    view! {
        <Show when=move || !result.get().is_valid>
            <div class="validation-error flex items-center gap-1 mt-1 text-sm text-danger-500 animate-fade-in">
                <i class="fa-solid fa-exclamation-circle"></i>
                <span>{move || result.get().error_message.unwrap_or_default()}</span>
            </div>
        </Show>
    }
}

/// Validated SmartField - SmartField with integrated validation
#[component]
pub fn ValidatedSmartField(
    field: FieldDef,
    #[prop(into)] value: Signal<JsonValue>,
    context: FieldContext,
    #[prop(optional)] on_change: Option<Callback<JsonValue>>,
    #[prop(optional)] on_validate: Option<Callback<ValidationResult>>,
    #[prop(optional)] disabled: bool,
    /// Whether to show validation on blur (true) or on every change (false)
    #[prop(default = true)] validate_on_blur: bool,
) -> impl IntoView {
    let field_clone = field.clone();
    let field_for_blur = field.clone();
    let validation_result = create_rw_signal(ValidationResult::valid());
    let (touched, set_touched) = create_signal(false);
    
    // Wrapped on_change that includes validation
    let wrapped_on_change = Callback::new(move |new_val: JsonValue| {
        if let Some(cb) = on_change {
            cb.call(new_val.clone());
        }
        if !validate_on_blur {
            let result = validate_field(&field_clone, &new_val);
            validation_result.set(result.clone());
            if let Some(cb) = on_validate {
                cb.call(result);
            }
        }
    });
    
    view! {
        <div 
            class="validated-field"
            class:has-error=move || !validation_result.get().is_valid && touched.get()
            on:blur=move |_| {
                set_touched.set(true);
                let result = validate_field(&field_for_blur, &value.get());
                validation_result.set(result.clone());
                if let Some(cb) = on_validate {
                    cb.call(result);
                }
            }
        >
            <SmartField
                field=field.clone()
                value=value
                context=context
                on_change=wrapped_on_change
                disabled=disabled
            />
            <Show when=move || touched.get()>
                <ValidationMessage result=Signal::from(validation_result) />
            </Show>
        </div>
    }
}

/// Form-level validation helper
/// Takes field definitions and a map of field_name -> value
pub fn validate_form(fields: &[FieldDef], values: &std::collections::HashMap<String, JsonValue>) -> Vec<(String, ValidationResult)> {
    fields.iter()
        .filter_map(|field| {
            // Use field.name for lookup since it's a String
            let value = values.get(&field.name).cloned().unwrap_or(JsonValue::Null);
            let result = validate_field(field, &value);
            if !result.is_valid {
                Some((field.name.clone(), result))
            } else {
                None
            }
        })
        .collect()
}


