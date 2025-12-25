//! Workflow module - Visual node graph editor

pub mod canvas;
pub mod pipe;
pub mod execution_panel;

pub use canvas::{NodeGraphCanvas, NodeInstance, Edge, Position, Port};
pub use pipe::WorkflowPipe;
pub use execution_panel::ExecutionPanel;
