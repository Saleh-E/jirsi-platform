//! Main Leptos App

use leptos::*;
use leptos_router::*;

use crate::pages::{HomePage, EntityListPage, EntityDetailPage, LoginPage};
use crate::components::shell::Shell;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/login" view=LoginPage/>
                <Route path="/" view=Shell>
                    <Route path="" view=HomePage/>
                    <Route path="/app/:app/entity/:entity" view=EntityListPage/>
                    <Route path="/app/:app/entity/:entity/:id" view=EntityDetailPage/>
                </Route>
            </Routes>
        </Router>
    }
}
