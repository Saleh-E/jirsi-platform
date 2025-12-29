//! Entity Registry - Single Source of Truth for all entity definitions
//!
//! This module defines all CRM and Real Estate entities in code.
//! The sidebar, API routes, and field rendering all derive from this registry.
//!
//! Benefits:
//! - Type-safe entity access
//! - Compile-time validation
//! - Auto-generated sidebar navigation
//! - Consistent field definitions across list/detail/create views

use serde::{Deserialize, Serialize};

// ============================================
// CORE TYPES
// ============================================

/// Category of the entity (determines sidebar grouping)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    CRM,
    RealEstate,
    System,
}

impl Category {
    pub fn label(&self) -> &'static str {
        match self {
            Category::CRM => "CRM",
            Category::RealEstate => "REAL ESTATE",
            Category::System => "SYSTEM",
        }
    }
}

/// Type of field for rendering and validation
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Text,
    TextArea,
    Number,
    Currency,
    Email,
    Phone,
    Date,
    DateTime,
    Boolean,
    Select(&'static [SelectOption]),
    MultiSelect(&'static [SelectOption]),
    Lookup { entity: &'static str },
    Address,
    Url,
    Image,
}

/// Option for Select/MultiSelect fields
#[derive(Debug, Clone, PartialEq)]
pub struct SelectOption {
    pub value: &'static str,
    pub label: &'static str,
    pub color: Option<&'static str>,
}

/// Definition of a single field
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: &'static str,
    pub label: &'static str,
    pub field_type: FieldType,
    pub required: bool,
    pub show_in_list: bool,
    pub show_in_detail: bool,
    pub show_in_create: bool,
    pub editable: bool,
}

impl FieldDef {
    pub const fn new(name: &'static str, label: &'static str, field_type: FieldType) -> Self {
        Self {
            name,
            label,
            field_type,
            required: false,
            show_in_list: true,
            show_in_detail: true,
            show_in_create: true,
            editable: true,
        }
    }

    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub const fn hidden_in_list(mut self) -> Self {
        self.show_in_list = false;
        self
    }

    pub const fn readonly(mut self) -> Self {
        self.editable = false;
        self
    }
}

/// Available view types for an entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewType {
    Table,
    Kanban,
    Calendar,
    Map,
}

impl ViewType {
    pub fn label(&self) -> &'static str {
        match self {
            ViewType::Table => "Table",
            ViewType::Kanban => "Kanban",
            ViewType::Calendar => "Calendar",
            ViewType::Map => "Map",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ViewType::Table => "fa-table",
            ViewType::Kanban => "fa-columns",
            ViewType::Calendar => "fa-calendar",
            ViewType::Map => "fa-map",
        }
    }
}

/// Complete entity definition
#[derive(Debug, Clone)]
pub struct EntityDef {
    pub code: &'static str,
    pub label: &'static str,
    pub label_plural: &'static str,
    pub icon: &'static str,
    pub category: Category,
    pub fields: &'static [FieldDef],
    pub views: &'static [ViewType],
    pub kanban_field: Option<&'static str>,
}

// ============================================
// COMMON SELECT OPTIONS
// ============================================

pub static CONTACT_TYPE_OPTIONS: &[SelectOption] = &[
    SelectOption { value: "buyer", label: "Buyer", color: Some("blue") },
    SelectOption { value: "seller", label: "Seller", color: Some("green") },
    SelectOption { value: "landlord", label: "Landlord", color: Some("purple") },
    SelectOption { value: "tenant", label: "Tenant", color: Some("amber") },
    SelectOption { value: "agent", label: "Agent", color: Some("cyan") },
    SelectOption { value: "other", label: "Other", color: None },
];

pub static LEAD_STATUS_OPTIONS: &[SelectOption] = &[
    SelectOption { value: "new", label: "New", color: Some("blue") },
    SelectOption { value: "contacted", label: "Contacted", color: Some("amber") },
    SelectOption { value: "qualified", label: "Qualified", color: Some("purple") },
    SelectOption { value: "converted", label: "Converted", color: Some("green") },
    SelectOption { value: "lost", label: "Lost", color: Some("red") },
];

pub static DEAL_STAGE_OPTIONS: &[SelectOption] = &[
    SelectOption { value: "discovery", label: "Discovery", color: Some("blue") },
    SelectOption { value: "proposal", label: "Proposal", color: Some("amber") },
    SelectOption { value: "negotiation", label: "Negotiation", color: Some("purple") },
    SelectOption { value: "closed_won", label: "Closed Won", color: Some("green") },
    SelectOption { value: "closed_lost", label: "Closed Lost", color: Some("red") },
];

