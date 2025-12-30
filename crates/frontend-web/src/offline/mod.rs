//! Offline-first functionality: Local database, sync, CRDT, and metadata index

pub mod crdt;
pub mod db;
pub mod metadata_index;
pub mod sync;

pub use crdt::{CrdtDocument, CrdtText, CrdtManager, AwarenessState};
pub use db::{LocalDatabase, DirtyRecord};
pub use metadata_index::{MetadataIndex, IndexEntry, provide_metadata_index, use_metadata_index, entity_to_index_entry};
pub use sync::{SyncManager, SyncResult, ConflictResolution};
