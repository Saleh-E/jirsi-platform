//! FieldDef model - Single source of truth for all field definitions
//! 
//! The Golden Rule: A field is defined once in FieldDef and reused everywhere.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// All supported field types in the system
/// Uses tagged serde format for type-specific configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum FieldType {
    // ==================
    // Basic Types
    // ==================
    
    /// Single line text
    Text,
    /// Multi-line text
    TextArea,
    /// Rich text with HTML formatting
    RichText,
    /// Numeric value with configurable decimals
    Number { decimals: Option<u8> },
    /// Currency amount with currency code
    Money { currency_code: Option<String> },
    /// True/false boolean
    Boolean,
    /// Date only (no time)
    Date,
    /// Date and time
    DateTime,
    
    // ==================
    // Selection Types
    // ==================
    
    /// Single select from dropdown options
    Select { options: Vec<String> },
    /// Multiple select from options
    MultiSelect { options: Vec<String> },
    
    // ==================
    // Relational Types
    // ==================
    
    /// Link to another entity (foreign key reference)
    Link { target_entity: String },
    /// Multiple links to entities
    MultiLink { target_entity: String },
    /// List of tags (free-form string chips)
    TagList,
    
    // ==================
    // Advanced Types
    // ==================
    
    /// Image URL
    Image,
    /// File/document attachment (URL + metadata)
    Attachment,
    /// Multiple attachments
    MultiAttachment,
    /// Email address with validation
    Email,
    /// Phone number
    Phone,
    /// URL with validation
    Url,
    /// Numeric score (e.g., lead score 1-100)
    Score { max_value: Option<i32> },
    /// JSON data (arbitrary structure)
    Json,
    
    // ==================
    // Computed Types
    // ==================
    
    /// Calculated field from formula
    Calculated { formula: String },
    /// Rollup from related records
    Rollup { 
        target_field: String, 
        operation: String,  // sum, count, avg, min, max
    },
}

impl Default for FieldType {
    fn default() -> Self {
        Self::Text
    }
}

/// Field definition - describes a single field on an EntityType
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// Parent EntityType
    pub entity_type_id: Uuid,
    /// Internal field name (snake_case)
    pub name: String,
    /// Display label
    pub label: String,
    /// Field data type
    pub field_type: FieldType,
    /// Is this field required?
    pub is_required: bool,
    /// Is field unique within entity?
    pub is_unique: bool,
    /// Show in list views by default?
    pub show_in_list: bool,
    /// Show in card views?
    pub show_in_card: bool,
    /// Include in search index?
    pub is_searchable: bool,
    /// Can be used for filtering?
    pub is_filterable: bool,
    /// Can be sorted?
    pub is_sortable: bool,
    /// Is read-only (system field)?
    pub is_readonly: bool,
    /// Default value (JSON)
    pub default_value: Option<serde_json::Value>,
    /// Placeholder text for forms
    pub placeholder: Option<String>,
    /// Help text / tooltip
    pub help_text: Option<String>,
    /// Validation rules
    pub validation: Option<FieldValidation>,
    /// Type-specific options (raw JSON - can be array of strings, array of objects, or FieldOptions struct)
    pub options: Option<serde_json::Value>,
    /// UI rendering hints
    pub ui_hints: Option<FieldUiHints>,
    /// Display order in forms/lists
    pub sort_order: i32,
    /// Group/section for form layout
    pub group: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Validation rules for a field
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldValidation {
    /// Minimum length for text fields
    pub min_length: Option<i32>,
    /// Maximum length for text fields
    pub max_length: Option<i32>,
    /// Minimum value for numeric fields
    pub min_value: Option<f64>,
    /// Maximum value for numeric fields
    pub max_value: Option<f64>,
    /// Regex pattern for validation
    pub pattern: Option<String>,
    /// Custom validation message
    pub message: Option<String>,
}

/// Type-specific field options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOptions {
    /// For Select/MultiSelect: list of options
    pub choices: Option<Vec<SelectChoice>>,
    /// For Link/MultiLink: target EntityType name
    pub link_target: Option<String>,
    /// For Link: display field from linked entity
    pub link_display_field: Option<String>,
    /// For Money: currency code
    pub currency: Option<String>,
    /// For Calculated: formula expression
    pub formula: Option<String>,
    /// For Score: max score value
    pub max_score: Option<i32>,
}

/// A choice for Select/MultiSelect fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectChoice {
    pub value: String,
    pub label: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub is_default: bool,
    pub sort_order: i32,
}

/// UI rendering hints
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldUiHints {
    /// Width in grid columns (1-12)
    pub width: Option<i32>,
    /// Custom component to use
    pub component: Option<String>,
    /// CSS class names
    pub class: Option<String>,
    /// Hide label in forms
    #[serde(default)]
    pub hide_label: bool,
    /// Display as read-only chip/badge
    #[serde(default)]
    pub as_badge: bool,
    /// For lookup fields: target entity type to fetch
    pub lookup_entity: Option<String>,
}

impl FieldDef {
    pub fn new(
        tenant_id: Uuid,
        entity_type_id: Uuid,
        name: &str,
        label: &str,
        field_type: FieldType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            entity_type_id,
            name: name.to_string(),
            label: label.to_string(),
            field_type,
            is_required: false,
            is_unique: false,
            show_in_list: false,
            show_in_card: false,
            is_searchable: false,
            is_filterable: false,
            is_sortable: false,
            is_readonly: false,
            default_value: None,
            placeholder: None,
            help_text: None,
            validation: None,
            options: None,
            ui_hints: None,
            sort_order: 0,
            group: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Builder: make field required
    pub fn required(mut self) -> Self {
        self.is_required = true;
        self
    }

    /// Builder: show in list view
    pub fn in_list(mut self) -> Self {
        self.show_in_list = true;
        self
    }

    /// Builder: make searchable
    pub fn searchable(mut self) -> Self {
        self.is_searchable = true;
        self
    }

    /// Builder: make filterable
    pub fn filterable(mut self) -> Self {
        self.is_filterable = true;
        self
    }

    /// Builder: make sortable
    pub fn sortable(mut self) -> Self {
        self.is_sortable = true;
        self
    }

    /// Builder: set sort order
    pub fn order(mut self, order: i32) -> Self {
        self.sort_order = order;
        self
    }

    /// Builder: set group/section
    pub fn group(mut self, group: &str) -> Self {
        self.group = Some(group.to_string());
        self
    }
}
