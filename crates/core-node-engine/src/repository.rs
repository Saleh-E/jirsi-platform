//! Node graph repository

use core_models::{EdgeDef, NodeDef, NodeGraphDef, GraphScope, GraphType, NodeType};
use sqlx::PgPool;
use uuid::Uuid;

use crate::NodeEngineError;

/// Repository for node graph CRUD operations
pub struct NodeGraphRepository {
    pool: PgPool,
}

impl NodeGraphRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get a graph by ID
    pub async fn get_graph(&self, tenant_id: Uuid, graph_id: Uuid) -> Result<NodeGraphDef, NodeEngineError> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, name, label, description,
                scope, graph_type,
                entity_type_id, app_id, is_enabled, version,
                created_at, updated_at
            FROM node_graph_defs
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(graph_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(NodeEngineError::GraphNotFound(graph_id))?;

        graph_from_row(&row)
    }

    /// Get graphs triggered by entity events
    pub async fn get_graphs_for_entity_event(
        &self,
        tenant_id: Uuid,
        entity_type_id: Uuid,
    ) -> Result<Vec<NodeGraphDef>, NodeEngineError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, tenant_id, name, label, description,
                scope, graph_type,
                entity_type_id, app_id, is_enabled, version,
                created_at, updated_at
            FROM node_graph_defs
            WHERE tenant_id = $1 
              AND entity_type_id = $2 
              AND is_enabled = true
            ORDER BY created_at
            "#,
        )
        .bind(tenant_id)
        .bind(entity_type_id)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(graph_from_row).collect()
    }

    /// Get all nodes for a graph
    pub async fn get_nodes(&self, graph_id: Uuid) -> Result<Vec<NodeDef>, NodeEngineError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, graph_id, node_type,
                label, x, y, config, is_enabled
            FROM node_defs
            WHERE graph_id = $1
            ORDER BY id
            "#,
        )
        .bind(graph_id)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(node_from_row).collect()
    }

    /// Get all edges for a graph
    pub async fn get_edges(&self, graph_id: Uuid) -> Result<Vec<EdgeDef>, NodeEngineError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, graph_id, source_node_id, source_port,
                target_node_id, target_port, label
            FROM edge_defs
            WHERE graph_id = $1
            "#,
        )
        .bind(graph_id)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(edge_from_row).collect()
    }

    /// Get complete graph with nodes and edges
    pub async fn get_graph_complete(
        &self,
        tenant_id: Uuid,
        graph_id: Uuid,
    ) -> Result<(NodeGraphDef, Vec<NodeDef>, Vec<EdgeDef>), NodeEngineError> {
        let graph = self.get_graph(tenant_id, graph_id).await?;
        let nodes = self.get_nodes(graph_id).await?;
        let edges = self.get_edges(graph_id).await?;

        Ok((graph, nodes, edges))
    }
}

fn graph_from_row(row: &sqlx::postgres::PgRow) -> Result<NodeGraphDef, NodeEngineError> {
    use sqlx::Row;
    
    let scope_str: String = row.try_get("scope")?;
    let graph_type_str: String = row.try_get("graph_type")?;
    
    Ok(NodeGraphDef {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        name: row.try_get("name")?,
        label: row.try_get("label")?,
        description: row.try_get("description")?,
        scope: serde_json::from_str(&format!("\"{}\"", scope_str)).unwrap_or(GraphScope::Global),
        graph_type: serde_json::from_str(&format!("\"{}\"", graph_type_str)).unwrap_or(GraphType::Logic),
        entity_type_id: row.try_get("entity_type_id")?,
        app_id: row.try_get("app_id")?,
        is_enabled: row.try_get("is_enabled")?,
        version: row.try_get("version")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn node_from_row(row: &sqlx::postgres::PgRow) -> Result<NodeDef, NodeEngineError> {
    use sqlx::Row;
    
    let node_type_str: String = row.try_get("node_type")?;
    
    Ok(NodeDef {
        id: row.try_get("id")?,
        graph_id: row.try_get("graph_id")?,
        node_type: serde_json::from_str(&format!("\"{}\"", node_type_str)).unwrap_or(NodeType::TriggerManual),
        label: row.try_get("label")?,
        x: row.try_get("x")?,
        y: row.try_get("y")?,
        config: row.try_get("config")?,
        is_enabled: row.try_get("is_enabled")?,
    })
}

fn edge_from_row(row: &sqlx::postgres::PgRow) -> Result<EdgeDef, NodeEngineError> {
    use sqlx::Row;
    Ok(EdgeDef {
        id: row.try_get("id")?,
        graph_id: row.try_get("graph_id")?,
        source_node_id: row.try_get("source_node_id")?,
        source_port: row.try_get("source_port")?,
        target_node_id: row.try_get("target_node_id")?,
        target_port: row.try_get("target_port")?,
        label: row.try_get("label")?,
    })
}
