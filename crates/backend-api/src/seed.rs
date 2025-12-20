//! Database seeding for development and new tenant onboarding

use chrono::Utc;
use core_auth::password::hash_password;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

// ============================================================================
// PUBLIC API: New Tenant Seeding (Transactional)
// ============================================================================

/// Seeds all required data for a new tenant within a transaction.
/// If any step fails, the entire operation is rolled back.
/// 
/// This is the main entry point for tenant onboarding.
pub async fn seed_new_tenant(tenant_id: Uuid, pool: &PgPool) -> Result<(), SeedError> {
    let mut tx = pool.begin().await?;
    
    // Seed in order of dependencies
    seed_entity_metadata_tx(&mut tx, tenant_id).await?;
    seed_associations_tx(&mut tx, tenant_id).await?;
    seed_views_tx(&mut tx, tenant_id).await?;
    seed_standard_workflows_tx(&mut tx, tenant_id).await?;
    seed_property_entity_tx(&mut tx, tenant_id).await?;
    
    tx.commit().await?;
    Ok(())
}

/// Error type for seeding operations
#[derive(Debug)]
pub enum SeedError {
    Database(sqlx::Error),
    PasswordHash(String),
}

impl From<sqlx::Error> for SeedError {
    fn from(e: sqlx::Error) -> Self {
        SeedError::Database(e)
    }
}

impl std::fmt::Display for SeedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeedError::Database(e) => write!(f, "Database error: {}", e),
            SeedError::PasswordHash(e) => write!(f, "Password hash error: {}", e),
        }
    }
}

impl std::error::Error for SeedError {}

// ============================================================================
// TRANSACTIONAL SEEDERS (for new tenant onboarding)
// ============================================================================

/// Seed entity types and field definitions within a transaction
async fn seed_entity_metadata_tx(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();

    // Create Contact entity type
    let contact_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon, flags, created_at, updated_at)
        VALUES ($1, $2, 'crm', 'contact', 'Contact', 'Contacts', 'user', $3, $4, $5)
        "#
    )
    .bind(contact_id)
    .bind(tenant_id)
    .bind(serde_json::json!({"has_activities": true, "has_tasks": true, "is_searchable": true, "show_in_nav": true}))
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    // Contact fields
    // Contact fields
    seed_field_tx(tx, tenant_id, contact_id, "first_name", "First Name", "text", true, true, 1, None).await?;
    seed_field_tx(tx, tenant_id, contact_id, "last_name", "Last Name", "text", true, true, 2, None).await?;
    seed_field_tx(tx, tenant_id, contact_id, "email", "Email", "email", false, true, 3, None).await?;
    seed_field_tx(tx, tenant_id, contact_id, "phone", "Phone", "phone", false, true, 4, None).await?;
    
    let lifecycle_options = serde_json::json!([
        {"value": "subscriber", "label": "Subscriber"},
        {"value": "lead", "label": "Lead"},
        {"value": "marketing_qualified", "label": "Marketing Qualified"},
        {"value": "opportunity", "label": "Opportunity"},
        {"value": "customer", "label": "Customer"},
        {"value": "evangelist", "label": "Evangelist"},
        {"value": "other", "label": "Other"}
    ]);
    seed_field_tx(tx, tenant_id, contact_id, "lifecycle_stage", "Lifecycle Stage", "select", false, true, 5, Some(lifecycle_options)).await?;

    // Create Company entity type
    let company_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon, flags, created_at, updated_at)
        VALUES ($1, $2, 'crm', 'company', 'Company', 'Companies', 'building', $3, $4, $5)
        "#
    )
    .bind(company_id)
    .bind(tenant_id)
    .bind(serde_json::json!({"has_activities": true, "has_tasks": true, "is_searchable": true, "show_in_nav": true}))
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    // Company fields
    // Company fields
    seed_field_tx(tx, tenant_id, company_id, "name", "Company Name", "text", true, true, 1, None).await?;
    seed_field_tx(tx, tenant_id, company_id, "domain", "Domain", "url", false, true, 2, None).await?;
    
    let industry_options = serde_json::json!([
        {"value": "tech", "label": "Technology"},
        {"value": "finance", "label": "Finance"},
        {"value": "health", "label": "Healthcare"},
        {"value": "retail", "label": "Retail"},
        {"value": "manufacturing", "label": "Manufacturing"},
        {"value": "other", "label": "Other"}
    ]);
    seed_field_tx(tx, tenant_id, company_id, "industry", "Industry", "select", false, true, 3, Some(industry_options)).await?;
    
    seed_field_tx(tx, tenant_id, company_id, "phone", "Phone", "phone", false, true, 4, None).await?;

    // Create Deal entity type
    let deal_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon, flags, created_at, updated_at)
        VALUES ($1, $2, 'crm', 'deal', 'Deal', 'Deals', 'dollar-sign', $3, $4, $5)
        "#
    )
    .bind(deal_id)
    .bind(tenant_id)
    .bind(serde_json::json!({"has_pipeline": true, "has_activities": true, "is_searchable": true, "show_in_nav": true}))
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    // Deal fields
    // Deal fields
    seed_field_tx(tx, tenant_id, deal_id, "name", "Deal Name", "text", true, true, 1, None).await?;
    seed_field_tx(tx, tenant_id, deal_id, "amount", "Amount", "money", false, true, 2, None).await?;
    
    let stage_options = serde_json::json!([
        {"value": "appointment_scheduled", "label": "Appointment Scheduled"},
        {"value": "qualified_to_buy", "label": "Qualified To Buy"},
        {"value": "presentation_scheduled", "label": "Presentation Scheduled"},
        {"value": "decision_maker_bought_in", "label": "Decision Maker Bought-In"},
        {"value": "contract_sent", "label": "Contract Sent"},
        {"value": "closed_won", "label": "Closed Won"},
        {"value": "closed_lost", "label": "Closed Lost"}
    ]);
    seed_field_tx(tx, tenant_id, deal_id, "stage", "Stage", "select", true, true, 3, Some(stage_options)).await?;
    
    seed_field_tx(tx, tenant_id, deal_id, "expected_close_date", "Expected Close", "date", false, true, 4, None).await?;

    Ok(())
}

