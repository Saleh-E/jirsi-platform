//! Metadata service - high-level API for metadata access

use core_models::{EntityType, FieldDef, ViewDef, AppDef};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::cache::MetadataCache;
use crate::repository::MetadataRepository;
use crate::MetadataError;

/// High-level service for accessing metadata with caching
#[derive(Clone)]
pub struct MetadataService {
    repo: MetadataRepository,
    cache: Arc<MetadataCache>,
}

impl MetadataService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            repo: MetadataRepository::new(pool),
            cache: Arc::new(MetadataCache::new()),
        }
    }

    pub fn with_cache(pool: PgPool, cache: Arc<MetadataCache>) -> Self {
        Self {
            repo: MetadataRepository::new(pool),
            cache,
        }
    }

    /// Get an EntityType by name, using cache
    pub async fn get_entity_type(
        &self,
        tenant_id: Uuid,
        name: &str,
    ) -> Result<EntityType, MetadataError> {
        // Check cache first
        if let Some(entity) = self.cache.get_entity_type(tenant_id, name) {
            return Ok(entity);
        }

        // Fetch from database
        let entity = self.repo.get_entity_type_by_name(tenant_id, name).await?;
        self.cache.set_entity_type(entity.clone());
        Ok(entity)
    }

    /// Get an EntityType by ID, using cache
    pub async fn get_entity_type_by_id(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<EntityType, MetadataError> {
        // Check cache first
        if let Some(entity) = self.cache.get_entity_type_by_id(id) {
            return Ok(entity);
        }

        // Fetch from database
        let entity = self.repo.get_entity_type_by_id(tenant_id, id).await?;
        self.cache.set_entity_type(entity.clone());
        Ok(entity)
    }

    /// List all EntityTypes for a tenant, optionally filtered by app
    pub async fn list_entity_types(
        &self,
        tenant_id: Uuid,
        app_id: Option<&str>,
    ) -> Result<Vec<EntityType>, MetadataError> {
        // For list queries, we don't cache the full result, but cache individual items
        let entities = self.repo.list_entity_types(tenant_id, app_id).await?;
        for entity in &entities {
            self.cache.set_entity_type(entity.clone());
        }
        Ok(entities)
    }

    /// Get all fields for an EntityType, using cache
    pub async fn get_fields(
        &self,
        tenant_id: Uuid,
        entity_type_id: Uuid,
    ) -> Result<Vec<FieldDef>, MetadataError> {
        // Check cache
        if let Some(fields) = self.cache.get_fields(entity_type_id) {
            return Ok(fields);
        }

        // Fetch from database
        let fields = self.repo.get_fields_for_entity(tenant_id, entity_type_id).await?;
        self.cache.set_fields(entity_type_id, fields.clone());
        Ok(fields)
    }

    /// Get fields for an EntityType by name
    pub async fn get_fields_by_entity_name(
        &self,
        tenant_id: Uuid,
        entity_name: &str,
    ) -> Result<Vec<FieldDef>, MetadataError> {
        let entity = self.get_entity_type(tenant_id, entity_name).await?;
        self.get_fields(tenant_id, entity.id).await
    }

    /// Get all views for an EntityType, using cache
    pub async fn get_views(
        &self,
        tenant_id: Uuid,
        entity_type_id: Uuid,
    ) -> Result<Vec<ViewDef>, MetadataError> {
        // Check cache
        if let Some(views) = self.cache.get_views(entity_type_id) {
            return Ok(views);
        }

        // Fetch from database
        let views = self.repo.get_views_for_entity(tenant_id, entity_type_id).await?;
        self.cache.set_views(entity_type_id, views.clone());
        Ok(views)
    }

    /// Get the default view for an EntityType
    pub async fn get_default_view(
        &self,
        tenant_id: Uuid,
        entity_type_id: Uuid,
    ) -> Result<Option<ViewDef>, MetadataError> {
        // Check cache for all views and find default
        if let Some(views) = self.cache.get_views(entity_type_id) {
            return Ok(views.into_iter().find(|v| v.is_default));
        }

        // Fetch from database
        self.repo.get_default_view(tenant_id, entity_type_id).await
    }

    /// List all apps for a tenant
    pub async fn list_apps(&self, tenant_id: Uuid) -> Result<Vec<AppDef>, MetadataError> {
        // Check cache
        if let Some(apps) = self.cache.get_apps(tenant_id) {
            return Ok(apps);
        }

        // Fetch from database
        let apps = self.repo.list_apps(tenant_id).await?;
        self.cache.set_apps(tenant_id, apps.clone());
        Ok(apps)
    }

    /// Get complete metadata for an entity: EntityType + Fields + Views
    pub async fn get_entity_metadata(
        &self,
        tenant_id: Uuid,
        entity_name: &str,
    ) -> Result<EntityMetadata, MetadataError> {
        let entity = self.get_entity_type(tenant_id, entity_name).await?;
        let fields = self.get_fields(tenant_id, entity.id).await?;
        let views = self.get_views(tenant_id, entity.id).await?;

        Ok(EntityMetadata {
            entity_type: entity,
            fields,
            views,
        })
    }

    // ============ Cache invalidation ============

    pub fn invalidate_entity(&self, tenant_id: Uuid, entity_name: &str) {
        self.cache.invalidate_entity_type(tenant_id, entity_name);
    }

    pub fn invalidate_fields(&self, entity_type_id: Uuid) {
        self.cache.invalidate_fields(entity_type_id);
    }

    pub fn invalidate_views(&self, entity_type_id: Uuid) {
        self.cache.invalidate_views(entity_type_id);
    }

    pub fn invalidate_tenant(&self, tenant_id: Uuid) {
        self.cache.invalidate_tenant(tenant_id);
    }
}

/// Complete metadata for an entity
#[derive(Debug, Clone)]
pub struct EntityMetadata {
    pub entity_type: EntityType,
    pub fields: Vec<FieldDef>,
    pub views: Vec<ViewDef>,
}
