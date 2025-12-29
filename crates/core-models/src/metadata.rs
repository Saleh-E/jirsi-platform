//! Diamond Metadata - Physics and Intelligence layers for Antigravity
//!
//! This module defines the additional "dimensions" of field definitions:
//! - **Physics**: How data synchronizes across clients (CRDT strategy)
//! - **Intelligence**: AI/LLM hints for smart features
//! - **Layout**: Adaptive UI configuration

use serde::{Deserialize, Serialize};

// ============================================================================
// PHYSICS LAYER - Sync Strategy
// ============================================================================

/// Merge strategy for conflict resolution
/// 
/// Determines how concurrent edits to the same field are resolved:
/// - `LastWriteWins`: Standard DB behavior, latest timestamp wins
/// - `TextMerge`: Yjs CRDT for character-level text merging
/// - `AppendOnly`: Never overwrite, only append (audit logs)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum MergeStrategy {
    /// Standard database behavior - latest write wins
    #[default]
    LastWriteWins,
    
    /// Yjs CRDT text merging for collaborative editing
    TextMerge,
    
    /// Append-only - never overwrite, always add
    AppendOnly,
}

// ============================================================================
// INTELLIGENCE LAYER - AI Metadata
// ============================================================================

/// AI/LLM metadata for intelligent features
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AiMetadata {
    /// Human-readable description for LLM context
    #[serde(default)]
    pub description: Option<String>,
    
    /// Is this Personally Identifiable Information?
    #[serde(default)]
    pub is_pii: bool,
    
    /// Should this field be vector-indexed for semantic search?
    #[serde(default)]
    pub embed: bool,
    
    /// Can AI auto-generate this field's value?
    #[serde(default)]
    pub auto_generate: bool,
}

// ============================================================================
// LAYOUT LAYER - Adaptive UI Configuration
// ============================================================================

/// Layout configuration for form/detail rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutConfig {
    /// Grid span (1-12), where 12 = full width
    pub form_span: u8,
    
    /// Section/group this field belongs to
    pub section: Option<String>,
    
    /// Condition for visibility (evaluates to bool)
    #[serde(default = "default_always")]
    pub visible_if: super::logic::LogicOp,
    
    /// Condition for readonly state (evaluates to bool)
    #[serde(default = "default_never")]
    pub readonly_if: super::logic::LogicOp,
}

fn default_always() -> super::logic::LogicOp {
    super::logic::LogicOp::Always
}

fn default_never() -> super::logic::LogicOp {
    super::logic::LogicOp::Never
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            form_span: 12,
            section: None,
            visible_if: default_always(),
            readonly_if: default_never(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merge_strategy_defaults() {
        let strategy: MergeStrategy = Default::default();
        assert_eq!(strategy, MergeStrategy::LastWriteWins);
    }
    
    #[test]
    fn test_ai_metadata_defaults() {
        let ai: AiMetadata = Default::default();
        assert!(!ai.is_pii);
        assert!(!ai.embed);
        assert!(!ai.auto_generate);
    }
    
    #[test]
    fn test_layout_config_defaults() {
        let layout: LayoutConfig = Default::default();
        assert_eq!(layout.form_span, 12);
        assert!(layout.section.is_none());
    }
}
