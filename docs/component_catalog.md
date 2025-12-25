# SmartField Component Catalog

Visual reference guide for all 24 SmartField types with examples and use cases.

---

## How to Use This Catalog

Each field type includes:
- **Description**: What it's for
- **Value Type**: JSON value format
- **Edit View**: How it renders in forms
- **List View**: How it appears in tables
- **Example Code**: Quick copy-paste snippet
- **Common Use Cases**: When to use this field

---

## Text & Communication

### 1. Text Field

**Description**: Single-line text input for short strings.

**Value Type**: `String`

**Edit View**: `<input type="text">` with placeholder

**List View**: Plain text, truncated if long

**Example**:
```rust
FieldDef {
    name: "full_name".to_string(),
    label: "Full Name".to_string(),
    field_type: FieldType::Text,
    placeholder: Some("Enter full name".to_string()),
    // ... other fields
}
```

**Use Cases**:
- Names (first, last, full)
- Titles and headlines
- Short descriptions
- Usernames
- Product names

---

### 2. Email Field

**Description**: Email address with validation.

**Value Type**: `String` (email format)

**Edit View**: `<input type="email">` with email validation

**List View**: Clickable mailto: link

**Example**:
```rust
FieldDef {
    name: "email".to_string(),
    label: "Email Address".to_string(),
    field_type: FieldType::Email,
    is_required: true,
    // ... other fields
}
```

**Use Cases**:
- Contact emails
- User accounts
- Support tickets
- Newsletter signups

---

### 3. Phone Field

**Description**: Phone number input.

**Value Type**: `String`

**Edit View**: `<input type="tel">` 

**List View**: Clickable tel: link

**Example**:
```rust
FieldDef {
    name: "phone".to_string(),
    label: "Phone Number".to_string(),
    field_type: FieldType::Phone,
    placeholder: Some("+1 (555) 123-4567".to_string()),
    // ... other fields
}
```

**Use Cases**:
- Contact numbers
- Mobile phones
- Office lines
- Emergency contacts

---

### 4. URL Field

**Description**: Website URL input.

**Value Type**: `String` (URL format)

**Edit View**: `<input type="url">` with URL validation

**List View**: Clickable link with icon

**Example**:
```rust
FieldDef {
    name: "website".to_string(),
    label: "Website".to_string(),
    field_type: FieldType::Url,
    placeholder: Some("https://example.com".to_string()),
    // ... other fields
}
```

**Use Cases**:
- Company websites
- Social media profiles
- Documentation links
- Resource URLs

---

### 5. TextArea Field

**Description**: Multi-line text input.

**Value Type**: `String`

**Edit View**: `<textarea>` with multiple rows

**List View**: First 100 characters with "..."

**Example**:
```rust
FieldDef {
    name: "description".to_string(),
    label: "Description".to_string(),
    field_type: FieldType::TextArea,
    placeholder: Some("Enter detailed description...".to_string()),
    // ... other fields
}
```

**Use Cases**:
- Descriptions
- Notes and comments
- Addresses
- Longer content

---

## Numbers & Currency

### 6. Number Field

**Description**: Numeric input (integer or decimal).

**Value Type**: `Number`

**Edit View**: `<input type="number">` with step control

**List View**: Formatted number

**Example**:
```rust
// Integer
FieldDef {
    name: "quantity".to_string(),
    label: "Quantity".to_string(),
    field_type: FieldType::Number { decimals: None },
    // ... other fields
}

// Decimal (2 places)
FieldDef {
    name: "rate".to_string(),
    label: "Rate".to_string(),
    field_type: FieldType::Number { decimals: Some(2) },
    // ... other fields
}
```

**Use Cases**:
- Quantities
- Scores
- Measurements
- Percentages
- Ratings

---

### 7. Money Field

**Description**: Currency amount with formatting.

**Value Type**: `Number` (f64)

**Edit View**: Number input with currency symbol

**List View**: Formatted currency ($99.99)

**Example**:
```rust
FieldDef {
    name: "price".to_string(),
    label: "Price".to_string(),
    field_type: FieldType::Money {
        currency_code: Some("USD".to_string()),
    },
    // ... other fields
}
```

**Use Cases**:
- Prices
- Salaries
- Budgets
- Invoices
- Revenue

---

## Date & Time

### 8. Date Field

**Description**: Date picker (no time).

**Value Type**: `String` (ISO 8601: "2024-12-24")

**Edit View**: `<input type="date">` with calendar picker

**List View**: Formatted date (Dec 24, 2024)

