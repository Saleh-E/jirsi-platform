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
/// - `CounterMerge`: CRDT counter for numeric aggregations
/// - `SetMerge`: CRDT set for multi-select/tags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum MergeStrategy {
    /// Standard database behavior - latest write wins
    /// Use for: IDs, foreign keys, status fields, most scalar values
    #[default]
    LastWriteWins,
    
    /// Yjs CRDT text merging for collaborative editing
    /// Use for: Descriptions, notes, rich text content
    TextMerge,
    
    /// Append-only - never overwrite, always add
    /// Use for: Audit logs, activity history, comments
    AppendOnly,
    
    /// CRDT counter for concurrent numeric operations
    /// Use for: View counts, like counts, inventory decrements
    CounterMerge,
    
    /// CRDT set for multi-value fields
    /// Use for: Tags, multi-select, assignees
    SetMerge,
    
    /// Custom merge function (identified by name)
    /// Use for: Complex domain-specific merge logic
    Custom { handler: String },
}

impl MergeStrategy {
    /// Returns true if this strategy requires CRDT infrastructure
    pub fn requires_crdt(&self) -> bool {
        matches!(
            self,
            MergeStrategy::TextMerge 
            | MergeStrategy::CounterMerge 
            | MergeStrategy::SetMerge
        )
    }
    
    /// Returns the Yjs type name for CRDT strategies
    pub fn yjs_type(&self) -> Option<&'static str> {
        match self {
            MergeStrategy::TextMerge => Some("YText"),
            MergeStrategy::CounterMerge => Some("YMap"), // Using map for counter
            MergeStrategy::SetMerge => Some("YArray"),
            _ => None,
        }
    }
}

// ============================================================================
// INTELLIGENCE LAYER - AI Metadata
// ============================================================================

/// AI/LLM metadata for intelligent features
/// 
/// Provides hints to AI systems about how to handle this field:
/// - Privacy (PII redaction)
/// - Embedding (vector search)
/// - Generation (magic fill)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AiMetadata {
    /// Human-readable description for LLM context
    /// Example: "The property's asking price in local currency"
    #[serde(default)]
    pub description: Option<String>,
    
    /// Is this Personally Identifiable Information?
    /// If true: Redact before sending to external AI, exclude from exports
    #[serde(default)]
    pub is_pii: bool,
    
    /// Should this field be vector-indexed for semantic search?
    /// If true: Include in embedding generation pipeline
    #[serde(default)]
    pub embed: bool,
    
    /// Can AI auto-generate this field's value?
    /// If true: Show "✨ Magic" button in UI
    #[serde(default)]
    pub auto_generate: bool,
    
    /// AI generation prompt template
    /// Example: "Generate a property description based on: {title}, {bedrooms}, {location}"
    #[serde(default)]
    pub generation_prompt: Option<String>,
    
    /// Semantic type hint for AI understanding
    /// Examples: "person_name", "company", "address", "currency_amount"
    #[serde(default)]
    pub semantic_type: Option<String>,
    
    /// Confidence threshold for AI suggestions (0.0 - 1.0)
    #[serde(default)]
    pub suggestion_threshold: Option<f32>,
}

impl AiMetadata {
    /// Create metadata for a PII field
    pub fn pii() -> Self {
        Self {
            is_pii: true,
            ..Default::default()
        }
    }
    
    /// Create metadata for an embeddable field
    pub fn embeddable(description: impl Into<String>) -> Self {
        Self {
            description: Some(description.into()),
            embed: true,
            ..Default::default()
        }
    }
    
    /// Create metadata for an auto-generatable field
    pub fn auto_gen(prompt: impl Into<String>) -> Self {
        Self {
            auto_generate: true,
            generation_prompt: Some(prompt.into()),
            ..Default::default()
        }
    }
}

// ============================================================================
// LAYOUT LAYER - Adaptive UI Configuration
// ============================================================================

