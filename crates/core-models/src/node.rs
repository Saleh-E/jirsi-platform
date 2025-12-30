//! Node model - Graph-based workflow/automation engine

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Scope of a node graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphScope {
    /// Global automation (cross-entity)
    Global,
    /// App-level automation
    App,
    /// Entity-specific automation
    EntityType,
    /// View-specific logic
    View,
    /// Record template (pre-fill)
    RecordTemplate,
}

/// Type of graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphType {
    /// UI layout/visibility logic
    Ui,
    /// Business logic/automation
    Logic,
    /// Data transformation
    Data,
    /// External integration
    Integration,
}

/// Node graph definition - a complete workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGraphDef {
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// Graph name
    pub name: String,
    /// Display label
    pub label: String,
    /// Description
    pub description: Option<String>,
    /// Scope of the graph
    pub scope: GraphScope,
    /// Type of graph
    pub graph_type: GraphType,
    /// For EntityType scope: which entity
    pub entity_type_id: Option<Uuid>,
    /// For App scope: which app
    pub app_id: Option<String>,
    /// Is graph active?
    pub is_enabled: bool,
    /// Version for history tracking
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Type of node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    // Triggers
    TriggerOnCreate,
    TriggerOnUpdate,
    TriggerOnDelete,
    TriggerOnFieldChange,
    TriggerOnEvent,
    TriggerSchedule,
    TriggerManual,
    /// State machine trigger - fires on state transitions
    TriggerStateChange,

    // Conditions
    ConditionIf,
    ConditionSwitch,
    ConditionFilter,

    // Data
    DataGetRecord,
    DataQueryRecords,
    DataSetField,
    DataCreateRecord,
    DataUpdateRecord,
    DataDeleteRecord,

    // Actions
    ActionSendEmail,
    ActionSendSms,
    /// WhatsApp Business API integration (Meta Cloud API)
    ActionWhatsapp,
    ActionSendWebhook,
    ActionCreateInteraction,
    ActionCreateTask,
    ActionScheduleMeeting,
    ActionDelay,
    /// Collect payment via Stripe/PaymentProvider
    ActionCollectPayment,

    // AI
    AiSummarize,
    AiClassify,
    AiExtract,
    AiGenerate,
    /// RAG-powered context-aware AI (replaces generic AiGenerate for production)
    AiContextAware,
    
    // Logic/Intelligence (Real Estate)
    /// Smart matching - matches demand (leads/requirements) with supply (properties)
    LogicMatch,
    /// Geo-fence check - validates coordinates within a target zone
    LogicGeoFence,
    
    // User-defined logic (WASM)
    ScriptNode,

    // Flow control
    FlowMerge,
    FlowSplit,
    FlowLoop,
    FlowSubGraph,

    // UI (for UI graphs)
    UiShowField,
    UiHideField,
    UiSetRequired,
    UiSetReadonly,
}

/// A node in a graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDef {
    pub id: Uuid,
    pub graph_id: Uuid,
    /// Node type
    pub node_type: NodeType,
    /// Display label
    pub label: String,
    /// X position on canvas
    pub x: f32,
    /// Y position on canvas
    pub y: f32,
    /// Node-specific configuration
    pub config: serde_json::Value,
    /// Is node enabled?
    pub is_enabled: bool,
}

/// An edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDef {
    pub id: Uuid,
    pub graph_id: Uuid,
    /// Source node ID
    pub source_node_id: Uuid,
    /// Source port name
    pub source_port: String,
    /// Target node ID
    pub target_node_id: Uuid,
    /// Target port name
    pub target_port: String,
    /// Edge label (for conditions)
    pub label: Option<String>,
}

/// Port definition for a node type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDef {
    /// Port name
    pub name: String,
    /// Display label
    pub label: String,
    /// Is this an input port?
    pub is_input: bool,
    /// Data type expected
    pub data_type: String,
    /// Is connection required?
    pub is_required: bool,
    /// Allow multiple connections?
    pub allow_multiple: bool,
}

/// Execution log for a graph run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExecution {
    pub id: Uuid,
    pub graph_id: Uuid,
    pub tenant_id: Uuid,
    /// The event that triggered this execution
    pub trigger_event_id: Option<Uuid>,
    /// Record that triggered (if applicable)
    pub trigger_record_id: Option<Uuid>,
    /// Execution status
    pub status: ExecutionStatus,
    /// When execution started
    pub started_at: DateTime<Utc>,
    /// When execution completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution log/trace
    pub log: serde_json::Value,
}

/// Status of a graph execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl NodeGraphDef {
    pub fn new(tenant_id: Uuid, name: &str, label: &str, graph_type: GraphType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            name: name.to_string(),
            label: label.to_string(),
            description: None,
            scope: GraphScope::Global,
            graph_type,
            entity_type_id: None,
            app_id: None,
            is_enabled: true,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn for_entity(mut self, entity_type_id: Uuid) -> Self {
        self.scope = GraphScope::EntityType;
        self.entity_type_id = Some(entity_type_id);
        self
    }

    pub fn for_app(mut self, app_id: &str) -> Self {
        self.scope = GraphScope::App;
        self.app_id = Some(app_id.to_string());
        self
    }
}

impl NodeDef {
    pub fn new(graph_id: Uuid, node_type: NodeType, label: &str, x: f32, y: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            graph_id,
            node_type,
            label: label.to_string(),
            x,
            y,
            config: serde_json::json!({}),
            is_enabled: true,
        }
    }

    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.config = config;
        self
    }
}

impl EdgeDef {
    pub fn new(
        graph_id: Uuid,
        source_node_id: Uuid,
        source_port: &str,
        target_node_id: Uuid,
        target_port: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            graph_id,
            source_node_id,
            source_port: source_port.to_string(),
            target_node_id,
            target_port: target_port.to_string(),
            label: None,
        }
    }
}
