//! Permission Context - Antigravity RBAC for Frontend
//!
//! Provides user permissions to components for visibility and access control.
//! Mirrors the backend permission.rs but runs in WASM.

use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User role from auth context - synced with backend
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Manager,
    Member,
    Agent,
    Broker,
    Landlord,
    Tenant,
    Vendor,
    Viewer,
}

impl UserRole {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => Self::Admin,
            "manager" => Self::Manager,
            "member" => Self::Member,
            "agent" => Self::Agent,
            "broker" => Self::Broker,
            "landlord" => Self::Landlord,
            "tenant" => Self::Tenant,
            "vendor" => Self::Vendor,
            _ => Self::Viewer,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Manager => "manager",
            Self::Member => "member",
            Self::Agent => "agent",
            Self::Broker => "broker",
            Self::Landlord => "landlord",
            Self::Tenant => "tenant",
            Self::Vendor => "vendor",
            Self::Viewer => "viewer",
        }
    }
    
    /// UI sidebar sections visible to this role (Chameleon Engine)
    pub fn sidebar_sections(&self) -> Vec<&'static str> {
        match self {
            Self::Admin => vec!["dashboard", "crm", "real_estate", "reports", "settings", "marketplace", "automation"],
            Self::Manager => vec!["dashboard", "crm", "real_estate", "reports", "automation"],
            Self::Agent => vec!["dashboard", "clients", "properties", "deals", "calendar", "dialer"],
            Self::Broker => vec!["dashboard", "agents", "properties", "deals", "commissions", "reports"],
            Self::Landlord => vec!["dashboard", "my_properties", "contracts", "financials", "maintenance"],
            Self::Tenant => vec!["my_home", "payments", "requests", "documents"],
            Self::Vendor => vec!["work_orders", "schedule", "invoices"],
            Self::Member => vec!["dashboard", "contacts", "tasks"],
            Self::Viewer => vec!["dashboard"],
        }
    }
    
    /// Entities this role can access
    pub fn accessible_entities(&self) -> Vec<&'static str> {
        match self {
            Self::Admin | Self::Manager => vec!["*"],
            Self::Agent => vec!["contact", "property", "deal", "contract", "viewing", "task", "requirement"],
            Self::Broker => vec!["contact", "property", "deal", "contract", "agent", "report", "commission"],
            Self::Landlord => vec!["property", "contract", "tenant", "maintenance_request", "payment"],
            Self::Tenant => vec!["contract", "payment", "maintenance_request", "viewing"],
            Self::Vendor => vec!["task", "property", "work_order"],
            Self::Member => vec!["contact", "deal", "task"],
            Self::Viewer => vec![],
        }
    }
    
    /// Check if role has dialer access
    pub fn has_dialer(&self) -> bool {
        matches!(self, Self::Admin | Self::Agent | Self::Broker)
    }
    
    /// Check if role can manage payments
    pub fn can_manage_payments(&self) -> bool {
        matches!(self, Self::Admin | Self::Manager | Self::Landlord | Self::Broker)
    }
    
    /// Get display name for role
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Admin => "Administrator",
            Self::Manager => "Manager",
            Self::Member => "Team Member",
            Self::Agent => "Agent",
            Self::Broker => "Broker",
            Self::Landlord => "Landlord",
            Self::Tenant => "Tenant",
            Self::Vendor => "Vendor",
            Self::Viewer => "Viewer",
        }
    }
}

/// Permission context available throughout the app
#[derive(Clone, Debug)]
pub struct PermissionContext {
    pub user_role: RwSignal<UserRole>,
    pub permissions: RwSignal<HashMap<String, bool>>,
}

impl PermissionContext {
    pub fn new() -> Self {
        // Get role from localStorage
        let stored_role = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            .and_then(|s| s.get_item("user_role").ok())
            .flatten()
            .unwrap_or_else(|| "viewer".to_string());
        
        let role = UserRole::from_str(&stored_role);
        let permissions = compute_permissions(&role);
        
        Self {
            user_role: create_rw_signal(role),
            permissions: create_rw_signal(permissions),
        }
    }
    
    /// Check if user has permission for resource:action
    pub fn can(&self, resource: &str, action: &str) -> bool {
        let key = format!("{}:{}", resource, action);
        self.permissions.get().get(&key).copied().unwrap_or(false)
    }
    
    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        matches!(self.user_role.get(), UserRole::Admin)
    }
    
    /// Check if user is admin or manager
    pub fn is_admin_or_manager(&self) -> bool {
        matches!(self.user_role.get(), UserRole::Admin | UserRole::Manager)
    }
    
    /// Check if nav item should be visible
    pub fn can_view_nav(&self, nav_id: &str) -> bool {
        match nav_id {
            // Admin-only items
            "settings" | "users" | "admin_panel" => self.is_admin(),
            
            // Manager+ items
            "workflows" | "reports" | "analytics" => self.is_admin_or_manager(),
            
            // Everyone can see CRM, Real Estate, Dashboard
            _ => true,
        }
    }
    
    /// Update role (e.g., after login)
    pub fn set_role(&self, role: UserRole) {
        // Store in localStorage
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.set_item("user_role", role.as_str());
        }
        
        self.user_role.set(role.clone());
        self.permissions.set(compute_permissions(&role));
    }
}

/// Compute permissions based on role
fn compute_permissions(role: &UserRole) -> HashMap<String, bool> {
    let mut perms = HashMap::new();
    
    let resources = ["contact", "company", "deal", "property", "workflow", "report", "settings"];
    let actions = ["read", "write", "delete"];
    
    for resource in &resources {
        for action in &actions {
            let key = format!("{}:{}", resource, action);
            let allowed = match role {
                UserRole::Admin => true,
                UserRole::Manager => true,
                UserRole::Broker => match *action {
                    "read" | "write" | "delete" => true,
                    _ => false,
                },
                UserRole::Member | UserRole::Agent => match *action {
                    "read" | "write" => true,
                    "delete" => false,
                    _ => false,
                },
                UserRole::Landlord => match (*resource, *action) {
                    ("property" | "deal" | "contact", "read" | "write") => true,
                    _ => false,
                },
                UserRole::Tenant => match (*resource, *action) {
                    ("property", "read") => true,
                    _ => false,
                },
                UserRole::Vendor => match (*resource, *action) {
                    ("property", "read") => true,
                    ("contact", "read" | "write") => true,
                    _ => false,
                },
                UserRole::Viewer => match *action {
                    "read" => true,
                    _ => false,
                },
            };
            perms.insert(key, allowed);
        }
    }
    
    // Special permissions
    perms.insert("admin_panel:access".to_string(), matches!(role, UserRole::Admin));
    perms.insert("workflows:execute".to_string(), matches!(role, UserRole::Admin | UserRole::Manager | UserRole::Broker));
    perms.insert("reports:export".to_string(), matches!(role, UserRole::Admin | UserRole::Manager | UserRole::Broker));
    perms.insert("dialer:access".to_string(), matches!(role, UserRole::Admin | UserRole::Agent | UserRole::Broker));
    perms.insert("payments:manage".to_string(), matches!(role, UserRole::Admin | UserRole::Manager | UserRole::Landlord | UserRole::Broker));
    
    perms
}

/// Provide permission context
pub fn provide_permission_context() {
    provide_context(PermissionContext::new());
}

/// Use permission context
pub fn use_permissions() -> PermissionContext {
    use_context::<PermissionContext>()
        .expect("PermissionContext not provided. Call provide_permission_context() first.")
}
