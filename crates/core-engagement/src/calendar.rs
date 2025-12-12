//! Calendar models and services

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Calendar event status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Tentative,
    Confirmed,
    Cancelled,
}

/// A calendar event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// Optional link to a user's calendar
    pub calendar_id: Option<Uuid>,
    /// Title of the event
    pub title: String,
    /// Description/notes
    pub description: Option<String>,
    /// Event location (physical or online meeting link)
    pub location: Option<String>,
    /// Start time
    pub start_at: DateTime<Utc>,
    /// End time
    pub end_at: DateTime<Utc>,
    /// Is this an all-day event?
    pub all_day: bool,
    /// Event status
    pub status: EventStatus,
    /// Online meeting URL (Zoom, Google Meet, etc.)
    pub meeting_url: Option<String>,
    /// The user who created/owns this event
    pub owner_id: Uuid,
    /// Linked entity type (e.g., "contact", "deal")
    pub linked_entity_type: Option<String>,
    /// Linked record ID
    pub linked_entity_id: Option<Uuid>,
    /// Recurrence rule (iCal RRULE format)
    pub recurrence_rule: Option<String>,
    /// External ID (for sync with Google/Microsoft)
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Attendee response status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttendeeStatus {
    Pending,
    Accepted,
    Declined,
    Tentative,
}

/// An attendee of a calendar event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarAttendee {
    pub id: Uuid,
    pub event_id: Uuid,
    /// Type of attendee: "user" or "contact"
    pub attendee_type: String,
    /// ID of the user or contact
    pub attendee_id: Uuid,
    /// Email address
    pub email: String,
    /// Display name
    pub name: Option<String>,
    /// Is this attendee required?
    pub is_required: bool,
    /// Is this the organizer?
    pub is_organizer: bool,
    /// Response status
    pub status: AttendeeStatus,
}

/// A user's calendar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub is_default: bool,
    pub is_visible: bool,
    /// External calendar ID (Google/Microsoft)
    pub external_id: Option<String>,
    /// External calendar provider
    pub external_provider: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CalendarEvent {
    pub fn new(
        tenant_id: Uuid,
        owner_id: Uuid,
        title: &str,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            calendar_id: None,
            title: title.to_string(),
            description: None,
            location: None,
            start_at,
            end_at,
            all_day: false,
            status: EventStatus::Confirmed,
            meeting_url: None,
            owner_id,
            linked_entity_type: None,
            linked_entity_id: None,
            recurrence_rule: None,
            external_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_link(mut self, entity_type: &str, entity_id: Uuid) -> Self {
        self.linked_entity_type = Some(entity_type.to_string());
        self.linked_entity_id = Some(entity_id);
        self
    }

    pub fn with_meeting(mut self, url: &str) -> Self {
        self.meeting_url = Some(url.to_string());
        self
    }
}
