//! Inbox Service - Thread aggregation for unified messaging
//!
//! Groups interactions into conversation threads by entity (Contact, Deal, etc.)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A conversation thread in the inbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxThread {
    /// Entity ID this thread is about (e.g., Contact ID)
    pub entity_id: Uuid,
    /// Entity type (contact, deal, company, etc.)
    pub entity_type: String,
    /// Display name for the entity
    pub entity_name: String,
    /// Preview of the last message (truncated)
    pub last_message_preview: String,
    /// When the last message was sent/received
    pub last_message_at: DateTime<Utc>,
    /// Number of unread messages in this thread
    pub unread_count: i64,
    /// Type of the last interaction (email, note, message, etc.)
    pub last_interaction_type: String,
}

/// Filter options for inbox threads
#[derive(Debug, Clone, Default, Deserialize)]
pub struct InboxFilter {
    /// Filter by status: all, unread, sent
    #[serde(default)]
    pub status: Option<String>,
    /// Filter by entity type
    #[serde(default)]
    pub entity_type: Option<String>,
    /// Filter by assigned user
    #[serde(default)]
    pub assigned_to: Option<Uuid>,
}

/// A message within a conversation thread
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMessage {
    pub id: Uuid,
    pub interaction_type: String,
    pub title: String,
    pub content: Option<String>,
    pub created_by: Uuid,
    pub occurred_at: DateTime<Utc>,
    /// Direction: inbound or outbound
    pub direction: String,
    pub duration_minutes: Option<i32>,
}

/// Request to send a reply in a thread
#[derive(Debug, Clone, Deserialize)]
pub struct ReplyRequest {
    pub interaction_type: String,
    pub title: String,
    pub content: Option<String>,
    pub created_by: Uuid,
}

impl InboxThread {
    /// Truncate message preview to specified length
    pub fn truncate_preview(content: &str, max_len: usize) -> String {
        if content.len() <= max_len {
            content.to_string()
        } else {
            format!("{}...", &content[..max_len.saturating_sub(3)])
        }
    }
}

impl Default for InboxThread {
    fn default() -> Self {
        Self {
            entity_id: Uuid::nil(),
            entity_type: String::new(),
            entity_name: String::new(),
            last_message_preview: String::new(),
            last_message_at: Utc::now(),
            unread_count: 0,
            last_interaction_type: String::new(),
        }
    }
}
