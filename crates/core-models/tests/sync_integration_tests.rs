//! Integration Tests for Sync Protocol

use core_models::sync::*;
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_sync_request_serialization() {
    let request = SyncRequest {
        client_id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        last_pulled_at: Some(Utc::now()),
        mutations: vec![
            Mutation::Create {
                temp_id: Uuid::new_v4(),
                entity_type: "contact".to_string(),
                field_values: serde_json::json!({"name": "John Doe"}),
                created_at: Utc::now(),
            }
        ],
        crdt_updates: vec![],
    };
    
    // Should serialize/deserialize correctly
    let json = serde_json::to_string(&request).unwrap();
    let deserialized: SyncRequest = serde_json::from_str(&json).unwrap();
    
    assert_eq!(request.client_id, deserialized.client_id);
    assert_eq!(request.mutations.len(), deserialized.mutations.len());
}

#[tokio::test]
async fn test_conflict_strategy() {
    let conflict = Conflict {
        entity_id: Uuid::new_v4(),
        entity_type: "deal".to_string(),
        field: "title".to_string(),
        client_value: serde_json::json!("Client Version"),
        client_version: 1,
        server_value: serde_json::json!("Server Version"),
        server_version: 2,
        strategy: ConflictStrategy::ServerWins,
    };
    
    // Serialize
    let json = serde_json::to_string(&conflict).unwrap();
    
    // Deserialize
    let deserialized: Conflict = serde_json::from_str(&json).unwrap();
    
    assert_eq!(conflict.entity_id, deserialized.entity_id);
    
    match deserialized.strategy {
        ConflictStrategy::ServerWins => {},
        _ => panic!("Wrong strategy"),
    }
}

#[tokio::test]
async fn test_sync_state_tracking() {
    let mut state = SyncState::new();
    
    assert_eq!(state.pending_mutations, 0);
    assert!(!state.is_syncing);
    
    // Start sync
    state.start_sync();
    assert!(state.is_syncing);
    
    // Add mutation
    state.add_pending_mutation();
    assert_eq!(state.pending_mutations, 1);
    
    // Sync success
    state.sync_success(Utc::now(), 1);
    assert!(!state.is_syncing);
    assert_eq!(state.pending_mutations, 0);
    assert!(state.last_pulled_at.is_some());
}

#[tokio::test]
async fn test_mutation_types() {
    // Create mutation
    let create = Mutation::Create {
        temp_id: Uuid::new_v4(),
        entity_type: "contact".to_string(),
        field_values: serde_json::json!({"name": "Test"}),
        created_at: Utc::now(),
    };
    
    // Update mutation
    let update = Mutation::Update {
        entity_id: Uuid::new_v4(),
        entity_type: "contact".to_string(),
        field_values: serde_json::json!({"name": "Updated"}),
        version: 2,
        updated_at: Utc::now(),
    };
    
    // Delete mutation
    let delete = Mutation::Delete {
        entity_id: Uuid::new_v4(),
        entity_type: "contact".to_string(),
        version: 3,
        deleted_at: Utc::now(),
    };
    
    // All should serialize
    assert!(serde_json::to_string(&create).is_ok());
    assert!(serde_json::to_string(&update).is_ok());
    assert!(serde_json::to_string(&delete).is_ok());
}
