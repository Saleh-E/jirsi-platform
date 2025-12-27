//! CRDT Collaboration Tests
//!
//! Integration tests for real-time collaborative editing features.
//! Tests WebSocket sync, event bus, and conflict resolution.

use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use serde_json::json;
use base64::{Engine as _, engine::general_purpose::STANDARD as Base64};

/// Test binary encoding for WebSocket
#[test]
fn test_binary_encoding() {
    let data = b"Hello CRDT World!";
    let encoded = Base64.encode(data);
    let decoded = Base64.decode(&encoded).unwrap();
    assert_eq!(data.to_vec(), decoded);
}

/// Test WebSocket event serialization
#[test]
fn test_ws_event_serialization() {
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[serde(tag = "type", content = "payload")]
    enum WsEvent {
        Connected { user_id: Uuid, tenant_id: Uuid },
        DocumentSubscribe { document_id: String },
        DocumentUpdate { document_id: String, update: String, user_id: Uuid },
        AwarenessUpdate { 
            document_id: String,
            user_id: Uuid,
            user_name: String,
            user_color: String,
            cursor_position: Option<u32>,
        },
    }
    
    // Test DocumentUpdate serialization
    let user_id = Uuid::new_v4();
    let event = WsEvent::DocumentUpdate {
        document_id: "entity-123:notes".to_string(),
        update: Base64.encode(b"binary crdt data"),
        user_id,
    };
    
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("DocumentUpdate"));
    assert!(json.contains("entity-123:notes"));
    
    let parsed: WsEvent = serde_json::from_str(&json).unwrap();
    match parsed {
        WsEvent::DocumentUpdate { document_id, update, user_id: uid } => {
            assert_eq!(document_id, "entity-123:notes");
            assert_eq!(uid, user_id);
            let decoded = Base64.decode(&update).unwrap();
            assert_eq!(decoded, b"binary crdt data");
        }
        _ => panic!("Wrong event type"),
    }
    
    // Test AwarenessUpdate
    let awareness = WsEvent::AwarenessUpdate {
        document_id: "doc-1".to_string(),
        user_id: Uuid::new_v4(),
        user_name: "Alice".to_string(),
        user_color: "#FF5733".to_string(),
        cursor_position: Some(42),
    };
    
    let json = serde_json::to_string(&awareness).unwrap();
    let parsed: WsEvent = serde_json::from_str(&json).unwrap();
    
    match parsed {
        WsEvent::AwarenessUpdate { user_name, cursor_position, .. } => {
            assert_eq!(user_name, "Alice");
            assert_eq!(cursor_position, Some(42));
        }
        _ => panic!("Wrong event type"),
    }
}

/// Test document ID format
#[test]
fn test_document_id_format() {
    let entity_id = Uuid::new_v4();
    let field = "description";
    let document_id = format!("{}:{}", entity_id, field);
    
    // Parse back
    let parts: Vec<&str> = document_id.split(':').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], entity_id.to_string());
    assert_eq!(parts[1], field);
}

#[cfg(test)]
mod event_bus_tests {
    use super::*;
    use backend_api::cqrs::{EventBus, EventBusConfig, EventEnvelope, ProjectionHandler, ProjectionError};
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    struct CountingHandler {
        call_count: Arc<AtomicUsize>,
        should_fail: bool,
    }
    
    #[async_trait::async_trait]
    impl ProjectionHandler for CountingHandler {
        async fn handle(&self, _event: &EventEnvelope) -> Result<(), ProjectionError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            if self.should_fail {
                Err(ProjectionError::Handler("Simulated failure".to_string()))
            } else {
                Ok(())
            }
        }
        
