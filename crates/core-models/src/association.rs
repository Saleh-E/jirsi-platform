//! Association model - Cross-entity/cross-app linking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Cardinality of an association
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

/// Association definition - defines how two EntityTypes can be linked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationDef {
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// Source EntityType name
    pub source_entity: String,
    /// Target EntityType name
    pub target_entity: String,
    /// Association name (e.g., "company_contacts")
    pub name: String,
    /// Display label from source perspective
    pub label_source: String,
    /// Display label from target perspective
    pub label_target: String,
    /// Relationship cardinality
    pub cardinality: Cardinality,
    /// Role name from source (e.g., "employer")
    pub source_role: Option<String>,
    /// Role name from target (e.g., "employee")
    pub target_role: Option<String>,
    /// Can one link be marked as primary?
    pub allow_primary: bool,
    /// Cascade delete?
    pub cascade_delete: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Runtime association instance - actual link between two records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Association {
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// The AssociationDef this instance belongs to
    pub association_def_id: Uuid,
    /// Source record ID
    pub source_id: Uuid,
    /// Target record ID
    pub target_id: Uuid,
    /// Role of target in this association (e.g., "buyer", "seller")
    pub role: Option<String>,
    /// Is this the primary association?
    pub is_primary: bool,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Association {
    pub fn new(
        tenant_id: Uuid,
        association_def_id: Uuid,
        source_id: Uuid,
        target_id: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            association_def_id,
            source_id,
            target_id,
            role: None,
            is_primary: false,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
    }

    pub fn as_primary(mut self) -> Self {
        self.is_primary = true;
        self
    }
}
