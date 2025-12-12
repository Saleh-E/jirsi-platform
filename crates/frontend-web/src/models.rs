//! Frontend-local model types
//! 
//! These are simplified versions of core-models types for use in WASM.
//! They are serialized/deserialized from the backend API.

use serde::{Deserialize, Serialize};

/// Field definition for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub is_required: bool,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
    pub sort_order: i32,
}

/// Field types supported in the UI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Text,
    TextArea,
    Email,
    Phone,
    Url,
    Number,
    Integer,
    Decimal,
    Money,
    Date,
    DateTime,
    Boolean,
    Select,
    MultiSelect,
    Link,
}

impl Default for FieldType {
    fn default() -> Self {
        Self::Text
    }
}

/// View column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewColumn {
    pub field: String,
    pub width: Option<i32>,
    pub visible: bool,
}

/// Entity type metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityType {
    pub id: String,
    pub name: String,
    pub label: String,
    pub label_plural: String,
    pub icon: Option<String>,
}
