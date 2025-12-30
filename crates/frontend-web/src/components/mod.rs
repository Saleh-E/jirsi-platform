//! UI Components
//!
//! ## Antigravity Integration
//! Includes CollaborativeTextField for TextMerge physics (CRDT).

pub mod shell;
pub mod sidebar;
pub mod field;
pub mod field_renderer;
pub mod smart_field;
pub mod async_select;
pub mod association_modal;
pub mod entity_hover_card;
pub mod signature_pad;
pub mod location_map;
pub mod rule_builder;
pub mod workflow;
pub mod table;
pub mod form;
pub mod kanban;
pub mod calendar;
pub mod map;
pub mod log_activity_modal;
pub mod view_switcher;
pub mod create_modal;
pub mod canvas;
pub mod smart_select;
pub mod public;
pub mod stat_card;
pub mod chart;
pub mod inbox_thread_list;
pub mod message_bubble;
pub mod composer;
pub mod toast;
pub mod action_bar;
pub mod editable_table;
pub mod bottom_nav;
pub mod mobile_card;
pub mod bottom_sheet;
pub mod smart_input;
pub mod node_graph;
pub mod async_entity_select;
pub mod sync_indicator;
pub mod timeline;
pub mod filter_builder;
pub mod column_selector;
pub mod command_palette;
pub mod rich_text_editor;
pub mod conflict_resolver;
pub mod audit_timeline;
pub mod pwa_install;
pub mod collaborative_field;
pub mod holographic_shell;
pub mod neural_status;
pub mod dialer_widget;

pub use collaborative_field::{CollaborativeTextField, should_use_crdt};
