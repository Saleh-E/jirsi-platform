//! Event Store Implementation - with Snapshot Support
//! 
//! PostgreSQL-based event store using the esrs pattern.
//! Includes aggregate snapshots for performance optimization.

use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value as JsonValue;
use super::{DealEvent, Deal};

/// Default snapshot interval (events)
/// Can be overridden via SNAPSHOT_INTERVAL env var
const DEFAULT_SNAPSHOT_INTERVAL: u64 = 50;

/// Get snapshot interval from environment or use default
fn get_snapshot_interval() -> u64 {
    std::env::var("SNAPSHOT_INTERVAL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_SNAPSHOT_INTERVAL)
}

pub struct EventStore {
    pool: PgPool,
}

impl EventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Load aggregate using snapshot (if available) + recent events
    pub async fn load_aggregate(&self, aggregate_id: Uuid) -> Result<Deal, EventStoreError> {
        // Try to load from snapshot first
        let (mut deal, snapshot_version) = self.load_from_snapshot(aggregate_id).await?;
        
        // Fetch only events after the snapshot
        let rows = sqlx::query(
            r#"
            SELECT event_data, aggregate_version
            FROM events
            WHERE aggregate_id = $1 AND aggregate_version > $2
            ORDER BY aggregate_version ASC
            "#
        )
        .bind(aggregate_id)
        .bind(snapshot_version as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
        
        let events_since_snapshot = rows.len();
        
        // Replay events since snapshot
        for row in rows {
            let event_data: JsonValue = row.try_get("event_data")
                .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
            let event: DealEvent = serde_json::from_value(event_data)
                .map_err(|e| EventStoreError::DeserializationError(e.to_string()))?;
            
            deal.apply(&event);
        }
        
        if deal.id.is_nil() {
            return Err(EventStoreError::AggregateNotFound(aggregate_id));
        }
        
        // Create new snapshot if we've replayed many events
        if events_since_snapshot as u64 >= get_snapshot_interval() {
            let _ = self.save_snapshot(aggregate_id, &deal).await;
        }
        
        Ok(deal)
    }
    
    /// Load from snapshot if available, otherwise return default
    async fn load_from_snapshot(&self, aggregate_id: Uuid) -> Result<(Deal, u64), EventStoreError> {
        let result = sqlx::query(
            r#"
            SELECT state_data, version
            FROM aggregate_snapshots
            WHERE aggregate_id = $1
            ORDER BY version DESC
            LIMIT 1
            "#
        )
        .bind(aggregate_id)
        .fetch_optional(&self.pool)
        .await;
        
        match result {
            Ok(Some(row)) => {
                let state_data: JsonValue = row.try_get("state_data")
                    .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
                let version: i64 = row.try_get("version")
                    .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
                
                let deal: Deal = serde_json::from_value(state_data)
                    .map_err(|e| EventStoreError::DeserializationError(e.to_string()))?;
                
                tracing::debug!(aggregate_id = %aggregate_id, version = version, "Loaded from snapshot");
                Ok((deal, version as u64))
            }
            Ok(None) => {
                // No snapshot, start from scratch
                Ok((Deal::default(), 0))
            }
            Err(e) => {
                // Table might not exist, continue without snapshot
                tracing::warn!(error = %e, "Snapshot lookup failed, replaying all events");
                Ok((Deal::default(), 0))
            }
        }
    }
    
    /// Save a snapshot of the current aggregate state
    pub async fn save_snapshot(&self, aggregate_id: Uuid, deal: &Deal) -> Result<(), EventStoreError> {
        let state_data = serde_json::to_value(deal)
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;
        
        let result = sqlx::query(
            r#"
            INSERT INTO aggregate_snapshots (aggregate_id, version, state_data, created_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (aggregate_id, version) DO UPDATE SET
                state_data = EXCLUDED.state_data,
                created_at = EXCLUDED.created_at
            "#
        )
        .bind(aggregate_id)
        .bind(deal.version as i64)
        .bind(state_data)
        .bind(Utc::now())
        .execute(&self.pool)
        .await;
        
        match result {
            Ok(_) => {
                tracing::info!(
                    aggregate_id = %aggregate_id, 
                    version = deal.version, 
                    "Snapshot saved"
                );
                Ok(())
            }
            Err(e) => {
                // Non-fatal: log and continue
                tracing::warn!(error = %e, "Failed to save snapshot");
                Ok(())
            }
        }
    }
    
    /// Force create a snapshot (for manual optimization)
    pub async fn create_snapshot(&self, aggregate_id: Uuid) -> Result<(), EventStoreError> {
        let deal = self.load_aggregate_full(aggregate_id).await?;
        self.save_snapshot(aggregate_id, &deal).await
    }
    
    /// Load aggregate by replaying ALL events (for snapshot creation)
    async fn load_aggregate_full(&self, aggregate_id: Uuid) -> Result<Deal, EventStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT event_data, aggregate_version
            FROM events
            WHERE aggregate_id = $1
            ORDER BY aggregate_version ASC
            "#
        )
        .bind(aggregate_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
        
        let mut deal = Deal::default();
        
        for row in rows {
            let event_data: JsonValue = row.try_get("event_data")
                .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
            let event: DealEvent = serde_json::from_value(event_data)
                .map_err(|e| EventStoreError::DeserializationError(e.to_string()))?;
            
            deal.apply(&event);
        }
        
        if deal.id.is_nil() {
            return Err(EventStoreError::AggregateNotFound(aggregate_id));
        }
        
        Ok(deal)
    }
    
    /// Save a new event to the store
    pub async fn save_event(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
        expected_version: u64,
        event: &DealEvent,
        created_by: Uuid,
    ) -> Result<(), EventStoreError> {
        let event_data = serde_json::to_value(event)
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;
        
        let event_type = match event {
            DealEvent::Created { .. } => "DealCreated",
            DealEvent::StageUpdated { .. } => "DealStageUpdated",
            DealEvent::ValueAdded { .. } => "DealValueAdded",
            DealEvent::ContactAssigned { .. } => "DealContactAssigned",
            DealEvent::PropertyAssigned { .. } => "DealPropertyAssigned",
            DealEvent::Closed { .. } => "DealClosed",
        };
        
        // Insert event with optimistic concurrency check
        let result = sqlx::query(
            r#"
            INSERT INTO events 
                (aggregate_id, aggregate_type, aggregate_version, event_type, event_data, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(aggregate_id)
        .bind(aggregate_type)
        .bind(expected_version as i64)
        .bind(event_type)
        .bind(event_data)
        .bind(created_by)
        .execute(&self.pool)
        .await;
        
        match result {
            Ok(_) => {
                // Check if we should create a snapshot
                if expected_version > 0 && expected_version % get_snapshot_interval() == 0 {
                    tokio::spawn({
                        let store = EventStore::new(self.pool.clone());
                        async move {
                            let _ = store.create_snapshot(aggregate_id).await;
                        }
                    });
                }
                Ok(())
            }
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(EventStoreError::ConcurrencyConflict {
                    aggregate_id,
                    expected_version,
                })
            }
            Err(e) => Err(EventStoreError::DatabaseError(e.to_string())),
        }
    }
    
    /// Get all events for time travel / audit
    pub async fn get_event_stream(
        &self,
        aggregate_id: Uuid,
    ) -> Result<Vec<(DealEvent, i64)>, EventStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT event_data, aggregate_version, created_at
            FROM events
            WHERE aggregate_id = $1
            ORDER BY aggregate_version ASC
            "#
        )
        .bind(aggregate_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
        
        let mut events = Vec::new();
        
        for row in rows {
            let event_data: JsonValue = row.try_get("event_data")
                .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
            let event: DealEvent = serde_json::from_value(event_data)
                .map_err(|e| EventStoreError::DeserializationError(e.to_string()))?;
            
            let version: i64 = row.try_get("aggregate_version")
                .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
            
            events.push((event, version));
        }
        
        Ok(events)
    }
    
    /// Get event count for an aggregate
    pub async fn get_event_count(&self, aggregate_id: Uuid) -> Result<u64, EventStoreError> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM events WHERE aggregate_id = $1"
        )
        .bind(aggregate_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
        
        let count: i64 = row.try_get("count")
            .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
        
        Ok(count as u64)
    }
    
    /// Cleanup old snapshots (keep only latest N)
    pub async fn cleanup_old_snapshots(&self, aggregate_id: Uuid, keep_count: i64) -> Result<u64, EventStoreError> {
        let result = sqlx::query(
            r#"
            DELETE FROM aggregate_snapshots
            WHERE aggregate_id = $1
            AND version NOT IN (
                SELECT version FROM aggregate_snapshots
                WHERE aggregate_id = $1
                ORDER BY version DESC
                LIMIT $2
            )
            "#
        )
        .bind(aggregate_id)
        .bind(keep_count)
        .execute(&self.pool)
        .await
        .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
        
        Ok(result.rows_affected())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("Aggregate not found: {0}")]
    AggregateNotFound(Uuid),
    
    #[error("Concurrency conflict for aggregate {aggregate_id} at version {expected_version}")]
    ConcurrencyConflict {
        aggregate_id: Uuid,
        expected_version: u64,
    },
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

