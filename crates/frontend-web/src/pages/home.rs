//! Home page / Dashboard with live data

use leptos::*;
use crate::api::{fetch_contacts, fetch_companies, fetch_deals, fetch_tasks, fetch_properties};

#[component]
pub fn HomePage() -> impl IntoView {
    // Fetch counts from API
    let contacts_count = create_resource(|| (), |_| async move {
        fetch_contacts().await.map(|r| r.total).unwrap_or(0)
    });

    let companies_count = create_resource(|| (), |_| async move {
        fetch_companies().await.map(|r| r.total).unwrap_or(0)
    });

    let deals_count = create_resource(|| (), |_| async move {
        fetch_deals().await.map(|r| r.total).unwrap_or(0)
    });

    let tasks_count = create_resource(|| (), |_| async move {
        fetch_tasks().await.map(|r| r.total).unwrap_or(0)
    });

    let properties_count = create_resource(|| (), |_| async move {
        fetch_properties().await.map(|r| r.total).unwrap_or(0)
    });

    view! {
        <div class="home-page">
            <h1>"Dashboard"</h1>
            <div class="dashboard-grid">
                <div class="dashboard-card">
                    <h3>"Contacts"</h3>
                    <div class="metric">
                        <Suspense fallback=|| "...">
                            {move || contacts_count.get().map(|c| c.to_string())}
                        </Suspense>
                    </div>
                </div>
                <div class="dashboard-card">
                    <h3>"Companies"</h3>
                    <div class="metric">
                        <Suspense fallback=|| "...">
                            {move || companies_count.get().map(|c| c.to_string())}
                        </Suspense>
                    </div>
                </div>
                <div class="dashboard-card">
                    <h3>"Deals"</h3>
                    <div class="metric">
                        <Suspense fallback=|| "...">
                            {move || deals_count.get().map(|c| c.to_string())}
                        </Suspense>
                    </div>
                </div>
                <div class="dashboard-card">
                    <h3>"Tasks"</h3>
                    <div class="metric">
                        <Suspense fallback=|| "...">
                            {move || tasks_count.get().map(|c| c.to_string())}
                        </Suspense>
                    </div>
                </div>
                <div class="dashboard-card">
                    <h3>"Properties"</h3>
                    <div class="metric">
                        <Suspense fallback=|| "...">
                            {move || properties_count.get().map(|c| c.to_string())}
                        </Suspense>
                    </div>
                </div>
            </div>
        </div>
    }
}
