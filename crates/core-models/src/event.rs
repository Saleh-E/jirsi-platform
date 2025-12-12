//! Event model - Entity lifecycle events for triggers

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Built-in event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Record created
    Created,
    /// Record updated
    Updated,
    /// Record deleted
    Deleted,
    /// Specific field changed
    FieldChanged,
    /// Record entered a stage (pipeline)
    StageEntered,
    /// Record left a stage (pipeline)
    StageLeft,
    /// Custom event triggered by node/API
    Custom,
    /// Time-based event (e.g., "no activity for 7 days")
    TimeBased,
}

/// Event definition - defines a type of event that can be triggered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDef {
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// Which EntityType this event applies to
    pub entity_type_id: Uuid,
    /// Event name (internal)
    pub name: String,
    /// Display label
    pub label: String,
    /// Type of event
    pub event_type: EventType,
    /// For FieldChanged: which field
    pub field_name: Option<String>,
    /// For StageEntered/Left: which stage
    pub stage_value: Option<String>,
    /// For TimeBased: cron expression or interval
    pub schedule: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Is event enabled?
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Event instance - actual event that occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// Which EventDef this is an instance of
    pub event_def_id: Uuid,
    /// Which EntityType the event is for
    pub entity_type_id: Uuid,
    /// Which record triggered the event
    pub record_id: Uuid,
    /// Who caused the event (can be null for system events)
    pub triggered_by: Option<Uuid>,
    /// Event data (old values, new values, etc.)
    pub payload: serde_json::Value,
    /// When the event occurred
    pub occurred_at: DateTime<Utc>,
    /// Was the event processed by workflows?
    pub processed: bool,
    /// Processing result/errors
    pub processing_result: Option<serde_json::Value>,
}

impl Event {
    pub fn new(
        tenant_id: Uuid,
        event_def_id: Uuid,
        entity_type_id: Uuid,
        record_id: Uuid,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            event_def_id,
            entity_type_id,
            record_id,
            triggered_by: None,
            payload,
            occurred_at: Utc::now(),
            processed: false,
            processing_result: None,
        }
    }

    pub fn triggered_by(mut self, user_id: Uuid) -> Self {
        self.triggered_by = Some(user_id);
        self
    }
}

impl EventDef {
    pub fn on_create(tenant_id: Uuid, entity_type_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            entity_type_id,
            name: "on_create".to_string(),
            label: "On Create".to_string(),
            event_type: EventType::Created,
            field_name: None,
            stage_value: None,
            schedule: None,
            description: Some("Triggered when a record is created".to_string()),
            is_enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn on_update(tenant_id: Uuid, entity_type_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            entity_type_id,
            name: "on_update".to_string(),
            label: "On Update".to_string(),
            event_type: EventType::Updated,
            field_name: None,
            stage_value: None,
            schedule: None,
            description: Some("Triggered when a record is updated".to_string()),
            is_enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn on_field_change(tenant_id: Uuid, entity_type_id: Uuid, field_name: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            entity_type_id,
            name: format!("on_{}_change", field_name),
            label: format!("On {} Change", field_name),
            event_type: EventType::FieldChanged,
            field_name: Some(field_name.to_string()),
            stage_value: None,
            schedule: None,
            description: Some(format!("Triggered when {} field changes", field_name)),
            is_enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}
