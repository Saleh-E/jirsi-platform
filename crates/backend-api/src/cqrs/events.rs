//! Deal Events
//! 
//! Immutable facts about what happened to Deals.
//! These are the source of truth - we can replay them to rebuild state.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// All events that can happen to a Deal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DealEvent {
    /// Deal was created
    Created {
        deal_id: Uuid,
        tenant_id: Uuid,
        title: String,
        value: Option<Decimal>,
        stage: String,
        contact_id: Option<Uuid>,
        property_id: Option<Uuid>,
        created_by: Uuid,
        created_at: DateTime<Utc>,
    },
    
    /// Deal stage changed
    StageUpdated {
        deal_id: Uuid,
        old_stage: String,
        new_stage: String,
        updated_by: Uuid,
        updated_at: DateTime<Utc>,
        reason: Option<String>,
    },
    
    /// Deal value added/updated
    ValueAdded {
        deal_id: Uuid,
        old_value: Option<Decimal>,
        new_value: Decimal,
        updated_by: Uuid,
        updated_at: DateTime<Utc>,
    },
    
    /// Contact assigned to deal
    ContactAssigned {
        deal_id: Uuid,
        contact_id: Uuid,
        previous_contact_id: Option<Uuid>,
        updated_by: Uuid,
        updated_at: DateTime<Utc>,
    },
    
    /// Property assigned to deal
    PropertyAssigned {
        deal_id: Uuid,
        property_id: Uuid,
        previous_property_id: Option<Uuid>,
        updated_by: Uuid,
        updated_at: DateTime<Utc>,
    },
    
    /// Deal closed (won or lost)
    Closed {
        deal_id: Uuid,
        outcome: DealOutcome,
        final_value: Option<Decimal>,
        closed_by: Uuid,
        closed_at: DateTime<Utc>,
        notes: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DealOutcome {
    Won,
    Lost,
}

impl DealEvent {
    /// Get the deal ID from any event
    pub fn deal_id(&self) -> Uuid {
        match self {
            DealEvent::Created { deal_id, .. } => *deal_id,
            DealEvent::StageUpdated { deal_id, .. } => *deal_id,
            DealEvent::ValueAdded { deal_id, .. } => *deal_id,
            DealEvent::ContactAssigned { deal_id, .. } => *deal_id,
            DealEvent::PropertyAssigned { deal_id, .. } => *deal_id,
            DealEvent::Closed { deal_id, .. } => *deal_id,
        }
    }
    
    /// Get the timestamp from any event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            DealEvent::Created { created_at, .. } => *created_at,
            DealEvent::StageUpdated { updated_at, .. } => *updated_at,
            DealEvent::ValueAdded { updated_at, .. } => *updated_at,
            DealEvent::ContactAssigned { updated_at, .. } => *updated_at,
            DealEvent::PropertyAssigned { updated_at, .. } => *updated_at,
            DealEvent::Closed { closed_at, .. } => *closed_at,
        }
    }
    
    /// Get the user who triggered this event
    pub fn triggered_by(&self) -> Uuid {
        match self {
            DealEvent::Created { created_by, .. } => *created_by,
            DealEvent::StageUpdated { updated_by, .. } => *updated_by,
            DealEvent::ValueAdded { updated_by, .. } => *updated_by,
            DealEvent::ContactAssigned { updated_by, .. } => *updated_by,
            DealEvent::PropertyAssigned { updated_by, .. } => *updated_by,
            DealEvent::Closed { closed_by, .. } => *closed_by,
        }
    }
}
