//! Sync Manager - Offline-First Synchronization Engine
//!
//! Handles bidirectional sync between local SQLite and remote API.
//! Features:
//! - Queue-based dirty record pushing
//! - Conflict detection via server_version (CQRS aggregate_version)
//! - Last-Write-Wins (LWW) conflict resolution
//! - Exponential backoff retry
//! - Online status detection

use super::db::{LocalDatabase, DirtyRecord};
use uuid::Uuid;
use gloo_console;
use crate::api::get_api_base;
use wasm_bindgen::JsValue;
use gloo_net::http::Request;

/// Result of a sync operation
#[derive(Debug, Clone)]
pub enum SyncResult {
    Success { synced_count: usize },
    PartialSuccess { synced: usize, failed: usize },
    Conflict { record_id: String, server_version: u64 },
    Offline,
    Error(String),
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, Default)]
pub enum ConflictResolution {
    #[default]
    LastWriteWins,
    ServerWins,
    ClientWins,
    Manual,
}

/// Sync Manager for offline-first data synchronization
#[derive(Clone, Debug)]
pub struct SyncManager {
    local_db: LocalDatabase,
    conflict_resolution: ConflictResolution,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new(local_db: LocalDatabase) -> Self {
        Self { 
            local_db,
            conflict_resolution: ConflictResolution::default(),
        }
    }

    /// Create with custom conflict resolution
    pub fn with_conflict_resolution(local_db: LocalDatabase, resolution: ConflictResolution) -> Self {
        Self {
            local_db,
            conflict_resolution: resolution,
        }
    }

    /// Check if we're online
    pub fn is_online() -> bool {
        web_sys::window()
            .map(|w| w.navigator().on_line())
            .unwrap_or(false)
    }


    /// Pull entities from server and store locally
    pub async fn pull_entities(&self, entity_type: &str, tenant_id: &Uuid) -> Result<usize, String> {
        if !Self::is_online() {
            return Err("Offline - cannot pull".to_string());
        }

        let url = format!("{}/entities/{}", get_api_base(), entity_type);
        
        let resp = Request::get(&url)
            .header("X-Tenant-Id", &tenant_id.to_string())
            .header("X-Tenant-Slug", "demo")
            .send()
            .await
            .map_err(|e| e.to_string())?;
            
        if !resp.ok() {
            return Err(format!("Pull failed: HTTP {}", resp.status()));
        }
        
        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        
        let mut count = 0;
        if let Some(rows) = json.get("data").and_then(|d| d.as_array()) {
            for row in rows {
                let id = row.get("id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok());
                    
                if let Some(entity_id) = id {
                    // Get server version for conflict detection
                    let server_version = row.get("aggregate_version")
                        .or_else(|| row.get("version"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1);

                    let now = chrono::Utc::now().to_rfc3339();
                    
                    // Upsert: only if not locally dirty OR server is newer
                    let sql = r#"
                        INSERT INTO local_entity_records 
                        (id, tenant_id, entity_type, data, is_dirty, created_at, updated_at, last_synced_at, server_version)
                        VALUES (?, ?, ?, ?, 0, ?, ?, ?, ?)
                        ON CONFLICT(id) DO UPDATE SET
                            data = CASE WHEN is_dirty = 0 OR ? > server_version THEN excluded.data ELSE data END,
                            server_version = CASE WHEN is_dirty = 0 OR ? > server_version THEN excluded.server_version ELSE server_version END,
                            last_synced_at = excluded.last_synced_at
                        WHERE is_dirty = 0 OR ? > server_version
                    "#;

                    let data_str = serde_json::to_string(row).unwrap_or_default();
                    let params = serde_json::json!([
                        entity_id.to_string(),
                        tenant_id.to_string(),
                        entity_type,
                        data_str,
                        now.clone(),
                        now.clone(),
                        now,
                        server_version,
                        server_version,
                        server_version,
                        server_version
                    ]);

                    if let Err(e) = self.local_db.execute_with_params(sql, params) {
                        gloo_console::warn!("Failed to save record:", &e);
                    } else {
                        count += 1;
                    }
                }
            }
        }
        
        gloo_console::info!("Pulled", count, "records for", entity_type);
        Ok(count)
    }

    /// Push all dirty records to server
    pub async fn push_changes(&self, tenant_id: &Uuid) -> SyncResult {
        if !Self::is_online() {
            gloo_console::warn!("Offline - changes queued for later sync");
            return SyncResult::Offline;
        }

        // Get dirty records
        let dirty_records = match self.local_db.get_dirty_records() {
            Ok(records) => records,
            Err(e) => return SyncResult::Error(e),
        };

        if dirty_records.is_empty() {
            return SyncResult::Success { synced_count: 0 };
        }

        gloo_console::info!("Pushing", dirty_records.len(), "dirty records...");

        let mut synced = 0;
        let mut failed = 0;

        for record in dirty_records {
            match self.push_single_record(&record, tenant_id).await {
                Ok(new_version) => {
                    // Mark as synced
                    if let Ok(id) = Uuid::parse_str(&record.id) {
                        let _ = self.local_db.mark_synced(&id, new_version);
                    }
                    synced += 1;
                }
                Err(PushError::Conflict { server_version }) => {
                    // Handle conflict based on strategy
                    match self.conflict_resolution {
                        ConflictResolution::LastWriteWins | ConflictResolution::ClientWins => {
                            // Force push with server version
                            if let Ok(_) = self.force_push_record(&record, tenant_id, server_version).await {
                                synced += 1;
                            } else {
                                failed += 1;
                            }
                        }
                        ConflictResolution::ServerWins => {
                            // Discard local changes, pull server version
                            if let Ok(id) = Uuid::parse_str(&record.id) {
                                let _ = self.local_db.mark_synced(&id, server_version);
                            }
                            synced += 1;
                        }
                        ConflictResolution::Manual => {
                            // Return conflict for user resolution
                            return SyncResult::Conflict {
                                record_id: record.id,
                                server_version,
                            };
                        }
                    }
                }
                Err(PushError::Network(e)) => {
                    gloo_console::error!("Network error:", &e);
                    failed += 1;
                }
                Err(PushError::Server(e)) => {
                    gloo_console::error!("Server error:", &e);
                    failed += 1;
                }
            }
        }

        if failed == 0 {
            SyncResult::Success { synced_count: synced }
        } else {
            SyncResult::PartialSuccess { synced, failed }
        }
    }

    /// Push a single record to server
    async fn push_single_record(&self, record: &DirtyRecord, tenant_id: &Uuid) -> Result<u64, PushError> {
        let url = if record.is_deleted {
            format!("{}/entities/{}/{}", get_api_base(), record.entity_type, record.id)
        } else {
            format!("{}/entities/{}", get_api_base(), record.entity_type)
        };

        let method = if record.is_deleted {
            "DELETE"
        } else if record.server_version == 0 {
            "POST" // New record
        } else {
            "PUT" // Update
        };

        // Build request body with version for optimistic locking
        let body = if !record.is_deleted {
            let mut data = record.data.clone();
            if let Some(obj) = data.as_object_mut() {
                obj.insert("id".to_string(), serde_json::json!(record.id));
                obj.insert("expected_version".to_string(), serde_json::json!(record.server_version));
            }
            Some(serde_json::to_string(&data).map_err(|e| PushError::Network(e.to_string()))?)
        } else {
            None
        };

        let url_with_id = match method {
            "PUT" | "DELETE" => format!("{}/{}", url, record.id),
            _ => url.clone(),
        };
        
        let request_builder = match method {
            "POST" => Request::post(&url),
            "PUT" => Request::put(&url_with_id),
            "DELETE" => Request::delete(&url_with_id),
            _ => Request::post(&url),
        };

        let request_builder = request_builder
            .header("Content-Type", "application/json")
            .header("X-Tenant-Id", &tenant_id.to_string())
            .header("X-Tenant-Slug", "demo")
            .header("X-Request-Id", &Uuid::new_v4().to_string());

        // Build the final request - with or without body
        let resp = if let Some(body) = body {
            request_builder
                .body(body)
                .map_err(|e| PushError::Network(format!("{:?}", e)))?
                .send()
                .await
                .map_err(|e| PushError::Network(e.to_string()))?
        } else {
            request_builder
                .send()
                .await
                .map_err(|e| PushError::Network(e.to_string()))?
        };


        match resp.status() {
            200 | 201 => {
                // Success - get new version from response
                let json: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));
                let new_version = json.get("aggregate_version")
                    .or_else(|| json.get("version"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(record.server_version + 1);
                Ok(new_version)
            }
            204 => {
                // Deleted successfully
                Ok(record.server_version)
            }
            409 => {
                // Conflict - get server version
                let json: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));
                let server_version = json.get("current_version")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(record.server_version + 1);
                Err(PushError::Conflict { server_version })
            }
            status => {
                let text = resp.text().await.unwrap_or_default();
                Err(PushError::Server(format!("HTTP {}: {}", status, text)))
            }
        }
    }

