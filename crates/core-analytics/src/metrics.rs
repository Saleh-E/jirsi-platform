//! Metrics and dashboard models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Aggregation function for metrics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregationType {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    CountDistinct,
}

/// Metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDef {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    /// Entity type to aggregate
    pub entity_type: String,
    /// Aggregation function
    pub aggregation: AggregationType,
    /// Field to aggregate (for sum/avg)
    pub field: Option<String>,
    /// Filter conditions (JSON)
    pub filters: serde_json::Value,
    /// Format string for display
    pub format: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dashboard definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardDef {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    /// Is this the default dashboard?
    pub is_default: bool,
    /// Is this a system dashboard?
    pub is_system: bool,
    /// Layout configuration
    pub layout: serde_json::Value,
    /// Widgets configuration
    pub widgets: Vec<DashboardWidget>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dashboard widget types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetType {
    Kpi,
    Chart,
    Table,
    List,
    Funnel,
    Progress,
}

/// A widget on a dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: Uuid,
    pub widget_type: WidgetType,
    pub title: String,
    /// Grid position (x, y, width, height)
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    /// Metric ID for KPI widgets
    pub metric_id: Option<Uuid>,
    /// Configuration for the widget
    pub config: serde_json::Value,
}

/// A computed metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub metric_id: Uuid,
    pub value: f64,
    pub formatted_value: String,
    pub computed_at: DateTime<Utc>,
}
