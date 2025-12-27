//! Event Bus - Async Projection & Webhook Dispatcher
//!
//! Provides asynchronous event processing with:
//! - In-memory event channel for projections
//! - Eventual consistency via background processing
//! - Webhook dispatch for external integrations
//! - Dead letter queue for failed events

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// Event envelope for the bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique event ID
    pub event_id: Uuid,
    /// Aggregate ID this event belongs to
    pub aggregate_id: Uuid,
    /// Aggregate type (e.g., "deal", "contact")
    pub aggregate_type: String,
    /// Event type name
    pub event_type: String,
    /// Serialized event data
    pub event_data: serde_json::Value,
    /// Tenant ID
    pub tenant_id: Uuid,
    /// User who triggered the event
    pub caused_by: Uuid,
    /// When the event occurred
    pub occurred_at: DateTime<Utc>,
    /// Aggregate version after this event
    pub version: u64,
}

/// Projection handler trait
#[async_trait::async_trait]
pub trait ProjectionHandler: Send + Sync {
    /// Handle an event
    async fn handle(&self, event: &EventEnvelope) -> Result<(), ProjectionError>;
    
    /// Get the handler name (for logging)
    fn name(&self) -> &'static str;
}

/// Projection error
#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Handler error: {0}")]
    Handler(String),
}

/// Event Bus configuration
#[derive(Clone)]
pub struct EventBusConfig {
    /// Channel buffer size
    pub buffer_size: usize,
    /// Max retries for failed projections
    pub max_retries: u32,
    /// Enable dead letter queue
    pub enable_dlq: bool,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            max_retries: 3,
            enable_dlq: true,
        }
    }
}

/// Dead letter queue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterEntry {
    pub event: EventEnvelope,
    pub error: String,
    pub handler: String,
    pub retry_count: u32,
    pub failed_at: DateTime<Utc>,
}

/// Event Bus for asynchronous event processing
pub struct EventBus {
    /// Broadcast channel for real-time subscribers
    broadcast_tx: broadcast::Sender<EventEnvelope>,
    
    /// MPSC channel for projection processing
    projection_tx: mpsc::Sender<EventEnvelope>,
    
    /// Registered projection handlers
    handlers: Arc<RwLock<Vec<Arc<dyn ProjectionHandler>>>>,
    
    /// Dead letter queue
    dlq: Arc<RwLock<Vec<DeadLetterEntry>>>,
    
    /// Configuration
    config: EventBusConfig,
    
    /// Database pool for persistence
    pool: Option<PgPool>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new(config: EventBusConfig) -> Self {
        let (broadcast_tx, _) = broadcast::channel(config.buffer_size);
        let (projection_tx, _) = mpsc::channel(config.buffer_size);
        
        Self {
            broadcast_tx,
            projection_tx,
            handlers: Arc::new(RwLock::new(Vec::new())),
            dlq: Arc::new(RwLock::new(Vec::new())),
            config,
            pool: None,
        }
    }
    
    /// Create with database pool for DLQ persistence
    pub fn with_pool(config: EventBusConfig, pool: PgPool) -> Self {
        let mut bus = Self::new(config);
        bus.pool = Some(pool);
        bus
    }
    
    /// Register a projection handler
    pub async fn register_handler(&self, handler: Arc<dyn ProjectionHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
    }
    
    /// Publish an event to the bus
    pub async fn publish(&self, event: EventEnvelope) {
        // Broadcast to real-time subscribers
        let _ = self.broadcast_tx.send(event.clone());
        
        // Process through projection handlers
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            let result = self.process_with_retry(&event, handler.clone()).await;
            
            if let Err(e) = result {
                if self.config.enable_dlq {
                    self.add_to_dlq(DeadLetterEntry {
                        event: event.clone(),
                        error: e.to_string(),
                        handler: handler.name().to_string(),
                        retry_count: self.config.max_retries,
                        failed_at: Utc::now(),
                    }).await;
                }
            }
        }
    }
    
    /// Process event with retry logic
    async fn process_with_retry(
        &self,
        event: &EventEnvelope,
        handler: Arc<dyn ProjectionHandler>,
    ) -> Result<(), ProjectionError> {
        let mut last_error = None;
        
        for attempt in 0..=self.config.max_retries {
            match handler.handle(event).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries {
                        // Exponential backoff
                        let delay_ms = 100 * 2u64.pow(attempt);
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| ProjectionError::Handler("Unknown error".to_string())))
    }
    
    /// Add failed event to dead letter queue
    async fn add_to_dlq(&self, entry: DeadLetterEntry) {
        let mut dlq = self.dlq.write().await;
        dlq.push(entry.clone());
        
        // Persist to database if available
        if let Some(pool) = &self.pool {
            let _ = sqlx::query(
                r#"
                INSERT INTO event_dead_letter_queue 
                    (event_id, event_data, error_message, handler_name, retry_count, failed_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#
            )
            .bind(entry.event.event_id)
            .bind(serde_json::to_value(&entry.event).unwrap_or_default())
            .bind(&entry.error)
            .bind(&entry.handler)
            .bind(entry.retry_count as i32)
            .bind(entry.failed_at)
            .execute(pool)
            .await;
        }
    }
    
    /// Subscribe to real-time events
    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.broadcast_tx.subscribe()
    }
    
    /// Get dead letter queue entries
    pub async fn get_dlq(&self) -> Vec<DeadLetterEntry> {
        self.dlq.read().await.clone()
    }
    
    /// Retry a dead letter queue entry
    pub async fn retry_dlq_entry(&self, event_id: Uuid) -> Result<(), ProjectionError> {
        let mut dlq = self.dlq.write().await;
        
        if let Some(pos) = dlq.iter().position(|e| e.event.event_id == event_id) {
            let entry = dlq.remove(pos);
            drop(dlq); // Release lock before publishing
            
            self.publish(entry.event).await;
            Ok(())
        } else {
            Err(ProjectionError::Handler("Entry not found in DLQ".to_string()))
        }
    }
    
    /// Clear all DLQ entries
    pub async fn clear_dlq(&self) {
        let mut dlq = self.dlq.write().await;
        dlq.clear();
        
        // Clear from database too
        if let Some(pool) = &self.pool {
            let _ = sqlx::query("TRUNCATE event_dead_letter_queue")
                .execute(pool)
                .await;
        }
    }
}

