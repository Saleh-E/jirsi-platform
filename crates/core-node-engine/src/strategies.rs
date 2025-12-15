//! Assignment Strategies - Round Robin and Load Balanced agent assignment
//!
//! Provides atomic, race-condition safe agent assignment for workflow automation.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

/// Assignment strategy types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentStrategy {
    /// Assign to agent who was assigned least recently
    RoundRobin,
    /// Assign to agent with fewest active deals
    LoadBalanced,
    /// Manual assignment (no auto-assignment)
    Manual,
}

impl Default for AssignmentStrategy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

/// Service for handling agent assignment with various strategies
pub struct AssignmentService;

impl AssignmentService {
    /// Find the next agent using the specified strategy
    pub async fn find_agent(
        pool: &PgPool,
        tenant_id: Uuid,
        strategy: AssignmentStrategy,
        pool_id: Option<Uuid>,
    ) -> Result<Option<Uuid>, String> {
        match strategy {
            AssignmentStrategy::RoundRobin => {
                Self::find_agent_round_robin(pool, tenant_id, pool_id).await
            }
            AssignmentStrategy::LoadBalanced => {
                Self::find_agent_load_balanced(pool, tenant_id, pool_id).await
            }
            AssignmentStrategy::Manual => Ok(None),
        }
    }

    /// Find next agent using Round Robin with atomic row locking
    /// 
    /// Uses FOR UPDATE SKIP LOCKED to prevent race conditions when
    /// multiple leads arrive simultaneously.
    pub async fn find_agent_round_robin(
        pool: &PgPool,
        tenant_id: Uuid,
        _pool_id: Option<Uuid>,
    ) -> Result<Option<Uuid>, String> {
        // Start a transaction for atomic operation
        let mut tx = pool.begin().await.map_err(|e| format!("Transaction failed: {}", e))?;
        
        // Find agent with oldest last_assigned_at, using row lock
        let result: Option<(Uuid,)> = sqlx::query_as(
            r#"
            WITH next_agent AS (
                SELECT u.id
                FROM users u
                LEFT JOIN agent_round_robin_state ars 
                    ON u.id = ars.user_id AND ars.tenant_id = $1
                WHERE u.tenant_id = $1 
                  AND u.role = 'agent' 
                  AND u.is_active = true
                ORDER BY COALESCE(ars.last_assigned_at, '1970-01-01'::timestamptz) ASC,
                         u.created_at ASC
                LIMIT 1
                FOR UPDATE OF u SKIP LOCKED
            )
            SELECT id FROM next_agent
            "#
        )
        .bind(tenant_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

        if let Some((agent_id,)) = result {
            // Update or insert the round robin state
            sqlx::query(
                r#"
                INSERT INTO agent_round_robin_state (tenant_id, user_id, last_assigned_at, assignment_count)
                VALUES ($1, $2, NOW(), 1)
                ON CONFLICT (tenant_id, user_id) 
                DO UPDATE SET 
                    last_assigned_at = NOW(),
                    assignment_count = agent_round_robin_state.assignment_count + 1
                "#
            )
            .bind(tenant_id)
            .bind(agent_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Update failed: {}", e))?;

            tx.commit().await.map_err(|e| format!("Commit failed: {}", e))?;
            
            info!(agent_id = %agent_id, "Round robin assigned agent");
            Ok(Some(agent_id))
        } else {
            tx.rollback().await.ok();
            warn!("No available agents for round robin assignment");
            Ok(None)
        }
    }

    /// Find agent with lowest active deal count (Load Balanced)
    /// 
    /// Counts deals where stage is not 'closed_won' or 'closed_lost'
    pub async fn find_agent_load_balanced(
        pool: &PgPool,
        tenant_id: Uuid,
        _pool_id: Option<Uuid>,
    ) -> Result<Option<Uuid>, String> {
        let result: Option<(Uuid, i64)> = sqlx::query_as(
            r#"
            SELECT u.id, COUNT(d.id) as active_deals
            FROM users u
            LEFT JOIN deals d ON d.owner_id = u.id 
                AND d.tenant_id = $1 
                AND d.stage NOT IN ('closed_won', 'closed_lost', 'lost')
                AND d.deleted_at IS NULL
            WHERE u.tenant_id = $1 
              AND u.role = 'agent' 
              AND u.is_active = true
            GROUP BY u.id
            ORDER BY active_deals ASC, u.created_at ASC
            LIMIT 1
            "#
        )
        .bind(tenant_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

        if let Some((agent_id, deal_count)) = result {
            info!(agent_id = %agent_id, active_deals = deal_count, "Load balanced assigned agent");
            Ok(Some(agent_id))
        } else {
            warn!("No available agents for load balanced assignment");
            Ok(None)
        }
    }

    /// Get assignment statistics for all agents
    pub async fn get_agent_stats(
        pool: &PgPool,
        tenant_id: Uuid,
    ) -> Result<Vec<AgentStats>, String> {
        let rows: Vec<(Uuid, String, String, i64, i32)> = sqlx::query_as(
            r#"
            SELECT 
                u.id,
                u.first_name,
                u.last_name,
                COUNT(d.id) as active_deals,
                COALESCE(ars.assignment_count, 0) as total_assignments
            FROM users u
            LEFT JOIN deals d ON d.owner_id = u.id 
                AND d.tenant_id = $1 
                AND d.stage NOT IN ('closed_won', 'closed_lost', 'lost')
                AND d.deleted_at IS NULL
            LEFT JOIN agent_round_robin_state ars 
                ON u.id = ars.user_id AND ars.tenant_id = $1
            WHERE u.tenant_id = $1 
              AND u.role = 'agent' 
              AND u.is_active = true
            GROUP BY u.id, u.first_name, u.last_name, ars.assignment_count
            ORDER BY u.first_name, u.last_name
            "#
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

        Ok(rows.into_iter().map(|(id, first_name, last_name, active_deals, total_assignments)| {
            AgentStats {
                id,
                name: format!("{} {}", first_name, last_name),
                active_deals,
                total_assignments,
            }
        }).collect())
    }
}

/// Statistics for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    pub id: Uuid,
    pub name: String,
    pub active_deals: i64,
    pub total_assignments: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_default() {
        assert_eq!(AssignmentStrategy::default(), AssignmentStrategy::RoundRobin);
    }

    #[test]
    fn test_strategy_serialization() {
        let strategy = AssignmentStrategy::LoadBalanced;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"load_balanced\"");
    }
}
