//! Main Leptos App with fixed routing

use leptos::*;
use leptos_router::*;

use crate::pages::{
    AutomationPage, EntityDetailPage, EntityListPage, InboxPage, LoginPage,
    ProfilePage, RegisterPage, ReportsPage, SettingsPage, WorkflowEditorPage, WorkflowListPage,
};
use crate::pages::component_playground::ComponentPlayground;
use crate::pages::dashboard::Dashboard;
use crate::pages::public::{listings::PublicListingsPage, detail::PublicDetailPage};
use crate::layouts::HolographicShell;
use crate::layouts::public_layout::PublicLayout;
use crate::core::shortcuts::ShortcutInitializer;

// Chameleon Engine: Persona Portals
use crate::pages::landlord_portal::LandlordPortal;
use crate::pages::tenant_portal::TenantPortal;
use crate::pages::agent_command_center::AgentCommandCenter;
use crate::pages::broker_dashboard::BrokerDashboard;
use crate::pages::landlord_onboarding::LandlordOnboarding;

use crate::context::theme::provide_theme_context;

#[component]
pub fn App() -> impl IntoView {
    // Initialize Theme System
    provide_theme_context();

    view! {
        <Router>
            // Initialize shortcuts inside Router context (Ghost Keys)
            <ShortcutInitializer />
            
            <Routes>
                // Public routes (no auth, tenant branding)
                <Route path="/listings" view=PublicLayout>
                    <Route path="" view=PublicListingsPage/>
                    <Route path=":id" view=PublicDetailPage/>
                </Route>
                
                // Auth routes (public)
                <Route path="/login" view=LoginPage/>
                <Route path="/register" view=RegisterPage/>
                
                // App routes (authenticated) - using HOLOGRAPHIC SHELL
                <Route path="/" view=HolographicShell>
                    // Default: Dashboard
                    <Route path="" view=Dashboard/>
                    
                    // Static app routes - MUST come before dynamic
                    <Route path="app/profile" view=ProfilePage/>
                    <Route path="app/settings" view=SettingsPage/>
                    <Route path="app/settings/workflows" view=WorkflowListPage/>
                    <Route path="app/settings/workflows/:id" view=WorkflowEditorPage/>
                    <Route path="app/dashboard" view=Dashboard/>
                    <Route path="app/reports" view=ReportsPage/>
                    <Route path="app/inbox" view=InboxPage/>
                    <Route path="app/automation" view=AutomationPage/>
                    
                    // === CHAMELEON ENGINE: Persona Portals ===
                    <Route path="app/portal/landlord" view=LandlordPortal/>
                    <Route path="app/portal/tenant" view=TenantPortal/>
                    <Route path="app/agent" view=AgentCommandCenter/>
                    <Route path="app/broker" view=BrokerDashboard/>
                    <Route path="app/onboarding/landlord" view=LandlordOnboarding/>
                    
                    // Component Playground (development/demo)
                    <Route path="app/playground" view=ComponentPlayground/>
                    
                    // Dynamic entity routes - MUST come last
                    <Route path="app/:app/entity/:entity" view=EntityListPage/>
                    <Route path="app/:app/entity/:entity/:id" view=EntityDetailPage/>
                </Route>
            </Routes>
        </Router>
    }
}
