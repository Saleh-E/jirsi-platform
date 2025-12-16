//! Metadata repository - database access layer

use core_models::{EntityType, FieldDef, ViewDef, AppDef, EntityFlags, FieldType, ViewType};
use sqlx::PgPool;
use uuid::Uuid;

use crate::MetadataError;

/// Repository for metadata CRUD operations
pub struct MetadataRepository {
    pool: PgPool,
}

impl MetadataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ============ EntityType ============

    pub async fn get_entity_type_by_name(
        &self,
        tenant_id: Uuid,
        name: &str,
    ) -> Result<EntityType, MetadataError> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, app_id, module_id, name, label, label_plural,
                icon, description, flags,
                default_sort_field, default_sort_desc, soft_delete,
                created_at, updated_at
            FROM entity_types
            WHERE tenant_id = $1 AND name = $2
            "#,
        )
        .bind(tenant_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| MetadataError::EntityTypeNotFound(name.to_string()))?;

        entity_type_from_row(&row)
    }

    pub async fn get_entity_type_by_id(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<EntityType, MetadataError> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, app_id, module_id, name, label, label_plural,
                icon, description, flags,
                default_sort_field, default_sort_desc, soft_delete,
                created_at, updated_at
            FROM entity_types
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| MetadataError::EntityTypeIdNotFound(id))?;

        entity_type_from_row(&row)
    }

    pub async fn list_entity_types(
        &self,
        tenant_id: Uuid,
        app_id: Option<&str>,
    ) -> Result<Vec<EntityType>, MetadataError> {
        let rows = if let Some(app) = app_id {
            sqlx::query(
                r#"
                SELECT 
                    id, tenant_id, app_id, module_id, name, label, label_plural,
                    icon, description, flags,
                    default_sort_field, default_sort_desc, soft_delete,
                    created_at, updated_at
                FROM entity_types
                WHERE tenant_id = $1 AND app_id = $2
                ORDER BY name
                "#,
            )
            .bind(tenant_id)
            .bind(app)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT 
                    id, tenant_id, app_id, module_id, name, label, label_plural,
                    icon, description, flags,
                    default_sort_field, default_sort_desc, soft_delete,
                    created_at, updated_at
                FROM entity_types
                WHERE tenant_id = $1
                ORDER BY app_id, name
                "#,
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?
        };

        rows.iter().map(entity_type_from_row).collect()
    }

    // ============ FieldDef ============

    pub async fn get_fields_for_entity(
        &self,
        tenant_id: Uuid,
        entity_type_id: Uuid,
    ) -> Result<Vec<FieldDef>, MetadataError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, entity_type_id, name, label, field_type,
                is_required, is_unique, show_in_list, show_in_card,
                is_searchable, is_filterable, is_sortable, is_readonly,
                default_value, placeholder, help_text,
                validation, options, ui_hints,
                sort_order, "group",
                created_at, updated_at
            FROM field_defs
            WHERE tenant_id = $1 AND entity_type_id = $2
            ORDER BY sort_order, name
            "#,
        )
        .bind(tenant_id)
        .bind(entity_type_id)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(field_def_from_row).collect()
    }

    // ============ ViewDef ============

    pub async fn get_views_for_entity(
        &self,
        tenant_id: Uuid,
        entity_type_id: Uuid,
    ) -> Result<Vec<ViewDef>, MetadataError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, entity_type_id, name, label, view_type,
                is_default, is_system, created_by,
                columns, filters, sort, settings,
                created_at, updated_at
            FROM view_defs
            WHERE tenant_id = $1 AND entity_type_id = $2
            ORDER BY is_default DESC, name
            "#,
        )
        .bind(tenant_id)
        .bind(entity_type_id)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(view_def_from_row).collect()
    }

    pub async fn get_default_view(
        &self,
        tenant_id: Uuid,
        entity_type_id: Uuid,
    ) -> Result<Option<ViewDef>, MetadataError> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, entity_type_id, name, label, view_type,
                is_default, is_system, created_by,
                columns, filters, sort, settings,
                created_at, updated_at
            FROM view_defs
            WHERE tenant_id = $1 AND entity_type_id = $2 AND is_default = true
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(entity_type_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(view_def_from_row(&r)?)),
            None => Ok(None),
        }
    }

    // ============ Apps ============

    pub async fn list_apps(&self, tenant_id: Uuid) -> Result<Vec<AppDef>, MetadataError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, name, label, icon, description,
                sort_order, is_enabled,
                created_at, updated_at
            FROM app_defs
            WHERE tenant_id = $1 AND is_enabled = true
            ORDER BY sort_order
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(app_def_from_row).collect()
    }
}

