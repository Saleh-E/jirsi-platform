# SmartField Quick Start Guide

## What is SmartField?

**SmartField** is a metadata-driven, context-aware field rendering system for Leptos/Rust web applications. It automatically renders the right UI component based on field type and context, eliminating the need to manually write form inputs.

### Key Features

✨ **24 Field Types** - Text, Number, Date, Dropdown, Association, Signature, Location, and more  
🎯 **Context-Aware** - Automatically adapts rendering for ListView, EditForm, DetailView, etc.  
⚡ **High Performance** - Virtual scrolling for 10,000+ items at 60fps  
♿ **Accessible** - WCAG AA compliant with full keyboard navigation  
🎨 **Themeable** - Built-in dark mode support

---

## Installation

SmartField is part of the Jirsi platform. It's located in:
```
crates/frontend-web/src/components/smart_field.rs
```

No additional dependencies required beyond the standard Jirsi stack.

---

## 5-Minute Quick Start

### 1. Define Your Field

```rust
use core_models::field::{FieldDef, FieldType, FieldContext};
use uuid::Uuid;

let field = FieldDef {
    id: Uuid::new_v4(),
    tenant_id: your_tenant_id,
    entity_type_id: your_entity_type_id,
    name: "email".to_string(),
    label: "Email Address".to_string(),
    field_type: FieldType::Email,
    is_required: true,
    placeholder: Some("Enter your email".to_string()),
    // ... other fields with defaults
};
```

### 2. Render the SmartField

```rust
use leptos::*;
use crate::components::smart_field::SmartField;

#[component]
pub fn MyForm() -> impl IntoView {
    let (email, set_email) = create_signal(json!(""));
    
    view! {
        <SmartField
            field=field
            value=Signal::from(email)
            context=FieldContext::CreateForm
            on_change=Callback::new(move |new_value| {
                set_email.set(new_value);
            })
        />
    }
}
```

### 3. That's It!

SmartField automatically renders:
- ✅ Proper input type (`<input type="email">`)
- ✅ Label and placeholder
- ✅ Validation indicators
- ✅ Help text
- ✅ Dark mode styling

---

## Common Recipes

### Recipe 1: Dropdown with Inline Creation

```rust
let status_field = FieldDef {
    // ... base fields
    field_type: FieldType::Dropdown {
        options: vec![
            SelectChoice {
                value: "active".to_string(),
                label: "Active".to_string(),
                color: Some("#10B981".to_string()),
                icon: None,
                is_default: false,
                sort_order: 0,
            },
            // ... more options
        ],
        allow_create: true, // Enables inline creation
    },
};

view! {
    <SmartField
        field=status_field
        value=Signal::from(status_signal)
        context=FieldContext::EditForm
        on_change=on_status_change
    />
}
```

**Result**: Dropdown with AsyncSelect, search, and "+ Create New" option.

---

### Recipe 2: Association (Foreign Key)

```rust
let customer_field = FieldDef {
    // ... base fields
    field_type: FieldType::Association {
        target_entity: "customers".to_string(),
        display_field: "name".to_string(),
        allow_create: true,
    },
};

view! {
    <SmartField
        field=customer_field
        value=Signal::from(customer_id_signal)
        context=FieldContext::EditForm
        on_change=on_customer_change
    />
}
```

**Result**: 
- AsyncSelect with entity lookup
- Inline creation modal
- Hover card preview (in ListView)

---

### Recipe 3: Context-Aware Rendering

```rust
// Same field, different contexts:

// In a form
view! {
    <SmartField field=name_field value=name context=FieldContext::EditForm />
}
// → Renders: <input type="text" />

// In a list
view! {
    <SmartField field=name_field value=name context=FieldContext::ListView />
}
// → Renders: <span>John Doe</span>

// In a Kanban card
view! {
    <SmartField field=name_field value=name context=FieldContext::KanbanCard />
}
// → Renders: Compact label + value
```

---

### Recipe 4: Signature Capture

```rust
let signature_field = FieldDef {
    // ... base fields
    field_type: FieldType::Signature,
};

view! {
    <SmartField
        field=signature_field
        value=Signal::from(signature_signal)
        context=FieldContext::EditForm
        on_change=Callback::new(move |base64_png| {
            // Save base64 PNG string
            signature_signal.set(base64_png);
        })
    />
}
```

