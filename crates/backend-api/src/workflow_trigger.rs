//! Workflow Trigger Service
//!
//! Connects database events to workflow execution.
//! Listens for CQRS events and triggers matching workflows.

use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use tokio::sync::RwLock;
use tracing::{info, error, warn};

use crate::cqrs::{EventEnvelope, ProjectionHandler, ProjectionError, SharedEventBus};

/// Workflow trigger type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    /// Entity created
    RecordCreated,
    /// Entity updated
    RecordUpdated,
    /// Entity deleted
    RecordDeleted,
    /// Field value changed
    FieldChanged { field: String },
    /// Scheduled (cron)
    Scheduled { cron: String },
    /// Webhook received
    Webhook { endpoint: String },
    /// Manual trigger
    Manual,
}

/// Workflow trigger definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTrigger {
    pub id: Uuid,
    pub graph_id: Uuid,
    pub tenant_id: Uuid,
    pub trigger_type: TriggerType,
    pub entity_type: Option<String>,
    pub filter_conditions: Option<JsonValue>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Matched workflow to execute
#[derive(Debug, Clone)]
pub struct MatchedWorkflow {
    pub graph_id: Uuid,
    pub trigger_id: Uuid,
    pub trigger_data: JsonValue,
}

/// Workflow Trigger Service
pub struct WorkflowTriggerService {
    pool: PgPool,
    /// Cached triggers (refreshed periodically)
    triggers: Arc<RwLock<Vec<WorkflowTrigger>>>,
}

impl WorkflowTriggerService {
    /// Create new trigger service
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            triggers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Load all active triggers from database
    pub async fn load_triggers(&self) -> Result<(), String> {
        let rows = sqlx::query(
            r#"
            SELECT 
                t.id, t.graph_id, g.tenant_id, t.trigger_type, t.entity_type,
                t.filter_conditions, t.is_active, t.created_at
            FROM workflow_triggers t
            JOIN node_graphs g ON t.graph_id = g.id
            WHERE t.is_active = true AND g.is_published = true
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        
        let mut triggers = Vec::new();
        for row in rows {
            use sqlx::Row;
            let trigger_type_json: serde_json::Value = row.get("trigger_type");
            let trigger_type: TriggerType = serde_json::from_value(trigger_type_json)
                .unwrap_or(TriggerType::Manual);
            
            triggers.push(WorkflowTrigger {
                id: row.get("id"),
                graph_id: row.get("graph_id"),
                tenant_id: row.get("tenant_id"),
                trigger_type,
                entity_type: row.get("entity_type"),
                filter_conditions: row.get("filter_conditions"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
            });
        }
        
        let mut cache = self.triggers.write().await;
        *cache = triggers;
        
        Ok(())
    }

    
    /// Find workflows matching an event
    pub async fn find_matching_workflows(&self, event: &EventEnvelope) -> Vec<MatchedWorkflow> {
        let triggers = self.triggers.read().await;
        let mut matches = Vec::new();
        
        for trigger in triggers.iter() {
            // Check tenant match
            if trigger.tenant_id != event.tenant_id {
                continue;
            }
            
            // Check entity type match
            if let Some(ref entity_type) = trigger.entity_type {
                if entity_type != &event.aggregate_type {
                    continue;
                }
            }
            
            // Check trigger type match
            let trigger_matches = match &trigger.trigger_type {
                TriggerType::RecordCreated => event.event_type.contains("Created"),
                TriggerType::RecordUpdated => {
                    event.event_type.contains("Updated") || 
                    event.event_type.contains("Changed")
                }
                TriggerType::RecordDeleted => event.event_type.contains("Deleted"),
                TriggerType::FieldChanged { field } => {
                    event.event_type.contains("Updated") && 
                    event.event_data.get(field).is_some()
                }
                TriggerType::Manual | TriggerType::Scheduled { .. } | TriggerType::Webhook { .. } => false,
            };
            
            if !trigger_matches {
                continue;
            }
            
            // Check filter conditions
            if let Some(ref conditions) = trigger.filter_conditions {
                if !self.evaluate_conditions(conditions, &event.event_data) {
                    continue;
                }
            }
            
            // Build trigger data
            let trigger_data = json!({
                "event_id": event.event_id,
                "aggregate_id": event.aggregate_id,
                "aggregate_type": event.aggregate_type,
                "event_type": event.event_type,
                "event_data": event.event_data,
                "occurred_at": event.occurred_at,
                "caused_by": event.caused_by,
            });
            
            matches.push(MatchedWorkflow {
                graph_id: trigger.graph_id,
                trigger_id: trigger.id,
                trigger_data,
            });
        }
        
        matches
    }
    
    /// Evaluate filter conditions
    fn evaluate_conditions(&self, conditions: &JsonValue, data: &JsonValue) -> bool {
        match conditions {
            JsonValue::Object(obj) => {
                for (key, expected) in obj {
                    let actual = data.get(key);
                    match actual {
                        Some(val) if val == expected => continue,
                        _ => return false,
                    }
                }
                true
            }
            _ => true,
        }
    }
    
    /// Queue a workflow execution
    pub async fn queue_workflow_execution(&self, matched: &MatchedWorkflow) -> Result<Uuid, String> {
        let execution_id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO workflow_executions 
                (id, graph_id, trigger_id, trigger_data, status, queued_at)
            VALUES ($1, $2, $3, $4, 'pending', NOW())
            "#
        )
        .bind(execution_id)
        .bind(matched.graph_id)
        .bind(matched.trigger_id)
        .bind(&matched.trigger_data)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        
        info!(
            execution_id = %execution_id,
            graph_id = %matched.graph_id,
            "Workflow execution queued"
        );
        
        Ok(execution_id)
    }
    
    /// Register trigger for a manual workflow
    pub async fn create_trigger(
        &self,
        graph_id: Uuid,
        trigger_type: TriggerType,
        entity_type: Option<String>,
        filter_conditions: Option<JsonValue>,
    ) -> Result<Uuid, String> {
        let trigger_id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO workflow_triggers 
                (id, graph_id, trigger_type, entity_type, filter_conditions, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5, true, NOW())
            "#
        )
        .bind(trigger_id)
        .bind(graph_id)
        .bind(serde_json::to_value(&trigger_type).unwrap())
        .bind(&entity_type)
        .bind(&filter_conditions)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        
        // Reload triggers
        let _ = self.load_triggers().await;
        
        Ok(trigger_id)
    }
}

