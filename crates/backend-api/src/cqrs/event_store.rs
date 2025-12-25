//! Event Store Implementation
//! 
//! PostgreSQL-based event store using the esrs pattern.

use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value as JsonValue;
use super::{DealEvent, Deal};

pub struct EventStore {
    pool: PgPool,
}

impl EventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Load all events for an aggregate and replay them
    pub async fn load_aggregate(&self, aggregate_id: Uuid) -> Result<Deal, EventStoreError> {
        // Fetch all events for this aggregate
        let rows = sqlx::query!(
            r#"
            SELECT event_data, aggregate_version
            FROM events
            WHERE aggregate_id = $1
            ORDER BY aggregate_version ASC
            "#,
            aggregate_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
        
        // Replay events to build current state
        let mut deal = Deal::default();
        
        for row in rows {
            let event: DealEvent = serde_json::from_value(row.event_data)
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
        let result = sqlx::query!(
            r#"
            INSERT INTO events 
                (aggregate_id, aggregate_type, aggregate_version, event_type, event_data, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            aggregate_id,
            aggregate_type,
            expected_version as i64,
            event_type,
            event_data,
            created_by,
        )
        .execute(&self.pool)
        .await;
        
        match result {
            Ok(_) => Ok(()),
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
        let rows = sqlx::query!(
            r#"
            SELECT event_data, aggregate_version, created_at
            FROM events
            WHERE aggregate_id = $1
            ORDER BY aggregate_version ASC
            "#,
            aggregate_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EventStoreError::DatabaseError(e.to_string()))?;
        
        let mut events = Vec::new();
        
        for row in rows {
            let event: DealEvent = serde_json::from_value(row.event_data)
                .map_err(|e| EventStoreError::DeserializationError(e.to_string()))?;
            
            events.push((event, row.aggregate_version));
        }
        
        Ok(events)
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
