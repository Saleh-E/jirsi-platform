//! Sync Worker - Rust wrapper for SQLite Web Worker communication

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::atomic::{AtomicU64, Ordering};

static MESSAGE_ID: AtomicU64 = AtomicU64::new(0);

#[wasm_bindgen(module = "/public/workers/sync-worker.js")]
extern "C" {
    pub type SyncWorker;
    
    #[wasm_bindgen(constructor)]
    pub fn new() -> SyncWorker;
    
    #[wasm_bindgen(method)]
    pub fn postMessage(this: &SyncWorker, message: &JsValue);
    
    #[wasm_bindgen(method, setter)]
    pub fn set_onmessage(this: &SyncWorker, callback: &Closure<dyn FnMut(JsValue)>);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerMessage {
    pub id: u64,
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResponse {
    pub id: u64,
    #[serde(rename = "type")]
    pub response_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub struct SyncWorkerClient {
    worker: SyncWorker,
    _callback: Closure<dyn FnMut(JsValue)>,
}

impl SyncWorkerClient {
    pub fn new() -> Self {
        let worker = SyncWorker::new();
        
        let callback = Closure::wrap(Box::new(move |msg: JsValue| {
            // Handle worker messages
            if let Ok(response) = serde_wasm_bindgen::from_value::<WorkerResponse>(msg) {
                tracing::info!("Worker response: {:?}", response);
            }
        }) as Box<dyn FnMut(JsValue)>);
        
        worker.set_onmessage(&callback);
        
        Self {
            worker,
            _callback: callback,
        }
    }
    
    pub fn create_entity(&self, entity_type: &str, field_values: serde_json::Value) -> Uuid {
        let id = Uuid::new_v4();
        
        let msg = WorkerMessage {
            id: MESSAGE_ID.fetch_add(1, Ordering::SeqCst),
            msg_type: "create_entity".to_string(),
            payload: Some(serde_json::json!({
                "id": id,
                "entity_type": entity_type,
                "field_values": field_values
            })),
        };
        
        let js_msg = serde_wasm_bindgen::to_value(&msg).unwrap();
        self.worker.postMessage(&js_msg);
        
        id
    }
    
    pub fn update_entity(&self, id: Uuid, field_values: serde_json::Value, version: u64) {
        let msg = WorkerMessage {
            id: MESSAGE_ID.fetch_add(1, Ordering::SeqCst),
            msg_type: "update_entity".to_string(),
            payload: Some(serde_json::json!({
                "id": id,
                "field_values": field_values,
                "version": version
            })),
        };
        
        let js_msg = serde_wasm_bindgen::to_value(&msg).unwrap();
        self.worker.postMessage(&js_msg);
    }
    
    pub fn delete_entity(&self, id: Uuid, version: u64) {
        let msg = WorkerMessage {
            id: MESSAGE_ID.fetch_add(1, Ordering::SeqCst),
            msg_type: "delete_entity".to_string(),
            payload: Some(serde_json::json!({
                "id": id,
                "version": version
            })),
        };
        
        let js_msg = serde_wasm_bindgen::to_value(&msg).unwrap();
        self.worker.postMessage(&js_msg);
    }
    
    pub fn get_pending_mutations(&self) {
        let msg = WorkerMessage {
            id: MESSAGE_ID.fetch_add(1, Ordering::SeqCst),
            msg_type: "get_pending_mutations".to_string(),
            payload: None,
        };
        
        let js_msg = serde_wasm_bindgen::to_value(&msg).unwrap();
        self.worker.postMessage(&js_msg);
    }
}
