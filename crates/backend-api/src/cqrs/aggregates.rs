//! Deal Aggregate
//! 
//! The Deal aggregate represents the current state of a Deal,
//! built by replaying all events from the event store.
//! 
//! This is the "Write Side" of CQRS - optimized for business logic validation.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use super::{DealEvent, DealOutcome};
use super::commands::*;

/// Deal Aggregate - current state built from events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Deal {
    // Identity
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub version: u64,
    
    // Business data
    pub title: String,
    pub value: Option<Decimal>,
    pub stage: String,
    pub contact_id: Option<Uuid>,
    pub property_id: Option<Uuid>,
    
    // Status
    pub is_closed: bool,
    pub outcome: Option<DealOutcome>,
    
    // Metadata
    pub created_by: Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Deal {
    /// Create a new Deal from a CreateDealCommand
    pub fn create(cmd: CreateDealCommand) -> Result<DealEvent, DealError> {
        // Validation
        if cmd.title.trim().is_empty() {
            return Err(DealError::InvalidTitle);
        }
        
        if let Some(value) = cmd.value {
            if value < Decimal::ZERO {
                return Err(DealError::InvalidValue);
            }
        }
        
        // Generate event
        Ok(DealEvent::Created {
            deal_id: cmd.deal_id,
            tenant_id: cmd.tenant_id,
            title: cmd.title,
            value: cmd.value,
            stage: cmd.stage,
            contact_id: cmd.contact_id,
            property_id: cmd.property_id,
            created_by: cmd.created_by,
            created_at: Utc::now(),
        })
    }
    
    /// Update deal stage
    pub fn update_stage(&self, cmd: UpdateDealStageCommand) -> Result<DealEvent, DealError> {
        // Validation
        if self.is_closed {
            return Err(DealError::DealAlreadyClosed);
        }
        
        if cmd.new_stage.trim().is_empty() {
            return Err(DealError::InvalidStage);
        }
        
        if self.stage == cmd.new_stage {
            return Err(DealError::StageUnchanged);
        }
        
        // Generate event
        Ok(DealEvent::StageUpdated {
            deal_id: self.id,
            old_stage: self.stage.clone(),
            new_stage: cmd.new_stage,
            updated_by: cmd.updated_by,
            updated_at: Utc::now(),
            reason: cmd.reason,
        })
    }
    
    /// Add/update value
    pub fn add_value(&self, cmd: AddValueCommand) -> Result<DealEvent, DealError> {
        // Validation
        if self.is_closed {
            return Err(DealError::DealAlreadyClosed);
        }
        
        if cmd.value < Decimal::ZERO {
            return Err(DealError::InvalidValue);
        }
        
        // Generate event
        Ok(DealEvent::ValueAdded {
            deal_id: self.id,
            old_value: self.value,
            new_value: cmd.value,
            updated_by: cmd.updated_by,
            updated_at: Utc::now(),
        })
    }
    
    /// Assign contact
    pub fn assign_contact(&self, cmd: AssignContactCommand) -> Result<DealEvent, DealError> {
        if self.is_closed {
            return Err(DealError::DealAlreadyClosed);
        }
        
        Ok(DealEvent::ContactAssigned {
            deal_id: self.id,
            contact_id: cmd.contact_id,
            previous_contact_id: self.contact_id,
            updated_by: cmd.updated_by,
            updated_at: Utc::now(),
        })
    }
    
    /// Assign property
    pub fn assign_property(&self, cmd: AssignPropertyCommand) -> Result<DealEvent, DealError> {
        if self.is_closed {
            return Err(DealError::DealAlreadyClosed);
        }
        
        Ok(DealEvent::PropertyAssigned {
            deal_id: self.id,
            property_id: cmd.property_id,
            previous_property_id: self.property_id,
            updated_by: cmd.updated_by,
            updated_at: Utc::now(),
        })
    }
    
    /// Close deal
    pub fn close(&self, cmd: CloseDealCommand) -> Result<DealEvent, DealError> {
        if self.is_closed {
            return Err(DealError::DealAlreadyClosed);
        }
        
        Ok(DealEvent::Closed {
            deal_id: self.id,
            outcome: cmd.outcome,
            final_value: cmd.final_value.or(self.value),
            closed_by: cmd.closed_by,
            closed_at: Utc::now(),
            notes: cmd.notes,
        })
    }
    
    /// Apply an event to update the aggregate state
    /// This is called when replaying events from the event store
    pub fn apply(&mut self, event: &DealEvent) {
        match event {
            DealEvent::Created {
                deal_id,
                tenant_id,
                title,
                value,
                stage,
                contact_id,
                property_id,
                created_by,
                created_at,
            } => {
                self.id = *deal_id;
                self.tenant_id = *tenant_id;
                self.title = title.clone();
                self.value = *value;
                self.stage = stage.clone();
                self.contact_id = *contact_id;
                self.property_id = *property_id;
                self.created_by = *created_by;
                self.created_at = Some(*created_at);
                self.updated_at = Some(*created_at);
            }
            
            DealEvent::StageUpdated {
                new_stage,
                updated_at,
                ..
            } => {
                self.stage = new_stage.clone();
                self.updated_at = Some(*updated_at);
            }
            
            DealEvent::ValueAdded {
                new_value,
                updated_at,
                ..
            } => {
                self.value = Some(*new_value);
                self.updated_at = Some(*updated_at);
            }
            
            DealEvent::ContactAssigned {
                contact_id,
                updated_at,
                ..
            } => {
                self.contact_id = Some(*contact_id);
                self.updated_at = Some(*updated_at);
            }
            
            DealEvent::PropertyAssigned {
                property_id,
                updated_at,
                ..
            } => {
                self.property_id = Some(*property_id);
                self.updated_at = Some(*updated_at);
            }
            
            DealEvent::Closed {
                outcome,
                final_value,
                closed_at,
                ..
            } => {
                self.is_closed = true;
                self.outcome = Some(outcome.clone());
                if let Some(value) = final_value {
                    self.value = Some(*value);
                }
                self.updated_at = Some(*closed_at);
            }
        }
        
        self.version += 1;
    }
}

/// Deal-specific errors
#[derive(Debug, thiserror::Error)]
pub enum DealError {
    #[error("Deal title cannot be empty")]
    InvalidTitle,
    
    #[error("Deal value must be positive")]
    InvalidValue,
    
    #[error("Deal stage cannot be empty")]
    InvalidStage,
    
    #[error("Stage is unchanged")]
    StageUnchanged,
    
    #[error("Deal is already closed")]
    DealAlreadyClosed,
}
