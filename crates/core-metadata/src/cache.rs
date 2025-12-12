//! Metadata cache - in-memory caching for frequently accessed metadata

use core_models::{EntityType, FieldDef, ViewDef, AppDef};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

/// Cache key for tenant-scoped items
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct TenantKey {
    tenant_id: Uuid,
    key: String,
}

impl TenantKey {
    fn new(tenant_id: Uuid, key: &str) -> Self {
        Self {
            tenant_id,
            key: key.to_string(),
        }
    }
}

/// In-memory metadata cache
/// 
/// Caches EntityTypes, FieldDefs, and ViewDefs per tenant.
/// Cache is invalidated on metadata changes.
pub struct MetadataCache {
    entity_types: RwLock<HashMap<TenantKey, EntityType>>,
    entity_types_by_id: RwLock<HashMap<Uuid, EntityType>>,
    fields: RwLock<HashMap<Uuid, Vec<FieldDef>>>, // entity_type_id -> fields
    views: RwLock<HashMap<Uuid, Vec<ViewDef>>>,   // entity_type_id -> views
    apps: RwLock<HashMap<Uuid, Vec<AppDef>>>,     // tenant_id -> apps
}

impl MetadataCache {
    pub fn new() -> Self {
        Self {
            entity_types: RwLock::new(HashMap::new()),
            entity_types_by_id: RwLock::new(HashMap::new()),
            fields: RwLock::new(HashMap::new()),
            views: RwLock::new(HashMap::new()),
            apps: RwLock::new(HashMap::new()),
        }
    }

    // ============ EntityType ============

    pub fn get_entity_type(&self, tenant_id: Uuid, name: &str) -> Option<EntityType> {
        let cache = self.entity_types.read().unwrap();
        cache.get(&TenantKey::new(tenant_id, name)).cloned()
    }

    pub fn get_entity_type_by_id(&self, id: Uuid) -> Option<EntityType> {
        let cache = self.entity_types_by_id.read().unwrap();
        cache.get(&id).cloned()
    }

    pub fn set_entity_type(&self, entity: EntityType) {
        let key = TenantKey::new(entity.tenant_id, &entity.name);
        {
            let mut cache = self.entity_types.write().unwrap();
            cache.insert(key, entity.clone());
        }
        {
            let mut cache = self.entity_types_by_id.write().unwrap();
            cache.insert(entity.id, entity);
        }
    }

    pub fn invalidate_entity_type(&self, tenant_id: Uuid, name: &str) {
        let key = TenantKey::new(tenant_id, name);
        let mut cache = self.entity_types.write().unwrap();
        if let Some(entity) = cache.remove(&key) {
            let mut by_id = self.entity_types_by_id.write().unwrap();
            by_id.remove(&entity.id);
        }
    }

    // ============ FieldDef ============

    pub fn get_fields(&self, entity_type_id: Uuid) -> Option<Vec<FieldDef>> {
        let cache = self.fields.read().unwrap();
        cache.get(&entity_type_id).cloned()
    }

    pub fn set_fields(&self, entity_type_id: Uuid, fields: Vec<FieldDef>) {
        let mut cache = self.fields.write().unwrap();
        cache.insert(entity_type_id, fields);
    }

    pub fn invalidate_fields(&self, entity_type_id: Uuid) {
        let mut cache = self.fields.write().unwrap();
        cache.remove(&entity_type_id);
    }

    // ============ ViewDef ============

    pub fn get_views(&self, entity_type_id: Uuid) -> Option<Vec<ViewDef>> {
        let cache = self.views.read().unwrap();
        cache.get(&entity_type_id).cloned()
    }

    pub fn set_views(&self, entity_type_id: Uuid, views: Vec<ViewDef>) {
        let mut cache = self.views.write().unwrap();
        cache.insert(entity_type_id, views);
    }

    pub fn invalidate_views(&self, entity_type_id: Uuid) {
        let mut cache = self.views.write().unwrap();
        cache.remove(&entity_type_id);
    }

    // ============ Apps ============

    pub fn get_apps(&self, tenant_id: Uuid) -> Option<Vec<AppDef>> {
        let cache = self.apps.read().unwrap();
        cache.get(&tenant_id).cloned()
    }

    pub fn set_apps(&self, tenant_id: Uuid, apps: Vec<AppDef>) {
        let mut cache = self.apps.write().unwrap();
        cache.insert(tenant_id, apps);
    }

    pub fn invalidate_apps(&self, tenant_id: Uuid) {
        let mut cache = self.apps.write().unwrap();
        cache.remove(&tenant_id);
    }

    // ============ Global ============

    pub fn invalidate_tenant(&self, tenant_id: Uuid) {
        // Clear all caches for a tenant
        {
            let mut cache = self.entity_types.write().unwrap();
            cache.retain(|k, _| k.tenant_id != tenant_id);
        }
        self.invalidate_apps(tenant_id);
        // Note: fields/views are keyed by entity_type_id, would need reverse lookup
    }

    pub fn clear(&self) {
        self.entity_types.write().unwrap().clear();
        self.entity_types_by_id.write().unwrap().clear();
        self.fields.write().unwrap().clear();
        self.views.write().unwrap().clear();
        self.apps.write().unwrap().clear();
    }
}

impl Default for MetadataCache {
    fn default() -> Self {
        Self::new()
    }
}
