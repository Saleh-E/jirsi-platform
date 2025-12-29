//! CRM metadata seeding
//!
//! Seeds the EntityTypes, FieldDefs, and ViewDefs for CRM entities.
//! Updated for Golden Rule: FieldType uses tagged serde format with config.

use core_models::{
    AppDef, EntityType, FieldDef, FieldType,
    ViewDef, ViewColumn,
};
use chrono::Utc;
use uuid::Uuid;

/// Seed CRM metadata for a tenant
pub fn seed_crm_metadata(tenant_id: Uuid) -> CrmMetadata {
    let now = Utc::now();

    // App definition
    let app = AppDef {
        id: "crm".to_string(),
        tenant_id,
        name: "crm".to_string(),
        label: "CRM".to_string(),
        icon: Some("users".to_string()),
        description: Some("Customer Relationship Management".to_string()),
        sort_order: 1,
        is_enabled: true,
        created_at: now,
        updated_at: now,
    };

    // Contact EntityType
    let contact_entity = EntityType::new(tenant_id, "crm", "contact", "Contact")
        .with_activities()
        .with_tasks()
        .with_nav()
        .searchable();

    let contact_entity_id = contact_entity.id;

    // Contact Fields - using new FieldType tagged format
    let contact_fields = vec![
        FieldDef::new(tenant_id, contact_entity_id, "first_name", "First Name", FieldType::Text)
            .required().in_list().searchable().order(1),
        FieldDef::new(tenant_id, contact_entity_id, "last_name", "Last Name", FieldType::Text)
            .required().in_list().searchable().order(2),
        FieldDef::new(tenant_id, contact_entity_id, "email", "Email", FieldType::Email)
            .in_list().searchable().order(3),
        FieldDef::new(tenant_id, contact_entity_id, "phone", "Phone", FieldType::Phone)
            .order(4),
        FieldDef::new(tenant_id, contact_entity_id, "company_id", "Company", 
            FieldType::Link { target_entity: "company".to_string() })
            .in_list().order(5),
        FieldDef::new(tenant_id, contact_entity_id, "job_title", "Job Title", FieldType::Text)
            .order(6),
        lifecycle_stage_field(tenant_id, contact_entity_id),
        FieldDef::new(tenant_id, contact_entity_id, "lead_source", "Lead Source", 
            FieldType::Select { options: vec!["Web".to_string(), "Referral".to_string(), "Event".to_string(), "Cold Call".to_string()] })
            .filterable().order(8),
        FieldDef::new(tenant_id, contact_entity_id, "owner_id", "Owner", 
            FieldType::Link { target_entity: "user".to_string() })
            .filterable().order(9),
        FieldDef::new(tenant_id, contact_entity_id, "tags", "Tags", FieldType::TagList)
            .order(10),
    ];

    // Contact default view
    let contact_view = ViewDef::table(tenant_id, contact_entity_id, "all_contacts", "All Contacts")
        .as_default()
        .as_system()
        .with_columns(vec![
            ViewColumn { field: "first_name".to_string(), width: None, visible: true, sort_order: 1 },
            ViewColumn { field: "last_name".to_string(), width: None, visible: true, sort_order: 2 },
            ViewColumn { field: "email".to_string(), width: None, visible: true, sort_order: 3 },
            ViewColumn { field: "company_id".to_string(), width: None, visible: true, sort_order: 4 },
            ViewColumn { field: "lifecycle_stage".to_string(), width: None, visible: true, sort_order: 5 },
        ]);

    // Company EntityType
    let company_entity = EntityType::new(tenant_id, "crm", "company", "Company")
        .with_activities()
        .with_tasks()
        .with_nav()
        .searchable();

    let company_entity_id = company_entity.id;

    let company_fields = vec![
        FieldDef::new(tenant_id, company_entity_id, "name", "Name", FieldType::Text)
            .required().in_list().searchable().order(1),
        FieldDef::new(tenant_id, company_entity_id, "domain", "Domain", FieldType::Url)
            .in_list().order(2),
        FieldDef::new(tenant_id, company_entity_id, "industry", "Industry", 
            FieldType::Select { options: vec!["Technology".to_string(), "Finance".to_string(), "Healthcare".to_string(), "Retail".to_string(), "Other".to_string()] })
            .filterable().order(3),
        FieldDef::new(tenant_id, company_entity_id, "size", "Company Size", 
            FieldType::Select { options: vec!["1-10".to_string(), "11-50".to_string(), "51-200".to_string(), "201-500".to_string(), "500+".to_string()] })
            .filterable().order(4),
        FieldDef::new(tenant_id, company_entity_id, "phone", "Phone", FieldType::Phone)
            .order(5),
        FieldDef::new(tenant_id, company_entity_id, "website", "Website", FieldType::Url)
            .order(6),
        FieldDef::new(tenant_id, company_entity_id, "address", "Address", FieldType::TextArea)
            .section("location").order(7),
        FieldDef::new(tenant_id, company_entity_id, "city", "City", FieldType::Text)
            .section("location").order(8),
        FieldDef::new(tenant_id, company_entity_id, "country", "Country", FieldType::Text)
            .section("location").order(9),
        FieldDef::new(tenant_id, company_entity_id, "owner_id", "Owner", 
            FieldType::Link { target_entity: "user".to_string() })
            .filterable().order(10),
        FieldDef::new(tenant_id, company_entity_id, "tags", "Tags", FieldType::TagList)
            .order(11),
    ];

    let company_view = ViewDef::table(tenant_id, company_entity_id, "all_companies", "All Companies")
        .as_default()
        .as_system();

    // Deal EntityType
    let deal_entity = EntityType::new(tenant_id, "crm", "deal", "Deal")
        .with_activities()
        .with_tasks()
        .with_pipeline()
        .with_nav()
        .searchable();

    let deal_entity_id = deal_entity.id;

    let deal_fields = vec![
        FieldDef::new(tenant_id, deal_entity_id, "name", "Deal Name", FieldType::Text)
            .required().in_list().searchable().order(1),
        FieldDef::new(tenant_id, deal_entity_id, "amount", "Amount", 
            FieldType::Money { currency_code: Some("USD".to_string()) })
            .in_list().sortable().order(2),
        FieldDef::new(tenant_id, deal_entity_id, "stage", "Stage", 
            FieldType::Select { options: vec!["New".to_string(), "Contacted".to_string(), "Qualified".to_string(), "Proposal".to_string(), "Negotiation".to_string(), "Won".to_string(), "Lost".to_string()] })
            .required().in_list().filterable().order(3),
        FieldDef::new(tenant_id, deal_entity_id, "probability", "Probability (%)", 
            FieldType::Number { decimals: Some(0) })
            .order(4),
        FieldDef::new(tenant_id, deal_entity_id, "expected_close_date", "Expected Close", FieldType::Date)
            .in_list().sortable().order(5),
        FieldDef::new(tenant_id, deal_entity_id, "contact_id", "Contact", 
            FieldType::Link { target_entity: "contact".to_string() })
            .in_list().order(6),
        FieldDef::new(tenant_id, deal_entity_id, "company_id", "Company", 
            FieldType::Link { target_entity: "company".to_string() })
            .order(7),
        FieldDef::new(tenant_id, deal_entity_id, "owner_id", "Owner", 
            FieldType::Link { target_entity: "user".to_string() })
            .filterable().order(8),
        FieldDef::new(tenant_id, deal_entity_id, "lost_reason", "Lost Reason", FieldType::Text)
            .order(9),
        FieldDef::new(tenant_id, deal_entity_id, "tags", "Tags", FieldType::TagList)
            .order(10),
    ];

    let deal_table_view = ViewDef::table(tenant_id, deal_entity_id, "all_deals", "All Deals")
        .as_default()
        .as_system();

    // ViewDef::kanban now requires group_by_field parameter
    let deal_kanban_view = ViewDef::kanban(tenant_id, deal_entity_id, "pipeline", "Pipeline", "stage")
        .as_system();

    // Task EntityType
    let task_entity = EntityType::new(tenant_id, "crm", "task", "Task")
        .with_nav()
        .searchable();

    let task_entity_id = task_entity.id;

    let task_fields = vec![
        FieldDef::new(tenant_id, task_entity_id, "title", "Title", FieldType::Text)
            .required().in_list().searchable().order(1),
        FieldDef::new(tenant_id, task_entity_id, "description", "Description", FieldType::TextArea)
            .order(2),
        FieldDef::new(tenant_id, task_entity_id, "due_date", "Due Date", FieldType::DateTime)
            .in_list().sortable().order(3),
        FieldDef::new(tenant_id, task_entity_id, "priority", "Priority", 
            FieldType::Select { options: vec!["Low".to_string(), "Medium".to_string(), "High".to_string(), "Urgent".to_string()] })
            .in_list().filterable().order(4),
        FieldDef::new(tenant_id, task_entity_id, "status", "Status", 
            FieldType::Select { options: vec!["Open".to_string(), "In Progress".to_string(), "Completed".to_string(), "Cancelled".to_string()] })
            .in_list().filterable().order(5),
        FieldDef::new(tenant_id, task_entity_id, "task_type", "Type", 
            FieldType::Select { options: vec!["Call".to_string(), "Email".to_string(), "Meeting".to_string(), "Follow Up".to_string(), "Other".to_string()] })
            .filterable().order(6),
        FieldDef::new(tenant_id, task_entity_id, "assignee_id", "Assignee", 
            FieldType::Link { target_entity: "user".to_string() })
            .in_list().filterable().order(7),
    ];

    let task_view = ViewDef::table(tenant_id, task_entity_id, "all_tasks", "All Tasks")
        .as_default()
        .as_system();

    CrmMetadata {
        app,
        entities: vec![contact_entity, company_entity, deal_entity, task_entity],
        fields: vec![contact_fields, company_fields, deal_fields, task_fields]
            .into_iter()
            .flatten()
            .collect(),
        views: vec![contact_view, company_view, deal_table_view, deal_kanban_view, task_view],
    }
}

