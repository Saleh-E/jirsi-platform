//! FieldDef model - Single source of truth for all field definitions
//! 
//! The Golden Rule: A field is defined once in FieldDef and reused everywhere.
//!
//! ## Antigravity Diamond Protocol
//! 
//! Each field definition now has four dimensions:
//! - **Data**: The shape (FieldType - Text, Number, Money, etc.)
//! - **Logic**: The rules (visible_if, readonly_if via LayoutConfig)
//! - **Physics**: The sync strategy (CRDT vs Last-Write-Wins)
//! - **Intelligence**: AI hints (PII, embeddings, auto-generation)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Import Antigravity layers
use crate::metadata::{AiMetadata, LayoutConfig, MergeStrategy};
use crate::validation::ValidationRule;

// ============================================================================
// FIELD TYPE - PRESERVED FROM LEGACY (The Valuable Logic)
// ============================================================================

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
    
    // ==================
    // Jirsi Enhanced Types
    // ==================
    
    /// Dropdown with search and inline creation (enhanced Select)
    Dropdown { 
        options: Vec<SelectChoice>,
        allow_create: bool,
    },
    /// Smart association link with inline entity creation
    Association {
        target_entity: String,
        display_field: String,
        allow_inline_create: bool,
    },
    /// Color picker field
    ColorPicker,
    /// JsonLogic formula for calculated fields
    JsonLogic { formula: String },
    /// Location/address with optional map display
    Location { show_map: bool },
    /// Progress indicator (0-100)
    Progress { max_value: i32 },
    /// Star rating (1-5 stars)
    Rating { max_stars: u8 },
    /// Digital signature capture
    Signature,
}

impl Default for FieldType {
    fn default() -> Self {
        Self::Text
    }
}

// ============================================================================
// FIELD CONTEXT - PRESERVED FROM LEGACY
// ============================================================================

/// Context in which a field is being rendered
/// Determines the visual representation of the field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FieldContext {
    /// Creating a new record in a form
    CreateForm,
    /// Editing an existing record in a form
    EditForm,
    /// Displaying in a list/table view (cell)
    ListView,
    /// Displaying in a detail/show view
    DetailView,
    /// Displaying on a Kanban card
    KanbanCard,
    /// Building a filter query
    FilterBuilder,
    /// Inline editing (click to edit)
    InlineEdit,
}

/// Context-specific rendering hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRenderHints {
    pub component: Option<String>,
    pub compact: bool,
    pub read_only: bool,
    pub show_label: bool,
}

impl Default for ContextRenderHints {
    fn default() -> Self {
        Self {
            component: None,
            compact: false,
            read_only: false,
            show_label: true,
        }
    }
}

// ============================================================================
// FIELD DEFINITION - ANTIGRAVITY DIAMOND EDITION
// ============================================================================

/// FieldDef model - Single source of truth (Antigravity Edition)
/// 
/// The Diamond Architecture: Each field has four layers:
/// - Data: `field_type` - The shape of data
/// - Logic: `layout.visible_if`, `layout.readonly_if` - Conditional rules
/// - Physics: `physics` - Sync/merge strategy
/// - Intelligence: `intelligence` - AI/LLM hints
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDef {
    // --- Identity ---
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub entity_type_id: Uuid,
    pub name: String,
    pub label: String,
    
    // --- Core Type (Data Layer) ---
    pub field_type: FieldType,
    
    // --- Diamond Layers (The New Power) ---
    
    /// LAYOUT: Adaptive UI configuration
    #[serde(default)]
    pub layout: LayoutConfig,
    
    /// PHYSICS: Sync/merge strategy
    #[serde(default)]
    pub physics: MergeStrategy,
    
    /// INTELLIGENCE: AI/LLM metadata
    #[serde(default)]
    pub intelligence: AiMetadata,
    
    /// RULES: New validation engine (replaces legacy validation)
    #[serde(default)]
    pub rules: Vec<ValidationRule>,
    
    // --- System Meta ---
    #[serde(default)]
    pub is_system: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // --- Legacy Fields (Keep for migration compatibility) ---
    pub default_value: Option<serde_json::Value>,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
    /// @deprecated Use `rules` instead
    pub validation: Option<FieldValidation>,
    /// @deprecated Use `layout` instead
    pub ui_hints: Option<FieldUiHints>,
    /// @deprecated Use `layout.section` instead
    pub group: Option<String>,
    pub options: Option<serde_json::Value>,
    pub context_hints: Option<HashMap<FieldContext, ContextRenderHints>>,
    
    // Legacy boolean flags (move to layout.visible_if/readonly_if)
    #[serde(default)]
    pub is_required: bool,
    #[serde(default)]
    pub is_unique: bool,
    #[serde(default)]
    pub show_in_list: bool,
    #[serde(default)]
    pub show_in_card: bool,
    #[serde(default)]
    pub is_searchable: bool,
    #[serde(default)]
    pub is_filterable: bool,
    #[serde(default)]
    pub is_sortable: bool,
    #[serde(default)]
    pub is_readonly: bool,
}

// ============================================================================
// LEGACY STRUCTS - PRESERVED FOR SERDE COMPATIBILITY
// ============================================================================

/// Validation rules for a field (Legacy - use ValidationRule instead)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldValidation {
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
    pub message: Option<String>,
}

/// Type-specific field options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOptions {
    pub choices: Option<Vec<SelectChoice>>,
    pub link_target: Option<String>,
    pub link_display_field: Option<String>,
    pub currency: Option<String>,
    pub formula: Option<String>,
    pub max_score: Option<i32>,
}

/// A choice for Select/MultiSelect fields
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectChoice {
    pub value: String,
    pub label: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub is_default: bool,
    pub sort_order: i32,
}

/// UI rendering hints (Legacy - use LayoutConfig instead)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldUiHints {
    pub width: Option<i32>,
    pub component: Option<String>,
    pub class: Option<String>,
    #[serde(default)]
    pub hide_label: bool,
    #[serde(default)]
    pub as_badge: bool,
    pub lookup_entity: Option<String>,
}

// ============================================================================
// IMPLEMENTATION
// ============================================================================

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
            
            // Antigravity Diamond Layers
            layout: LayoutConfig::default(),
            physics: MergeStrategy::default(),
            intelligence: AiMetadata::default(),
            rules: Vec::new(),
            
            // System Meta
            is_system: false,
            sort_order: 0,
            created_at: now,
            updated_at: now,
            
            // Legacy Fields (for compatibility)
            default_value: None,
            placeholder: None,
            help_text: None,
            validation: None,
            ui_hints: None,
            group: None,
            options: None,
            context_hints: None,
            
            // Legacy boolean flags
            is_required: false,
            is_unique: false,
            show_in_list: false,
            show_in_card: false,
            is_searchable: false,
            is_filterable: false,
            is_sortable: false,
            is_readonly: false,
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
    pub fn section(mut self, section: &str) -> Self {
        self.layout.section = Some(section.to_string());
        self
    }
    
    /// Builder: mark as system field
    pub fn system(mut self) -> Self {
        self.is_system = true;
        self
    }
}
