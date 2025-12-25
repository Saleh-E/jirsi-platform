
use super::db::LocalDatabase;
use uuid::Uuid;
use gloo_console;
use crate::api::get_api_base;

#[derive(Clone, Debug)]
pub struct SyncManager {
    local_db: LocalDatabase,
}

impl SyncManager {
    pub fn new(local_db: LocalDatabase) -> Self {
        Self { local_db }
    }

    pub async fn pull_entities(&self, entity_type: &str, tenant_id: &Uuid) -> Result<(), String> {
        // 1. Fetch from server API (mocked for now, assumes API exists)
        // In real impl: fetch /api/v1/sync/pull?entity_type=...&since=...
        // For MVP: We just fetch the list via regular API and upsert all
        
        let url = format!("{}/records/{}?tenant_id={}&limit=1000", get_api_base(), entity_type, tenant_id);
        let resp = gloo_net::http::Request::get(&url)
            .header("X-Tenant-Id", &tenant_id.to_string())
            .header("X-Tenant-Slug", "demo")
            .send()
            .await
            .map_err(|e| e.to_string())?;
            
        if !resp.ok() {
            return Err(format!("Failed to fetch {}: {}", entity_type, resp.status()));
        }
        
        let json: serde_json::Value = resp.json::<serde_json::Value>().await.map_err(|e| e.to_string())?;
        
        if let Some(rows) = json.get("data").and_then(|d: &serde_json::Value| d.as_array()) {
            for row in rows {
                let id = row.get("id").and_then(|v: &serde_json::Value| v.as_str()).unwrap_or_default();
                let _data_str = serde_json::to_string(row).unwrap_or_default();
                let now = chrono::Utc::now().to_rfc3339();
                
                // Upsert into local DB
                // We use REPLACE INTO to overwrite
                let sql = "INSERT OR REPLACE INTO local_entity_records (id, tenant_id, entity_type, data, created_at, updated_at, last_synced_at) VALUES (?, ?, ?, ?, ?, ?, ?)";
                
                let params = serde_json::json!([
                    id,
                    tenant_id.to_string(),
                    entity_type,
                    row, // data (as json)
                    now, // created
                    now, // updated (using now for local mirror)
                    now
                ]);

                if let Err(e) = self.local_db.execute_with_params(sql, params) {
                    gloo_console::error!("Failed to save record:", e);
                }
            }
        }
        
        Ok(())
    }

    pub async fn push_changes(&self) -> Result<(), String> {
        // 1. Read pending changes from LocalDatabase
        let dirty_rows = self.local_db.query("SELECT * FROM local_entity_records WHERE is_dirty = 1")?;
        
        if dirty_rows.is_empty() {
            return Ok(());
        }
        
        // 2. POST to server
        // ... implementation pending valid API endpoint ...
        
        Ok(())
    }
}