/// Layout configuration for form/detail rendering
/// 
/// Controls how the field appears in different UI contexts:
/// - Grid span (1-12 columns)
/// - Section grouping
/// - Conditional visibility
/// - Dynamic readonly state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutConfig {
    /// Grid span (1-12), where 12 = full width
    #[serde(default = "default_form_span")]
    pub form_span: u8,
    
    /// Section/group this field belongs to
    /// Example: "basic_info", "pricing", "location"
    #[serde(default)]
    pub section: Option<String>,
    
    /// Tab this field appears in (for tabbed forms)
    #[serde(default)]
    pub tab: Option<String>,
    
    /// Display order within section (lower = earlier)
    #[serde(default)]
    pub order: i32,
    
    /// Condition for visibility (evaluates to bool)
    /// Default: Always visible
    #[serde(default = "default_always")]
    pub visible_if: super::logic::LogicOp,
    
    /// Condition for readonly state (evaluates to bool)
    /// Default: Never readonly (always editable)
    #[serde(default = "default_never")]
    pub readonly_if: super::logic::LogicOp,
    
    /// Condition for requirement (dynamic required)
    /// Default: Uses field's is_required
    #[serde(default)]
    pub required_if: Option<super::logic::LogicOp>,
    
    /// CSS class names for custom styling
    #[serde(default)]
    pub css_class: Option<String>,
    
    /// Mobile-specific span override (1-12)
    #[serde(default)]
    pub mobile_span: Option<u8>,
    
    /// Hide label in form
    #[serde(default)]
    pub hide_label: bool,
    
    /// Render as inline element
    #[serde(default)]
    pub inline: bool,
}

fn default_form_span() -> u8 {
    12 // Full width by default
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
            tab: None,
            order: 0,
            visible_if: default_always(),
            readonly_if: default_never(),
            required_if: None,
            css_class: None,
            mobile_span: None,
            hide_label: false,
            inline: false,
        }
    }
}

impl LayoutConfig {
    /// Create a layout with specific column span
    pub fn span(columns: u8) -> Self {
        Self {
            form_span: columns.min(12),
            ..Default::default()
        }
    }
    
    /// Create a layout in a specific section
    pub fn in_section(section: impl Into<String>) -> Self {
        Self {
            section: Some(section.into()),
            ..Default::default()
        }
    }
    
    /// Builder: Set visibility condition
    pub fn visible_when(mut self, condition: super::logic::LogicOp) -> Self {
        self.visible_if = condition;
        self
    }
    
    /// Builder: Set readonly condition
    pub fn readonly_when(mut self, condition: super::logic::LogicOp) -> Self {
        self.readonly_if = condition;
        self
    }
    
    /// Builder: Set section
    pub fn section(mut self, section: impl Into<String>) -> Self {
        self.section = Some(section.into());
        self
    }
    
    /// Builder: Set order
    pub fn order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::LogicOp;
    
    #[test]
    fn test_merge_strategy_defaults() {
        let strategy: MergeStrategy = Default::default();
        assert_eq!(strategy, MergeStrategy::LastWriteWins);
        assert!(!strategy.requires_crdt());
    }
    
    #[test]
    fn test_crdt_detection() {
        assert!(MergeStrategy::TextMerge.requires_crdt());
        assert!(MergeStrategy::SetMerge.requires_crdt());
        assert!(!MergeStrategy::LastWriteWins.requires_crdt());
    }
    
    #[test]
    fn test_ai_metadata_builders() {
        let pii = AiMetadata::pii();
        assert!(pii.is_pii);
        assert!(!pii.embed);
        
        let embed = AiMetadata::embeddable("Property description");
        assert!(embed.embed);
        assert_eq!(embed.description, Some("Property description".into()));
    }
    
    #[test]
    fn test_layout_config_builders() {
        let layout = LayoutConfig::span(6)
            .section("pricing")
            .visible_when(LogicOp::HasRole { role: "admin".into() });
        
        assert_eq!(layout.form_span, 6);
        assert_eq!(layout.section, Some("pricing".into()));
    }
    
    #[test]
    fn test_serde_roundtrip() {
        let layout = LayoutConfig {
            form_span: 6,
            section: Some("details".into()),
            visible_if: LogicOp::HasRole { role: "admin".into() },
            ..Default::default()
        };
        
        let json = serde_json::to_string(&layout).unwrap();
        let parsed: LayoutConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.form_span, 6);
        assert_eq!(parsed.section, Some("details".into()));
    }
}