/// Projection handler that triggers workflows
pub struct WorkflowTriggerProjection {
    trigger_service: Arc<WorkflowTriggerService>,
}

impl WorkflowTriggerProjection {
    pub fn new(trigger_service: Arc<WorkflowTriggerService>) -> Self {
        Self { trigger_service }
    }
}

#[async_trait::async_trait]
impl ProjectionHandler for WorkflowTriggerProjection {
    async fn handle(&self, event: &EventEnvelope) -> Result<(), ProjectionError> {
        // Find matching workflows
        let matches = self.trigger_service.find_matching_workflows(event).await;
        
        // Queue each matched workflow
        for matched in matches {
            if let Err(e) = self.trigger_service.queue_workflow_execution(&matched).await {
                error!(error = %e, "Failed to queue workflow execution");
                return Err(ProjectionError::Handler(e));
            }
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "WorkflowTriggerProjection"
    }
}

/// Background worker to process queued workflow executions
pub struct WorkflowExecutionWorker {
    pool: PgPool,
    event_bus: SharedEventBus,
}

impl WorkflowExecutionWorker {
    pub fn new(pool: PgPool, event_bus: SharedEventBus) -> Self {
        Self { pool, event_bus }
    }
    
    /// Start the worker loop
    pub async fn start(self) {
        info!("Starting workflow execution worker");
        
        loop {
            // Fetch pending executions
            match self.process_pending().await {
                Ok(count) if count > 0 => {
                    info!(count, "Processed workflow executions");
                }
                Err(e) => {
                    error!(error = %e, "Workflow execution error");
                }
                _ => {}
            }
            
            // Wait before next poll
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
    
    /// Process pending workflow executions
    async fn process_pending(&self) -> Result<usize, String> {
        // Claim pending executions
        let executions: Vec<(Uuid, Uuid, JsonValue)> = sqlx::query_as(
            r#"
            UPDATE workflow_executions
            SET status = 'running', started_at = NOW()
            WHERE id IN (
                SELECT id FROM workflow_executions
                WHERE status = 'pending'
                ORDER BY queued_at
                LIMIT 10
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, graph_id, trigger_data
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        
        let count = executions.len();
        
        for (execution_id, graph_id, trigger_data) in executions {
            // Load graph and execute
            match self.execute_workflow(execution_id, graph_id, trigger_data).await {
                Ok(()) => {
                    sqlx::query(
                        "UPDATE workflow_executions SET status = 'completed', completed_at = NOW() WHERE id = $1"
                    )
                    .bind(execution_id)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| e.to_string())?;
                }
                Err(e) => {
                    sqlx::query(
                        "UPDATE workflow_executions SET status = 'failed', error = $1, completed_at = NOW() WHERE id = $2"
                    )
                    .bind(&e)
                    .bind(execution_id)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| e.to_string())?;
                    
                    warn!(execution_id = %execution_id, error = %e, "Workflow failed");
                }
            }
        }
        
        Ok(count)
    }
    
    /// Execute a single workflow
    async fn execute_workflow(
        &self,
        execution_id: Uuid,
        graph_id: Uuid,
        trigger_data: JsonValue,
    ) -> Result<(), String> {
        // Load graph definition
        let graph: Option<(Uuid, Uuid, String, bool)> = sqlx::query_as(
            "SELECT id, tenant_id, name, is_published FROM node_graphs WHERE id = $1"
        )
        .bind(graph_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        
        let Some((_id, tenant_id, name, is_published)) = graph else {
            return Err("Graph not found".to_string());
        };
        
        if !is_published {
            return Err("Graph is not published".to_string());
        }
        
        info!(
            execution_id = %execution_id,
            graph_id = %graph_id,
            graph_name = %name,
            tenant_id = %tenant_id,
            "Executing workflow"
        );
        
        // TODO: Actually execute the graph using GraphExecutor
        // For now, just mark as successful
        
        Ok(())
    }
}
