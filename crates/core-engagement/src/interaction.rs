//! Interaction model - Activity tracking for entities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of interaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionType {
    Call,
    Email,
    Meeting,
    Note,
    Task,
    Message,
    Other,
}

impl Default for InteractionType {
    fn default() -> Self {
        Self::Note
    }
}

/// An interaction record (activity on an entity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// Entity type this interaction is on
    pub entity_type: String,
    /// Record ID this interaction is on
    pub record_id: Uuid,
    /// Type of interaction
    pub interaction_type: InteractionType,
    /// Title/subject
    pub title: String,
    /// Optional content/notes
    pub content: Option<String>,
    /// Who created this interaction
    pub created_by: Uuid,
    /// When it occurred
    pub occurred_at: DateTime<Utc>,
    /// Duration in minutes (for calls/meetings)
    pub duration_minutes: Option<i32>,
    /// Outcome (for calls/meetings)
    pub outcome: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Interaction {
    pub fn new(
        tenant_id: Uuid,
        entity_type: String,
        record_id: Uuid,
        interaction_type: InteractionType,
        title: String,
        created_by: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            entity_type,
            record_id,
            interaction_type,
            title,
            content: None,
            created_by,
            occurred_at: now,
            duration_minutes: None,
            outcome: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn with_duration(mut self, minutes: i32) -> Self {
        self.duration_minutes = Some(minutes);
        self
    }
}
