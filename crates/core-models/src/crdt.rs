//! CRDT Module - Production-Grade Yjs/Yrs Implementation
//!
//! Provides conflict-free replicated data types (CRDTs) using the Yrs library,
//! which implements the Yjs protocol for real-time collaboration.
//!
//! Features:
//! - Document-level state management with yrs::Doc
//! - Binary update encoding for efficient network sync
//! - State vector for incremental delta updates
//! - Server-side document persistence
//! - Multi-field document support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use yrs::{Doc, GetString, ReadTxn, StateVector, Text, Transact, Update};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;

/// Error type for CRDT operations
#[derive(Debug, thiserror::Error)]
pub enum CrdtError {
    #[error("Failed to decode update: {0}")]
    DecodeError(String),
    
    #[error("Failed to encode state: {0}")]
    EncodeError(String),
    
    #[error("Invalid state vector: {0}")]
    InvalidStateVector(String),
    
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    
    #[error("Lock error")]
    LockError,
}

/// Serializable CRDT document state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtDocumentState {
    /// Document identifier (entity_id:field_name)
    pub document_id: String,
    /// Full document state as bytes (Yjs encoded)
    pub state: Vec<u8>,
    /// State vector for delta sync
    pub state_vector: Vec<u8>,
    /// Version counter for optimistic locking
    pub version: u64,
    /// Timestamp of last update
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl CrdtDocumentState {
    /// Create new empty document state
    pub fn new(document_id: &str) -> Self {
        let doc = Doc::new();
        let txn = doc.transact();
        
        Self {
            document_id: document_id.to_string(),
            state: txn.encode_state_as_update_v1(&StateVector::default()),
            state_vector: txn.state_vector().encode_v1(),
            version: 1,
            updated_at: chrono::Utc::now(),
        }
    }
    
    /// Create from existing Yrs Doc
    pub fn from_doc(document_id: &str, doc: &Doc) -> Self {
        let txn = doc.transact();
        Self {
            document_id: document_id.to_string(),
            state: txn.encode_state_as_update_v1(&StateVector::default()),
            state_vector: txn.state_vector().encode_v1(),
            version: 1,
            updated_at: chrono::Utc::now(),
        }
    }
    
    /// Load into a Yrs Doc
    pub fn load(&self) -> Result<Doc, CrdtError> {
        let doc = Doc::new();
        let update = Update::decode_v1(&self.state)
            .map_err(|e| CrdtError::DecodeError(format!("{:?}", e)))?;
        
        let mut txn = doc.transact_mut();
        txn.apply_update(update);
        drop(txn);
        
        Ok(doc)
    }
}

/// In-memory CRDT document manager
/// Manages active documents and handles sync operations
pub struct CrdtDocumentManager {
    /// Active documents in memory
    documents: Mutex<HashMap<String, Arc<Mutex<Doc>>>>,
}

impl CrdtDocumentManager {
    /// Create new document manager
    pub fn new() -> Self {
        Self {
            documents: Mutex::new(HashMap::new()),
        }
    }
    
    /// Get or create a document by ID
    pub fn get_or_create(&self, document_id: &str) -> Result<Arc<Mutex<Doc>>, CrdtError> {
        let mut docs = self.documents.lock().map_err(|_| CrdtError::LockError)?;
        
        if let Some(doc) = docs.get(document_id) {
            Ok(Arc::clone(doc))
        } else {
            let doc = Arc::new(Mutex::new(Doc::new()));
            docs.insert(document_id.to_string(), Arc::clone(&doc));
            Ok(doc)
        }
    }
    
    /// Load a document from persisted state
    pub fn load_from_state(&self, state: &CrdtDocumentState) -> Result<Arc<Mutex<Doc>>, CrdtError> {
        let doc = state.load()?;
        let doc = Arc::new(Mutex::new(doc));
        
        let mut docs = self.documents.lock().map_err(|_| CrdtError::LockError)?;
        docs.insert(state.document_id.clone(), Arc::clone(&doc));
        
        Ok(doc)
    }
    
