//! Workflow module - Visual node graph editor
//! 
//! ## Antigravity Integration
//! Includes LogicBuilder for visual rule editing.

pub mod canvas;
pub mod pipe;
pub mod execution_panel;
pub mod workflow_canvas;
pub mod node_inspector;
pub mod node_palette;
pub mod workflow_node;
pub mod logic_builder;

pub use canvas::{NodeGraphCanvas, NodeInstance, Edge, Position, Port};
pub use pipe::WorkflowPipe;
pub use execution_panel::ExecutionPanel;
pub use workflow_canvas::{WorkflowCanvas, NodeUI, EdgeUI};
pub use node_inspector::NodeInspector;
pub use node_palette::NodePalette;
pub use logic_builder::LogicBuilder;
