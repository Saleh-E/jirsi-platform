//! Delta Sync Protocol
//! 
//! For local-first offline operation with eventual consistency

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Client sync request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    /// Client ID (device/browser identifier)
    pub client_id: Uuid,
    
    /// Tenant ID
    pub tenant_id: Uuid,
    
    /// Last successful pull timestamp
    pub last_pulled_at: Option<DateTime<Utc>>,
    
    /// Pending mutations from client
    pub mutations: Vec<Mutation>,
    
    /// CRDT updates for collaborative fields
    #[serde(default)]
    pub crdt_updates: Vec<CrdtUpdate>,
}

/// Client sync response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    /// Server's latest timestamp
    pub server_timestamp: DateTime<Utc>,
    
    /// Changes from server since last_pulled_at
    pub changes: Vec<ServerChange>,
    
    /// CRDT updates for collaborative fields
    #[serde(default)]
    pub crdt_updates: Vec<CrdtUpdate>,
    
    /// Conflicts that need resolution
    #[serde(default)]
    pub conflicts: Vec<Conflict>,
}

/// Client mutation (create, update, delete)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Mutation {
    Create {
        temp_id: Uuid,
        entity_type: String,
        field_values: serde_json::Value,
        created_at: DateTime<Utc>,
    },
    Update {
        entity_id: Uuid,
        entity_type: String,
        field_values: serde_json::Value,
        version: u64,
        updated_at: DateTime<Utc>,
    },
    Delete {
        entity_id: Uuid,
        entity_type: String,
        version: u64,
        deleted_at: DateTime<Utc>,
    },
}

/// Server change (delta)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerChange {
    Created {
        entity_id: Uuid,
        entity_type: String,
        field_values: serde_json::Value,
        created_at: DateTime<Utc>,
        version: u64,
    },
    Updated {
        entity_id: Uuid,
        entity_type: String,
        field_values: serde_json::Value,
        updated_at: DateTime<Utc>,
        version: u64,
    },
    Deleted {
        entity_id: Uuid,
        entity_type: String,
        deleted_at: DateTime<Utc>,
        version: u64,
    },
}

/// CRDT update for collaborative text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtUpdate {
    pub entity_id: Uuid,
    pub field: String,
    pub update: Vec<u8>,
    pub state_vector: Vec<u8>,
}

/// Conflict detected during sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub field: String,
    
    /// Client's value
    pub client_value: serde_json::Value,
    pub client_version: u64,
    
    /// Server's value
    pub server_value: serde_json::Value,
    pub server_version: u64,
    
    /// Suggested resolution strategy
    pub strategy: ConflictStrategy,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStrategy {
    /// Server wins (last write wins)
    ServerWins,
    
    /// Client wins (force push)
    ClientWins,
    
    /// Manual resolution required
    Manual,
    
    /// Merge (for arrays/objects)
    Merge,
    
    /// CRDT (already resolved)
    Crdt,
}

/// Sync state (stored on client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    /// Last successful pull timestamp
    pub last_pulled_at: Option<DateTime<Utc>>,
    
    /// Last successful push timestamp
    pub last_pushed_at: Option<DateTime<Utc>>,
    
    /// Number of pending mutations
    pub pending_mutations: usize,
    
    /// Is currently syncing
    pub is_syncing: bool,
    
    /// Last sync error
    pub last_error: Option<String>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            last_pulled_at: None,
            last_pushed_at: None,
            pending_mutations: 0,
            is_syncing: false,
            last_error: None,
        }
    }
}

impl SyncState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn start_sync(&mut self) {
        self.is_syncing = true;
        self.last_error = None;
    }
    
    pub fn sync_success(&mut self, pulled_at: DateTime<Utc>, pushed_count: usize) {
        self.is_syncing = false;
        self.last_pulled_at = Some(pulled_at);
        self.last_pushed_at = Some(Utc::now());
        self.pending_mutations = self.pending_mutations.saturating_sub(pushed_count);
        self.last_error = None;
    }
    
    pub fn sync_error(&mut self, error: String) {
        self.is_syncing = false;
        self.last_error = Some(error);
    }
    
    pub fn add_pending_mutation(&mut self) {
        self.pending_mutations += 1;
    }
}