// Helper functions to convert rows to types using JSON deserialization for complex fields
fn entity_type_from_row(row: &sqlx::postgres::PgRow) -> Result<EntityType, MetadataError> {
    use sqlx::Row;
    
    // Get the flags as JSON and deserialize
    let flags_json: serde_json::Value = row.try_get("flags")?;
    let flags: EntityFlags = serde_json::from_value(flags_json).unwrap_or_default();
    
    Ok(EntityType {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        app_id: row.try_get("app_id")?,
        module_id: row.try_get("module_id")?,
        name: row.try_get("name")?,
        label: row.try_get("label")?,
        label_plural: row.try_get("label_plural")?,
        icon: row.try_get("icon")?,
        description: row.try_get("description")?,
        flags,
        default_sort_field: row.try_get("default_sort_field")?,
        default_sort_desc: row.try_get("default_sort_desc")?,
        soft_delete: row.try_get("soft_delete")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn field_def_from_row(row: &sqlx::postgres::PgRow) -> Result<FieldDef, MetadataError> {
    use sqlx::Row;
    
    // Get field_type as string - it might be JSON or simple string
    let field_type_str: String = row.try_get("field_type")?;
    
    // Try parsing as JSON first (new format: {"type": "Select", ...})
    let field_type: FieldType = serde_json::from_str(&field_type_str)
        .unwrap_or_else(|_| {
            // Fallback: try as simple string (old format: "select", "text", etc.)
            match field_type_str.to_lowercase().as_str() {
                "select" | "status" => FieldType::Select { options: vec![] },
                "multiselect" | "multi_select" => FieldType::MultiSelect { options: vec![] },
                "text" => FieldType::Text,
                "textarea" | "longtext" => FieldType::TextArea,
                "richtext" => FieldType::RichText,
                "number" | "integer" | "decimal" => FieldType::Number { decimals: None },
                "money" | "currency" => FieldType::Money { currency_code: Some("USD".to_string()) },
                "boolean" => FieldType::Boolean,
                "date" => FieldType::Date,
                "datetime" => FieldType::DateTime,
                "email" => FieldType::Email,
                "phone" => FieldType::Phone,
                "url" => FieldType::Url,
                "link" | "lookup" => FieldType::Link { target_entity: "contact".to_string() },
                "multilink" | "multi_link" => FieldType::MultiLink { target_entity: "contact".to_string() },
                "taglist" | "tag_list" | "tags" => FieldType::TagList,
                "image" => FieldType::Image,
                "attachment" | "file" | "file_array" => FieldType::Attachment,
                "score" => FieldType::Score { max_value: Some(100) },
                "json" => FieldType::Json,
                _ => FieldType::Text,
            }
        });
    
    // Get optional JSON fields
    let validation: Option<serde_json::Value> = row.try_get("validation")?;
    let options: Option<serde_json::Value> = row.try_get("options")?;
    let ui_hints: Option<serde_json::Value> = row.try_get("ui_hints")?;
    
    Ok(FieldDef {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        entity_type_id: row.try_get("entity_type_id")?,
        name: row.try_get("name")?,
        label: row.try_get("label")?,
        field_type,
        is_required: row.try_get("is_required")?,
        is_unique: row.try_get("is_unique")?,
        show_in_list: row.try_get("show_in_list")?,
        show_in_card: row.try_get("show_in_card")?,
        is_searchable: row.try_get("is_searchable")?,
        is_filterable: row.try_get("is_filterable")?,
        is_sortable: row.try_get("is_sortable")?,
        is_readonly: row.try_get("is_readonly")?,
        default_value: row.try_get("default_value")?,
        placeholder: row.try_get("placeholder")?,
        help_text: row.try_get("help_text")?,
        validation: validation.and_then(|v| serde_json::from_value(v).ok()),
        options: options.and_then(|v| serde_json::from_value(v).ok()),
        ui_hints: ui_hints.and_then(|v| serde_json::from_value(v).ok()),
        sort_order: row.try_get("sort_order")?,
        group: row.try_get("group")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn view_def_from_row(row: &sqlx::postgres::PgRow) -> Result<ViewDef, MetadataError> {
    use sqlx::Row;
    
    // Get view_type as string and parse
    let view_type_str: String = row.try_get("view_type")?;
    let view_type: ViewType = serde_json::from_str(&format!("\"{}\"", view_type_str))
        .unwrap_or(ViewType::Table);
    
    // Get JSON fields
    let columns: serde_json::Value = row.try_get("columns")?;
    let filters: serde_json::Value = row.try_get("filters")?;
    let sort: serde_json::Value = row.try_get("sort")?;
    
    Ok(ViewDef {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        entity_type_id: row.try_get("entity_type_id")?,
        name: row.try_get("name")?,
        label: row.try_get("label")?,
        view_type,
        is_default: row.try_get("is_default")?,
        is_system: row.try_get("is_system")?,
        created_by: row.try_get("created_by")?,
        owner_id: row.try_get("created_by").ok(), // Use created_by as owner
        is_favorite: false, // Default to not favorite
        group_by: None, // Will read from settings if needed
        columns: serde_json::from_value(columns).unwrap_or_default(),
        filters: serde_json::from_value(filters).unwrap_or_default(),
        sort: serde_json::from_value(sort).unwrap_or_default(),
        settings: row.try_get("settings")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn app_def_from_row(row: &sqlx::postgres::PgRow) -> Result<AppDef, MetadataError> {
    use sqlx::Row;
    Ok(AppDef {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        name: row.try_get("name")?,
        label: row.try_get("label")?,
        icon: row.try_get("icon")?,
        description: row.try_get("description")?,
        sort_order: row.try_get("sort_order")?,
        is_enabled: row.try_get("is_enabled")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