    /// Apply a binary update to a document
    pub fn apply_update(&self, document_id: &str, update_bytes: &[u8]) -> Result<(), CrdtError> {
        let doc = self.get_or_create(document_id)?;
        let doc = doc.lock().map_err(|_| CrdtError::LockError)?;
        
        let update = Update::decode_v1(update_bytes)
            .map_err(|e| CrdtError::DecodeError(format!("{:?}", e)))?;
        
        let mut txn = doc.transact_mut();
        txn.apply_update(update);
        
        Ok(())
    }
    
    /// Get delta update since a specific state vector
    pub fn get_update_since(&self, document_id: &str, state_vector_bytes: &[u8]) -> Result<Vec<u8>, CrdtError> {
        let doc = self.get_or_create(document_id)?;
        let doc = doc.lock().map_err(|_| CrdtError::LockError)?;
        
        let state_vector = StateVector::decode_v1(state_vector_bytes)
            .map_err(|e| CrdtError::InvalidStateVector(format!("{:?}", e)))?;
        
        let txn = doc.transact();
        Ok(txn.encode_state_as_update_v1(&state_vector))
    }
    
    /// Get full document state
    pub fn get_state(&self, document_id: &str) -> Result<CrdtDocumentState, CrdtError> {
        let doc = self.get_or_create(document_id)?;
        let doc = doc.lock().map_err(|_| CrdtError::LockError)?;
        
        let txn = doc.transact();
        
        Ok(CrdtDocumentState {
            document_id: document_id.to_string(),
            state: txn.encode_state_as_update_v1(&StateVector::default()),
            state_vector: txn.state_vector().encode_v1(),
            version: 1,
            updated_at: chrono::Utc::now(),
        })
    }
    
    /// Get text content from a field in a document
    pub fn get_text(&self, document_id: &str, field: &str) -> Result<String, CrdtError> {
        let doc = self.get_or_create(document_id)?;
        let doc = doc.lock().map_err(|_| CrdtError::LockError)?;
        
        let txn = doc.transact();
        if let Some(text) = txn.get_text(field) {
            Ok(text.get_string(&txn))
        } else {
            Ok(String::new())
        }
    }
    
    /// Set text content in a field (creates update)
    pub fn set_text(&self, document_id: &str, field: &str, content: &str) -> Result<Vec<u8>, CrdtError> {
        let doc = self.get_or_create(document_id)?;
        let doc = doc.lock().map_err(|_| CrdtError::LockError)?;
        
        let text = doc.get_or_insert_text(field);
        
        // Get state before update
        let txn = doc.transact();
        let state_before = txn.state_vector();
        drop(txn);
        
        // Apply update
        let mut txn = doc.transact_mut();
        let len = text.len(&txn);
        if len > 0 {
            text.remove_range(&mut txn, 0, len);
        }
        text.insert(&mut txn, 0, content);
        drop(txn);
        
        // Get delta update
        let txn = doc.transact();
        Ok(txn.encode_state_as_update_v1(&state_before))
    }
    
    /// Remove a document from memory (cleanup)
    pub fn remove(&self, document_id: &str) -> Result<(), CrdtError> {
        let mut docs = self.documents.lock().map_err(|_| CrdtError::LockError)?;
        docs.remove(document_id);
        Ok(())
    }
    
    /// Get all active document IDs
    pub fn active_documents(&self) -> Result<Vec<String>, CrdtError> {
        let docs = self.documents.lock().map_err(|_| CrdtError::LockError)?;
        Ok(docs.keys().cloned().collect())
    }
}

