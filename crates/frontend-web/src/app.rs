//! Main Leptos App with fixed routing

use leptos::*;
use leptos_router::*;

use crate::pages::{HomePage, EntityListPage, EntityDetailPage, LoginPage, ProfilePage, SettingsPage, ReportsPage};
use crate::pages::dashboard::DashboardPage;
use crate::pages::public::{listings::PublicListingsPage, detail::PublicDetailPage};
use crate::components::shell::Shell;
use crate::layouts::public_layout::PublicLayout;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                // Public routes (no auth, tenant branding)
                <Route path="/listings" view=PublicLayout>
                    <Route path="" view=PublicListingsPage/>
                    <Route path=":id" view=PublicDetailPage/>
                </Route>
                
                // Auth routes
                <Route path="/login" view=LoginPage/>
                
                // App routes (authenticated) - explicit routes BEFORE dynamic
                <Route path="/" view=Shell>
                    // Default: Dashboard
                    <Route path="" view=DashboardPage/>
                    
                    // Static app routes - MUST come before dynamic
                    <Route path="app/profile" view=ProfilePage/>
                    <Route path="app/settings" view=SettingsPage/>
                    <Route path="app/dashboard" view=DashboardPage/>
                    <Route path="app/reports" view=ReportsPage/>
                    
                    // Dynamic entity routes - MUST come last
                    <Route path="app/:app/entity/:entity" view=EntityListPage/>
                    <Route path="app/:app/entity/:entity/:id" view=EntityDetailPage/>
                </Route>
            </Routes>
        </Router>
    }
}
