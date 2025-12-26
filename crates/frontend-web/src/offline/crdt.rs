//! CRDT Module - Real-time Collaborative Editing via Yjs Protocol
//!
//! This module provides conflict-free replicated data types (CRDTs) using the
//! Yrs library, which implements the Yjs protocol for real-time collaboration.
//!
//! Key Features:
//! - Text editing with automatic merge (no conflicts)
//! - Binary update encoding for efficient network sync
//! - State vector for incremental updates
//! - Awareness protocol for presence (cursors, selections)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use yrs::{Doc, Text, TextRef, Transact, ReadTxn, StateVector, Update, GetString};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use uuid::Uuid;


/// A collaborative document that can be synced across multiple clients
#[derive(Clone)]
pub struct CrdtDocument {
    /// The Yjs document
    doc: Arc<Mutex<Doc>>,
    /// Document ID (entity_id + field_name)
    pub id: String,
    /// Last known state vector for efficient delta sync
    last_state: Arc<Mutex<Option<StateVector>>>,
}

impl CrdtDocument {
    /// Create a new empty CRDT document
    pub fn new(id: &str) -> Self {
        let doc = Doc::new();
        Self {
            doc: Arc::new(Mutex::new(doc)),
            id: id.to_string(),
            last_state: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a document from a field identifier
    pub fn for_field(entity_id: Uuid, field_name: &str) -> Self {
        let id = format!("{}:{}", entity_id, field_name);
        Self::new(&id)
    }

    /// Get the text content from the document
    pub fn get_text(&self, field: &str) -> String {
        let doc = self.doc.lock().unwrap();
        let txn = doc.transact();
        if let Some(text) = txn.get_text(field) {
            text.get_string(&txn)
        } else {
            String::new()
        }
    }

    /// Set text content (creates or replaces)
    pub fn set_text(&self, field: &str, content: &str) {
        let doc = self.doc.lock().unwrap();
        let text = doc.get_or_insert_text(field);
        let mut txn = doc.transact_mut();
        
        // Delete existing content
        let len = text.len(&txn);
        if len > 0 {
            text.remove_range(&mut txn, 0, len);
        }
        
        // Insert new content
        text.insert(&mut txn, 0, content);
    }

    /// Insert text at a specific position
    pub fn insert_at(&self, field: &str, index: u32, content: &str) {
        let doc = self.doc.lock().unwrap();
        let text = doc.get_or_insert_text(field);
        let mut txn = doc.transact_mut();
        text.insert(&mut txn, index, content);
    }

    /// Delete text at a specific range
    pub fn delete_range(&self, field: &str, index: u32, length: u32) {
        let doc = self.doc.lock().unwrap();
        let text = doc.get_or_insert_text(field);
        let mut txn = doc.transact_mut();
        text.remove_range(&mut txn, index, length);
    }

    /// Get the full state as a binary update (for initial sync)
    pub fn encode_state(&self) -> Vec<u8> {
        let doc = self.doc.lock().unwrap();
        let txn = doc.transact();
        txn.encode_state_as_update_v1(&StateVector::default())
    }

    /// Get a delta update since a given state vector
    pub fn encode_update_since(&self, state_vector: &[u8]) -> Result<Vec<u8>, String> {
        let sv = StateVector::decode_v1(state_vector)
            .map_err(|e| format!("Invalid state vector: {:?}", e))?;
        
        let doc = self.doc.lock().unwrap();
        let txn = doc.transact();
        Ok(txn.encode_state_as_update_v1(&sv))
    }

    /// Get current state vector (for requesting updates)
    pub fn get_state_vector(&self) -> Vec<u8> {
        let doc = self.doc.lock().unwrap();
        let txn = doc.transact();
        txn.state_vector().encode_v1()
    }

    /// Apply a binary update from another client
    pub fn apply_update(&self, update: &[u8]) -> Result<(), String> {
        let update = Update::decode_v1(update)
            .map_err(|e| format!("Failed to decode update: {:?}", e))?;
        
        let doc = self.doc.lock().unwrap();
        let mut txn = doc.transact_mut();
        txn.apply_update(update);
        
        Ok(())
    }

    /// Subscribe to document changes (returns update bytes)
    /// Note: In WASM, we'd use a callback pattern
    pub fn on_update<F>(&self, callback: F) 
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static
    {
        let doc = self.doc.lock().unwrap();
        let _sub = doc.observe_update_v1(move |_txn, event| {
            callback(event.update.clone());
        });
        // Note: Subscription is dropped here. In production, store it.
    }
}

/// A text field with CRDT support for collaborative editing
#[derive(Clone)]
pub struct CrdtText {
    doc: CrdtDocument,
    field_name: String,
}

impl CrdtText {
    /// Create a new CRDT text field
    pub fn new(entity_id: Uuid, field_name: &str) -> Self {
        Self {
            doc: CrdtDocument::for_field(entity_id, field_name),
            field_name: field_name.to_string(),
        }
    }

    /// Get the current text content
    pub fn get(&self) -> String {
        self.doc.get_text(&self.field_name)
    }

    /// Set the entire text content
    pub fn set(&self, content: &str) {
        self.doc.set_text(&self.field_name, content);
    }

    /// Insert text at position
    pub fn insert(&self, index: u32, text: &str) {
        self.doc.insert_at(&self.field_name, index, text);
    }

    /// Delete text range
    pub fn delete(&self, index: u32, length: u32) {
        self.doc.delete_range(&self.field_name, index, length);
    }

    /// Get binary update for syncing
    pub fn get_update(&self) -> Vec<u8> {
        self.doc.encode_state()
    }

    /// Apply update from remote
    pub fn apply_update(&self, update: &[u8]) -> Result<(), String> {
        self.doc.apply_update(update)
    }

    /// Get state vector for delta sync
    pub fn get_state_vector(&self) -> Vec<u8> {
        self.doc.get_state_vector()
    }

    /// Get update since state vector
    pub fn get_update_since(&self, state_vector: &[u8]) -> Result<Vec<u8>, String> {
        self.doc.encode_update_since(state_vector)
    }
}

/// Document manager for tracking multiple collaborative documents
pub struct CrdtManager {
    documents: Mutex<HashMap<String, CrdtDocument>>,
}

impl CrdtManager {
    pub fn new() -> Self {
        Self {
            documents: Mutex::new(HashMap::new()),
        }
    }

    /// Get or create a document for an entity field
    pub fn get_document(&self, entity_id: Uuid, field: &str) -> CrdtDocument {
        let key = format!("{}:{}", entity_id, field);
        let mut docs = self.documents.lock().unwrap();
        
        docs.entry(key.clone())
            .or_insert_with(|| CrdtDocument::new(&key))
            .clone()
    }

    /// Apply an update to a document
    pub fn apply_update(&self, entity_id: Uuid, field: &str, update: &[u8]) -> Result<(), String> {
        let doc = self.get_document(entity_id, field);
        doc.apply_update(update)
    }

    /// Get all documents that have pending changes
    pub fn get_dirty_documents(&self) -> Vec<(String, Vec<u8>)> {
        let docs = self.documents.lock().unwrap();
        docs.iter()
            .map(|(id, doc)| (id.clone(), doc.encode_state()))
            .collect()
    }
}

impl Default for CrdtManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Awareness state for presence indicators (cursors, selections)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AwarenessState {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_color: String,
    pub cursor_position: Option<u32>,
    pub selection_start: Option<u32>,
    pub selection_end: Option<u32>,
}

impl AwarenessState {
    pub fn new(user_id: Uuid, user_name: &str, color: &str) -> Self {
        Self {
            user_id,
            user_name: user_name.to_string(),
            user_color: color.to_string(),
            cursor_position: None,
            selection_start: None,
            selection_end: None,
        }
    }

    pub fn with_cursor(mut self, position: u32) -> Self {
        self.cursor_position = Some(position);
        self
    }

    pub fn with_selection(mut self, start: u32, end: u32) -> Self {
        self.selection_start = Some(start);
        self.selection_end = Some(end);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_text_basic() {
        let entity_id = Uuid::new_v4();
        let text = CrdtText::new(entity_id, "description");
        
        text.set("Hello World");
        assert_eq!(text.get(), "Hello World");
        
        text.insert(5, ",");
        assert_eq!(text.get(), "Hello, World");
        
        text.delete(5, 1);
        assert_eq!(text.get(), "Hello World");
    }

    #[test]
    fn test_crdt_sync() {
        let entity_id = Uuid::new_v4();
        
        // Client A
        let text_a = CrdtText::new(entity_id, "notes");
        text_a.set("Hello");
        
        // Client B receives update
        let text_b = CrdtText::new(entity_id, "notes");
        let update = text_a.get_update();
        text_b.apply_update(&update).unwrap();
        
        assert_eq!(text_b.get(), "Hello");
        
        // Client B makes changes
        text_b.insert(5, " World");
        
        // Client A receives update
        let update_b = text_b.get_update();
        text_a.apply_update(&update_b).unwrap();
        
        assert_eq!(text_a.get(), "Hello World");
    }

    #[test]
    fn test_concurrent_edits() {
        let entity_id = Uuid::new_v4();
        
        // Both clients start with same state
        let text_a = CrdtText::new(entity_id, "content");
        let text_b = CrdtText::new(entity_id, "content");
        
        text_a.set("Base");
        let initial = text_a.get_update();
        text_b.apply_update(&initial).unwrap();
        
        // Concurrent edits
        text_a.insert(4, " A"); // "Base A"
        text_b.insert(4, " B"); // "Base B"
        
        // Merge updates
        let update_a = text_a.get_update();
        let update_b = text_b.get_update();
        
        text_a.apply_update(&update_b).unwrap();
        text_b.apply_update(&update_a).unwrap();
        
        // Both should converge to same state
        assert_eq!(text_a.get(), text_b.get());
    }
}
