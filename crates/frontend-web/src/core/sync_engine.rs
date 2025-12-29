//! Sync Engine: Hybrid Zero-Latency Backend Integration
//! 
//! Supports TWO sync modes:
//! - Option A: Simple JSON Value (Last Write Wins) - For SmartField
//! - Option B: CRDT Delta (Real-time Collab) - For live editing

use leptos::*;
use serde::{Serialize, Deserialize};
use crate::api;

/// Sync Context provided at app root
#[derive(Clone, Copy)]
pub struct SyncContext {
    pub push_update: Callback<SyncOp>,
}

/// Represents a sync operation - supports BOTH simple values AND CRDTs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOp {
    pub entity_type: String,
    pub entity_id: String,
    pub field: String,
    /// Option A: Simple JSON value (Last Write Wins)
    pub value: Option<serde_json::Value>,
    /// Option B: Binary CRDT delta (for real-time collab)
    pub delta: Option<Vec<u8>>,
    pub timestamp: i64,
}

/// Provide the sync engine at app root
pub fn provide_sync_engine() {
    let (queue, set_queue) = create_signal(Vec::<SyncOp>::new());
    let (pending_count, set_pending) = create_signal(0usize);

    // The Hybrid Updater - handles both value and delta
    let push_update = Callback::new(move |op: SyncOp| {
        // Log what kind of update we are doing
        if let Some(ref v) = op.value {
            logging::log!("💾 Standard Update: {}.{} = {:?}", op.entity_type, op.field, v);
        } else if op.delta.is_some() {
            logging::log!("✨ CRDT Update: {}.{}", op.entity_type, op.field);
        }
        
        // Queue for Network Sync
        set_queue.update(|q| q.push(op.clone()));
        set_pending.update(|c| *c += 1);
        
        // Trigger Background Sync
        spawn_local(async move {
            process_sync_op(op).await;
            set_pending.update(|c| *c = c.saturating_sub(1));
        });
    });

    provide_context(SyncContext { push_update });
    provide_context(pending_count);
}

/// Process a sync operation - handles BOTH modes
async fn process_sync_op(op: SyncOp) {
    // Option B: CRDT Delta (Real-time Collab)
    if let Some(_delta) = op.delta {
        // TODO: Implement CRDT merge when backend supports it
        logging::log!("✅ CRDT Delta received for {}.{}", op.entity_type, op.field);
        return;
    }
    
    // Option A: Simple JSON update (Last Write Wins)
    if let Some(val) = op.value {
        let body = serde_json::json!({ op.field.clone(): val });
        let result = api::update_entity(&op.entity_type, &op.entity_id, body).await;
        
        match result {
            Ok(_) => logging::log!("✅ Synced: {}.{}", op.entity_type, op.field),
            Err(e) => logging::error!("❌ Sync Failed: {:?}", e),
        }
    }
}

/// Hook for components to use the sync engine
pub fn use_sync() -> SyncContext {
    use_context::<SyncContext>().expect("SyncContext not provided. Call provide_sync_engine() in App.")
}

/// Get pending sync count (for UI indicators)
pub fn use_pending_sync_count() -> ReadSignal<usize> {
    use_context::<ReadSignal<usize>>().expect("Pending count not found")
}

/// Convenience: Sync a simple JSON value (SmartField uses this)
pub fn sync_field_value(entity_type: &str, entity_id: &str, field: &str, value: serde_json::Value) {
    let sync = use_sync();
    let now = js_sys::Date::now() as i64;
    
    sync.push_update.call(SyncOp {
        entity_type: entity_type.to_string(),
        entity_id: entity_id.to_string(),
        field: field.to_string(),
        value: Some(value),
        delta: None,
        timestamp: now,
    });
}

/// Convenience: Sync a CRDT delta (for real-time collab)
pub fn sync_crdt_delta(entity_type: &str, entity_id: &str, field: &str, delta: Vec<u8>) {
    let sync = use_sync();
    let now = js_sys::Date::now() as i64;
    
    sync.push_update.call(SyncOp {
        entity_type: entity_type.to_string(),
        entity_id: entity_id.to_string(),
        field: field.to_string(),
        value: None,
        delta: Some(delta),
        timestamp: now,
    });
}

/// Helper: Convert string to JSON value for SmartField
pub fn string_to_value(value: &str) -> serde_json::Value {
    serde_json::Value::String(value.to_string())
}