pub static TASK_STATUS_OPTIONS: &[SelectOption] = &[
    SelectOption { value: "todo", label: "To Do", color: Some("neutral") },
    SelectOption { value: "in_progress", label: "In Progress", color: Some("blue") },
    SelectOption { value: "completed", label: "Completed", color: Some("green") },
    SelectOption { value: "cancelled", label: "Cancelled", color: Some("red") },
];

pub static TASK_PRIORITY_OPTIONS: &[SelectOption] = &[
    SelectOption { value: "low", label: "Low", color: Some("neutral") },
    SelectOption { value: "medium", label: "Medium", color: Some("amber") },
    SelectOption { value: "high", label: "High", color: Some("red") },
];

pub static PROPERTY_TYPE_OPTIONS: &[SelectOption] = &[
    SelectOption { value: "apartment", label: "Apartment", color: None },
    SelectOption { value: "villa", label: "Villa", color: None },
    SelectOption { value: "townhouse", label: "Townhouse", color: None },
    SelectOption { value: "penthouse", label: "Penthouse", color: None },
    SelectOption { value: "land", label: "Land", color: None },
    SelectOption { value: "commercial", label: "Commercial", color: None },
];

pub static PROPERTY_STATUS_OPTIONS: &[SelectOption] = &[
    SelectOption { value: "draft", label: "Draft", color: Some("neutral") },
    SelectOption { value: "active", label: "Active", color: Some("green") },
    SelectOption { value: "reserved", label: "Reserved", color: Some("amber") },
    SelectOption { value: "under_offer", label: "Under Offer", color: Some("purple") },
    SelectOption { value: "sold", label: "Sold", color: Some("blue") },
    SelectOption { value: "rented", label: "Rented", color: Some("cyan") },
    SelectOption { value: "withdrawn", label: "Withdrawn", color: Some("red") },
];

pub static USAGE_OPTIONS: &[SelectOption] = &[
    SelectOption { value: "sale", label: "Sale", color: Some("green") },
    SelectOption { value: "rent", label: "Rent", color: Some("blue") },
    SelectOption { value: "both", label: "Both", color: Some("purple") },
];

pub static VIEWING_STATUS_OPTIONS: &[SelectOption] = &[
    SelectOption { value: "scheduled", label: "Scheduled", color: Some("blue") },
    SelectOption { value: "completed", label: "Completed", color: Some("green") },
    SelectOption { value: "cancelled", label: "Cancelled", color: Some("red") },
    SelectOption { value: "no_show", label: "No Show", color: Some("amber") },
];

pub static OFFER_STATUS_OPTIONS: &[SelectOption] = &[
    SelectOption { value: "pending", label: "Pending", color: Some("amber") },
    SelectOption { value: "accepted", label: "Accepted", color: Some("green") },
    SelectOption { value: "rejected", label: "Rejected", color: Some("red") },
    SelectOption { value: "countered", label: "Countered", color: Some("purple") },
    SelectOption { value: "expired", label: "Expired", color: Some("neutral") },
];

pub static CONTRACT_STATUS_OPTIONS: &[SelectOption] = &[
    SelectOption { value: "draft", label: "Draft", color: Some("neutral") },
    SelectOption { value: "pending_signature", label: "Pending Signature", color: Some("amber") },
    SelectOption { value: "active", label: "Active", color: Some("green") },
    SelectOption { value: "completed", label: "Completed", color: Some("blue") },
    SelectOption { value: "terminated", label: "Terminated", color: Some("red") },
];

// ============================================
// ENTITY FIELD DEFINITIONS
// ============================================

pub static CONTACT_FIELDS: &[FieldDef] = &[
    FieldDef { name: "first_name", label: "First Name", field_type: FieldType::Text, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "last_name", label: "Last Name", field_type: FieldType::Text, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "email", label: "Email", field_type: FieldType::Email, required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "phone", label: "Phone", field_type: FieldType::Phone, required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "contact_type", label: "Type", field_type: FieldType::Select(CONTACT_TYPE_OPTIONS), required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "lead_status", label: "Lead Status", field_type: FieldType::Select(LEAD_STATUS_OPTIONS), required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "company_id", label: "Company", field_type: FieldType::Lookup { entity: "company" }, required: false, show_in_list: false, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "notes", label: "Notes", field_type: FieldType::TextArea, required: false, show_in_list: false, show_in_detail: true, show_in_create: true, editable: true },
];

pub static COMPANY_FIELDS: &[FieldDef] = &[
    FieldDef { name: "name", label: "Company Name", field_type: FieldType::Text, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "industry", label: "Industry", field_type: FieldType::Text, required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "website", label: "Website", field_type: FieldType::Url, required: false, show_in_list: false, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "phone", label: "Phone", field_type: FieldType::Phone, required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "address", label: "Address", field_type: FieldType::Address, required: false, show_in_list: false, show_in_detail: true, show_in_create: true, editable: true },
];

