//! EntityType model - The core of the metadata-driven system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An EntityType defines a type of business object (like Contact, Company, Deal)
/// This is the "DocType" concept - everything is configured, not hard-coded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityType {
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// Which app owns this entity (e.g., "crm", "properties")
    pub app_id: String,
    /// Module within the app (e.g., "sales", "contacts")
    pub module_id: Option<String>,
    /// Internal name (snake_case, e.g., "contact")
    pub name: String,
    /// Display label (e.g., "Contact")
    pub label: String,
    /// Plural label (e.g., "Contacts")
    pub label_plural: String,
    /// Icon identifier
    pub icon: Option<String>,
    /// Description for documentation
    pub description: Option<String>,
    /// Feature flags
    pub flags: EntityFlags,
    /// Default sort field
    pub default_sort_field: Option<String>,
    /// Default sort direction
    pub default_sort_desc: bool,
    /// Soft delete support
    pub soft_delete: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Feature flags for an EntityType
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityFlags {
    /// Can have activity timeline (interactions, notes)
    pub has_activities: bool,
    /// Can have pipeline/stages
    pub has_pipeline: bool,
    /// Can appear in calendar
    pub has_calendar: bool,
    /// Can have tasks associated
    pub has_tasks: bool,
    /// Can have file attachments
    pub has_attachments: bool,
    /// Can be searched globally
    pub is_searchable: bool,
    /// Appears in global navigation
    pub show_in_nav: bool,
}

impl EntityType {
    pub fn new(tenant_id: Uuid, app_id: &str, name: &str, label: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            app_id: app_id.to_string(),
            module_id: None,
            name: name.to_string(),
            label: label.to_string(),
            label_plural: format!("{}s", label),
            icon: None,
            description: None,
            flags: EntityFlags::default(),
            default_sort_field: None,
            default_sort_desc: false,
            soft_delete: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Builder method to enable activity timeline
    pub fn with_activities(mut self) -> Self {
        self.flags.has_activities = true;
        self
    }

    /// Builder method to enable pipeline
    pub fn with_pipeline(mut self) -> Self {
        self.flags.has_pipeline = true;
        self
    }

    /// Builder method to enable tasks
    pub fn with_tasks(mut self) -> Self {
        self.flags.has_tasks = true;
        self
    }

    /// Builder method to show in navigation
    pub fn with_nav(mut self) -> Self {
        self.flags.show_in_nav = true;
        self
    }

    /// Builder method to enable global search
    pub fn searchable(mut self) -> Self {
        self.flags.is_searchable = true;
        self
    }
}

/// App definition - groups related modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDef {
    pub id: String,
    pub tenant_id: Uuid,
    pub name: String,
    pub label: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    /// Display order in navigation
    pub sort_order: i32,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Module definition - groups related entities within an app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDef {
    pub id: String,
    pub app_id: String,
    pub tenant_id: Uuid,
    pub name: String,
    pub label: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_enabled: bool,
}
