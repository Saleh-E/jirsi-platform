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

/// Feature flags for an EntityType - Updated for Real Estate/CRM
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityFlags {
    // -- STANDARD CRM FLAGS --
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
    
    // -- REAL ESTATE SPECIALIZED FLAGS --
    /// Can be published to external portals (Zillow, Rightmove, Dubizzle)
    #[serde(default)]
    pub is_publishable: bool,
    /// Has geo-location data (Lat/Long) for Map View
    #[serde(default)]
    pub has_geo: bool,
    /// Supports image gallery/media management (Listings)
    #[serde(default)]
    pub has_gallery: bool,
    /// Is a "Contract" type (Supports digital signatures/Versioning)
    #[serde(default)]
    pub is_contract: bool,
    /// Supports recurring billing/payments (Rent/Subscriptions)
    #[serde(default)]
    pub has_payments: bool,
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
    
    /// Builder method to enable geo-location features
    pub fn with_geo(mut self) -> Self {
        self.flags.has_geo = true;
        self
    }
    
    /// Builder method to enable image gallery
    pub fn with_gallery(mut self) -> Self {
        self.flags.has_gallery = true;
        self
    }
    
    /// Builder method to mark as publishable to external portals
    pub fn publishable(mut self) -> Self {
        self.flags.is_publishable = true;
        self
    }
    
    /// Builder method to mark as a contract type
    pub fn as_contract(mut self) -> Self {
        self.flags.is_contract = true;
        self
    }
    
    /// Builder method to enable payments/billing
    pub fn with_payments(mut self) -> Self {
        self.flags.has_payments = true;
        self
    }
}

/// App definition - Marketplace Ready
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
    
    // -- MARKETPLACE FIELDS --
    /// ID in the global Jirsi Marketplace
    #[serde(default)]
    pub marketplace_id: Option<String>,
    /// Version of the installed app (e.g., "2.1.0")
    #[serde(default = "default_version")]
    pub version: String,
    /// Vendor/Publisher name (e.g., "Jirsi Core", "Third Party")
    #[serde(default = "default_publisher")]
    pub publisher: String,
    /// Auto-update policy
    #[serde(default)]
    pub auto_update: bool,
    /// Core apps cannot be uninstalled
    #[serde(default)]
    pub is_core: bool,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_publisher() -> String {
    "Local".to_string()
}

impl AppDef {
    pub fn new(id: &str, tenant_id: Uuid, name: &str, label: &str) -> Self {
        let now = Utc::now();
        Self {
            id: id.to_string(),
            tenant_id,
            name: name.to_string(),
            label: label.to_string(),
            icon: None,
            description: None,
            sort_order: 0,
            is_enabled: true,
            marketplace_id: None,
            version: default_version(),
            publisher: default_publisher(),
            auto_update: false,
            is_core: false,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Mark as a core app (cannot be uninstalled)
    pub fn as_core(mut self) -> Self {
        self.is_core = true;
        self
    }
    
    /// Link to marketplace
    pub fn from_marketplace(mut self, marketplace_id: &str) -> Self {
        self.marketplace_id = Some(marketplace_id.to_string());
        self
    }
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
