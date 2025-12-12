//! Field rendering component
//!
//! Renders fields based on FieldDef metadata in various contexts:
//! - List cell
//! - Form input
//! - Detail view
//! - Card cell

use leptos::*;
use crate::models::{FieldDef, FieldType};

/// Context in which to render the field
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldContext {
    ListCell,
    FormInput,
    DetailView,
    CardCell,
}

/// Render a field value based on its definition
#[component]
pub fn FieldRenderer(
    field: FieldDef,
    value: serde_json::Value,
    context: FieldContext,
    #[prop(optional)] on_change: Option<Callback<serde_json::Value>>,
) -> impl IntoView {
    let is_display = matches!(
        context,
        FieldContext::ListCell | FieldContext::DetailView | FieldContext::CardCell
    );

    let display_value = get_display_value(&field, &value);
    let input_type = get_input_type(&field);
    let initial_value = value.as_str().unwrap_or("").to_string();
    let required = field.is_required;
    let placeholder = field.placeholder.clone().unwrap_or_default();
    let label = field.label.clone();
    let help_text = field.help_text.clone();

    view! {
        {if is_display {
            view! { <span class="field-value">{display_value}</span> }.into_view()
        } else {
            view! {
                <div class="form-field">
                    <label class="form-label">{label}</label>
                    <input
                        type=input_type
                        class="form-input"
                        value=initial_value
                        placeholder=placeholder
                        required=required
                    />
                    {help_text.as_ref().map(|help| view! {
                        <span class="form-help">{help.clone()}</span>
                    })}
                </div>
            }.into_view()
        }}
    }
}

fn get_display_value(field: &FieldDef, value: &serde_json::Value) -> String {
    match &field.field_type {
        FieldType::Text | FieldType::TextArea => {
            value.as_str().unwrap_or("").to_string()
        }
        FieldType::Email => {
            value.as_str().unwrap_or("").to_string()
        }
        FieldType::Phone => {
            value.as_str().unwrap_or("").to_string()
        }
        FieldType::Integer => {
            value.as_i64().map(|n| n.to_string()).unwrap_or_default()
        }
        FieldType::Decimal => {
            value.as_f64().map(|n| format!("{:.2}", n)).unwrap_or_default()
        }
        FieldType::Money => {
            // Convert cents to dollars
            value.as_i64()
                .map(|cents| format!("${:.2}", cents as f64 / 100.0))
                .unwrap_or_default()
        }
        FieldType::Boolean => {
            if value.as_bool().unwrap_or(false) { "Yes" } else { "No" }.to_string()
        }
        FieldType::Date => {
            value.as_str().unwrap_or("").to_string()
        }
        FieldType::Select => {
            // TODO: Look up label from options
            value.as_str().unwrap_or("").to_string()
        }
        _ => value.to_string(),
    }
}

fn get_input_type(field: &FieldDef) -> &'static str {
    match &field.field_type {
        FieldType::Email => "email",
        FieldType::Phone => "tel",
        FieldType::Integer | FieldType::Decimal | FieldType::Money => "number",
        FieldType::Date => "date",
        FieldType::DateTime => "datetime-local",
        _ => "text",
    }
}
