//! CQRS Integration Tests
//! 
//! End-to-end tests for the Deal aggregate with Event Sourcing

#[cfg(test)]
mod tests {
    use super::super::*;
    use uuid::Uuid;
    use rust_decimal_macros::dec;
    
    #[test]
    fn test_create_deal_command() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        let cmd = CreateDealCommand::new(
            tenant_id,
            "Big Enterprise Deal".to_string(),
            user_id,
        )
        .with_value(dec!(50000.00))
        .with_stage("proposal".to_string());
        
        assert_eq!(cmd.tenant_id, tenant_id);
        assert_eq!(cmd.title, "Big Enterprise Deal");
        assert_eq!(cmd.value, Some(dec!(50000.00)));
        assert_eq!(cmd.stage, "proposal");
    }
    
    #[test]
    fn test_deal_creation() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        let cmd = CreateDealCommand::new(
            tenant_id,
            "Test Deal".to_string(),
            user_id,
        );
        
        let result = Deal::create(cmd);
        assert!(result.is_ok());
        
        let event = result.unwrap();
        
        if let DealEvent::Created { title, tenant_id: tid, .. } = event {
            assert_eq!(title, "Test Deal");
            assert_eq!(tid, tenant_id);
        } else {
            panic!("Expected DealCreated event");
        }
    }
    
    #[test]
    fn test_deal_creation_validation() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        // Empty title should fail
        let cmd = CreateDealCommand::new(
            tenant_id,
            "".to_string(),
            user_id,
        );
        
        let result = Deal::create(cmd);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DealError::InvalidTitle));
        
        // Negative value should fail
        let cmd = CreateDealCommand::new(
            tenant_id,
            "Test".to_string(),
            user_id,
        ).with_value(dec!(-100.00));
        
        let result = Deal::create(cmd);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DealError::InvalidValue));
    }
    
    #[test]
    fn test_deal_event_replay() {
        let deal_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        // Create initial state
        let mut deal = Deal::default();
        
        // Apply creation event
        let created_event = DealEvent::Created {
            deal_id,
            tenant_id,
            title: "Test Deal".to_string(),
            value: Some(dec!(10000.00)),
            stage: "lead".to_string(),
            contact_id: None,
            property_id: None,
            created_by: user_id,
            created_at: chrono::Utc::now(),
        };
        
        deal.apply(&created_event);
        
        assert_eq!(deal.id, deal_id);
        assert_eq!(deal.title, "Test Deal");
        assert_eq!(deal.value, Some(dec!(10000.00)));
        assert_eq!(deal.stage, "lead");
        assert_eq!(deal.version, 1);
        
        // Apply stage update event
        let stage_event = DealEvent::StageUpdated {
            deal_id,
            old_stage: "lead".to_string(),
            new_stage: "proposal".to_string(),
            updated_by: user_id,
            updated_at: chrono::Utc::now(),
            reason: Some("Sent proposal".to_string()),
        };
        
        deal.apply(&stage_event);
        
        assert_eq!(deal.stage, "proposal");
        assert_eq!(deal.version, 2);
        
        // Apply value update event
        let value_event = DealEvent::ValueAdded {
            deal_id,
            old_value: Some(dec!(10000.00)),
            new_value: dec!(15000.00),
            updated_by: user_id,
            updated_at: chrono::Utc::now(),
        };
        
        deal.apply(&value_event);
        
        assert_eq!(deal.value, Some(dec!(15000.00)));
        assert_eq!(deal.version, 3);
    }
    
    #[test]
    fn test_update_stage_command() {
        let deal_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        // Create a deal
        let mut deal = Deal::default();
        deal.id = deal_id;
        deal.stage = "lead".to_string();
        deal.is_closed = false;
        
        // Update stage
        let cmd = UpdateDealStageCommand::new(
            deal_id,
            "proposal".to_string(),
            user_id,
        ).with_reason("Customer interested".to_string());
        
        let result = deal.update_stage(cmd);
        assert!(result.is_ok());
        
        let event = result.unwrap();
        
        if let DealEvent::StageUpdated { old_stage, new_stage, reason, .. } = event {
            assert_eq!(old_stage, "lead");
            assert_eq!(new_stage, "proposal");
            assert_eq!(reason, Some("Customer interested".to_string()));
        } else {
            panic!("Expected StageUpdated event");
        }
    }
    
    #[test]
    fn test_closed_deal_cannot_be_updated() {
        let deal_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        let mut deal = Deal::default();
        deal.id = deal_id;
        deal.is_closed = true;
        
        // Try to update stage
        let cmd = UpdateDealStageCommand::new(
            deal_id,
            "proposal".to_string(),
            user_id,
        );
        
        let result = deal.update_stage(cmd);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DealError::DealAlreadyClosed));
        
        // Try to update value
        let cmd = AddValueCommand::new(deal_id, dec!(1000.00), user_id);
        
        let result = deal.add_value(cmd);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DealError::DealAlreadyClosed));
    }
    
    #[test]
    fn test_deal_lifecycle() {
        let deal_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        let mut deal = Deal::default();
        
        // Create
        let created = DealEvent::Created {
            deal_id,
            tenant_id,
            title: "Lifecycle Test".to_string(),
            value: Some(dec!(5000.00)),
            stage: "lead".to_string(),
            contact_id: None,
            property_id: None,
            created_by: user_id,
            created_at: chrono::Utc::now(),
        };
        deal.apply(&created);
        assert_eq!(deal.version, 1);
        assert!(!deal.is_closed);
        
        // Update stage
        let stage_updated = DealEvent::StageUpdated {
            deal_id,
            old_stage: "lead".to_string(),
            new_stage: "negotiation".to_string(),
            updated_by: user_id,
            updated_at: chrono::Utc::now(),
            reason: None,
        };
        deal.apply(&stage_updated);
        assert_eq!(deal.version, 2);
        assert_eq!(deal.stage, "negotiation");
        
        // Close deal
        let closed = DealEvent::Closed {
            deal_id,
            outcome: DealOutcome::Won,
            final_value: Some(dec!(6000.00)),
            closed_by: user_id,
            closed_at: chrono::Utc::now(),
            notes: Some("Successfully closed!".to_string()),
        };
        deal.apply(&closed);
        assert_eq!(deal.version, 3);
        assert!(deal.is_closed);
        assert_eq!(deal.value, Some(dec!(6000.00)));
    }
}
