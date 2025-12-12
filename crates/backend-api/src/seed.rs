//! Database seeding for development

use chrono::Utc;
use core_auth::password::hash_password;
use sqlx::PgPool;
use uuid::Uuid;

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

/// Seed entity types and field definitions
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
    seed_field(pool, tenant_id, contact_id, "first_name", "First Name", "text", true, true, 1).await?;
    seed_field(pool, tenant_id, contact_id, "last_name", "Last Name", "text", true, true, 2).await?;
    seed_field(pool, tenant_id, contact_id, "email", "Email", "email", false, true, 3).await?;
    seed_field(pool, tenant_id, contact_id, "phone", "Phone", "phone", false, true, 4).await?;
    seed_field(pool, tenant_id, contact_id, "lifecycle_stage", "Lifecycle Stage", "select", false, true, 5).await?;

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
    seed_field(pool, tenant_id, company_id, "name", "Company Name", "text", true, true, 1).await?;
    seed_field(pool, tenant_id, company_id, "domain", "Domain", "url", false, true, 2).await?;
    seed_field(pool, tenant_id, company_id, "industry", "Industry", "select", false, true, 3).await?;
    seed_field(pool, tenant_id, company_id, "phone", "Phone", "phone", false, true, 4).await?;

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
    seed_field(pool, tenant_id, deal_id, "name", "Deal Name", "text", true, true, 1).await?;
    seed_field(pool, tenant_id, deal_id, "amount", "Amount", "money", false, true, 2).await?;
    seed_field(pool, tenant_id, deal_id, "stage", "Stage", "select", true, true, 3).await?;
    seed_field(pool, tenant_id, deal_id, "expected_close_date", "Expected Close", "date", false, true, 4).await?;

    Ok(())
}

/// Helper to seed a single field definition
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
) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    let id = Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, sort_order, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
