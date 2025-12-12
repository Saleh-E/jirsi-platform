//! Graph executor - runs node graphs

use chrono::Utc;
use core_models::{
    EdgeDef, ExecutionStatus, GraphExecution, NodeDef, NodeGraphDef, NodeType,
};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::nodes::NodeRegistry;
use crate::NodeEngineError;

/// Graph executor - runs node graphs
pub struct GraphExecutor {
    pool: PgPool,
    registry: NodeRegistry,
}

impl GraphExecutor {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            registry: NodeRegistry::new(),
        }
    }

    /// Execute a graph for a given trigger event
    #[instrument(skip(self, trigger_data))]
    pub async fn execute(
        &self,
        graph: &NodeGraphDef,
        nodes: &[NodeDef],
        edges: &[EdgeDef],
        trigger_data: Value,
    ) -> Result<GraphExecution, NodeEngineError> {
        info!(graph_id = %graph.id, "Starting graph execution");

        let execution_id = Uuid::new_v4();
        let started_at = Utc::now();

        // Create execution record
        let mut execution = GraphExecution {
            id: execution_id,
            graph_id: graph.id,
            tenant_id: graph.tenant_id,
            trigger_event_id: None,
            trigger_record_id: None,
            status: ExecutionStatus::Running,
            started_at,
            completed_at: None,
            error: None,
            log: serde_json::json!({ "steps": [] }),
        };

        // Build execution context
        let mut context = ExecutionContext {
            values: HashMap::new(),
            logs: Vec::new(),
        };

        // Initialize with trigger data
        context.values.insert("$trigger".to_string(), trigger_data);

        // Topological sort
        let sorted_nodes = self.topological_sort(nodes, edges)?;

        // Execute nodes in order
        for node in sorted_nodes {
            if !node.is_enabled {
                debug!(node_id = %node.id, "Skipping disabled node");
                continue;
            }

            let result = self.execute_node(&node, edges, &mut context).await;

            match result {
                Ok(output) => {
                    context.values.insert(node.id.to_string(), output.clone());
                    context.logs.push(serde_json::json!({
                        "node_id": node.id,
                        "label": node.label,
                        "status": "success",
                        "output": output,
                    }));
                }
                Err(e) => {
                    error!(node_id = %node.id, error = %e, "Node execution failed");
                    execution.status = ExecutionStatus::Failed;
                    execution.error = Some(e.to_string());
                    execution.completed_at = Some(Utc::now());
                    execution.log = serde_json::json!({ "steps": context.logs });
                    return Ok(execution);
                }
            }
        }

        execution.status = ExecutionStatus::Completed;
        execution.completed_at = Some(Utc::now());
        execution.log = serde_json::json!({ "steps": context.logs });

        info!(
            graph_id = %graph.id,
            execution_id = %execution.id,
            "Graph execution completed"
        );

        Ok(execution)
    }

    /// Execute a single node
    async fn execute_node(
        &self,
        node: &NodeDef,
        edges: &[EdgeDef],
        context: &mut ExecutionContext,
    ) -> Result<Value, NodeEngineError> {
        // Gather inputs from connected nodes
        let inputs = self.gather_inputs(node, edges, context)?;

        // Execute the node
        let handler = self.registry.get_handler(&node.node_type)?;
        handler.execute(node, inputs, context).await
    }

    /// Gather inputs for a node from connected edges
    fn gather_inputs(
        &self,
        node: &NodeDef,
        edges: &[EdgeDef],
        context: &ExecutionContext,
    ) -> Result<HashMap<String, Value>, NodeEngineError> {
        let mut inputs = HashMap::new();

        // Find all edges targeting this node
        for edge in edges.iter().filter(|e| e.target_node_id == node.id) {
            let source_value = context
                .values
                .get(&edge.source_node_id.to_string())
                .cloned()
                .unwrap_or(Value::Null);

            inputs.insert(edge.target_port.clone(), source_value);
        }

        Ok(inputs)
    }

    /// Topological sort of nodes
    fn topological_sort(
        &self,
        nodes: &[NodeDef],
        edges: &[EdgeDef],
    ) -> Result<Vec<NodeDef>, NodeEngineError> {
        let mut result = Vec::new();
        let mut visited = HashMap::new();
        let mut temp_visited = HashMap::new();

        let node_map: HashMap<_, _> = nodes.iter().map(|n| (n.id, n.clone())).collect();

        for node in nodes {
            if !visited.contains_key(&node.id) {
                self.visit_node(
                    &node.id,
                    &node_map,
                    edges,
                    &mut visited,
                    &mut temp_visited,
                    &mut result,
                )?;
            }
        }

        Ok(result)
    }

    fn visit_node(
        &self,
        node_id: &Uuid,
        node_map: &HashMap<Uuid, NodeDef>,
        edges: &[EdgeDef],
        visited: &mut HashMap<Uuid, bool>,
        temp_visited: &mut HashMap<Uuid, bool>,
        result: &mut Vec<NodeDef>,
    ) -> Result<(), NodeEngineError> {
        if temp_visited.get(node_id).copied().unwrap_or(false) {
            return Err(NodeEngineError::CycleDetected);
        }

        if visited.get(node_id).copied().unwrap_or(false) {
            return Ok(());
        }

        temp_visited.insert(*node_id, true);

        // Visit all dependencies (nodes that feed into this one)
        for edge in edges.iter().filter(|e| e.target_node_id == *node_id) {
            self.visit_node(
                &edge.source_node_id,
                node_map,
                edges,
                visited,
                temp_visited,
                result,
            )?;
        }

        temp_visited.insert(*node_id, false);
        visited.insert(*node_id, true);

        if let Some(node) = node_map.get(node_id) {
            result.push(node.clone());
        }

        Ok(())
    }
}

/// Execution context - holds values during graph execution
pub struct ExecutionContext {
    pub values: HashMap<String, Value>,
    pub logs: Vec<Value>,
}