**Result**: HTML5 canvas with mouse/touch drawing, clear button, save to base64.

---

### Recipe 5: Location Map

```rust
let location_field = FieldDef {
    // ... base fields
    field_type: FieldType::Location,
};

view! {
    <SmartField
        field=location_field
        value=Signal::from(json!({"lat": 40.7128, "lng": -74.0060}))
        context=FieldContext::EditForm
        on_change=on_location_change
    />
}
```

**Result**: Map with manual lat/lng inputs (Leaflet integration ready).

---

## All 24 Field Types at a Glance

| Field Type | Use Case | Edit Render | List Render |
|------------|----------|-------------|-------------|
| `Text` | Single-line text | `<input type="text">` | Truncated text |
| `Email` | Email addresses | `<input type="email">` | Mailto link |
| `Phone` | Phone numbers | `<input type="tel">` | Tel link |
| `Url` | Website URLs | `<input type="url">` | Clickable link |
| `TextArea` | Multi-line text | `<textarea>` | First 100 chars |
| `Number` | Integers/decimals | `<input type="number">` | Formatted number |
| `Money` | Currency | Number + currency | $99.99 |
| `Date` | Date picker | `<input type="date">` | Formatted date |
| `DateTime` | Date + time | `<input type="datetime-local">` | Formatted datetime |
| `Boolean` | Yes/No | Checkbox/toggle | ✓ / ✗ icon |
| `ColorPicker` | Color selection | Color input | Color swatch + hex |
| `Dropdown` | Single choice | AsyncSelect | Badge/pill |
| `Select` | Single choice (legacy) | `<select>` | Plain text |
| `MultiSelect` | Multiple choices | Checkboxes | Comma-separated |
| `TagList` | Multiple tags | Tag input | Tag pills |
| `Rating` | Star rating | Interactive stars | ★★★★☆ |
| `Progress` | Progress bar | Slider + input | Progress bar |
| `RichText` | Markdown/HTML | Textarea + preview | Rendered HTML |
| `Image` | Single image | File upload + preview | Thumbnail |
| `Attachment` | Multiple files | Multi-file upload | File count |
| `Location` | Geographic point | Map + lat/lng inputs | Address text |
| `JsonLogic` | Business rules | Visual rule builder | JSON preview |
| `Signature` | Handwritten signature | Canvas drawing pad | Image preview |
| `Association` | Foreign key | AsyncSelect + modal | Clickable link |
| `MultiLink` | Many-to-many | Multi AsyncSelect | Comma-separated links |

---

## Context-Aware Rendering

SmartField renders differently based on `FieldContext`:

### 1. **CreateForm** / **EditForm**
Full input controls with labels, validation, help text.

### 2. **ListView**
Compact, read-only representation:
- Badges for dropdowns
- Icons for booleans
- Clickable links for associations

### 3. **DetailView**
Rich display with full labels and formatted values.

### 4. **KanbanCard**
Compact label + value pairs for card display.

### 5. **FilterBuilder**
Comparison operators for filtering.

### 6. **InlineEdit**
Simplified inline editing experience.

---

## Next Steps

- 📖 [SmartField API Reference](./smartfield_api.md) - Complete prop documentation
- 🎨 [Component Catalog](./component_catalog.md) - Visual examples of all field types
- 🔧 [Developer Guide](./developer_guide.md) - How to customize and extend
- 🚀 [Deployment Guide](./deployment.md) - Production deployment

---

## Live Demo

Visit the **Component Playground** at `/app/playground` to:
- See all 24 field types in action
- Switch between contexts in real-time
- Toggle light/dark themes
- View JSON output
- Experiment with different configurations

---

## Getting Help

- **Issues**: Check compilation errors in `cargo build`
- **Examples**: See `component_playground.rs` for reference implementations
- **Community**: [Add your community link]

---

## What Makes SmartField Special?

### Traditional Approach (Without SmartField)
```rust
// You write this 50+ times:
match field_type {
    "text" => view! { <input type="text" .../> },
    "email" => view! { <input type="email" .../> },
    "dropdown" => view! { <select>...</select> },
    // ... 20 more cases
}
```

### With SmartField
```rust
// You write this once:
view! { <SmartField field=field_def /> }
```

**That's the power of metadata-driven UI!** 🚀

---

**Version**: 1.0.0  
**Last Updated**: 2024-12-24  
**License**: [Your License]