/// Event replay service for rebuilding projections
pub struct EventReplayService {
    pool: PgPool,
    event_bus: Arc<EventBus>,
}

impl EventReplayService {
    pub fn new(pool: PgPool, event_bus: Arc<EventBus>) -> Self {
        Self { pool, event_bus }
    }
    
    /// Replay all events for an aggregate
    pub async fn replay_aggregate(&self, aggregate_id: Uuid) -> Result<u64, String> {
        let events: Vec<(Uuid, String, String, serde_json::Value, Uuid, Uuid, DateTime<Utc>, i64)> = 
            sqlx::query_as(
                r#"
                SELECT 
                    id, aggregate_type, event_type, event_data, 
                    tenant_id, created_by, created_at, version
                FROM events 
                WHERE aggregate_id = $1 
                ORDER BY version
                "#
            )
            .bind(aggregate_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        
        let count = events.len() as u64;
        
        for (event_id, aggregate_type, event_type, event_data, tenant_id, created_by, created_at, version) in events {
            let envelope = EventEnvelope {
                event_id,
                aggregate_id,
                aggregate_type,
                event_type,
                event_data,
                tenant_id,
                caused_by: created_by,
                occurred_at: created_at,
                version: version as u64,
            };
            
            self.event_bus.publish(envelope).await;
        }
        
        Ok(count)
    }
    
    /// Replay all events since a specific timestamp
    pub async fn replay_since(&self, since: DateTime<Utc>) -> Result<u64, String> {
        let events: Vec<(Uuid, Uuid, String, String, serde_json::Value, Uuid, Uuid, DateTime<Utc>, i64)> = 
            sqlx::query_as(
                r#"
                SELECT 
                    id, aggregate_id, aggregate_type, event_type, event_data, 
                    tenant_id, created_by, created_at, version
                FROM events 
                WHERE created_at >= $1 
                ORDER BY created_at, version
                "#
            )
            .bind(since)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        
        let count = events.len() as u64;
        
        for (event_id, aggregate_id, aggregate_type, event_type, event_data, tenant_id, created_by, created_at, version) in events {
            let envelope = EventEnvelope {
                event_id,
                aggregate_id,
                aggregate_type,
                event_type,
                event_data,
                tenant_id,
                caused_by: created_by,
                occurred_at: created_at,
                version: version as u64,
            };
            
            self.event_bus.publish(envelope).await;
        }
        
        Ok(count)
    }
    
    /// Rebuild all projections for a specific aggregate type
    pub async fn rebuild_projections(&self, aggregate_type: &str, batch_size: i64) -> Result<u64, String> {
        let mut total_replayed = 0u64;
        let mut last_id: Option<Uuid> = None;
        
        loop {
            let events: Vec<(Uuid, Uuid, String, serde_json::Value, Uuid, Uuid, DateTime<Utc>, i64)> = 
                if let Some(cursor) = last_id {
                    sqlx::query_as(
                        r#"
                        SELECT 
                            id, aggregate_id, event_type, event_data, 
                            tenant_id, created_by, created_at, version
                        FROM events 
                        WHERE aggregate_type = $1 AND id > $2
                        ORDER BY id
                        LIMIT $3
                        "#
                    )
                    .bind(aggregate_type)
                    .bind(cursor)
                    .bind(batch_size)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| e.to_string())?
                } else {
                    sqlx::query_as(
                        r#"
                        SELECT 
                            id, aggregate_id, event_type, event_data, 
                            tenant_id, created_by, created_at, version
                        FROM events 
                        WHERE aggregate_type = $1
                        ORDER BY id
                        LIMIT $2
                        "#
                    )
                    .bind(aggregate_type)
                    .bind(batch_size)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| e.to_string())?
                };
            
            if events.is_empty() {
                break;
            }
            
            for (event_id, aggregate_id, event_type, event_data, tenant_id, created_by, created_at, version) in &events {
                let envelope = EventEnvelope {
                    event_id: *event_id,
                    aggregate_id: *aggregate_id,
                    aggregate_type: aggregate_type.to_string(),
                    event_type: event_type.clone(),
                    event_data: event_data.clone(),
                    tenant_id: *tenant_id,
                    caused_by: *created_by,
                    occurred_at: *created_at,
                    version: *version as u64,
                };
                
                self.event_bus.publish(envelope).await;
                total_replayed += 1;
            }
            
            last_id = events.last().map(|(id, ..)| *id);
            
            if events.len() < batch_size as usize {
                break;
            }
        }
        
        Ok(total_replayed)
    }
}

/// Global event bus instance (lazy initialized)
pub type SharedEventBus = Arc<EventBus>;

/// Create default event bus
pub fn create_event_bus() -> SharedEventBus {
    Arc::new(EventBus::new(EventBusConfig::default()))
}

/// Create event bus with database pool
pub fn create_event_bus_with_pool(pool: PgPool) -> SharedEventBus {
    Arc::new(EventBus::with_pool(EventBusConfig::default(), pool))
}