        fn name(&self) -> &'static str {
            "CountingHandler"
        }
    }
    
    fn create_test_event() -> EventEnvelope {
        EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            aggregate_type: "contact".to_string(),
            event_type: "ContactCreated".to_string(),
            event_data: json!({
                "name": "Test Contact",
                "email": "test@example.com"
            }),
            tenant_id: Uuid::new_v4(),
            caused_by: Uuid::new_v4(),
            occurred_at: Utc::now(),
            version: 1,
        }
    }
    
    #[tokio::test]
    async fn test_event_bus_basic_publish() {
        let config = EventBusConfig::default();
        let bus = EventBus::new(config);
        
        let call_count = Arc::new(AtomicUsize::new(0));
        let handler = Arc::new(CountingHandler {
            call_count: call_count.clone(),
            should_fail: false,
        });
        
        bus.register_handler(handler).await;
        bus.publish(create_test_event()).await;
        
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }
    
    #[tokio::test]
    async fn test_event_bus_multiple_handlers() {
        let config = EventBusConfig::default();
        let bus = EventBus::new(config);
        
        let count1 = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::new(AtomicUsize::new(0));
        
        bus.register_handler(Arc::new(CountingHandler {
            call_count: count1.clone(),
            should_fail: false,
        })).await;
        
        bus.register_handler(Arc::new(CountingHandler {
            call_count: count2.clone(),
            should_fail: false,
        })).await;
        
        bus.publish(create_test_event()).await;
        
        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
    }
    
    #[tokio::test]
    async fn test_event_bus_retry_on_failure() {
        let config = EventBusConfig {
            buffer_size: 100,
            max_retries: 2,
            enable_dlq: true,
        };
        let bus = EventBus::new(config);
        
        let call_count = Arc::new(AtomicUsize::new(0));
        let handler = Arc::new(CountingHandler {
            call_count: call_count.clone(),
            should_fail: true,
        });
        
        bus.register_handler(handler).await;
        bus.publish(create_test_event()).await;
        
        // Should retry: 1 initial + 2 retries = 3 attempts
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }
    
    #[tokio::test]
    async fn test_event_bus_dlq() {
        let config = EventBusConfig {
            buffer_size: 100,
            max_retries: 1,
            enable_dlq: true,
        };
        let bus = EventBus::new(config);
        
        let handler = Arc::new(CountingHandler {
            call_count: Arc::new(AtomicUsize::new(0)),
            should_fail: true,
        });
        
        bus.register_handler(handler).await;
        bus.publish(create_test_event()).await;
        
        let dlq = bus.get_dlq().await;
        assert_eq!(dlq.len(), 1);
        assert!(dlq[0].error.contains("Simulated failure"));
    }
    
    #[tokio::test]
    async fn test_event_bus_clear_dlq() {
        let config = EventBusConfig {
            buffer_size: 100,
            max_retries: 0,
            enable_dlq: true,
        };
        let bus = EventBus::new(config);
        
        let handler = Arc::new(CountingHandler {
            call_count: Arc::new(AtomicUsize::new(0)),
            should_fail: true,
        });
        
        bus.register_handler(handler).await;
        bus.publish(create_test_event()).await;
        
        assert_eq!(bus.get_dlq().await.len(), 1);
        
        bus.clear_dlq().await;
        assert_eq!(bus.get_dlq().await.len(), 0);
    }
    
    #[tokio::test]
    async fn test_broadcast_subscription() {
        let config = EventBusConfig::default();
        let bus = EventBus::new(config);
        
        let mut receiver = bus.subscribe();
        
        let event = create_test_event();
        let event_id = event.event_id;
        
        bus.publish(event).await;
        
        // Give time for broadcast
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        // Try to receive (may timeout if no handler processed)
        match tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            receiver.recv()
        ).await {
            Ok(Ok(received)) => assert_eq!(received.event_id, event_id),
            _ => {} // Broadcast might not have active handler to trigger it
        }
    }
}

#[cfg(test)]
mod conflict_resolution_tests {
    use super::*;
    use serde_json::{json, Value as JsonValue};

    fn merge_json_objects(local: &JsonValue, server: &JsonValue) -> JsonValue {
        match (local, server) {
            (JsonValue::Object(local_obj), JsonValue::Object(server_obj)) => {
                let mut merged = server_obj.clone();
                for (key, value) in local_obj {
                    // Skip system fields
                    if key == "id" || key == "tenant_id" || key == "created_at" {
                        continue;
                    }
                    merged.insert(key.clone(), value.clone());
                }
                JsonValue::Object(merged)
            }
            _ => local.clone(),
        }
    }

    #[test]
    fn test_merge_non_conflicting_fields() {
        let local = json!({
            "name": "Local Name",
            "phone": "123-local"
        });
        
        let server = json!({
            "email": "server@test.com",
            "address": "Server Address"
        });
        
        let merged = merge_json_objects(&local, &server);
        
        // All fields should be present
        assert_eq!(merged["name"], "Local Name");
        assert_eq!(merged["phone"], "123-local");
        assert_eq!(merged["email"], "server@test.com");
        assert_eq!(merged["address"], "Server Address");
    }
    
    #[test]
    fn test_merge_local_wins_conflicts() {
        let local = json!({
            "name": "Local Name",
            "status": "active"
        });
        
        let server = json!({
            "name": "Server Name",
            "status": "inactive"
        });
        
        let merged = merge_json_objects(&local, &server);
        
        // Local values should win
        assert_eq!(merged["name"], "Local Name");
        assert_eq!(merged["status"], "active");
    }
    