**Example**:
```rust
FieldDef {
    name: "birth_date".to_string(),
    label: "Birth Date".to_string(),
    field_type: FieldType::Date,
    // ... other fields
}
```

**Use Cases**:
- Birth dates
- Deadlines
- Event dates
- Start/end dates

---

### 9. DateTime Field

**Description**: Date and time picker.

**Value Type**: `String` (ISO 8601: "2024-12-24T21:00:00")

**Edit View**: `<input type="datetime-local">`

**List View**: Formatted datetime (Dec 24, 2024 9:00 PM)

**Example**:
```rust
FieldDef {
    name: "meeting_time".to_string(),
    label: "Meeting Time".to_string(),
    field_type: FieldType::DateTime,
    // ... other fields
}
```

**Use Cases**:
- Appointments
- Meetings
- Created/updated timestamps
- Scheduled tasks

---

## Boolean & Selection

### 10. Boolean Field

**Description**: Yes/No, True/False toggle.

**Value Type**: `Boolean`

**Edit View**: Checkbox or toggle switch

**List View**: ✓ or ✗ icon

**Example**:
```rust
FieldDef {
    name: "is_active".to_string(),
    label: "Active".to_string(),
    field_type: FieldType::Boolean,
    default_value: Some(json!(true)),
    // ... other fields
}
```

**Use Cases**:
- Active/inactive status
- Feature flags
- Agreements/consents
- Permissions

---

### 11. Dropdown Field ⭐ RECOMMENDED

**Description**: Single-select dropdown with search, virtual scrolling, and inline creation.

**Value Type**: `String` (selected value)

**Edit View**: AsyncSelect component with search

**List View**: Colored badge/pill

**Example**:
```rust
FieldDef {
    name: "status".to_string(),
    label: "Status".to_string(),
    field_type: FieldType::Dropdown {
        options: vec![
            SelectChoice {
                value: "new".to_string(),
                label: "New".to_string(),
                color: Some("#3B82F6".to_string()),
                icon: None,
                is_default: true,
                sort_order: 0,
            },
            SelectChoice {
                value: "in_progress".to_string(),
                label: "In Progress".to_string(),
                color: Some("#F59E0B".to_string()),
                icon: None,
                is_default: false,
                sort_order: 1,
            },
            SelectChoice {
                value: "completed".to_string(),
                label: "Completed".to_string(),
                color: Some("#10B981".to_string()),
                icon: None,
                is_default: false,
                sort_order: 2,
            },
        ],
        allow_create: true,
    },
    // ... other fields
}
```

**Use Cases**:
- Statuses
- Categories
- Priorities
- Types/Classifications
- Any single-choice selection

**Why Use This**:
- ✅ Virtual scrolling (handles 10,000+ options)
- ✅ Fuzzy search
- ✅ Inline creation
- ✅ Keyboard navigation
- ✅ Beautiful badges in list view

---

### 12. Select Field (Legacy)

**Description**: Simple dropdown (use Dropdown instead).

**Value Type**: `String`

**Edit View**: Basic `<select>` element

**List View**: Plain text

**Example**:
```rust
FieldDef {
    name: "priority".to_string(),
    label: "Priority".to_string(),
    field_type: FieldType::Select {
        options: vec!["Low".to_string(), "Medium".to_string(), "High".to_string()],
    },
    // ... other fields
}
```

**Use Cases**:
- Small option sets (<10 items)
- Simple selections
- Legacy compatibility

---

### 13. MultiSelect Field

**Description**: Multiple selection with checkboxes.

**Value Type**: `Array<String>`

**Edit View**: Checkbox list

**List View**: Comma-separated values

**Example**:
```rust
FieldDef {
    name: "skills".to_string(),
    label: "Skills".to_string(),
    field_type: FieldType::MultiSelect {
        options: vec![
            "JavaScript".to_string(),
            "Rust".to_string(),
            "Python".to_string(),
            "SQL".to_string(),
        ],
    },
    // ... other fields
}
```

**Use Cases**:
- Skills
- Tags
- Features
- Multiple categories

---

### 14. TagList Field

**Description**: Tag-based multi-selection.

**Value Type**: `Array<String>`

**Edit View**: Tag input with autocomplete

**List View**: Tag pills

**Example**:
```rust
FieldDef {
    name: "tags".to_string(),
    label: "Tags".to_string(),
    field_type: FieldType::TagList {
        predefined_tags: Some(vec!["urgent".to_string(), "review".to_string()]),
        allow_create: true,
    },
    // ... other fields
}
```