/// Seed Property entity type for Real Estate
async fn seed_property_entity_tx(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();

    // Create Property entity type
    let property_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon, flags, created_at, updated_at)
        VALUES ($1, $2, 'real_estate', 'property', 'Property', 'Properties', 'home', $3, $4, $5)
        "#
    )
    .bind(property_id)
    .bind(tenant_id)
    .bind(serde_json::json!({"has_activities": true, "has_tasks": true, "is_searchable": true, "show_in_nav": true, "has_map": true}))
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    // Property fields
    // Property fields
    seed_field_tx(tx, tenant_id, property_id, "title", "Title", "text", true, true, 1, None).await?;
    seed_field_tx(tx, tenant_id, property_id, "price", "Price", "money", true, true, 2, None).await?;
    
    let status_options = serde_json::json!([
        {"value": "active", "label": "Active"},
        {"value": "under_offer", "label": "Under Offer"},
        {"value": "sold", "label": "Sold"},
        {"value": "rented", "label": "Rented"},
        {"value": "withdrawn", "label": "Withdrawn"}
    ]);
    seed_field_tx(tx, tenant_id, property_id, "status", "Status", "select", true, true, 3, Some(status_options)).await?;
    
    let type_options = serde_json::json!([
        {"value": "apartment", "label": "Apartment"},
        {"value": "house", "label": "House"},
        {"value": "commercial", "label": "Commercial"},
        {"value": "land", "label": "Land"}
    ]);
    seed_field_tx(tx, tenant_id, property_id, "property_type", "Property Type", "select", false, true, 4, Some(type_options)).await?;
    
    seed_field_tx(tx, tenant_id, property_id, "bedrooms", "Bedrooms", "number", false, true, 5, None).await?;
    seed_field_tx(tx, tenant_id, property_id, "bathrooms", "Bathrooms", "number", false, true, 6, None).await?;
    seed_field_tx(tx, tenant_id, property_id, "area_sqm", "Area (sqm)", "number", false, true, 7, None).await?;
    seed_field_tx(tx, tenant_id, property_id, "address", "Address", "text", false, true, 8, None).await?;
    seed_field_tx(tx, tenant_id, property_id, "city", "City", "text", false, true, 9, None).await?;
    seed_field_tx(tx, tenant_id, property_id, "description", "Description", "textarea", false, false, 10, None).await?;

    Ok(())
}