impl Default for CrdtDocumentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Simplified CRDT Text wrapper (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtText {
    /// Current text content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    
    /// State vector (Yjs encoded)
    pub state_vector: Vec<u8>,
    
    /// Document state (Yjs encoded)
    pub doc_state: Vec<u8>,
}

impl CrdtText {
    /// Create new empty CRDT text
    pub fn new() -> Self {
        let doc = Doc::new();
        let _ = doc.get_or_insert_text("content");
        let txn = doc.transact();
        
        Self {
            text: Some(String::new()),
            state_vector: txn.state_vector().encode_v1(),
            doc_state: txn.encode_state_as_update_v1(&StateVector::default()),
        }
    }
    
    /// Create from existing text
    pub fn from_text(text: &str) -> Result<Self, CrdtError> {
        let doc = Doc::new();
        let text_ref = doc.get_or_insert_text("content");
        
        let mut txn = doc.transact_mut();
        text_ref.insert(&mut txn, 0, text);
        drop(txn);
        
        let txn = doc.transact();
        Ok(Self {
            text: Some(text.to_string()),
            state_vector: txn.state_vector().encode_v1(),
            doc_state: txn.encode_state_as_update_v1(&StateVector::default()),
        })
    }
    
    /// Apply an update from another client
    pub fn apply_update(&mut self, update: &[u8]) -> Result<(), CrdtError> {
        // Load current state into doc
        let doc = Doc::new();
        if !self.doc_state.is_empty() {
            if let Ok(existing_update) = Update::decode_v1(&self.doc_state) {
                let mut txn = doc.transact_mut();
                txn.apply_update(existing_update);
            }
        }
        
        // Apply new update
        let new_update = Update::decode_v1(update)
            .map_err(|e| CrdtError::DecodeError(format!("{:?}", e)))?;
        
        let mut txn = doc.transact_mut();
        txn.apply_update(new_update);
        drop(txn);
        
        // Extract new state
        let txn = doc.transact();
        self.doc_state = txn.encode_state_as_update_v1(&StateVector::default());
        self.state_vector = txn.state_vector().encode_v1();
        
        // Update text
        if let Some(text_ref) = txn.get_text("content") {
            self.text = Some(text_ref.get_string(&txn));
        }
        
        Ok(())
    }
    
    /// Get update since a specific state vector
    pub fn get_update_since(&self, state_vector: &[u8]) -> Result<Vec<u8>, CrdtError> {
        // Load state into doc
        let doc = Doc::new();
        if !self.doc_state.is_empty() {
            if let Ok(update) = Update::decode_v1(&self.doc_state) {
                let mut txn = doc.transact_mut();
                txn.apply_update(update);
            }
        }
        
        let sv = StateVector::decode_v1(state_vector)
            .map_err(|e| CrdtError::InvalidStateVector(format!("{:?}", e)))?;
        
        let txn = doc.transact();
        Ok(txn.encode_state_as_update_v1(&sv))
    }
    
    /// Get current text content
    pub fn get_text(&self) -> Option<&str> {
        self.text.as_deref()
    }
    
    /// Merge with another CRDT text
    pub fn merge(&mut self, other: &CrdtText) -> Result<(), CrdtError> {
        // Apply other's state as an update
        self.apply_update(&other.doc_state)
    }
}

impl Default for CrdtText {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_crdt_text_new() {
        let crdt = CrdtText::new();
        assert_eq!(crdt.get_text(), Some(""));
    }
    
    #[test]
    fn test_crdt_text_from_text() {
        let crdt = CrdtText::from_text("Hello, World!").unwrap();
        assert_eq!(crdt.get_text(), Some("Hello, World!"));
    }
    
    #[test]
    fn test_document_manager_basic() {
        let manager = CrdtDocumentManager::new();
        
        // Set text
        let update = manager.set_text("test:doc", "content", "Hello").unwrap();
        assert!(!update.is_empty());
        
        // Get text
        let text = manager.get_text("test:doc", "content").unwrap();
        assert_eq!(text, "Hello");
    }
    
    #[test]
    fn test_document_manager_sync() {
        let manager1 = CrdtDocumentManager::new();
        let manager2 = CrdtDocumentManager::new();
        
        // Create document on manager1
        let update1 = manager1.set_text("sync:doc", "field", "Hello from 1").unwrap();
        
        // Apply to manager2
        manager2.apply_update("sync:doc", &update1).unwrap();
        
        // Verify sync
        let text2 = manager2.get_text("sync:doc", "field").unwrap();
        assert_eq!(text2, "Hello from 1");
    }
    
    #[test]
    fn test_delta_sync() {
        let manager = CrdtDocumentManager::new();
        
        // Initial content
        let _ = manager.set_text("delta:doc", "content", "Hello").unwrap();
        
        // Get state vector
        let state = manager.get_state("delta:doc").unwrap();
        
        // Make another change
        let _ = manager.set_text("delta:doc", "content", "Hello World").unwrap();
        
        // Get delta since first state
        let delta = manager.get_update_since("delta:doc", &state.state_vector).unwrap();
        
        // Delta should be non-empty (contains the change)
        assert!(!delta.is_empty());
    }
}