**Use Cases**:
- Article tags
- Categories
- Keywords
- Labels

---

## Visual & Interactive

### 15. ColorPicker Field

**Description**: Color selection with hex input.

**Value Type**: `String` (#RRGGBB)

**Edit View**: Color picker input

**List View**: Color swatch + hex value

**Example**:
```rust
FieldDef {
    name: "brand_color".to_string(),
    label: "Brand Color".to_string(),
    field_type: FieldType::ColorPicker,
    default_value: Some(json!("#3B82F6")),
    // ... other fields
}
```

**Use Cases**:
- Brand colors
- Theme customization
- UI preferences
- Status colors

---

### 16. Rating Field

**Description**: Star rating system.

**Value Type**: `Number` (1-5)

**Edit View**: Interactive stars

**List View**: ★★★★☆

**Example**:
```rust
FieldDef {
    name: "rating".to_string(),
    label: "Rating".to_string(),
    field_type: FieldType::Rating {
        max_stars: Some(5),
    },
    // ... other fields
}
```

**Use Cases**:
- Product ratings
- Reviews
- Quality scores
- Satisfaction levels

---

### 17. Progress Field

**Description**: Progress bar with slider.

**Value Type**: `Number` (0-100)

**Edit View**: Slider + number input

**List View**: Progress bar

**Example**:
```rust
FieldDef {
    name: "completion".to_string(),
    label: "Completion %".to_string(),
    field_type: FieldType::Progress,
    default_value: Some(json!(0)),
    // ... other fields
}
```

**Use Cases**:
- Task completion
- Project progress
- Loading indicators
- Percentage complete

---

### 18. Image Field

**Description**: Single image upload.

**Value Type**: `String` (URL or base64)

**Edit View**: File upload + preview

**List View**: Thumbnail

**Example**:
```rust
FieldDef {
    name: "avatar".to_string(),
    label: "Profile Picture".to_string(),
    field_type: FieldType::Image,
    // ... other fields
}
```

**Use Cases**:
- Profile pictures
- Product images
- Logos
- Thumbnails

---

### 19. Attachment Field

**Description**: Multiple file uploads.

**Value Type**: `Array<Object>` (file metadata)

**Edit View**: Multi-file upload with preview

**List View**: File count + icons

**Example**:
```rust
FieldDef {
    name: "documents".to_string(),
    label: "Attachments".to_string(),
    field_type: FieldType::Attachment,
    // ... other fields
}
```

**Use Cases**:
- Document uploads
- Multiple attachments
- File collections
- Media galleries

---

### 20. Signature Field

**Description**: Handwritten signature capture.

**Value Type**: `String` (base64 PNG)

**Edit View**: Canvas drawing pad

**List View**: Signature image preview

**Example**:
```rust
FieldDef {
    name: "signature".to_string(),
    label: "Signature".to_string(),
    field_type: FieldType::Signature,
    is_required: true,
    // ... other fields
}
```

**Use Cases**:
- Contract signatures
- Approvals
- Agreements
- Authentication

---

## Relationships

### 21. Association Field ⭐ RECOMMENDED

**Description**: Foreign key relationship to another entity.

**Value Type**: `String` (entity ID)

**Edit View**: AsyncSelect with API lookup + inline creation

**List View**: Clickable link to related entity

**Example**:
```rust
FieldDef {
    name: "customer_id".to_string(),
    label: "Customer".to_string(),
    field_type: FieldType::Association {
        target_entity: "customers".to_string(),
        display_field: "name".to_string(),
        allow_create: true,
    },
    // ... other fields
}
```

**Features**:
- ✅ Async API lookup
- ✅ Inline creation modal
- ✅ Hover card preview
- ✅ Auto-select after creation

**Use Cases**:
- Customer relationships
- Parent/child entities
- Foreign keys
- Lookups

---

### 22. MultiLink Field

**Description**: Many-to-many relationship.

**Value Type**: `Array<String>` (entity IDs)

**Edit View**: Multi AsyncSelect

**List View**: Comma-separated links

**Example**:
```rust
FieldDef {
    name: "assigned_users".to_string(),
    label: "Assigned To".to_string(),
    field_type: FieldType::MultiLink {
        target_entity: "users".to_string(),
        display_field: "name".to_string(),
    },
    // ... other fields
}
```

**Use Cases**:
- Multiple assignees
- Tags to entities
- Related items
- Many-to-many relationships

---

## Advanced

### 23. Location Field

**Description**: Geographic location with lat/lng.

**Value Type**: `Object { lat: Number, lng: Number }`

**Edit View**: Map + manual lat/lng inputs

**List View**: Address text or coordinates

**Example**:
```rust
FieldDef {
    name: "location".to_string(),
    label: "Location".to_string(),
    field_type: FieldType::Location,
    // ... other fields
}
```

**Use Cases**:
- Store locations
- Event venues
- Delivery addresses
- Geographic data

---

### 24. JsonLogic Field

**Description**: Visual business rule builder.

**Value Type**: `Object` (JsonLogic format)

**Edit View**: Visual rule builder with AND/OR logic

**List View**: JSON preview

**Example**:
```rust
FieldDef {
    name: "approval_rules".to_string(),
    label: "Approval Rules".to_string(),
    field_type: FieldType::JsonLogic,
    // ... other fields
}
```

**Use Cases**:
- Business rules
- Conditional logic
- Workflow conditions
- Advanced filtering

---

### 25. RichText Field

**Description**: Markdown/HTML editor.

**Value Type**: `String` (markdown)

**Edit View**: Textarea with markdown preview

**List View**: Rendered HTML (sanitized)

**Example**:
```rust
FieldDef {
    name: "content".to_string(),
    label: "Content".to_string(),
    field_type: FieldType::RichText,
    // ... other fields
}
```

**Use Cases**:
- Blog posts
- Article content
- Rich descriptions
- Formatted text

---

## Quick Reference Table

| Field Type | Best For | Edit UI | List UI | Searchable |
|-----------|----------|---------|---------|------------|
| Text | Names, titles | Input | Text | ✅ |
| Email | Email addresses | Email input | Mailto link | ✅ |
| Phone | Phone numbers | Tel input | Tel link | ✅ |
| Url | Website links | URL input | Link | ✅ |
| TextArea | Long text | Textarea | Truncated | ✅ |
| Number | Quantities | Number input | Formatted | ✅ |
| Money | Currency | Number + symbol | $99.99 | ✅ |
| Date | Dates | Date picker | Formatted date | ✅ |
| DateTime | Timestamps | DateTime picker | Formatted datetime | ✅ |
| Boolean | Yes/No | Checkbox | ✓/✗ | ❌ |
| ColorPicker | Colors | Color input | Swatch | ❌ |
| **Dropdown** | Single choice | **AsyncSelect** | **Badge** | ✅ |
| Select | Legacy choice | Select | Text | ✅ |
| MultiSelect | Multiple choice | Checkboxes | List | ✅ |
| TagList | Tags | Tag input | Pills | ✅ |
| Rating | Star rating | Stars | ★★★☆☆ | ❌ |
| Progress | Percentage | Slider | Progress bar | ❌ |
| Image | Pictures | Upload | Thumbnail | ❌ |
| Attachment | Files | Multi-upload | File count | ❌ |
| Signature | Signatures | Canvas | Image | ❌ |
| **Association** | **Foreign key** | **AsyncSelect + Modal** | **Link** | ✅ |
| MultiLink | Many-to-many | Multi AsyncSelect | Links | ✅ |
| Location | Geographic | Map + inputs | Coordinates | ❌ |
| JsonLogic | Business rules | Visual builder | JSON | ❌ |
| RichText | Formatted text | Markdown editor | HTML | ✅ |

---

## Context Rendering Examples

### Same Field, Different Contexts

```rust
let name_field = FieldDef {
    name: "company_name".to_string(),
    label: "Company Name".to_string(),
    field_type: FieldType::Text,
    // ... other fields
};

// In CreateForm
<SmartField field=name_field value=value context=FieldContext::CreateForm />
// → Renders: <input type="text" placeholder="Enter company name" />

// In ListView
<SmartField field=name_field value=value context=FieldContext::ListView />
// → Renders: <span>Acme Corp</span>

// In DetailView
<SmartField field=name_field value=value context=FieldContext::DetailView />
// → Renders: <div><label>Company Name</label><div>Acme Corp</div></div>

// In KanbanCard
<SmartField field=name_field value=value context=FieldContext::KanbanCard />
// → Renders: <div><span class="label">Company:</span> Acme Corp</div>
```

---

## See Also

- [Quick Start Guide](../README_SMARTFIELD.md)
- [API Reference](./smartfield_api.md)
- [Developer Guide](./developer_guide.md)
- [Component Playground](/app/playground) - Live interactive demo

---

**Version**: 1.0.0  
**Last Updated**: 2024-12-24
