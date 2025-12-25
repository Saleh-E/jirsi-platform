# SmartField API Reference

Complete API documentation for the SmartField component system.

---

## Core Component

### `SmartField`

The main polymorphic field component that renders based on type and context.

```rust
#[component]
pub fn SmartField(
    field: FieldDef,
    #[prop(into)] value: Signal<JsonValue>,
    context: FieldContext,
    #[prop(optional)] on_change: Option<Callback<JsonValue>>,
    #[prop(optional)] disabled: bool,
) -> impl IntoView
```

#### Props

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| `field` | `FieldDef` | ✅ | Field definition with type, label, validation, etc. |
| `value` | `Signal<JsonValue>` | ✅ | Current field value as reactive signal |
| `context` | `FieldContext` | ✅ | Rendering context (EditForm, ListView, etc.) |
| `on_change` | `Option<Callback<JsonValue>>` | ❌ | Callback when value changes |
| `disabled` | `bool` | ❌ | Whether field is disabled (default: false) |

#### Example

```rust
use leptos::*;
use serde_json::json;
use core_models::field::{FieldDef, FieldType, FieldContext};
use crate::components::smart_field::SmartField;

#[component]
pub fn MyForm() -> impl IntoView {
    let (email, set_email) = create_signal(json!("user@example.com"));
    
    let field = FieldDef {
        // ... field definition
        field_type: FieldType::Email,
    };
    
    view! {
        <SmartField
            field=field
            value=Signal::from(email)
            context=FieldContext::EditForm
            on_change=Callback::new(move |new_val| {
                set_email.set(new_val);
                // Additional logic here
            })
            disabled=false
        />
    }
}
```

---

## Field Definition (`FieldDef`)

### Core Fields

```rust
pub struct FieldDef {
    pub id: Uuid,                    // Unique field ID
    pub tenant_id: Uuid,             // Multi-tenant ID
    pub entity_type_id: Uuid,        // Parent entity type
    pub name: String,                // Internal field name (snake_case)
    pub label: String,               // Display label
    pub field_type: FieldType,       // Field type (see below)
    
    // Behavior flags
    pub is_required: bool,           // Validation: required
    pub is_unique: bool,             // Validation: unique constraint
    pub is_readonly: bool,           // Cannot be edited
    pub is_searchable: bool,         // Included in search
    pub is_filterable: bool,         // Can be used in filters
    pub is_sortable: bool,           // Can be sorted
    
    // Display flags
    pub show_in_list: bool,          // Show in ListView
    pub show_in_card: bool,          // Show in KanbanCard
    
    // UI Configuration
    pub placeholder: Option<String>, // Input placeholder
    pub help_text: Option<String>,   // Help text below field
    pub default_value: Option<JsonValue>, // Default value
    pub validation: Option<JsonValue>, // Validation rules
    pub options: Option<JsonValue>,  // Field-specific options
    pub ui_hints: Option<UiHints>,   // UI customization
    pub context_hints: Option<ContextHints>, // Context overrides
    
    // Organization
    pub sort_order: i32,             // Display order
    pub group: Option<String>,       // Field group/section
    
    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## Field Types

### Text-Based Fields

#### `FieldType::Text`
Simple single-line text input.

**Value Type**: `String`

```rust
FieldType::Text
```

**Renders**: `<input type="text">`

---

#### `FieldType::Email`
Email address with validation.

**Value Type**: `String`

```rust
FieldType::Email
```

**Renders**: `<input type="email">` with mailto link in ListView

---

#### `FieldType::Phone`
Phone number input.

**Value Type**: `String`

```rust
FieldType::Phone
```

**Renders**: `<input type="tel">` with tel link in ListView

---

#### `FieldType::Url`
Website URL input.

**Value Type**: `String`

```rust
FieldType::Url
```

**Renders**: `<input type="url">` with clickable link in ListView

---

#### `FieldType::TextArea`
Multi-line text input.

**Value Type**: `String`

```rust
FieldType::TextArea
```

**Renders**: `<textarea>` with configurable rows

---

### Number Fields

#### `FieldType::Number`
Numeric input with optional decimal places.

**Value Type**: `Number` (i64 or f64)

```rust
FieldType::Number {
    decimals: Option<u8>, // Number of decimal places (None = integer)
}
```

**Example**:
```rust
// Integer
FieldType::Number { decimals: None }