/// Helper to seed a single field definition within a transaction
async fn seed_field_tx(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    entity_type_id: Uuid,
    name: &str,
    label: &str,
    field_type: &str,
    is_required: bool,
    show_in_list: bool,
    sort_order: i32,
    options: Option<serde_json::Value>,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    let id = Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order, options, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#
    )
    .bind(id)
    .bind(tenant_id)
    .bind(entity_type_id)
    .bind(name)
    .bind(label)
    .bind(field_type)
    .bind(is_required)
    .bind(show_in_list)
    .bind(sort_order)
    .bind(options)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Seed association definitions within a transaction
async fn seed_associations_tx(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();

    // Contact ↔ Company (many contacts can work at one company)
    sqlx::query(
        r#"
        INSERT INTO association_defs (id, tenant_id, source_entity, target_entity, name, label_source, label_target, cardinality, source_role, target_role, allow_primary, cascade_delete, created_at, updated_at)
        VALUES ($1, $2, 'contact', 'company', 'contact_company', 'Company', 'Contacts', 'many_to_one', 'employee', 'employer', true, false, $3, $4)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(tenant_id)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    // Deal ↔ Contact (deals can be linked to multiple contacts)
    sqlx::query(
        r#"
        INSERT INTO association_defs (id, tenant_id, source_entity, target_entity, name, label_source, label_target, cardinality, source_role, target_role, allow_primary, cascade_delete, created_at, updated_at)
        VALUES ($1, $2, 'deal', 'contact', 'deal_contact', 'Contacts', 'Deals', 'many_to_many', NULL, NULL, true, false, $3, $4)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(tenant_id)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    // Deal ↔ Company
    sqlx::query(
        r#"
        INSERT INTO association_defs (id, tenant_id, source_entity, target_entity, name, label_source, label_target, cardinality, source_role, target_role, allow_primary, cascade_delete, created_at, updated_at)
        VALUES ($1, $2, 'deal', 'company', 'deal_company', 'Company', 'Deals', 'many_to_one', NULL, NULL, true, false, $3, $4)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(tenant_id)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    // Contact ↔ Property (buyer interest)
    sqlx::query(
        r#"
        INSERT INTO association_defs (id, tenant_id, source_entity, target_entity, name, label_source, label_target, cardinality, source_role, target_role, allow_primary, cascade_delete, created_at, updated_at)
        VALUES ($1, $2, 'contact', 'property', 'contact_property', 'Properties', 'Interested Contacts', 'many_to_many', 'buyer', NULL, false, false, $3, $4)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(tenant_id)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Seed view definitions within a transaction
async fn seed_views_tx(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    use sqlx::Row;
    let now = Utc::now();

    // Get entity type IDs
    let contact_id: Option<Uuid> = sqlx::query("SELECT id FROM entity_types WHERE tenant_id = $1 AND name = 'contact'")
        .bind(tenant_id)
        .fetch_optional(&mut **tx)
        .await?
        .map(|row| row.try_get("id").unwrap_or_default());
    
    let deal_id: Option<Uuid> = sqlx::query("SELECT id FROM entity_types WHERE tenant_id = $1 AND name = 'deal'")
        .bind(tenant_id)
        .fetch_optional(&mut **tx)
        .await?
        .map(|row| row.try_get("id").unwrap_or_default());

    let company_id: Option<Uuid> = sqlx::query("SELECT id FROM entity_types WHERE tenant_id = $1 AND name = 'company'")
        .bind(tenant_id)
        .fetch_optional(&mut **tx)
        .await?
        .map(|row| row.try_get("id").unwrap_or_default());

    let property_id: Option<Uuid> = sqlx::query("SELECT id FROM entity_types WHERE tenant_id = $1 AND name = 'property'")
        .bind(tenant_id)
        .fetch_optional(&mut **tx)
        .await?
        .map(|row| row.try_get("id").unwrap_or_default());

    // Contact - Default Table View
    if let Some(entity_id) = contact_id {
        let columns = serde_json::json!([
            {"field": "first_name", "width": "150", "visible": true, "sort_order": 1},
            {"field": "last_name", "width": "150", "visible": true, "sort_order": 2},
            {"field": "email", "width": "200", "visible": true, "sort_order": 3},
            {"field": "phone", "width": "150", "visible": true, "sort_order": 4}
        ]);

        sqlx::query(
            r#"
            INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings, created_at, updated_at)
            VALUES ($1, $2, $3, 'default_table', 'All Contacts', 'table', true, true, $4, '[]', '[]', '{}', $5, $6)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(entity_id)
        .bind(columns)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await?;
    }

    // Company - Default Table View
    if let Some(entity_id) = company_id {
        let columns = serde_json::json!([
            {"field": "name", "width": "200", "visible": true, "sort_order": 1},
            {"field": "domain", "width": "150", "visible": true, "sort_order": 2},
            {"field": "industry", "width": "150", "visible": true, "sort_order": 3}
        ]);

        sqlx::query(
            r#"
            INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings, created_at, updated_at)
            VALUES ($1, $2, $3, 'default_table', 'All Companies', 'table', true, true, $4, '[]', '[]', '{}', $5, $6)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(entity_id)
        .bind(columns)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await?;
    }

    // Deal - Kanban View (Pipeline)
    if let Some(entity_id) = deal_id {
        let kanban_settings = serde_json::json!({
            "group_by_field": "stage",
            "title_field": "name",
            "description_field": null,
            "card_fields": ["amount", "expected_close_date"],
            "allow_drag": true
        });

        // Kanban view
        sqlx::query(
            r#"
            INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings, created_at, updated_at)
            VALUES ($1, $2, $3, 'pipeline', 'Pipeline', 'kanban', true, true, '[]', '[]', '[]', $4, $5, $6)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(entity_id)
        .bind(kanban_settings)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await?;

        // Also add a table view for deals
        let table_columns = serde_json::json!([
            {"field": "name", "width": "200", "visible": true, "sort_order": 1},
            {"field": "amount", "width": "120", "visible": true, "sort_order": 2},
            {"field": "stage", "width": "120", "visible": true, "sort_order": 3},
            {"field": "expected_close_date", "width": "150", "visible": true, "sort_order": 4}
        ]);

        sqlx::query(
            r#"
            INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings, created_at, updated_at)
            VALUES ($1, $2, $3, 'deals_table', 'All Deals', 'table', false, true, $4, '[]', '[]', '{}', $5, $6)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(entity_id)
        .bind(table_columns)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await?;
    }

    // Property - Table and Map Views
    if let Some(entity_id) = property_id {
        let columns = serde_json::json!([
            {"field": "title", "width": "200", "visible": true, "sort_order": 1},
            {"field": "price", "width": "120", "visible": true, "sort_order": 2},
            {"field": "status", "width": "100", "visible": true, "sort_order": 3},
            {"field": "property_type", "width": "120", "visible": true, "sort_order": 4},
            {"field": "city", "width": "120", "visible": true, "sort_order": 5}
        ]);

        sqlx::query(
            r#"
            INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings, created_at, updated_at)
            VALUES ($1, $2, $3, 'default_table', 'All Properties', 'table', true, true, $4, '[]', '[]', '{}', $5, $6)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(entity_id)
        .bind(columns)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await?;

        // Map view for properties
        let map_settings = serde_json::json!({
            "lat_field": "latitude",
            "lng_field": "longitude",
            "title_field": "title",
            "popup_fields": ["price", "status", "property_type"]
        });

        sqlx::query(
            r#"
            INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings, created_at, updated_at)
            VALUES ($1, $2, $3, 'map_view', 'Map View', 'map', false, true, '[]', '[]', '[]', $4, $5, $6)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(entity_id)
        .bind(map_settings)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

/// Seed standard workflows within a transaction
async fn seed_standard_workflows_tx(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();

    // WORKFLOW 1: New Lead Intake (CRM)
    let lead_intake_actions = serde_json::json!([
        {
            "type": "create_task",
            "config": {
                "title": "Follow up with new lead",
                "due_in_hours": 24,
                "assign_to": "record_owner"
            }
        },
        {
            "type": "send_notification",
            "config": {
                "channel": "in_app",
                "message": "New lead created: {{record.first_name}} {{record.last_name}}"
            }
        }
    ]);

    sqlx::query(
        r#"
        INSERT INTO workflow_defs (id, tenant_id, name, description, is_active, is_system, trigger_type, trigger_entity, conditions, actions, created_at, updated_at)
        VALUES ($1, $2, 'New Lead Intake', 'Automatically creates follow-up task when a new contact is created', true, true, 'record_created', 'contact', '{}', $3, $4, $5)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(tenant_id)
    .bind(lead_intake_actions)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    // WORKFLOW 2: Deal Won (CRM)
    let deal_won_conditions = serde_json::json!({
        "field": "stage",
        "operator": "equals",
        "value": "closed_won"
    });

    let deal_won_actions = serde_json::json!([
        {
            "type": "update_record",
            "config": {
                "entity": "contact",
                "field": "lifecycle_stage",
                "value": "customer"
            }
        },
        {
            "type": "send_notification",
            "config": {
                "channel": "in_app",
                "message": "🎉 Deal won: {{record.name}} for {{record.amount}}"
            }
        }
    ]);

    sqlx::query(
        r#"
        INSERT INTO workflow_defs (id, tenant_id, name, description, is_active, is_system, trigger_type, trigger_entity, conditions, actions, created_at, updated_at)
        VALUES ($1, $2, 'Deal Won', 'Updates contact lifecycle and sends celebration notification', true, true, 'field_changed', 'deal', $3, $4, $5, $6)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(tenant_id)
    .bind(deal_won_conditions)
    .bind(deal_won_actions)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    // WORKFLOW 3: Offer Accepted (Real Estate)
    let offer_conditions = serde_json::json!({
        "field": "status",
        "operator": "equals",
        "value": "offer_accepted"
    });

    let offer_actions = serde_json::json!([
        {
            "type": "create_task",
            "config": {
                "title": "Prepare sales contract",
                "due_in_hours": 48,
                "assign_to": "record_owner"
            }
        },
        {
            "type": "send_notification",
            "config": {
                "channel": "in_app",
                "message": "Offer accepted on {{record.title}}! Next: Prepare contract."
            }
        }
    ]);

    sqlx::query(
        r#"
        INSERT INTO workflow_defs (id, tenant_id, name, description, is_active, is_system, trigger_type, trigger_entity, conditions, actions, created_at, updated_at)
        VALUES ($1, $2, 'Offer Accepted', 'Creates contract preparation task when offer is accepted', true, true, 'field_changed', 'property', $3, $4, $5, $6)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(tenant_id)
    .bind(offer_conditions)
    .bind(offer_actions)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

// ============================================================================
// DEVELOPMENT SEEDING (existing functionality)
// ============================================================================

/// Seed the database with sample data for development
pub async fn seed_database(pool: &PgPool) -> Result<SeedResult, sqlx::Error> {
    use sqlx::Row;
    let now = Utc::now();
    
    // Check if demo tenant already exists
    let existing_tenant: Option<Uuid> = sqlx::query(
        "SELECT id FROM tenants WHERE subdomain = 'demo'"
    )
    .fetch_optional(pool)
    .await?
    .map(|row| row.try_get("id").unwrap_or_default());

    let tenant_id = if let Some(id) = existing_tenant {
        id
    } else {
        // Create demo tenant
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO tenants (id, name, subdomain, custom_domain, plan, status, settings, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(id)
        .bind("Demo Company")
        .bind("demo")
        .bind(None::<String>)
        .bind("professional")
        .bind("active")
        .bind(serde_json::json!({}))
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;
        id
    };

    // Check if admin user exists
    let existing_admin: Option<String> = sqlx::query(
        "SELECT email FROM users WHERE tenant_id = $1 AND email = 'admin@demo.com'"
    )
    .bind(tenant_id)
    .fetch_optional(pool)
    .await?
    .map(|row| row.try_get("email").unwrap_or_default());

    if existing_admin.is_none() {
        // Create admin user (password: "Admin123!")
        let admin_password_hash = hash_password("Admin123!")
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let admin_id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO users (id, tenant_id, email, name, password_hash, role, status, avatar_url, preferences, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(admin_id)
        .bind(tenant_id)
        .bind("admin@demo.com")
        .bind("Admin User")
        .bind(&admin_password_hash)
        .bind("admin")
        .bind("active")
        .bind(None::<String>)
        .bind(serde_json::json!({}))
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;
    }

    // Seed entity metadata (entity_types, field_defs)
    seed_entity_metadata(pool, tenant_id).await?;

    // Seed association definitions
    seed_associations(pool, tenant_id).await?;

    // Seed view definitions
    seed_views(pool, tenant_id).await?;

    // Create sample contacts
    seed_sample_contacts(pool, tenant_id).await?;

    Ok(SeedResult {
        tenant_id,
        tenant_subdomain: "demo".to_string(),
        admin_email: "admin@demo.com".to_string(),
        admin_password: "Admin123!".to_string(),
    })
}

/// Seed entity types and field definitions (non-transactional, for dev seeding)
async fn seed_entity_metadata(pool: &PgPool, tenant_id: Uuid) -> Result<(), sqlx::Error> {
    use sqlx::Row;
    let now = Utc::now();

    // Check if already seeded
    let existing: i64 = sqlx::query("SELECT COUNT(*) as count FROM entity_types WHERE tenant_id = $1")
        .bind(tenant_id)
        .fetch_one(pool)
        .await?
        .try_get("count")
        .unwrap_or(0);

    if existing > 0 {
        return Ok(());
    }

    // Create Contact entity type
    let contact_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon, flags, created_at, updated_at)
        VALUES ($1, $2, 'crm', 'contact', 'Contact', 'Contacts', 'user', $3, $4, $5)
        "#
    )
    .bind(contact_id)
    .bind(tenant_id)
    .bind(serde_json::json!({"has_activities": true, "has_tasks": true, "is_searchable": true, "show_in_nav": true}))
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    // Contact fields
    seed_field(pool, tenant_id, contact_id, "first_name", "First Name", "text", true, true, 1, None).await?;
    seed_field(pool, tenant_id, contact_id, "last_name", "Last Name", "text", true, true, 2, None).await?;
    seed_field(pool, tenant_id, contact_id, "email", "Email", "email", false, true, 3, None).await?;
    seed_field(pool, tenant_id, contact_id, "phone", "Phone", "phone", false, true, 4, None).await?;
    
    let lifecycle_options = serde_json::json!([
        {"value": "subscriber", "label": "Subscriber"},
        {"value": "lead", "label": "Lead"},
        {"value": "marketing_qualified", "label": "Marketing Qualified"},
        {"value": "opportunity", "label": "Opportunity"},
        {"value": "customer", "label": "Customer"},
        {"value": "evangelist", "label": "Evangelist"},
        {"value": "other", "label": "Other"}
    ]);
    seed_field(pool, tenant_id, contact_id, "lifecycle_stage", "Lifecycle Stage", "select", false, true, 5, Some(lifecycle_options)).await?;

    // Create Company entity type
    let company_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon, flags, created_at, updated_at)
        VALUES ($1, $2, 'crm', 'company', 'Company', 'Companies', 'building', $3, $4, $5)
        "#
    )
    .bind(company_id)
    .bind(tenant_id)
    .bind(serde_json::json!({"has_activities": true, "has_tasks": true, "is_searchable": true, "show_in_nav": true}))
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    // Company fields
    seed_field(pool, tenant_id, company_id, "name", "Company Name", "text", true, true, 1, None).await?;
    seed_field(pool, tenant_id, company_id, "domain", "Domain", "url", false, true, 2, None).await?;
    
    let industry_options = serde_json::json!([
        {"value": "tech", "label": "Technology"},
        {"value": "finance", "label": "Finance"},
        {"value": "health", "label": "Healthcare"},
        {"value": "retail", "label": "Retail"},
        {"value": "manufacturing", "label": "Manufacturing"},
        {"value": "other", "label": "Other"}
    ]);
    seed_field(pool, tenant_id, company_id, "industry", "Industry", "select", false, true, 3, Some(industry_options)).await?;
    
    seed_field(pool, tenant_id, company_id, "phone", "Phone", "phone", false, true, 4, None).await?;

    // Create Deal entity type
    let deal_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon, flags, created_at, updated_at)
        VALUES ($1, $2, 'crm', 'deal', 'Deal', 'Deals', 'dollar-sign', $3, $4, $5)
        "#
    )
    .bind(deal_id)
    .bind(tenant_id)
    .bind(serde_json::json!({"has_pipeline": true, "has_activities": true, "is_searchable": true, "show_in_nav": true}))
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    // Deal fields
    seed_field(pool, tenant_id, deal_id, "name", "Deal Name", "text", true, true, 1, None).await?;
    seed_field(pool, tenant_id, deal_id, "amount", "Amount", "money", false, true, 2, None).await?;
    
    let stage_options = serde_json::json!([
        {"value": "appointment_scheduled", "label": "Appointment Scheduled"},
        {"value": "qualified_to_buy", "label": "Qualified To Buy"},
        {"value": "presentation_scheduled", "label": "Presentation Scheduled"},
        {"value": "decision_maker_bought_in", "label": "Decision Maker Bought-In"},
        {"value": "contract_sent", "label": "Contract Sent"},
        {"value": "closed_won", "label": "Closed Won"},
        {"value": "closed_lost", "label": "Closed Lost"}
    ]);
    seed_field(pool, tenant_id, deal_id, "stage", "Stage", "select", true, true, 3, Some(stage_options)).await?;
    
    seed_field(pool, tenant_id, deal_id, "expected_close_date", "Expected Close", "date", false, true, 4, None).await?;

    Ok(())
}

// Helper to seed a single field definition (UPDATED signature)
async fn seed_field(
    pool: &PgPool,
    tenant_id: Uuid,
    entity_type_id: Uuid,
    name: &str,
    label: &str,
    field_type: &str,
    is_required: bool,
    show_in_list: bool,
    sort_order: i32,
    options: Option<serde_json::Value>,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    let id = Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order, options, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#
    )
    .bind(id)
    .bind(tenant_id)
    .bind(entity_type_id)
    .bind(name)
    .bind(label)
    .bind(field_type)
    .bind(is_required)
    .bind(show_in_list)
    .bind(sort_order)
    .bind(options)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}


/// Seed association definitions
async fn seed_associations(pool: &PgPool, tenant_id: Uuid) -> Result<(), sqlx::Error> {
    use sqlx::Row;
    let now = Utc::now();

    // Check if already seeded
    let existing: i64 = sqlx::query("SELECT COUNT(*) as count FROM association_defs WHERE tenant_id = $1")
        .bind(tenant_id)
        .fetch_one(pool)
        .await?
        .try_get("count")
        .unwrap_or(0);

    if existing > 0 {
        return Ok(());
    }

    // Contact ↔ Company (many contacts can work at one company)
    sqlx::query(
        r#"
        INSERT INTO association_defs (id, tenant_id, source_entity, target_entity, name, label_source, label_target, cardinality, source_role, target_role, allow_primary, cascade_delete, created_at, updated_at)
        VALUES ($1, $2, 'contact', 'company', 'contact_company', 'Company', 'Contacts', 'many_to_one', 'employee', 'employer', true, false, $3, $4)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(tenant_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    // Deal ↔ Contact (deals can be linked to multiple contacts)
    sqlx::query(
        r#"
        INSERT INTO association_defs (id, tenant_id, source_entity, target_entity, name, label_source, label_target, cardinality, source_role, target_role, allow_primary, cascade_delete, created_at, updated_at)
        VALUES ($1, $2, 'deal', 'contact', 'deal_contact', 'Contacts', 'Deals', 'many_to_many', NULL, NULL, true, false, $3, $4)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(tenant_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    // Deal ↔ Company
    sqlx::query(
        r#"
        INSERT INTO association_defs (id, tenant_id, source_entity, target_entity, name, label_source, label_target, cardinality, source_role, target_role, allow_primary, cascade_delete, created_at, updated_at)
        VALUES ($1, $2, 'deal', 'company', 'deal_company', 'Company', 'Deals', 'many_to_one', NULL, NULL, true, false, $3, $4)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(tenant_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

/// Seed view definitions
async fn seed_views(pool: &PgPool, tenant_id: Uuid) -> Result<(), sqlx::Error> {
    use sqlx::Row;
    let now = Utc::now();

    // Check if already seeded
    let existing: i64 = sqlx::query("SELECT COUNT(*) as count FROM view_defs WHERE tenant_id = $1")
        .bind(tenant_id)
        .fetch_one(pool)
        .await?
        .try_get("count")
        .unwrap_or(0);

    if existing > 0 {
        return Ok(());
    }

    // Get entity type IDs
    let contact_row = sqlx::query("SELECT id FROM entity_types WHERE tenant_id = $1 AND name = 'contact'")
        .bind(tenant_id)
        .fetch_optional(pool)
        .await?;
    
    let deal_row = sqlx::query("SELECT id FROM entity_types WHERE tenant_id = $1 AND name = 'deal'")
        .bind(tenant_id)
        .fetch_optional(pool)
        .await?;

    let company_row = sqlx::query("SELECT id FROM entity_types WHERE tenant_id = $1 AND name = 'company'")
        .bind(tenant_id)
        .fetch_optional(pool)
        .await?;

    // Contact - Default Table View
    if let Some(row) = contact_row {
        let entity_id: Uuid = row.try_get("id").unwrap_or_default();
        let columns = serde_json::json!([
            {"field": "first_name", "width": "150", "visible": true, "sort_order": 1},
            {"field": "last_name", "width": "150", "visible": true, "sort_order": 2},
            {"field": "email", "width": "200", "visible": true, "sort_order": 3},
            {"field": "phone", "width": "150", "visible": true, "sort_order": 4}
        ]);

        sqlx::query(
            r#"
            INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings, created_at, updated_at)
            VALUES ($1, $2, $3, 'default_table', 'All Contacts', 'table', true, true, $4, '[]', '[]', '{}', $5, $6)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(entity_id)
        .bind(columns)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;
    }

    // Company - Default Table View
    if let Some(row) = company_row {
        let entity_id: Uuid = row.try_get("id").unwrap_or_default();
        let columns = serde_json::json!([
            {"field": "name", "width": "200", "visible": true, "sort_order": 1},
            {"field": "domain", "width": "150", "visible": true, "sort_order": 2},
            {"field": "industry", "width": "150", "visible": true, "sort_order": 3}
        ]);

        sqlx::query(
            r#"
            INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings, created_at, updated_at)
            VALUES ($1, $2, $3, 'default_table', 'All Companies', 'table', true, true, $4, '[]', '[]', '{}', $5, $6)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(entity_id)
        .bind(columns)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;
    }

    // Deal - Kanban View (Pipeline)
    if let Some(row) = deal_row {
        let entity_id: Uuid = row.try_get("id").unwrap_or_default();
        let kanban_settings = serde_json::json!({
            "group_by_field": "stage",
            "title_field": "name",
            "description_field": null,
            "card_fields": ["amount", "expected_close_date"],
            "allow_drag": true
        });

        // Kanban view
        sqlx::query(
            r#"
            INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings, created_at, updated_at)
            VALUES ($1, $2, $3, 'pipeline', 'Pipeline', 'kanban', true, true, '[]', '[]', '[]', $4, $5, $6)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(entity_id)
        .bind(kanban_settings)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // Also add a table view for deals
        let table_columns = serde_json::json!([
            {"field": "name", "width": "200", "visible": true, "sort_order": 1},
            {"field": "amount", "width": "120", "visible": true, "sort_order": 2},
            {"field": "stage", "width": "120", "visible": true, "sort_order": 3},
            {"field": "expected_close_date", "width": "150", "visible": true, "sort_order": 4}
        ]);

        sqlx::query(
            r#"
            INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings, created_at, updated_at)
            VALUES ($1, $2, $3, 'deals_table', 'All Deals', 'table', false, true, $4, '[]', '[]', '{}', $5, $6)
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(entity_id)
        .bind(table_columns)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_sample_contacts(pool: &PgPool, tenant_id: Uuid) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    
    let contacts = vec![
        ("John", "Smith", "john.smith@example.com", "+1-555-0101"),
        ("Jane", "Doe", "jane.doe@example.com", "+1-555-0102"),
        ("Bob", "Johnson", "bob.johnson@example.com", "+1-555-0103"),
        ("Alice", "Williams", "alice.williams@example.com", "+1-555-0104"),
        ("Charlie", "Brown", "charlie.brown@example.com", "+1-555-0105"),
    ];

    for (first_name, last_name, email, phone) in contacts {
        // Skip if contact with this email already exists
        let existing: Option<Uuid> = sqlx::query(
            "SELECT id FROM contacts WHERE tenant_id = $1 AND email = $2"
        )
        .bind(tenant_id)
        .bind(email)
        .fetch_optional(pool)
        .await?
        .map(|row| {
            use sqlx::Row;
            row.try_get("id").unwrap_or_default()
        });

        if existing.is_none() {
            let id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO contacts (id, tenant_id, first_name, last_name, email, phone, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(id)
            .bind(tenant_id)
            .bind(first_name)
            .bind(last_name)
            .bind(email)
            .bind(phone)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}


/// Result of seeding operation
#[derive(Debug, Clone)]
pub struct SeedResult {
    pub tenant_id: Uuid,
    pub tenant_subdomain: String,
    pub admin_email: String,
    pub admin_password: String,
}