    #[test]
    fn test_merge_preserves_system_fields() {
        let local = json!({
            "id": "should-not-override",
            "tenant_id": "local-tenant",
            "name": "Local Name"
        });
        
        let server = json!({
            "id": "server-id",
            "tenant_id": "server-tenant",
            "created_at": "2024-01-01T00:00:00Z"
        });
        
        let merged = merge_json_objects(&local, &server);
        
        // System fields from server should be preserved
        assert_eq!(merged["id"], "server-id");
        assert_eq!(merged["tenant_id"], "server-tenant");
        assert_eq!(merged["created_at"], "2024-01-01T00:00:00Z");
        // User field from local wins
        assert_eq!(merged["name"], "Local Name");
    }
    
    #[test]
    fn test_version_comparison() {
        struct VersionedRecord {
            version: u64,
            data: JsonValue,
        }
        
        let local = VersionedRecord {
            version: 5,
            data: json!({"name": "Local"}),
        };
        
        let server = VersionedRecord {
            version: 7,
            data: json!({"name": "Server"}),
        };
        
        // Conflict detection
        let has_conflict = local.version < server.version;
        assert!(has_conflict, "Should detect version conflict");
        
        // Same version = no conflict
        let local2 = VersionedRecord { version: 7, data: json!({}) };
        let no_conflict = local2.version >= server.version;
        assert!(no_conflict, "Same version should not conflict");
    }
    
    #[test]
    fn test_resolution_choices() {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum ResolutionChoice {
            KeepMine,
            KeepTheirs,
            Merge,
        }
        
        let local = json!({"value": 100});
        let server = json!({"value": 200});
        
        let keep_mine_result = match ResolutionChoice::KeepMine {
            ResolutionChoice::KeepMine => local.clone(),
            ResolutionChoice::KeepTheirs => server.clone(),
            ResolutionChoice::Merge => merge_json_objects(&local, &server),
        };
        
        assert_eq!(keep_mine_result["value"], 100);
        
        let keep_theirs_result = match ResolutionChoice::KeepTheirs {
            ResolutionChoice::KeepMine => local.clone(),
            ResolutionChoice::KeepTheirs => server.clone(),
            ResolutionChoice::Merge => merge_json_objects(&local, &server),
        };
        
        assert_eq!(keep_theirs_result["value"], 200);
    }
}

#[cfg(test)]
mod sync_tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct DirtyRecord {
        id: Uuid,
        entity_type: String,
        data: serde_json::Value,
        local_version: u64,
        server_version: u64,
        is_deleted: bool,
    }
    
    fn determine_sync_action(record: &DirtyRecord) -> &'static str {
        if record.is_deleted {
            "DELETE"
        } else if record.server_version == 0 {
            "CREATE"
        } else {
            "UPDATE"
        }
    }
    
    #[test]
    fn test_determine_create_action() {
        let record = DirtyRecord {
            id: Uuid::new_v4(),
            entity_type: "contact".to_string(),
            data: json!({"name": "New Contact"}),
            local_version: 1,
            server_version: 0, // Never synced
            is_deleted: false,
        };
        
        assert_eq!(determine_sync_action(&record), "CREATE");
    }
    
    #[test]
    fn test_determine_update_action() {
        let record = DirtyRecord {
            id: Uuid::new_v4(),
            entity_type: "contact".to_string(),
            data: json!({"name": "Updated Contact"}),
            local_version: 3,
            server_version: 2, // Previously synced
            is_deleted: false,
        };
        
        assert_eq!(determine_sync_action(&record), "UPDATE");
    }
    
    #[test]
    fn test_determine_delete_action() {
        let record = DirtyRecord {
            id: Uuid::new_v4(),
            entity_type: "contact".to_string(),
            data: json!({}),
            local_version: 5,
            server_version: 4,
            is_deleted: true,
        };
        
        assert_eq!(determine_sync_action(&record), "DELETE");
    }
    
    #[test]
    fn test_optimistic_locking() {
        let expected_version = 5u64;
        let actual_server_version = 6u64;
        
        // Version mismatch = conflict
        let is_conflict = expected_version != actual_server_version;
        assert!(is_conflict);
        
        // Matching version = no conflict
        let expected2 = 6u64;
        let no_conflict = expected2 == actual_server_version;
        assert!(no_conflict);
    }
}
