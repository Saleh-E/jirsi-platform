//! Events module - CRUD event types and publisher for triggers

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of entity event that can trigger workflows
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Record was created
    Create,
    /// Record was updated
    Update,
    /// Record was deleted
    Delete,
    /// Custom event (user-defined)
    Custom(String),
}

impl ToString for EventType {
    fn to_string(&self) -> String {
        match self {
            Self::Create => "create".to_string(),
            Self::Update => "update".to_string(),
            Self::Delete => "delete".to_string(),
            Self::Custom(name) => format!("custom:{}", name),
        }
    }
}

/// An event that occurred on an entity record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityEvent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// Entity type (e.g., "contact", "deal")
    pub entity_type: String,
    /// Record ID that was affected
    pub record_id: Uuid,
    /// Type of event
    pub event_type: EventType,
    /// User who triggered the event
    pub triggered_by: Option<Uuid>,
    /// Old values (for update/delete)
    pub old_values: Option<serde_json::Value>,
    /// New values (for create/update)
    pub new_values: Option<serde_json::Value>,
    /// Changed fields (for update)
    pub changed_fields: Option<Vec<String>>,
    /// When the event occurred
    pub occurred_at: DateTime<Utc>,
}

impl EntityEvent {
    /// Create a new create event
    pub fn create(
        tenant_id: Uuid,
        entity_type: &str,
        record_id: Uuid,
        new_values: serde_json::Value,
        triggered_by: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            entity_type: entity_type.to_string(),
            record_id,
            event_type: EventType::Create,
            triggered_by,
            old_values: None,
            new_values: Some(new_values),
            changed_fields: None,
            occurred_at: Utc::now(),
        }
    }

    /// Create a new update event
    pub fn update(
        tenant_id: Uuid,
        entity_type: &str,
        record_id: Uuid,
        old_values: serde_json::Value,
        new_values: serde_json::Value,
        changed_fields: Vec<String>,
        triggered_by: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            entity_type: entity_type.to_string(),
            record_id,
            event_type: EventType::Update,
            triggered_by,
            old_values: Some(old_values),
            new_values: Some(new_values),
            changed_fields: Some(changed_fields),
            occurred_at: Utc::now(),
        }
    }

    /// Create a new delete event
    pub fn delete(
        tenant_id: Uuid,
        entity_type: &str,
        record_id: Uuid,
        old_values: serde_json::Value,
        triggered_by: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            entity_type: entity_type.to_string(),
            record_id,
            event_type: EventType::Delete,
            triggered_by,
            old_values: Some(old_values),
            new_values: None,
            changed_fields: None,
            occurred_at: Utc::now(),
        }
    }

    /// Create a custom event
    pub fn custom(
        tenant_id: Uuid,
        entity_type: &str,
        record_id: Uuid,
        event_name: &str,
        data: serde_json::Value,
        triggered_by: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            entity_type: entity_type.to_string(),
            record_id,
            event_type: EventType::Custom(event_name.to_string()),
            triggered_by,
            old_values: None,
            new_values: Some(data),
            changed_fields: None,
            occurred_at: Utc::now(),
        }
    }

    /// Convert to trigger data for graph execution
    pub fn to_trigger_data(&self) -> serde_json::Value {
        serde_json::json!({
            "event_id": self.id,
            "entity_type": self.entity_type,
            "record_id": self.record_id,
            "event_type": self.event_type.to_string(),
            "triggered_by": self.triggered_by,
            "old_values": self.old_values,
            "new_values": self.new_values,
            "changed_fields": self.changed_fields,
            "occurred_at": self.occurred_at,
        })
    }
}

/// Event publisher - stores events and triggers workflows
pub struct EventPublisher {
    pool: sqlx::PgPool,
}

impl EventPublisher {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Publish an event and store it in the database
    pub async fn publish(&self, event: &EntityEvent) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO events (id, tenant_id, entity_type, record_id, event_type, triggered_by, old_values, new_values, changed_fields, occurred_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(event.id)
        .bind(event.tenant_id)
        .bind(&event.entity_type)
        .bind(event.record_id)
        .bind(event.event_type.to_string())
        .bind(event.triggered_by)
        .bind(&event.old_values)
        .bind(&event.new_values)
        .bind(&event.changed_fields.as_ref().map(|v| serde_json::json!(v)))
        .bind(event.occurred_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get events for a record (for audit trail)
    pub async fn get_events_for_record(
        &self,
        tenant_id: Uuid,
        record_id: Uuid,
        limit: i32,
    ) -> Result<Vec<EntityEvent>, sqlx::Error> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"
            SELECT id, tenant_id, entity_type, record_id, event_type, triggered_by, old_values, new_values, changed_fields, occurred_at
            FROM events
            WHERE tenant_id = $1 AND record_id = $2
            ORDER BY occurred_at DESC
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(record_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let events = rows.iter().map(|row| {
            let event_type_str: String = row.try_get("event_type").unwrap_or_default();
            let event_type = match event_type_str.as_str() {
                "create" => EventType::Create,
                "update" => EventType::Update,
                "delete" => EventType::Delete,
                s if s.starts_with("custom:") => EventType::Custom(s[7..].to_string()),
                _ => EventType::Create,
            };

            EntityEvent {
                id: row.try_get("id").unwrap_or_default(),
                tenant_id: row.try_get("tenant_id").unwrap_or_default(),
                entity_type: row.try_get("entity_type").unwrap_or_default(),
                record_id: row.try_get("record_id").unwrap_or_default(),
                event_type,
                triggered_by: row.try_get("triggered_by").ok(),
                old_values: row.try_get("old_values").ok(),
                new_values: row.try_get("new_values").ok(),
                changed_fields: row.try_get::<Option<serde_json::Value>, _>("changed_fields")
                    .ok()
                    .flatten()
                    .and_then(|v| serde_json::from_value(v).ok()),
                occurred_at: row.try_get("occurred_at").unwrap_or_else(|_| Utc::now()),
            }
        }).collect();

        Ok(events)
    }
}
