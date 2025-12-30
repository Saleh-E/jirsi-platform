//! Client-side Metadata Index
//!
//! Provides instant local search by caching entity metadata in IndexedDB.
//! This enables offline search and reduces server load.

use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A searchable entity entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexEntry {
    pub id: Uuid,
    pub entity_type: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub keywords: Vec<String>,
    pub url: String,
    pub icon: String,
    pub updated_at: i64,
}

/// The metadata index store
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MetadataIndex {
    entries: Vec<IndexEntry>,
    last_sync: Option<i64>,
}

impl MetadataIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            last_sync: None,
        }
    }
    
    /// Add or update an entry
    pub fn upsert(&mut self, entry: IndexEntry) {
        if let Some(pos) = self.entries.iter().position(|e| e.id == entry.id) {
            self.entries[pos] = entry;
        } else {
            self.entries.push(entry);
        }
    }
    
    /// Remove an entry by ID
    pub fn remove(&mut self, id: Uuid) {
        self.entries.retain(|e| e.id != id);
    }
    
    /// Search the index
    pub fn search(&self, query: &str, limit: usize) -> Vec<&IndexEntry> {
        if query.len() < 2 {
            return vec![];
        }
        
        let q = query.to_lowercase();
        let mut results: Vec<(&IndexEntry, u8)> = self
            .entries
            .iter()
            .filter_map(|entry| {
                let title_match = entry.title.to_lowercase().contains(&q);
                let subtitle_match = entry.subtitle.as_ref()
                    .map(|s| s.to_lowercase().contains(&q))
                    .unwrap_or(false);
                let keyword_match = entry.keywords.iter()
                    .any(|k| k.to_lowercase().contains(&q));
                
                if title_match || subtitle_match || keyword_match {
                    // Score: exact title match = 3, title contains = 2, keyword = 1
                    let score = if entry.title.to_lowercase() == q {
                        3
                    } else if title_match {
                        2
                    } else {
                        1
                    };
                    Some((entry, score))
                } else {
                    None
                }
            })
            .collect();
        
        // Sort by score descending
        results.sort_by(|a, b| b.1.cmp(&a.1));
        
        results.into_iter().take(limit).map(|(e, _)| e).collect()
    }
    
    /// Get count of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Get entries by type
    pub fn get_by_type(&self, entity_type: &str) -> Vec<&IndexEntry> {
        self.entries
            .iter()
            .filter(|e| e.entity_type == entity_type)
            .collect()
    }
    
    /// Bulk update from API response
    pub fn bulk_update(&mut self, entries: Vec<IndexEntry>) {
        for entry in entries {
            self.upsert(entry);
        }
        self.last_sync = Some(chrono::Utc::now().timestamp());
    }
    
    /// Get last sync timestamp
    pub fn last_synced(&self) -> Option<i64> {
        self.last_sync
    }
}

/// Create a reactive metadata index context
pub fn provide_metadata_index() {
    let index = create_rw_signal(MetadataIndex::new());
    provide_context(index);
}

/// Use the metadata index from context
pub fn use_metadata_index() -> Option<RwSignal<MetadataIndex>> {
    use_context::<RwSignal<MetadataIndex>>()
}

/// Helper to build index entries from entity data
pub fn entity_to_index_entry(
    id: Uuid,
    entity_type: &str,
    data: &serde_json::Value,
) -> IndexEntry {
    let (title, subtitle, keywords, icon, url) = match entity_type {
        "contact" => {
            let name = data.get("name")
                .or_else(|| data.get("first_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed")
                .to_string();
            let email = data.get("email")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let phone = data.get("phone")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let keywords = vec![name.clone(), email.clone().unwrap_or_default(), phone];
            (
                name,
                email,
                keywords,
                "👤".to_string(),
                format!("/app/crm/entity/contact/{}", id),
            )
        }
        "deal" => {
            let title = data.get("title")
                .or_else(|| data.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed Deal")
                .to_string();
            let value = data.get("value")
                .and_then(|v| v.as_f64())
                .map(|v| format!("${:.0}", v));
            (
                title.clone(),
                value,
                vec![title],
                "💰".to_string(),
                format!("/app/crm/entity/deal/{}", id),
            )
        }
        "property" => {
            let title = data.get("title")
                .or_else(|| data.get("address"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed Property")
                .to_string();
            let city = data.get("city")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let address = data.get("address")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            (
                title.clone(),
                city,
                vec![title, address],
                "🏠".to_string(),
                format!("/app/realestate/entity/property/{}", id),
            )
        }
        "company" => {
            let name = data.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed Company")
                .to_string();
            let industry = data.get("industry")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            (
                name.clone(),
                industry,
                vec![name],
                "🏢".to_string(),
                format!("/app/crm/entity/company/{}", id),
            )
        }
        _ => {
            let name = data.get("name")
                .or_else(|| data.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed")
                .to_string();
            (
                name.clone(),
                None,
                vec![name],
                "📄".to_string(),
                format!("/app/entity/{}/{}", entity_type, id),
            )
        }
    };
    
    IndexEntry {
        id,
        entity_type: entity_type.to_string(),
        title,
        subtitle,
        keywords,
        url,
        icon,
        updated_at: chrono::Utc::now().timestamp(),
    }
}
