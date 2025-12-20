
use super::db::LocalDatabase;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SyncManager {
    local_db: LocalDatabase,
}

impl SyncManager {
    pub fn new(local_db: LocalDatabase) -> Self {
        Self { local_db }
    }

    pub async fn pull_entities(&self, entity_type: &str) -> Result<(), String> {
        // 1. Fetch from server API
        // 2. Insert into LocalDatabase
        Ok(())
    }

    pub async fn push_changes(&self) -> Result<(), String> {
        // 1. Read pending changes from LocalDatabase
        // 2. POST/PATCH to server API
        Ok(())
    }
}
