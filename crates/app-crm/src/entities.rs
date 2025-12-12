//! CRM entity definitions

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// CRM Contact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company_id: Option<Uuid>,
    pub job_title: Option<String>,
    pub lifecycle_stage: String,
    pub lead_source: Option<String>,
    pub owner_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CRM Company
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub industry: Option<String>,
    pub size: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub owner_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CRM Deal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub amount: Option<i64>, // cents
    pub currency: String,
    pub stage: String,
    pub pipeline_id: Uuid,
    pub probability: Option<i32>,
    pub expected_close_date: Option<chrono::NaiveDate>,
    pub actual_close_date: Option<chrono::NaiveDate>,
    pub contact_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub lost_reason: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CRM Task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: String,
    pub status: String,
    pub task_type: String,
    /// Linked entity type (contact, company, deal)
    pub linked_entity_type: Option<String>,
    pub linked_entity_id: Option<Uuid>,
    pub assignee_id: Option<Uuid>,
    pub created_by: Uuid,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sales Pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub is_default: bool,
    pub stages: Vec<PipelineStage>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pipeline Stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub id: String,
    pub name: String,
    pub order: i32,
    pub probability: i32,
    pub color: Option<String>,
}
