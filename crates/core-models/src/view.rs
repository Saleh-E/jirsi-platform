//! View model - View definitions for rendering entity lists

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of view for rendering entity lists
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewType {
    /// Standard table/grid view
    Table,
    /// Card layout
    Card,
    /// Kanban board (stages/columns)
    Kanban,
    /// Calendar view
    Calendar,
    /// Map view (geo-located data)
    Map,
    /// Gantt chart (timeline)
    Gantt,
    /// Free-form canvas
    Canvas,
}

impl Default for ViewType {
    fn default() -> Self {
        Self::Table
    }
}

/// Sort direction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        Self::Asc
    }
}

/// View definition - how to display an entity list
/// The Golden Rule: Views are metadata-driven configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDef {
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// Which EntityType this view is for
    pub entity_type_id: Uuid,
    /// View name (internal)
    pub name: String,
    /// Display label
    pub label: String,
    /// Type of view
    pub view_type: ViewType,
    /// Is this the default view for the entity?
    pub is_default: bool,
    /// Is this a system view (non-deletable)?
    pub is_system: bool,
    /// Creator user ID (None for system views)
    pub created_by: Option<Uuid>,
    /// Owner for personal views (None = global/system)
    pub owner_id: Option<Uuid>,
    /// User favorite flag
    pub is_favorite: bool,
    /// Field to group by (for Kanban views)
    pub group_by: Option<String>,
    /// Column configuration
    pub columns: Vec<ViewColumn>,
    /// Default filters
    pub filters: Vec<ViewFilter>,
    /// Default sort
    pub sort: Vec<ViewSort>,
    /// View-specific settings (JSON)
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Column in a table/card view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewColumn {
    /// Field name to display
    pub field: String,
    /// Display width (pixels or percentage)
    pub width: Option<String>,
    /// Is column visible?
    pub visible: bool,
    /// Display order
    pub sort_order: i32,
}

/// Filter condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

/// Filter operators
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    IsNull,
    IsNotNull,
    In,
    NotIn,
    Between,
}

/// Sort configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewSort {
    pub field: String,
    pub direction: SortDirection,
}

/// Kanban-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanSettings {
    /// Field to group by (usually a Select field)
    pub group_by_field: String,
    /// Card title field
    pub title_field: String,
    /// Card description field
    pub description_field: Option<String>,
    /// Fields to show on cards
    pub card_fields: Vec<String>,
    /// Allow drag between columns
    pub allow_drag: bool,
}

/// Calendar-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSettings {
    /// Date/DateTime field for start
    pub start_field: String,
    /// Date/DateTime field for end (optional)
    pub end_field: Option<String>,
    /// Field for event title
    pub title_field: String,
    /// Field for event color
    pub color_field: Option<String>,
}

impl ViewDef {
    pub fn table(tenant_id: Uuid, entity_type_id: Uuid, name: &str, label: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            entity_type_id,
            name: name.to_string(),
            label: label.to_string(),
            view_type: ViewType::Table,
            is_default: false,
            is_system: false,
            created_by: None,
            owner_id: None,
            is_favorite: false,
            group_by: None,
            columns: Vec::new(),
            filters: Vec::new(),
            sort: Vec::new(),
            settings: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn kanban(tenant_id: Uuid, entity_type_id: Uuid, name: &str, label: &str, group_by_field: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            entity_type_id,
            name: name.to_string(),
            label: label.to_string(),
            view_type: ViewType::Kanban,
            is_default: false,
            is_system: false,
            created_by: None,
            owner_id: None,
            is_favorite: false,
            group_by: Some(group_by_field.to_string()),
            columns: Vec::new(),
            filters: Vec::new(),
            sort: Vec::new(),
            settings: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn map(tenant_id: Uuid, entity_type_id: Uuid, name: &str, label: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            entity_type_id,
            name: name.to_string(),
            label: label.to_string(),
            view_type: ViewType::Map,
            is_default: false,
            is_system: false,
            created_by: None,
            owner_id: None,
            is_favorite: false,
            group_by: None,
            columns: Vec::new(),
            filters: Vec::new(),
            sort: Vec::new(),
            settings: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn as_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    pub fn as_system(mut self) -> Self {
        self.is_system = true;
        self
    }

    pub fn with_columns(mut self, columns: Vec<ViewColumn>) -> Self {
        self.columns = columns;
        self
    }
    
    pub fn with_group_by(mut self, field: &str) -> Self {
        self.group_by = Some(field.to_string());
        self
    }
    
    pub fn owned_by(mut self, user_id: Uuid) -> Self {
        self.owner_id = Some(user_id);
        self
    }
}

