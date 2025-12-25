//! Deal Commands
//! 
//! All write operations for Deals go through these commands.
//! This makes the system:
//! - Testable (commands are structs)
//! - Auditable (every change is tracked)
//! - Exposable to Node Engine (commands as workflow actions)

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

// Import DealOutcome from events module to avoid duplication
use super::events::DealOutcome;

/// Create a new deal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDealCommand {
    pub deal_id: Uuid,
    pub tenant_id: Uuid,
    pub title: String,
    pub value: Option<Decimal>,
    pub stage: String,
    pub contact_id: Option<Uuid>,
    pub property_id: Option<Uuid>,
    pub created_by: Uuid,
}

impl CreateDealCommand {
    pub fn new(
        tenant_id: Uuid,
        title: String,
        created_by: Uuid,
    ) -> Self {
        Self {
            deal_id: Uuid::new_v4(),
            tenant_id,
            title,
            value: None,
            stage: "lead".to_string(),
            contact_id: None,
            property_id: None,
            created_by,
        }
    }
    
    pub fn with_value(mut self, value: Decimal) -> Self {
        self.value = Some(value);
        self
    }
    
    pub fn with_stage(mut self, stage: String) -> Self {
        self.stage = stage;
        self
    }
    
    pub fn with_contact(mut self, contact_id: Uuid) -> Self {
        self.contact_id = Some(contact_id);
        self
    }
    
    pub fn with_property(mut self, property_id: Uuid) -> Self {
        self.property_id = Some(property_id);
        self
    }
}

/// Update the deal stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDealStageCommand {
    pub deal_id: Uuid,
    pub new_stage: String,
    pub updated_by: Uuid,
    pub reason: Option<String>,
}

impl UpdateDealStageCommand {
    pub fn new(deal_id: Uuid, new_stage: String, updated_by: Uuid) -> Self {
        Self {
            deal_id,
            new_stage,
            updated_by,
            reason: None,
        }
    }
    
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
}

/// Update the deal value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddValueCommand {
    pub deal_id: Uuid,
    pub value: Decimal,
    pub updated_by: Uuid,
}

impl AddValueCommand {
    pub fn new(deal_id: Uuid, value: Decimal, updated_by: Uuid) -> Self {
        Self {
            deal_id,
            value,
            updated_by,
        }
    }
}

/// Assign contact to deal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignContactCommand {
    pub deal_id: Uuid,
    pub contact_id: Uuid,
    pub updated_by: Uuid,
}

/// Assign property to deal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignPropertyCommand {
    pub deal_id: Uuid,
    pub property_id: Uuid,
    pub updated_by: Uuid,
}

/// Close deal (win or lost)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseDealCommand {
    pub deal_id: Uuid,
    pub outcome: DealOutcome,
    pub final_value: Option<Decimal>,
    pub closed_by: Uuid,
    pub notes: Option<String>,
}