// Decimal (2 places)
FieldType::Number { decimals: Some(2) }
```

---

#### `FieldType::Money`
Currency amount with formatting.

**Value Type**: `Number` (f64)

```rust
FieldType::Money {
    currency_code: Option<String>, // ISO currency code (e.g., "USD")
}
```

**Renders**: Formatted currency ($99.99)

---

### Date/Time Fields

#### `FieldType::Date`
Date picker (no time).

**Value Type**: `String` (ISO 8601 date: "2024-12-24")

```rust
FieldType::Date
```

**Renders**: `<input type="date">`

---

#### `FieldType::DateTime`
Date and time picker.

**Value Type**: `String` (ISO 8601 datetime: "2024-12-24T21:00:00")

```rust
FieldType::DateTime
```

**Renders**: `<input type="datetime-local">`

---

### Boolean Field

#### `FieldType::Boolean`
Checkbox or toggle switch.

**Value Type**: `Boolean`

```rust
FieldType::Boolean
```

**Renders**:
- EditForm: Checkbox/Toggle
- ListView: ✓ or ✗ icon

---

### Selection Fields

#### `FieldType::Dropdown`
Single-select dropdown with search and inline creation.

**Value Type**: `String` (selected value)

```rust
FieldType::Dropdown {
    options: Vec<SelectChoice>,
    allow_create: bool, // Enable inline creation
}

