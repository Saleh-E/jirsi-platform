//! Main Leptos App

use leptos::*;
use leptos_router::*;

use crate::pages::{HomePage, EntityListPage, EntityDetailPage, LoginPage};
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
                
                // App routes (authenticated)
                <Route path="/" view=Shell>
                    <Route path="" view=HomePage/>
                    <Route path="/app/:app/entity/:entity" view=EntityListPage/>
                    <Route path="/app/:app/entity/:entity/:id" view=EntityDetailPage/>
                </Route>
            </Routes>
        </Router>
    }
}
