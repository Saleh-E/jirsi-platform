# SmartField Developer Guide

Architecture, customization, and best practices for extending the SmartField system.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Adding New Field Types](#adding-new-field-types)
3. [Customizing Rendering](#customizing-rendering)
4. [Context-Aware Rendering](#context-aware-rendering)
5. [Best Practices](#best-practices)
6. [Performance Optimization](#performance-optimization)
7. [Troubleshooting](#troubleshooting)

---

## Architecture Overview

### System Design

SmartField follows a **metadata-driven, context-aware** architecture:

```
┌─────────────────┐
│   FieldDef      │ ← Metadata definition
│  (Database)     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  SmartField     │ ← Polymorphic renderer
│   Component     │
└────────┬────────┘
         │
    ┌────┴────┐
    │ Context │ (CreateForm, ListView, etc.)
    └────┬────┘
         │
    ┌────▼────────────┐
    │ Field Type      │
    │ (Text, Dropdown,│
    │  Association)   │
    └────┬────────────┘
         │
    ┌────▼────┐
    │ Render  │ ← Appropriate UI component
    └─────────┘
```

### Key Components

| Component | Responsibility |
|-----------|---------------|
| `FieldDef` | Metadata definition (type, label, validation) |
| `SmartField` | Main component, routes to correct renderer |
| `render_field_by_context()` | Context-based routing |
| `render_form_input()` | Edit mode rendering |
| `render_list_cell()` | ListView rendering |
| `AsyncSelect` | Virtual scrolling dropdown |
| `AssociationModal` | Inline entity creation |
| `SignaturePad` | Canvas signature capture |

### Data Flow

```
User Interaction
      ↓
SmartField Component
      ↓
on_change Callback
      ↓
Parent Component (updates signal)
      ↓
Re-render with new value
```

---

## Adding New Field Types

### Step 1: Define the Field Type

Edit `crates/core-models/src/field.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FieldType {
    // ... existing types
    
    // New field type
    MyCustomField {
        // Configuration options
        option1: String,
        option2: bool,
    },
}
```

### Step 2: Add Rendering Logic

Edit `crates/frontend-web/src/components/smart_field.rs`:

Find the `render_input_control()` function and add your case:

```rust
fn render_input_control(
    field: &FieldDef,
    value: JsonValue,
    field_id: String,
    is_readonly: bool,
    on_change: Option<Callback<JsonValue>>,
) -> View {
    match &field.field_type {
        // ... existing types
        
        FieldType::MyCustomField { option1, option2 } => {
            let current_value = value.as_str().unwrap_or("").to_string();
            
            view! {
                <div class="my-custom-field">
                    <input
                        type="text"
                        id=field_id
                        class="form-input"
                        value=current_value.clone()
                        disabled=is_readonly
                        on:input=move |ev| {
                            if let Some(cb) = on_change {
                                cb.call(json!(event_target_value(&ev)));
                            }
                        }
                    />
                    // Your custom UI here
                </div>
            }.into_view()
        },
    }
}
```

### Step 3: Add List View Rendering

In the same file, update `render_list_cell()`:

```rust
fn render_list_cell(field: &FieldDef, value: &JsonValue) -> View {
    match &field.field_type {
        // ... existing types
        
        FieldType::MyCustomField { .. } => {
            let display = value.as_str().unwrap_or("").to_string();
            view! {
                <span class="my-custom-field-display">{display}</span>
            }.into_view()
        },
    }
}
```

### Step 4: Add Database Migration

Create a migration to support the new field type in PostgreSQL.

### Step 5: Update Tests

Add tests for your new field type:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_custom_field() {
        let field = FieldDef {
            field_type: FieldType::MyCustomField {
                option1: "value".to_string(),
                option2: true,
            },
            // ... other fields
        };
        
        // Test rendering logic
        assert!(/* your assertions */);
    }
}
```

---

## Customizing Rendering

### Per-Field Customization with UI Hints

Use `ui_hints` to customize individual fields:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiHints {
    pub hide_label: bool,
    pub inline: bool,
    pub width: Option<String>,
    pub css_class: Option<String>,
    pub icon: Option<String>,
}

let field = FieldDef {
    // ... base fields
    ui_hints: Some(UiHints {
        hide_label: false,
        inline: true,
        width: Some("50%".to_string()),
        css_class: Some("custom-field-class".to_string()),
        icon: Some("user-icon".to_string()),
    }),
};
```

### Per-Context Customization

Use `context_hints` for context-specific overrides:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextHints {
    pub list_view: Option<ListViewHints>,
    pub edit_form: Option<FormHints>,
    // ... other contexts
}

let field = FieldDef {
    // ... base fields
    context_hints: Some(ContextHints {
        list_view: Some(ListViewHints {
            format: Some("badge".to_string()),
            truncate_at: Some(50),
        }),
        edit_form: Some(FormHints {
            rows: Some(5),
            autocomplete: Some("email".to_string()),
        }),
    }),
};
```

---

## Context-Aware Rendering

### Understanding Contexts

| Context | When Used | Rendering Goal |
|---------|-----------|----------------|
| `CreateForm` | New entity creation | Full input controls, empty defaults |
| `EditForm` | Editing existing entity | Full input controls, current values |
| `InlineEdit` | Quick table edit | Simplified inline input |
| `ListView` | Table/list display | Compact, read-only, scannable |
| `DetailView` | Entity detail page | Rich formatted display |
| `KanbanCard` | Kanban boards | Compact label + value |
| `FilterBuilder` | Filter UI | Comparison operators |

### Custom Context Rendering

Override rendering for specific contexts:

```rust
fn render_field_by_context(
    field: &FieldDef,
    value: JsonValue,
    context: FieldContext,
    field_id: String,
    is_readonly: bool,
    on_change: Option<Callback<JsonValue>>,
) -> View {
    // Add custom logic for specific context + type combinations
    match (&field.field_type, context) {
        (FieldType::MyCustomField { .. }, FieldContext::ListView) => {
            // Custom list view rendering
            view! {
                <span class="custom-list-view">
                    // Your custom list display
                </span>
            }.into_view()
        },
        
        (FieldType::MyCustomField { .. }, _) => {
            // Default rendering for other contexts
            render_form_input(field, value, field_id, is_readonly, on_change)
        },
        
        _ => {
            // Default behavior
            // ...
        }
    }
}
```

---

## Best Practices

### 1. Always Use Signals for Values

✅ **Correct**:
```rust
let (email, set_email) = create_signal(json!(""));

view! {
    <SmartField
        field=field
        value=Signal::from(email)
        on_change=Callback::new(move |val| set_email.set(val))
    />
}
```

❌ **Incorrect**:
```rust
let email = "static_value".to_string();
// SmartField won't be reactive!
```

### 2. Use Dropdown Over Select

For new code, always use `FieldType::Dropdown` instead of `FieldType::Select`:

✅ **Preferred**:
```rust
FieldType::Dropdown {
    options: vec![...],
    allow_create: true,
}
```

❌ **Legacy** (only for compatibility):
```rust
FieldType::Select {
    options: vec![...],
}
```

**Why?** Dropdown includes:
- Virtual scrolling
- Search
- Inline creation
- Better UX

### 3. Use Association Over Plain Text for IDs

✅ **Correct**:
```rust
FieldType::Association {
    target_entity: "customers".to_string(),
    display_field: "name".to_string(),
    allow_create: true,
}
```

❌ **Antipattern**:
```rust
// Storing customer ID as plain text
FieldType::Text
```

### 4. Leverage Context-Aware Rendering

Let SmartField handle rendering instead of manual logic:

✅ **Correct**:
```rust
// Same component works everywhere
<SmartField field=field value=value context=current_context />
```

❌ **Verbose**:
```rust
{if is_edit_mode {
    view! { <input type="text" /> }
} else {
    view! { <span>{value}</span> }
}}
```

### 5. Validate Early

Use `is_required` and `validation` in FieldDef instead of custom logic:

✅ **Correct**:
```rust
FieldDef {
    is_required: true,
    validation: Some(json!({
        "min_length": 5,
        "max_length": 100,
        "pattern": "^[a-zA-Z0-9]+$"
    })),
    // ...
}
```

###6. Keep Field Names Consistent

Use `snake_case` for field names, matching database columns:

✅ **Correct**:
```rust
name: "customer_email".to_string(),
```

❌ **Incorrect**:
```rust
name: "CustomerEmail".to_string(), // PascalCase
name: "customer-email".to_string(), // kebab-case
```

---

## Performance Optimization

### 1. Virtual Scrolling Threshold

AsyncSelect automatically enables virtual scrolling at 50+ items:

```rust
const VIRTUAL_SCROLL_THRESHOLD: usize = 50;
```

Adjust if needed for your use case.

### 2. Lazy Load Heavy Components

Use `Suspense` for heavy field types:

```rust
<Suspense fallback=move || view! { <div>"Loading..."</div> }>
    <SignaturePad ... />
</Suspense>
```

### 3. Memoize Expensive Computations

```rust
let formatted_value = create_memo(move |_| {
    // Expensive formatting
    format_currency(value.get())
});

view! {
    <span>{move || formatted_value.get()}</span>
}
```

### 4. Debounce API Calls

AsyncSelect already debounces search (300ms). For custom fields:

```rust
use leptos_use::use_debounce_fn;

let debounced_search = use_debounce_fn(
    move || {
        // API call
    },
    300.0, // milliseconds
);
```

---

## Troubleshooting

### Issue: Field Not Reactive

**Problem**: Value updates don't reflect in UI.

**Solution**: Ensure you're using `Signal` and `on_change` callback:

```rust
let (value, set_value) = create_signal(json!(""));

<SmartField
    value=Signal::from(value) // Must be Signal
    on_change=Callback::new(move |v| set_value.set(v)) // Update parent
/>
```

### Issue: Context Not Applying

**Problem**: Field renders incorrectly for context.

**Solution**: Check `render_field_by_context` implementation for your field type.

### Issue: Virtual Scrolling Not Working

**Problem**: Dropdown doesn't scroll efficiently with 1000+ items.

**Solution**: Ensure you're using `FieldType::Dropdown`, not `FieldType::Select`:

```rust
// This triggers virtual scrolling:
FieldType::Dropdown { options, allow_create: true }

// This doesn't:
FieldType::Select { options }
```

### Issue: Association Modal Not Showing

**Problem**: Inline creation doesn't work.

**Solution**: Set `allow_create: true`:

```rust
FieldType::Association {
    target_entity: "customers".to_string(),
    display_field: "name".to_string(),
    allow_create: true, // Required for modal
}
```

### Issue: Compilation Errors with Closures

**Problem**: "cannot move out of captured variable" errors.

**Solution**: Clone values before move:

```rust
let field_id = field.id.clone();
let on_change_clone = on_change.clone();

move |ev| {
    // Use cloned values
    on_change_clone.call(value);
}
```

---

## Testing

### Unit Testing Fields

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use leptos::*;
    
    #[test]
    fn test_text_field_rendering() {
        let field = FieldDef {
            field_type: FieldType::Text,
            label: "Test".to_string(),
            // ... other fields
        };
        
        let (value, _) = create_signal(json!("test value"));
        
        let rendered = view! {
            <SmartField
                field=field
                value=Signal::from(value)
                context=FieldContext::EditForm
            />
        };
        
        // Assertions
        assert!(/* check rendered output */);
    }
}
```

### Integration Testing

Test the full form flow:

```rust
#[cfg(test)]
mod integration_tests {
    #[test]
    fn test_form_submission() {
        // 1. Render form with SmartFields
        // 2. Simulate user input
        // 3. Verify on_change callbacks
        // 4. Check final values
    }
}
```

---

## Code Organization

### Recommended Structure

```
crates/
├── core-models/
│   └── src/
│       └── field.rs          ← FieldDef, FieldType enums
├── frontend-web/
│   └── src/
│       ├── components/
│       │   ├── smart_field.rs    ← Main SmartField component
│       │   ├── async_select.rs   ← Virtual scrolling dropdown
│       │   ├── association_modal.rs
│       │   ├── entity_hover_card.rs
│       │   ├── signature_pad.rs
│       │   ├── location_map.rs
│       │   └── rule_builder.rs
│       └── pages/
│           └── component_playground.rs  ← Live demo
└── docs/
    ├── smartfield_api.md
    ├── component_catalog.md
    └── developer_guide.md       ← This file
```

---

## Advanced Topics

### Custom Field Validators

```rust
pub fn validate_field(field: &FieldDef, value: &JsonValue) -> Result<(), String> {
    if field.is_required && value.is_null() {
        return Err(format!("{} is required", field.label));
    }
    
    if let Some(validation) = &field.validation {
        // Apply custom validation rules
        if let Some(min_length) = validation.get("min_length") {
            // Validate min_length
        }
    }
    
    Ok(())
}
```

### Dynamic Field Configuration

Load field configs from API:

```rust
#[component]
pub fn DynamicForm(entity_type: String) -> impl IntoView {
    let (fields, set_fields) = create_signal(Vec::new());
    
    create_effect(move |_| {
        spawn_local(async move {
            match fetch_entity_metadata(&entity_type).await {
                Ok(metadata) => set_fields.set(metadata.fields),
                Err(e) => log::error!("Failed to load metadata: {}", e),
            }
        });
    });
    
    view! {
        <For
            each=move || fields.get()
            key=|f| f.id
            children=move |field| {
                view! {
                    <SmartField
                        field=field
                        value=Signal::from(create_signal(json!(null)).0)
                        context=FieldContext::CreateForm
                    />
                }
            }
        />
    }
}
```

---

## Migration Guide

### From Manual Forms to SmartField

**Before**:
```rust
view! {
    <div>
        <label>"Email"</label>
        <input type="email" value=email />
    </div>
    <div>
        <label>"Phone"</label>
        <input type="tel" value=phone />
    </div>
    // ... 20 more fields
}
```

**After**:
```rust
view! {
    <For
        each=move || fields.get()
        key=|f| f.id
        children=move |field| {
            view! {
                <SmartField
                    field=field
                    value=get_field_value(&field.name)
                    context=FieldContext::EditForm
                    on_change=create_field_updater(&field.name)
                />
            }
        }
    />
}
```

**Benefits**:
- ✅ 90% less code
- ✅ Consistent styling
- ✅ Automatic validation
- ✅ Context-aware rendering
- ✅ Metadata-driven (no hardcoding)

---

## Resources

- [SmartField API Reference](./smartfield_api.md)
- [Component Catalog](./component_catalog.md)
- [Quick Start Guide](../README_SMARTFIELD.md)
- [Component Playground](/app/playground)
- [Leptos Documentation](https://leptos-rs.github.io/leptos/)

---

## Contributing

### Adding Documentation

When adding features, update:
1. This Developer Guide
2. API Reference
3. Component Catalog
4. Quick Start (if user-facing)

### Coding Standards

- Use `clippy` for linting
- Run `cargo fmt` before committing
- Write tests for new field types
- Document all public APIs

---

## Getting Help

- **GitHub Issues**: Report bugs
- **Discord**: Join community discussions
- **Documentation**: Check existing guides first

---

**Version**: 1.0.0  
**Last Updated**: 2024-12-24  
**Maintainer**: Jirsi Platform Team
