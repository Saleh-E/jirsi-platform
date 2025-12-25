//! Projections - Async Read Model Updaters
//! 
//! When events are saved to the event store, these projections
//! update the denormalized read models (entity_records table).
//! 
//! This keeps the existing UI working while we migrate to CQRS!

use sqlx::PgPool;
use uuid::Uuid;
use serde_json::json;
use super::DealEvent;

/// Deal Projection - updates entity_records table
pub struct DealProjection {
    pool: PgPool,
}

impl DealProjection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Project a Deal event into the read model
    pub async fn project(&self, event: &DealEvent) -> Result<(), ProjectionError> {
        match event {
            DealEvent::Created {
                deal_id,
                tenant_id,
                title,
                value,
                stage,
                contact_id,
                property_id,
                created_by,
                created_at,
            } => {
                // Insert into entity_records table
                let field_values = json!({
                    "title": title,
                    "value": value,
                    "stage": stage,
                    "contact_id": contact_id,
                    "property_id": property_id,
                    "status": "active",
                });
                
                sqlx::query(
                    r#"
                    INSERT INTO entity_records 
                        (id, tenant_id, entity_type, field_values, created_by, created_at, updated_at)
                    VALUES ($1, $2, 'deal', $3, $4, $5, $5)
                    "#
                )
                .bind(deal_id)
                .bind(tenant_id)
                .bind(field_values)
                .bind(created_by)
                .bind(created_at)
                .execute(&self.pool)
                .await
                .map_err(|e| ProjectionError::DatabaseError(e.to_string()))?;
            }
            
            DealEvent::StageUpdated {
                deal_id,
                new_stage,
                updated_at,
                ..
            } => {
                // Update stage in read model
                sqlx::query(
                    r#"
                    UPDATE entity_records
                    SET field_values = jsonb_set(field_values, '{stage}', $1),
                        updated_at = $2
                    WHERE id = $3
                    "#
                )
                .bind(json!(new_stage))
                .bind(updated_at)
                .bind(deal_id)
                .execute(&self.pool)
                .await
                .map_err(|e| ProjectionError::DatabaseError(e.to_string()))?;
            }
            
            DealEvent::ValueAdded {
                deal_id,
                new_value,
                updated_at,
                ..
            } => {
                // Update value in read model
                sqlx::query(
                    r#"
                    UPDATE entity_records
                    SET field_values = jsonb_set(field_values, '{value}', $1),
                        updated_at = $2
                    WHERE id = $3
                    "#
                )
                .bind(json!(new_value))
                .bind(updated_at)
                .bind(deal_id)
                .execute(&self.pool)
                .await
                .map_err(|e| ProjectionError::DatabaseError(e.to_string()))?;
            }
            
            DealEvent::ContactAssigned {
                deal_id,
                contact_id,
                updated_at,
                ..
            } => {
                sqlx::query(
                    r#"
                    UPDATE entity_records
                    SET field_values = jsonb_set(field_values, '{contact_id}', $1),
                        updated_at = $2
                    WHERE id = $3
                    "#
                )
                .bind(json!(contact_id))
                .bind(updated_at)
                .bind(deal_id)
                .execute(&self.pool)
                .await
                .map_err(|e| ProjectionError::DatabaseError(e.to_string()))?;
            }
            
            DealEvent::PropertyAssigned {
                deal_id,
                property_id,
                updated_at,
                ..
            } => {
                sqlx::query(
                    r#"
                    UPDATE entity_records
                    SET field_values = jsonb_set(field_values, '{property_id}', $1),
                        updated_at = $2
                    WHERE id = $3
                    "#
                )
                .bind(json!(property_id))
                .bind(updated_at)
                .bind(deal_id)
                .execute(&self.pool)
                .await
                .map_err(|e| ProjectionError::DatabaseError(e.to_string()))?;
            }
            
            DealEvent::Closed {
                deal_id,
                outcome,
                final_value,
                closed_at,
                ..
            } => {
                // Update status to closed
                let mut updates = json!({
                    "status": match outcome {
                        super::DealOutcome::Won => "won",
                        super::DealOutcome::Lost => "lost",
                    }
                });
                
                if let Some(value) = final_value {
                    updates["value"] = json!(value);
                }
                
                sqlx::query(
                    r#"
                    UPDATE entity_records
                    SET field_values = field_values || $1,
                        updated_at = $2
                    WHERE id = $3
                    "#
                )
                .bind(updates)
                .bind(closed_at)
                .bind(deal_id)
                .execute(&self.pool)
                .await
                .map_err(|e| ProjectionError::DatabaseError(e.to_string()))?;
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("Database error: {0}")]
    DatabaseError(String),
}