fn lifecycle_stage_field(tenant_id: Uuid, entity_type_id: Uuid) -> FieldDef {
    let mut field = FieldDef::new(
        tenant_id,
        entity_type_id,
        "lifecycle_stage",
        "Lifecycle Stage",
        FieldType::Select { 
            options: vec![
                "Subscriber".to_string(), 
                "Lead".to_string(), 
                "MQL".to_string(), 
                "SQL".to_string(), 
                "Opportunity".to_string(), 
                "Customer".to_string(), 
                "Evangelist".to_string()
            ] 
        },
    )
    .in_list()
    .filterable()
    .order(7);

    // Additional options with colors/icons as JSON array
    field.options = Some(serde_json::json!([
        {"value": "subscriber", "label": "Subscriber", "color": "#6b7280"},
        {"value": "lead", "label": "Lead", "color": "#3b82f6"},
        {"value": "mql", "label": "Marketing Qualified", "color": "#8b5cf6"},
        {"value": "sql", "label": "Sales Qualified", "color": "#f59e0b"},
        {"value": "opportunity", "label": "Opportunity", "color": "#10b981"},
        {"value": "customer", "label": "Customer", "color": "#059669"},
        {"value": "evangelist", "label": "Evangelist", "color": "#0ea5e9"}
    ]));

    field
}

/// CRM Metadata bundle
pub struct CrmMetadata {
    pub app: AppDef,
    pub entities: Vec<EntityType>,
    pub fields: Vec<FieldDef>,
    pub views: Vec<ViewDef>,
}
