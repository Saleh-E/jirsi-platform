//! Workflow module - Visual node graph editor

pub mod canvas;
pub mod pipe;
pub mod execution_panel;
pub mod workflow_canvas;
pub mod node_inspector;
pub mod node_palette;
pub mod workflow_node;

pub use canvas::{NodeGraphCanvas, NodeInstance, Edge, Position, Port};
pub use pipe::WorkflowPipe;
pub use execution_panel::ExecutionPanel;
pub use workflow_canvas::{WorkflowCanvas, NodeUI, EdgeUI};
pub use node_inspector::NodeInspector;
pub use node_palette::NodePalette;

