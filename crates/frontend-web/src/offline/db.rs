//! Local Database - OPFS-backed SQLite for offline-first functionality
//!
//! Provides a JavaScript-bridged interface to SQLite WASM running on OPFS.
//! Supports dirty record tracking for sync queue management.

use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::to_value;
use uuid::Uuid;

#[wasm_bindgen(module = "/assets/offline_adapter.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn initOfflineDb() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    fn executeSql(sql: &str, params: JsValue) -> Result<JsValue, JsValue>;
}

/// Local database for offline-first functionality
#[derive(Clone, Debug)]
pub struct LocalDatabase {
    // We don't hold the JS object directly here, it's global in JS for now
    // In a cleaner implementation we'd pass a handle
    initialized: bool,
}

impl LocalDatabase {
    /// Initialize the local database
    pub async fn new() -> Result<Self, String> {
        match initOfflineDb().await {
            Ok(val) => {
                if val.is_truthy() {
                    let db = Self { initialized: true };
                    db.ensure_schema()?;
                    Ok(db)
                } else {
                    Err("Failed to init OPFS DB".to_string())
                }
            },
            Err(e) => Err(format!("JS Error: {:?}", e))
        }
    }

    /// Ensure required tables exist
    fn ensure_schema(&self) -> Result<(), String> {
        // Main records table with dirty tracking
        self.execute(r#"
            CREATE TABLE IF NOT EXISTS local_entity_records (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                data TEXT NOT NULL,
                is_dirty INTEGER DEFAULT 0,
                is_deleted INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_synced_at TEXT,
                server_version INTEGER DEFAULT 0
            )
        "#)?;

        // Create indexes for efficient querying
        self.execute("CREATE INDEX IF NOT EXISTS idx_dirty ON local_entity_records(is_dirty)")?;
        self.execute("CREATE INDEX IF NOT EXISTS idx_entity_type ON local_entity_records(entity_type, tenant_id)")?;

        // Pending operations queue for offline actions
        self.execute(r#"
            CREATE TABLE IF NOT EXISTS pending_operations (
                id TEXT PRIMARY KEY,
                operation_type TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                payload TEXT NOT NULL,
                created_at TEXT NOT NULL,
                retry_count INTEGER DEFAULT 0
            )
        "#)?;

        Ok(())
    }

    /// Execute SQL without parameters
    pub fn execute(&self, sql: &str) -> Result<(), String> {
        executeSql(sql, JsValue::NULL)
            .map(|_| ())
            .map_err(|e| format!("{:?}", e))
    }

    /// Execute SQL with parameters
    pub fn execute_with_params(&self, sql: &str, params: serde_json::Value) -> Result<(), String> {
        let js_params = to_value(&params).map_err(|e| e.to_string())?;
        executeSql(sql, js_params)
            .map(|_| ())
            .map_err(|e| format!("{:?}", e))
    }
    
    /// Query and return results
    pub fn query(&self, sql: &str) -> Result<Vec<serde_json::Value>, String> {
        let res = executeSql(sql, JsValue::NULL)
            .map_err(|e| format!("{:?}", e))?;
            
        serde_wasm_bindgen::from_value(res)
            .map_err(|e| e.to_string())
    }

    /// Save a record locally (marks as dirty for sync)
    pub fn save_record(
        &self,
        entity_type: &str,
        entity_id: &Uuid,
        tenant_id: &Uuid,
        data: &serde_json::Value,
    ) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        let data_str = serde_json::to_string(data).map_err(|e| e.to_string())?;

        let sql = r#"
            INSERT OR REPLACE INTO local_entity_records 
            (id, tenant_id, entity_type, data, is_dirty, created_at, updated_at)
            VALUES (?, ?, ?, ?, 1, COALESCE((SELECT created_at FROM local_entity_records WHERE id = ?), ?), ?)
        "#;

        let params = serde_json::json!([
            entity_id.to_string(),
            tenant_id.to_string(),
            entity_type,
            data_str,
            entity_id.to_string(),
            now.clone(),
            now
        ]);

        self.execute_with_params(sql, params)
    }

    /// Mark a record as deleted (soft delete)
    pub fn delete_record(&self, entity_id: &Uuid) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        let sql = "UPDATE local_entity_records SET is_deleted = 1, is_dirty = 1, updated_at = ? WHERE id = ?";
        let params = serde_json::json!([now, entity_id.to_string()]);
        self.execute_with_params(sql, params)
    }

    /// Get all dirty records that need to be synced
    pub fn get_dirty_records(&self) -> Result<Vec<DirtyRecord>, String> {
        let rows = self.query("SELECT * FROM local_entity_records WHERE is_dirty = 1")?;
        
        rows.into_iter()
            .map(|row| {
                Ok(DirtyRecord {
                    id: row.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    tenant_id: row.get("tenant_id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    entity_type: row.get("entity_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    data: row.get("data").cloned().unwrap_or(serde_json::Value::Null),
                    is_deleted: row.get("is_deleted").and_then(|v| v.as_i64()).unwrap_or(0) == 1,
                    server_version: row.get("server_version").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
                })
            })
            .collect()
    }

    /// Mark record as synced (clear dirty flag)
    pub fn mark_synced(&self, entity_id: &Uuid, server_version: u64) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        let sql = "UPDATE local_entity_records SET is_dirty = 0, last_synced_at = ?, server_version = ? WHERE id = ?";
        let params = serde_json::json!([now, server_version, entity_id.to_string()]);
        self.execute_with_params(sql, params)
    }

    /// Add operation to pending queue (for offline actions)
    pub fn queue_operation(
        &self,
        operation_type: &str,
        entity_type: &str,
        entity_id: &Uuid,
        payload: &serde_json::Value,
    ) -> Result<(), String> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();
        let payload_str = serde_json::to_string(payload).map_err(|e| e.to_string())?;

        let sql = r#"
            INSERT INTO pending_operations (id, operation_type, entity_type, entity_id, payload, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
        "#;

        let params = serde_json::json!([
            id.to_string(),
            operation_type,
            entity_type,
            entity_id.to_string(),
            payload_str,
            now
        ]);

        self.execute_with_params(sql, params)
    }

    /// Get pending operations
    pub fn get_pending_operations(&self) -> Result<Vec<serde_json::Value>, String> {
        self.query("SELECT * FROM pending_operations ORDER BY created_at ASC")
    }

    /// Remove operation from queue after successful sync
    pub fn remove_operation(&self, operation_id: &str) -> Result<(), String> {
        let sql = "DELETE FROM pending_operations WHERE id = ?";
        let params = serde_json::json!([operation_id]);
        self.execute_with_params(sql, params)
    }
}

/// Represents a dirty record that needs to be synced
#[derive(Debug, Clone)]
pub struct DirtyRecord {
    pub id: String,
    pub tenant_id: String,
    pub entity_type: String,
    pub data: serde_json::Value,
    pub is_deleted: bool,
    pub server_version: u64,
}
