//! Offline-first functionality: Local database, sync, and CRDT

pub mod crdt;
pub mod db;
pub mod sync;

pub use crdt::{CrdtDocument, CrdtText, CrdtManager, AwarenessState};
pub use db::{LocalDatabase, DirtyRecord};
pub use sync::{SyncManager, SyncResult, ConflictResolution};
