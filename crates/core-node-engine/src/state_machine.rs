//! State Machine Logic
//! 
//! Provides state transition validation and handling for entity workflows.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a state transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Source state
    pub from: String,
    /// Target state
    pub to: String,
    /// Optional conditions that must be met
    pub conditions: Vec<TransitionCondition>,
    /// Actions to perform on transition
    pub actions: Vec<TransitionAction>,
}

/// Condition that must be met for a transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionCondition {
    /// Field to check
    pub field: String,
    /// Operator (eq, ne, gt, lt, gte, lte, contains, empty, not_empty)
    pub operator: String,
    /// Value to compare against
    pub value: serde_json::Value,
}

/// Action to perform on transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionAction {
    /// Action type (set_field, send_notification, trigger_workflow)
    pub action_type: String,
    /// Action configuration
    pub config: serde_json::Value,
}

/// State machine definition for an entity type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachineDefinition {
    /// Entity type this state machine applies to
    pub entity_type: String,
    /// Field that holds the state value
    pub state_field: String,
    /// Initial state for new entities
    pub initial_state: String,
    /// List of valid states
    pub states: Vec<StateInfo>,
    /// Allowed transitions
    pub transitions: Vec<StateTransition>,
}

/// Information about a state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateInfo {
    pub code: String,
    pub name: String,
    pub color: Option<String>,
    pub is_final: bool,
}

impl StateMachineDefinition {
    /// Check if a transition is allowed
    pub fn can_transition(&self, from: &str, to: &str) -> bool {
        self.transitions.iter().any(|t| t.from == from && t.to == to)
    }
    
    /// Get valid transitions from a state
    pub fn get_valid_transitions(&self, from: &str) -> Vec<&StateTransition> {
        self.transitions.iter().filter(|t| t.from == from || t.from == "*").collect()
    }
    
    /// Validate a transition against conditions
    pub fn validate_transition(
        &self,
        from: &str,
        to: &str,
        entity_data: &serde_json::Value,
    ) -> Result<&StateTransition, String> {
        let transition = self.transitions.iter()
            .find(|t| (t.from == from || t.from == "*") && t.to == to)
            .ok_or_else(|| format!("Transition from '{}' to '{}' is not allowed", from, to))?;
        
        // Check all conditions
        for condition in &transition.conditions {
            if !evaluate_condition(condition, entity_data) {
                return Err(format!(
                    "Condition not met: {} {} {:?}",
                    condition.field, condition.operator, condition.value
                ));
            }
        }
        
        Ok(transition)
    }
}

