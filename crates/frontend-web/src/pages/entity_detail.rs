//! Entity detail page

use leptos::*;
use leptos_router::*;

#[component]
pub fn EntityDetailPage() -> impl IntoView {
    let params = use_params_map();
    
    let app = move || params.with(|p| p.get("app").cloned().unwrap_or_default());
    let entity = move || params.with(|p| p.get("entity").cloned().unwrap_or_default());
    let id = move || params.with(|p| p.get("id").cloned().unwrap_or_default());

    view! {
        <div class="entity-detail-page">
            <header class="page-header">
                <a href=move || format!("/app/{}/entity/{}", app(), entity()) class="back-link">
                    "← Back"
                </a>
                <h1>{move || format!("{} Details", entity())}</h1>
            </header>
            <div class="page-content">
                <div class="detail-tabs">
                    <button class="tab active">"Details"</button>
                    <button class="tab">"Activity"</button>
                    <button class="tab">"Tasks"</button>
                    <button class="tab">"Associations"</button>
                </div>
                <div class="detail-content">
                    <p>"Loading record " {id} "..."</p>
                </div>
            </div>
            <aside class="detail-sidebar">
                <h3>"Activity Timeline"</h3>
                // TODO: Render interactions
            </aside>
        </div>
    }
}