pub static DEAL_FIELDS: &[FieldDef] = &[
    FieldDef { name: "title", label: "Deal Title", field_type: FieldType::Text, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "value", label: "Value", field_type: FieldType::Currency, required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "stage", label: "Stage", field_type: FieldType::Select(DEAL_STAGE_OPTIONS), required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "contact_id", label: "Contact", field_type: FieldType::Lookup { entity: "contact" }, required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "property_id", label: "Property", field_type: FieldType::Lookup { entity: "property" }, required: false, show_in_list: false, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "expected_close_date", label: "Expected Close", field_type: FieldType::Date, required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
];

pub static TASK_FIELDS: &[FieldDef] = &[
    FieldDef { name: "title", label: "Title", field_type: FieldType::Text, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "status", label: "Status", field_type: FieldType::Select(TASK_STATUS_OPTIONS), required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "priority", label: "Priority", field_type: FieldType::Select(TASK_PRIORITY_OPTIONS), required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "due_date", label: "Due Date", field_type: FieldType::Date, required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "description", label: "Description", field_type: FieldType::TextArea, required: false, show_in_list: false, show_in_detail: true, show_in_create: true, editable: true },
];

pub static PROPERTY_FIELDS: &[FieldDef] = &[
    FieldDef { name: "reference", label: "Reference", field_type: FieldType::Text, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: false },
    FieldDef { name: "title", label: "Title", field_type: FieldType::Text, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "property_type", label: "Property Type", field_type: FieldType::Select(PROPERTY_TYPE_OPTIONS), required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "usage", label: "Usage", field_type: FieldType::Select(USAGE_OPTIONS), required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "status", label: "Status", field_type: FieldType::Select(PROPERTY_STATUS_OPTIONS), required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "sale_price", label: "Sale Price", field_type: FieldType::Currency, required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "rent_price", label: "Rent Price", field_type: FieldType::Currency, required: false, show_in_list: false, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "city", label: "City", field_type: FieldType::Text, required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "area", label: "Area", field_type: FieldType::Text, required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "bedrooms", label: "Bedrooms", field_type: FieldType::Number, required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "bathrooms", label: "Bathrooms", field_type: FieldType::Number, required: false, show_in_list: false, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "size_sqft", label: "Size (sqft)", field_type: FieldType::Number, required: false, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
];

pub static LISTING_FIELDS: &[FieldDef] = &[
    FieldDef { name: "property_id", label: "Property", field_type: FieldType::Lookup { entity: "property" }, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "title", label: "Title", field_type: FieldType::Text, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "description", label: "Description", field_type: FieldType::TextArea, required: false, show_in_list: false, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "status", label: "Status", field_type: FieldType::Select(PROPERTY_STATUS_OPTIONS), required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "published_date", label: "Published", field_type: FieldType::Date, required: false, show_in_list: true, show_in_detail: true, show_in_create: false, editable: false },
];

pub static VIEWING_FIELDS: &[FieldDef] = &[
    FieldDef { name: "property_id", label: "Property", field_type: FieldType::Lookup { entity: "property" }, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "contact_id", label: "Contact", field_type: FieldType::Lookup { entity: "contact" }, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "scheduled_at", label: "Scheduled At", field_type: FieldType::DateTime, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "status", label: "Status", field_type: FieldType::Select(VIEWING_STATUS_OPTIONS), required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "notes", label: "Notes", field_type: FieldType::TextArea, required: false, show_in_list: false, show_in_detail: true, show_in_create: true, editable: true },
];

pub static OFFER_FIELDS: &[FieldDef] = &[
    FieldDef { name: "property_id", label: "Property", field_type: FieldType::Lookup { entity: "property" }, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "contact_id", label: "Contact", field_type: FieldType::Lookup { entity: "contact" }, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "amount", label: "Offer Amount", field_type: FieldType::Currency, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "status", label: "Status", field_type: FieldType::Select(OFFER_STATUS_OPTIONS), required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "submitted_at", label: "Submitted", field_type: FieldType::Date, required: false, show_in_list: true, show_in_detail: true, show_in_create: false, editable: false },
    FieldDef { name: "valid_until", label: "Valid Until", field_type: FieldType::Date, required: false, show_in_list: false, show_in_detail: true, show_in_create: true, editable: true },
];

pub static CONTRACT_FIELDS: &[FieldDef] = &[
    FieldDef { name: "reference", label: "Contract #", field_type: FieldType::Text, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: false },
    FieldDef { name: "property_id", label: "Property", field_type: FieldType::Lookup { entity: "property" }, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "contact_id", label: "Client", field_type: FieldType::Lookup { entity: "contact" }, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "contract_type", label: "Type", field_type: FieldType::Text, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "status", label: "Status", field_type: FieldType::Select(CONTRACT_STATUS_OPTIONS), required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "value", label: "Value", field_type: FieldType::Currency, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "start_date", label: "Start Date", field_type: FieldType::Date, required: true, show_in_list: true, show_in_detail: true, show_in_create: true, editable: true },
    FieldDef { name: "end_date", label: "End Date", field_type: FieldType::Date, required: false, show_in_list: false, show_in_detail: true, show_in_create: true, editable: true },
];

// ============================================
// ENTITY REGISTRY - ALL 9 ENTITIES
// ============================================

pub static ENTITIES: &[EntityDef] = &[
    // CRM Entities
    EntityDef {
        code: "contact",
        label: "Contact",
        label_plural: "Contacts",
        icon: "fa-users",
        category: Category::CRM,
        fields: CONTACT_FIELDS,
        views: &[ViewType::Table, ViewType::Kanban],
        kanban_field: Some("lead_status"),
    },
    EntityDef {
        code: "company",
        label: "Company",
        label_plural: "Companies",
        icon: "fa-building",
        category: Category::CRM,
        fields: COMPANY_FIELDS,
        views: &[ViewType::Table],
        kanban_field: None,
    },
    EntityDef {
        code: "deal",
        label: "Deal",
        label_plural: "Deals",
        icon: "fa-handshake",
        category: Category::CRM,
        fields: DEAL_FIELDS,
        views: &[ViewType::Table, ViewType::Kanban],
        kanban_field: Some("stage"),
    },
    EntityDef {
        code: "task",
        label: "Task",
        label_plural: "Tasks",
        icon: "fa-tasks",
        category: Category::CRM,
        fields: TASK_FIELDS,
        views: &[ViewType::Table, ViewType::Kanban, ViewType::Calendar],
        kanban_field: Some("status"),
    },
    // Real Estate Entities
    EntityDef {
        code: "property",
        label: "Property",
        label_plural: "Properties",
        icon: "fa-home",
        category: Category::RealEstate,
        fields: PROPERTY_FIELDS,
        views: &[ViewType::Table, ViewType::Kanban, ViewType::Map],
        kanban_field: Some("status"),
    },
    EntityDef {
        code: "listing",
        label: "Listing",
        label_plural: "Listings",
        icon: "fa-bullhorn",
        category: Category::RealEstate,
        fields: LISTING_FIELDS,
        views: &[ViewType::Table, ViewType::Kanban],
        kanban_field: Some("status"),
    },
    EntityDef {
        code: "viewing",
        label: "Viewing",
        label_plural: "Viewings",
        icon: "fa-eye",
        category: Category::RealEstate,
        fields: VIEWING_FIELDS,
        views: &[ViewType::Table, ViewType::Calendar],
        kanban_field: Some("status"),
    },
    EntityDef {
        code: "offer",
        label: "Offer",
        label_plural: "Offers",
        icon: "fa-file-invoice-dollar",
        category: Category::RealEstate,
        fields: OFFER_FIELDS,
        views: &[ViewType::Table, ViewType::Kanban],
        kanban_field: Some("status"),
    },
    EntityDef {
        code: "contract",
        label: "Contract",
        label_plural: "Contracts",
        icon: "fa-file-contract",
        category: Category::RealEstate,
        fields: CONTRACT_FIELDS,
        views: &[ViewType::Table, ViewType::Kanban],
        kanban_field: Some("status"),
    },
];

// ============================================
// HELPER FUNCTIONS
// ============================================

/// Get entity by code
pub fn get_entity(code: &str) -> Option<&'static EntityDef> {
    ENTITIES.iter().find(|e| e.code == code)
}

/// Get all entities in a category
pub fn get_entities_by_category(category: Category) -> impl Iterator<Item = &'static EntityDef> {
    ENTITIES.iter().filter(move |e| e.category == category)
}

/// Get field definition from entity
pub fn get_field(entity_code: &str, field_name: &str) -> Option<&'static FieldDef> {
    get_entity(entity_code)?.fields.iter().find(|f| f.name == field_name)
}

/// Get all CRM entities
pub fn crm_entities() -> impl Iterator<Item = &'static EntityDef> {
    get_entities_by_category(Category::CRM)
}

/// Get all Real Estate entities
pub fn real_estate_entities() -> impl Iterator<Item = &'static EntityDef> {
    get_entities_by_category(Category::RealEstate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_count() {
        assert_eq!(ENTITIES.len(), 9);
    }

    #[test]
    fn test_get_entity() {
        let contact = get_entity("contact");
        assert!(contact.is_some());
        assert_eq!(contact.unwrap().label, "Contact");
    }

    #[test]
    fn test_crm_entities() {
        let crm: Vec<_> = crm_entities().collect();
        assert_eq!(crm.len(), 4); // contact, company, deal, task
    }

    #[test]
    fn test_real_estate_entities() {
        let re: Vec<_> = real_estate_entities().collect();
        assert_eq!(re.len(), 5); // property, listing, viewing, offer, contract
    }
}