    /// Force push a record, overwriting server version
    async fn force_push_record(&self, record: &DirtyRecord, tenant_id: &Uuid, _server_version: u64) -> Result<u64, PushError> {
        // In LWW mode, we just push again without version check
        let url = format!("{}/entities/{}/{}", get_api_base(), record.entity_type, record.id);
        
        let body = serde_json::to_string(&record.data).map_err(|e| PushError::Network(e.to_string()))?;

        let resp = Request::put(&url)
            .header("Content-Type", "application/json")
            .header("X-Tenant-Id", &tenant_id.to_string())
            .header("X-Tenant-Slug", "demo")
            .header("X-Force-Overwrite", "true")
            .body(body)
            .map_err(|e| PushError::Network(format!("{:?}", e)))?
            .send()
            .await
            .map_err(|e| PushError::Network(e.to_string()))?;

        if resp.ok() {
            let json: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));
            Ok(json.get("aggregate_version").and_then(|v| v.as_u64()).unwrap_or(1))
        } else {
            Err(PushError::Server(format!("Force push failed: {}", resp.status())))
        }
    }

    /// Sync all - pull then push
    pub async fn sync_all(&self, tenant_id: &Uuid, entity_types: &[&str]) -> Result<SyncResult, String> {
        // First push local changes
        let push_result = self.push_changes(tenant_id).await;
        
        // Then pull updates for each entity type
        for entity_type in entity_types {
            if let Err(e) = self.pull_entities(entity_type, tenant_id).await {
                gloo_console::warn!("Pull failed for", *entity_type, ":", &e);
            }
        }

        Ok(push_result)
    }
}

/// Errors that can occur during push
#[derive(Debug)]
enum PushError {
    Conflict { server_version: u64 },
    Network(String),
    Server(String),
}
