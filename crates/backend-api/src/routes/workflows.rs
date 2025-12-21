//! Workflow Execution Engine
//! Handles automatic workflow execution when triggers fire

use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use tracing::{info, warn, error};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowDef {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub trigger_type: String,  // "field_changed", "record_created", "form_submitted"
    pub trigger_entity: String,
    pub trigger_config: Value,
    pub conditions: Value,
    pub actions: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowAction {
    pub id: String,
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(flatten)]
    pub config: Value,
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // old_values used in future comparison logic
pub struct WorkflowContext {
    pub tenant_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub old_values: Option<Value>,
    pub new_values: Value,
    pub variables: std::collections::HashMap<String, Value>,
}

/// Execute workflows triggered by an entity change
pub async fn execute_triggered_workflows(
    pool: &PgPool,
    tenant_id: Uuid,
    trigger_type: &str,
    entity_type: &str,
    entity_id: Uuid,
    old_values: Option<Value>,
    new_values: Value,
) -> Result<Vec<Uuid>, String> {
    // Find matching workflows
    let workflows = find_matching_workflows(pool, tenant_id, trigger_type, entity_type, &old_values, &new_values).await?;
    
    if workflows.is_empty() {
        return Ok(vec![]);
    }
    
    let mut executed_ids = Vec::new();
    
    for workflow in workflows {
        info!("Executing workflow: {} ({})", workflow.name, workflow.id);
        
        // Create execution context
        let mut context = WorkflowContext {
            tenant_id,
            entity_type: entity_type.to_string(),
            entity_id,
            old_values: old_values.clone(),
            new_values: new_values.clone(),
            variables: std::collections::HashMap::new(),
        };
        
        // Execute workflow actions
        match execute_workflow_actions(pool, &workflow, &mut context).await {
            Ok(_) => {
                info!("Workflow {} completed successfully", workflow.id);
                log_workflow_execution(pool, &workflow, &context, true, None).await;
                executed_ids.push(workflow.id);
            }
            Err(e) => {
                error!("Workflow {} failed: {}", workflow.id, e);
                log_workflow_execution(pool, &workflow, &context, false, Some(&e)).await;
            }
        }
    }
    
    Ok(executed_ids)
}

/// Find workflows matching the current trigger
async fn find_matching_workflows(
    pool: &PgPool,
    tenant_id: Uuid,
    trigger_type: &str,
    entity_type: &str,
    old_values: &Option<Value>,
    new_values: &Value,
) -> Result<Vec<WorkflowDef>, String> {
    // Use runtime query instead of compile-time query_as! macro
    let rows: Vec<(Uuid, Uuid, String, String, String, Value, Value, Value)> = sqlx::query_as(
        r#"
        SELECT 
            id, tenant_id, name, 
            trigger_type, trigger_entity, trigger_config,
            conditions, actions
        FROM workflow_defs
        WHERE tenant_id = $1 
          AND trigger_type = $2 
          AND trigger_entity = $3
          AND is_active = true
        "#
    )
    .bind(tenant_id)
    .bind(trigger_type)
    .bind(entity_type)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch workflows: {}", e))?;
    
    let workflows: Vec<WorkflowDef> = rows.into_iter().map(|row| {
        WorkflowDef {
            id: row.0,
            tenant_id: row.1,
            name: row.2,
            trigger_type: row.3,
            trigger_entity: row.4,
            trigger_config: row.5,
            conditions: row.6,
            actions: row.7,
        }
    }).collect();
    
    // Filter by specific trigger conditions
    let matching: Vec<WorkflowDef> = workflows.into_iter()
        .filter(|wf| check_trigger_condition(wf, old_values, new_values))
        .collect();
    
    Ok(matching)
}

/// Check if workflow trigger condition matches
fn check_trigger_condition(
    workflow: &WorkflowDef,
    old_values: &Option<Value>,
    new_values: &Value,
) -> bool {
    // For field_changed triggers, check if specific field changed to target value
    if workflow.trigger_type == "field_changed" {
        let field = workflow.trigger_config.get("field")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let to_value = workflow.trigger_config.get("to")
            .and_then(|v| v.as_str());
        
        let new_field_value = new_values.get(field)
            .and_then(|v| v.as_str());
        let old_field_value = old_values.as_ref()
            .and_then(|ov| ov.get(field))
            .and_then(|v| v.as_str());
        
        // Field must have changed to target value
        if let Some(target) = to_value {
            return new_field_value == Some(target) && old_field_value != Some(target);
        }
    }
    
    // For record_created, always match
    if workflow.trigger_type == "record_created" {
        return old_values.is_none();
    }
    
    true
}

/// Execute all actions in a workflow
async fn execute_workflow_actions(
    pool: &PgPool,
    workflow: &WorkflowDef,
    context: &mut WorkflowContext,
) -> Result<(), String> {
    let actions: Vec<WorkflowAction> = serde_json::from_value(workflow.actions.clone())
        .map_err(|e| format!("Invalid actions JSON: {}", e))?;
    
    for action in actions {
        execute_action(pool, &action, context).await?;
    }
    
    Ok(())
}

/// Execute a single workflow action
async fn execute_action(
    pool: &PgPool,
    action: &WorkflowAction,
    context: &mut WorkflowContext,
) -> Result<(), String> {
    info!("Executing action: {} ({})", action.id, action.action_type);
    
    match action.action_type.as_str() {
        "update_record" => {
            execute_update_record(pool, action, context).await
        }
        "create_record" => {
            execute_create_record(pool, action, context).await
        }
        "log_activity" => {
            execute_log_activity(pool, action, context).await
        }
        "send_notification" => {
            execute_send_notification(pool, action, context).await
        }
        "upsert_record" => {
            execute_upsert_record(pool, action, context).await
        }
        "assign_agent" => {
            execute_assign_agent(pool, action, context).await
        }
        _ => {
            warn!("Unknown action type: {}", action.action_type);
            Ok(())
        }
    }
}

// Action: Update an existing record
async fn execute_update_record(
    pool: &PgPool,
    action: &WorkflowAction,
    context: &mut WorkflowContext,
) -> Result<(), String> {
    let set_fields = action.config.get("set_fields")
        .ok_or("Missing set_fields in update_record")?;
    
    // Resolve record_id from context variables
    let record_id_str = action.config.get("record_id")
        .and_then(|v| v.as_str())
        .map(|s| resolve_variable(s, context))
        .unwrap_or_else(|| context.entity_id.to_string());
    
    let record_id = Uuid::parse_str(&record_id_str)
        .map_err(|e| format!("Invalid record_id: {}", e))?;
    
    let resolved_fields = resolve_fields(set_fields, context);
    
    // Update using JSONB merge on entity_records table
    sqlx::query(
        "UPDATE entity_records SET data = data || $1, updated_at = NOW() WHERE id = $2 AND tenant_id = $3"
    )
    .bind(resolved_fields)
    .bind(record_id)
    .bind(context.tenant_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update record: {}", e))?;
    
    Ok(())
}

// Action: Create a new record
async fn execute_create_record(
    pool: &PgPool,
    action: &WorkflowAction,
    context: &mut WorkflowContext,
) -> Result<(), String> {
    let entity_code = action.config.get("entity")
        .and_then(|v| v.as_str())
        .ok_or("Missing entity in create_record")?;
    
    let set_fields = action.config.get("set_fields")
        .ok_or("Missing set_fields in create_record")?;
    
    let resolved_fields = resolve_fields(set_fields, context);
    let new_id = Uuid::new_v4();
    
    // Store output variable if specified
    if let Some(output_var) = action.config.get("output_var").and_then(|v| v.as_str()) {
        context.variables.insert(
            format!("{}.id", output_var),
            json!(new_id.to_string())
        );
    }
    
    // Create record in entity_records
    sqlx::query(
        "INSERT INTO entity_records (id, tenant_id, entity_type_id, data, created_at, updated_at) 
         SELECT $1, $2, id, $3, NOW(), NOW() FROM entity_types WHERE name = $4 AND tenant_id = $5"
    )
    .bind(new_id)
    .bind(context.tenant_id)
    .bind(resolved_fields)
    .bind(entity_code)
    .bind(context.tenant_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create record in entity_records: {}", e))?;
    
    info!("Created {} record: {}", entity_code, new_id);
    Ok(())
}

// Action: Log activity
async fn execute_log_activity(
    pool: &PgPool,
    action: &WorkflowAction,
    context: &mut WorkflowContext,
) -> Result<(), String> {
    let entity_type = action.config.get("entity_type")
        .and_then(|v| v.as_str())
        .unwrap_or(&context.entity_type);
    
    let entity_id_str = action.config.get("entity_id")
        .and_then(|v| v.as_str())
        .map(|s| resolve_variable(s, context))
        .unwrap_or_else(|| context.entity_id.to_string());
    
    let entity_id = Uuid::parse_str(&entity_id_str).ok();
    
    let activity_type = action.config.get("activity_type")
        .and_then(|v| v.as_str())
        .unwrap_or("workflow_action");
    
    let title = action.config.get("title")
        .and_then(|v| v.as_str())
        .map(|s| resolve_variable(s, context))
        .unwrap_or_else(|| "Workflow executed".to_string());
    
    let content = action.config.get("content")
        .and_then(|v| v.as_str())
        .map(|s| resolve_variable(s, context));

    sqlx::query(
        "INSERT INTO interactions (id, tenant_id, interaction_type, title, content, entity_type, entity_id, occurred_at, created_at, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW(), '00000000-0000-0000-0000-000000000000')"
    )
    .bind(Uuid::new_v4())
    .bind(context.tenant_id)
    .bind(activity_type)
    .bind(&title)
    .bind(&content)
    .bind(entity_type)
    .bind(entity_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to log interaction: {}", e))?;
    
    info!("Logged activity: {}", title);
    Ok(())
}

// Action: Send notification (placeholder - would integrate with email/SMS service)
async fn execute_send_notification(
    _pool: &PgPool,
    action: &WorkflowAction,
    context: &mut WorkflowContext,
) -> Result<(), String> {
    let channel = action.config.get("channel")
        .and_then(|v| v.as_str())
        .unwrap_or("email");
    
    let template = action.config.get("template")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    
    let to = action.config.get("to")
        .and_then(|v| v.as_str())
        .map(|s| resolve_variable(s, context))
        .unwrap_or_default();
    
    // In production, this would call an email service
    info!("Would send {} notification using template '{}' to: {}", channel, template, to);
    
    Ok(())
}

// Action: Upsert record (create or update)
async fn execute_upsert_record(
    pool: &PgPool,
    action: &WorkflowAction,
    context: &mut WorkflowContext,
) -> Result<(), String> {
    // For now, just create - in production would check match_fields first
    execute_create_record(pool, action, context).await
}

// Action: Assign agent
async fn execute_assign_agent(
    _pool: &PgPool,
    action: &WorkflowAction,
    context: &mut WorkflowContext,
) -> Result<(), String> {
    let method = action.config.get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("round_robin");
    
    // In production, would implement actual assignment logic
    info!("Would assign agent using method: {}", method);
    
    // Store placeholder agent in context
    if let Some(output_var) = action.config.get("output_var").and_then(|v| v.as_str()) {
        context.variables.insert(
            format!("{}.id", output_var),
            json!(Uuid::new_v4().to_string())
        );
    }
    
    Ok(())
}

/// Resolve a template variable like {{entity.field}} or {{variable.property}}
fn resolve_variable(template: &str, context: &WorkflowContext) -> String {
    let mut result = template.to_string();
    
    // Replace {{entity.field}} patterns
    let re = regex::Regex::new(r"\{\{(\w+)\.(\w+)\}\}").unwrap();
    for cap in re.captures_iter(template) {
        let full_match = cap.get(0).unwrap().as_str();
        let object = cap.get(1).unwrap().as_str();
        let field = cap.get(2).unwrap().as_str();
        
        let value = match object {
            "offer" | "property" | "contact" | "deal" => {
                context.new_values.get(field)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }
            _ => {
                context.variables.get(&format!("{}.{}", object, field))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }
        };
        
        if let Some(v) = value {
            result = result.replace(full_match, &v);
        }
    }
    
    result
}

/// Resolve all fields in a set_fields object
fn resolve_fields(set_fields: &Value, context: &WorkflowContext) -> Value {
    if let Some(obj) = set_fields.as_object() {
        let mut resolved = serde_json::Map::new();
        for (key, value) in obj {
            if let Some(s) = value.as_str() {
                resolved.insert(key.clone(), json!(resolve_variable(s, context)));
            } else {
                resolved.insert(key.clone(), value.clone());
            }
        }
        Value::Object(resolved)
    } else {
        set_fields.clone()
    }
}

// Removed entity_to_table as we now use entity_records for everything

/// Log workflow execution to database
async fn log_workflow_execution(
    pool: &PgPool,
    workflow: &WorkflowDef,
    _context: &WorkflowContext,
    success: bool,
    error: Option<&str>,
) {
    let _ = sqlx::query(
        "INSERT INTO workflow_executions (id, tenant_id, workflow_id, status, error_message, executed_at)
         VALUES ($1, $2, $3, $4, $5, NOW())"
    )
    .bind(Uuid::new_v4())
    .bind(workflow.tenant_id)
    .bind(workflow.id)
    .bind(if success { "completed" } else { "failed" })
    .bind(error)
    .execute(pool)
    .await;
}