pub struct SelectChoice {
    pub value: String,             // Internal value
    pub label: String,             // Display label
    pub color: Option<String>,     // Badge color (#hex)
    pub icon: Option<String>,      // Icon name
    pub is_default: bool,          // Default selection
    pub sort_order: i32,           // Display order
}
```

**Example**:
```rust
FieldType::Dropdown {
    options: vec![
        SelectChoice {
            value: "active".to_string(),
            label: "Active".to_string(),
            color: Some("#10B981".to_string()),
            icon: None,
            is_default: true,
            sort_order: 0,
        },
        SelectChoice {
            value: "inactive".to_string(),
            label: "Inactive".to_string(),
            color: Some("#EF4444".to_string()),
            icon: None,
            is_default: false,
            sort_order: 1,
        },
    ],
    allow_create: true,
}
```

**Features**:
- AsyncSelect component
- Virtual scrolling (50+ items)
- Fuzzy search
- Inline creation modal
- Keyboard navigation

---

#### `FieldType::Select`
Legacy simple dropdown (use Dropdown instead).

**Value Type**: `String`

```rust
FieldType::Select {
    options: Vec<String>,
}
```

---

#### `FieldType::MultiSelect`
Multiple selection with checkboxes.

**Value Type**: `Array<String>`

```rust
FieldType::MultiSelect {
    options: Vec<String>,
}
```

---

#### `FieldType::TagList`
Tag-based multi-selection with inline creation.

**Value Type**: `Array<String>`

```rust
FieldType::TagList {
    predefined_tags: Option<Vec<String>>,
    allow_create: bool,
}
```

---

### Relationship Fields

#### `FieldType::Association`
Foreign key relationship to another entity.

**Value Type**: `String` (entity ID)

```rust
FieldType::Association {
    target_entity: String,    // Target entity type code
    display_field: String,    // Field to display from target
    allow_create: bool,       // Enable inline creation
}
```

**Example**:
```rust
FieldType::Association {
    target_entity: "customers".to_string(),
    display_field: "name".to_string(),
    allow_create: true,
}
```

**Features**:
- AsyncSelect with API lookup
- Inline creation modal
- Hover card preview
- Auto-select after creation

---

#### `FieldType::MultiLink`
Many-to-many relationship.

**Value Type**: `Array<String>` (entity IDs)

```rust
FieldType::MultiLink {
    target_entity: String,
    display_field: String,
}
```

---

### Visual Fields

#### `FieldType::ColorPicker`
Color selection with hex input.

**Value Type**: `String` (#hexcolor)

```rust
FieldType::ColorPicker
```

**Renders**: Color input with swatch preview

---

#### `FieldType::Image`
Single image upload with preview.

**Value Type**: `String` (URL or base64)

```rust
FieldType::Image
```

**Features**:
- File upload
- Image preview
- Remove button
- Object URL handling

---

#### `FieldType::Attachment`
Multiple file uploads.

**Value Type**: `Array<Object>` (file metadata)

```rust
FieldType::Attachment
```

**Features**:
- Multi-file selection
- File preview
- Individual remove
- File type filtering

---

#### `FieldType::Signature`
Handwritten signature capture.

**Value Type**: `String` (base64 PNG)

```rust
FieldType::Signature
```

**Features**:
- HTML5 canvas drawing
- Mouse and touch support
- Clear button
- Save to base64 PNG

---

### Advanced Fields

#### `FieldType::Location`
Geographic location.

**Value Type**: `Object { lat: Number, lng: Number }`

```rust
FieldType::Location
```

**Features**:
- Manual lat/lng inputs
- Map placeholder
- Leaflet integration ready

---

#### `FieldType::JsonLogic`
Visual business rule builder.

**Value Type**: `Object` (JsonLogic format)

```rust
FieldType::JsonLogic
```

**Features**:
- Visual rule builder
- AND/OR logic groups
- Field/operator/value selection
- Live JSON preview

---

#### `FieldType::Rating`
Star rating system.

**Value Type**: `Number` (1-5)

```rust
FieldType::Rating {
    max_stars: Option<u8>, // Default: 5
}
```

**Renders**: Interactive star rating (★★★☆☆)

---

#### `FieldType::Progress`
Progress bar with input.

**Value Type**: `Number` (0-100)

```rust
FieldType::Progress
```

**Renders**: Visual progress bar + slider

---

#### `FieldType::RichText`
Markdown/HTML editor.

**Value Type**: `String` (markdown)

```rust
FieldType::RichText
```

**Features**:
- Textarea input
- Markdown preview
- Basic formatting

---

## Field Contexts

### `FieldContext` Enum

```rust
pub enum FieldContext {
    CreateForm,      // New entity creation
    EditForm,        // Existing entity editing
    InlineEdit,      // Quick inline edit
    ListView,        // Compact list display
    DetailView,      // Full detail display
    KanbanCard,      // Kanban card display
    FilterBuilder,   // Filter comparison
}
```

### Context Rendering Behavior

| Context | Purpose | Rendering Style |
|---------|---------|-----------------|
| `CreateForm` | New entities | Full input with empty defaults |
| `EditForm` | Edit existing | Full input with current values |
| `InlineEdit` | Quick edit | Simplified inline input |
| `ListView` | Table rows | Compact, read-only display |
| `DetailView` | Entity detail | Rich formatted display |
| `KanbanCard` | Kanban boards | Label + value compact format |
| `FilterBuilder` | Filtering UI | Comparison operators |

---

## Supporting Components

### `AsyncSelect`

Virtual scrolling dropdown for large option sets.

```rust
#[component]
pub fn AsyncSelect(
    options: Vec<SelectOption>,
    #[prop(into)] selected: Signal<Option<String>>,
    on_select: Callback<String>,
    #[prop(optional)] allow_create: bool,
    #[prop(optional)] on_create: Option<Callback<String>>,
    #[prop(optional)] placeholder: String,
) -> impl IntoView
```

**Features**:
- Virtual scrolling (O(1) performance)
- Fuzzy search with 300ms debounce
- Keyboard navigation
- Inline creation

---

### `AssociationModal`

Modal for creating new entities inline.

```rust
#[component]
pub fn AssociationModal(
    entity_type: String,
    show: Signal<bool>,
    on_close: Callback<()>,
    on_created: Callback<String>, // Returns new entity ID
) -> impl IntoView
```

---

### `EntityHoverCard`

Preview card for entity associations.

```rust
#[component]
pub fn EntityHoverCard(
    entity_type: String,
    entity_id: String,
    #[prop(into)] position: Signal<(f64, f64)>,
) -> impl IntoView
```

---

### `SignaturePad`

Canvas-based signature capture.

```rust
#[component]
pub fn SignaturePad(
    on_save: Callback<String>, // Base64 PNG
    #[prop(default = 600)] width: u32,
    #[prop(default = 200)] height: u32,
    #[prop(default = "#000000".to_string())] pen_color: String,
    #[prop(default = 2.0)] pen_width: f64,
) -> impl IntoView
```

---

### `LocationMap`

Map component for location selection.

```rust
#[component]
pub fn LocationMap(
    #[prop(into)] lat: Signal<f64>,
    #[prop(into)] lng: Signal<f64>,
    #[prop(optional)] on_change: Option<Callback<(f64, f64)>>,
    #[prop(default = "400px".to_string())] height: String,
    #[prop(default = true)] editable: bool,
) -> impl IntoView
```

---

### `RuleBuilder`

Visual JsonLogic rule builder.

```rust
#[component]
pub fn RuleBuilder(
    on_change: Callback<JsonValue>,
    #[prop(default = vec![])] available_fields: Vec<String>,
) -> impl IntoView
```

---

## Performance Characteristics

| Component | Dataset Size | Render Time | FPS |
|-----------|--------------|-------------|-----|
| AsyncSelect | 10,000 items | <100ms | 60 |
| SmartField (basic) | N/A | <5ms | 60 |
| SignaturePad | N/A | <5ms drawing latency | 60 |
| Virtual Scroll | 50+ items | O(1) | 60 |

---

## Browser Support

- ✅ Chrome 90+
- ✅ Firefox 88+
- ✅ Safari 14+
- ✅ Edge 90+

---

## Accessibility

All SmartField components are WCAG AA compliant:
- ✅ ARIA labels
- ✅ Keyboard navigation
- ✅ Screen reader support
- ✅ Color contrast compliant

---

## See Also

- [Quick Start Guide](../README_SMARTFIELD.md)
- [Component Catalog](./component_catalog.md)
- [Developer Guide](./developer_guide.md)

---

**Version**: 1.0.0  
**Last Updated**: 2024-12-24