/// Evaluate a condition against entity data
fn evaluate_condition(condition: &TransitionCondition, data: &serde_json::Value) -> bool {
    let field_value = data.get(&condition.field);
    
    match condition.operator.as_str() {
        "eq" => field_value == Some(&condition.value),
        "ne" => field_value != Some(&condition.value),
        "empty" => field_value.is_none() || field_value == Some(&serde_json::Value::Null),
        "not_empty" => field_value.is_some() && field_value != Some(&serde_json::Value::Null),
        "contains" => {
            if let (Some(serde_json::Value::String(s)), serde_json::Value::String(v)) = (field_value, &condition.value) {
                s.contains(v)
            } else {
                false
            }
        }
        "gt" | "lt" | "gte" | "lte" => {
            if let (Some(serde_json::Value::Number(field_num)), serde_json::Value::Number(cond_num)) = (field_value, &condition.value) {
                let f = field_num.as_f64().unwrap_or(0.0);
                let c = cond_num.as_f64().unwrap_or(0.0);
                match condition.operator.as_str() {
                    "gt" => f > c,
                    "lt" => f < c,
                    "gte" => f >= c,
                    "lte" => f <= c,
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Common state machine definitions
pub mod presets {
    use super::*;
    
    /// Deal state machine (sales pipeline)
    pub fn deal_state_machine() -> StateMachineDefinition {
        StateMachineDefinition {
            entity_type: "deal".to_string(),
            state_field: "stage".to_string(),
            initial_state: "lead".to_string(),
            states: vec![
                StateInfo { code: "lead".to_string(), name: "Lead".to_string(), color: Some("#94a3b8".to_string()), is_final: false },
                StateInfo { code: "qualified".to_string(), name: "Qualified".to_string(), color: Some("#3b82f6".to_string()), is_final: false },
                StateInfo { code: "proposal".to_string(), name: "Proposal".to_string(), color: Some("#8b5cf6".to_string()), is_final: false },
                StateInfo { code: "negotiation".to_string(), name: "Negotiation".to_string(), color: Some("#f59e0b".to_string()), is_final: false },
                StateInfo { code: "closed_won".to_string(), name: "Closed Won".to_string(), color: Some("#22c55e".to_string()), is_final: true },
                StateInfo { code: "closed_lost".to_string(), name: "Closed Lost".to_string(), color: Some("#ef4444".to_string()), is_final: true },
            ],
            transitions: vec![
                StateTransition { from: "lead".to_string(), to: "qualified".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "qualified".to_string(), to: "proposal".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "proposal".to_string(), to: "negotiation".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "negotiation".to_string(), to: "closed_won".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "*".to_string(), to: "closed_lost".to_string(), conditions: vec![], actions: vec![] },
            ],
        }
    }
    
    /// Contract state machine
    pub fn contract_state_machine() -> StateMachineDefinition {
        StateMachineDefinition {
            entity_type: "contract".to_string(),
            state_field: "status".to_string(),
            initial_state: "draft".to_string(),
            states: vec![
                StateInfo { code: "draft".to_string(), name: "Draft".to_string(), color: Some("#94a3b8".to_string()), is_final: false },
                StateInfo { code: "pending_landlord".to_string(), name: "Pending Landlord".to_string(), color: Some("#f59e0b".to_string()), is_final: false },
                StateInfo { code: "pending_tenant".to_string(), name: "Pending Tenant".to_string(), color: Some("#8b5cf6".to_string()), is_final: false },
                StateInfo { code: "active".to_string(), name: "Active".to_string(), color: Some("#22c55e".to_string()), is_final: false },
                StateInfo { code: "completed".to_string(), name: "Completed".to_string(), color: Some("#3b82f6".to_string()), is_final: true },
                StateInfo { code: "cancelled".to_string(), name: "Cancelled".to_string(), color: Some("#ef4444".to_string()), is_final: true },
            ],
            transitions: vec![
                StateTransition { from: "draft".to_string(), to: "pending_landlord".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "pending_landlord".to_string(), to: "pending_tenant".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "pending_tenant".to_string(), to: "active".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "active".to_string(), to: "completed".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "*".to_string(), to: "cancelled".to_string(), conditions: vec![], actions: vec![] },
            ],
        }
    }
    
    /// Property state machine
    pub fn property_state_machine() -> StateMachineDefinition {
        StateMachineDefinition {
            entity_type: "property".to_string(),
            state_field: "status".to_string(),
            initial_state: "draft".to_string(),
            states: vec![
                StateInfo { code: "draft".to_string(), name: "Draft".to_string(), color: Some("#94a3b8".to_string()), is_final: false },
                StateInfo { code: "available".to_string(), name: "Available".to_string(), color: Some("#22c55e".to_string()), is_final: false },
                StateInfo { code: "reserved".to_string(), name: "Reserved".to_string(), color: Some("#f59e0b".to_string()), is_final: false },
                StateInfo { code: "rented".to_string(), name: "Rented".to_string(), color: Some("#3b82f6".to_string()), is_final: false },
                StateInfo { code: "sold".to_string(), name: "Sold".to_string(), color: Some("#8b5cf6".to_string()), is_final: true },
                StateInfo { code: "off_market".to_string(), name: "Off Market".to_string(), color: Some("#ef4444".to_string()), is_final: false },
            ],
            transitions: vec![
                StateTransition { from: "draft".to_string(), to: "available".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "available".to_string(), to: "reserved".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "reserved".to_string(), to: "rented".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "reserved".to_string(), to: "sold".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "rented".to_string(), to: "available".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "*".to_string(), to: "off_market".to_string(), conditions: vec![], actions: vec![] },
                StateTransition { from: "off_market".to_string(), to: "available".to_string(), conditions: vec![], actions: vec![] },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deal_transitions() {
        let sm = presets::deal_state_machine();
        
        assert!(sm.can_transition("lead", "qualified"));
        assert!(sm.can_transition("negotiation", "closed_lost")); // wildcard
        assert!(!sm.can_transition("closed_won", "lead")); // invalid
    }
    
    #[test]
    fn test_condition_evaluation() {
        let condition = TransitionCondition {
            field: "amount".to_string(),
            operator: "gt".to_string(),
            value: serde_json::json!(1000),
        };
        
        let data = serde_json::json!({"amount": 1500});
        assert!(evaluate_condition(&condition, &data));
        
        let data2 = serde_json::json!({"amount": 500});
        assert!(!evaluate_condition(&condition, &data2));
    }
}
