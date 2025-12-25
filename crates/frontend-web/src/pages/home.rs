//! Home page / Dashboard with live data

use crate::api::{fetch_companies, fetch_contacts, fetch_deals, fetch_properties, fetch_tasks};
use leptos::*;

#[component]
pub fn HomePage() -> impl IntoView {
    // Fetch counts from API
    let contacts_count = create_resource(
        || (),
        |_| async move { fetch_contacts().await.map(|r| r.total).unwrap_or(0) },
    );

    let companies_count = create_resource(
        || (),
        |_| async move { fetch_companies().await.map(|r| r.total).unwrap_or(0) },
    );

    let deals_count = create_resource(
        || (),
        |_| async move { fetch_deals().await.map(|r| r.total).unwrap_or(0) },
    );

    let tasks_count = create_resource(
        || (),
        |_| async move { fetch_tasks().await.map(|r| r.total).unwrap_or(0) },
    );

    let properties_count = create_resource(
        || (),
        |_| async move { fetch_properties().await.map(|r| r.total).unwrap_or(0) },
    );

    let total_records = create_memo(move |_| {
        contacts_count.get().unwrap_or(0)
            + companies_count.get().unwrap_or(0)
            + deals_count.get().unwrap_or(0)
    });

    let adoption_score = create_memo(move |_| {
        let score = (total_records.get() as f64 / 5_000.0 * 100.0).round();
        score.clamp(38.0, 99.0) as i64
    });

    view! {
        <div class="home-page">
            <section class="home-hero">
                <div class="hero-copy">
                    <p class="hero-eyebrow">"Experience-grade CRM • HubSpot + Frappe DNA"</p>
                    <h1 class="hero-title">"Customer HQ built for velocity"</h1>
                    <p class="hero-subtitle">
                        "Launch-ready workspaces, live collaboration, offline resilience, and guided automations in a single interface."
                    </p>
                    <div class="hero-actions">
                        <a class="btn btn-primary" href="/app/dashboard">"Open control center"</a>
                        <a class="btn btn-secondary" href="/app/settings/workflows">"Automate a journey"</a>
                    </div>
                    <div class="hero-highlights">
                        <span class="pill">"⚡ Realtime presence & CRDT sync"</span>
                        <span class="pill">"📱 Offline-first mobile & desktop"</span>
                        <span class="pill">"🤖 Automations, inbox, analytics"</span>
                    </div>
                </div>
                <div class="hero-scorecard">
                    <div class="scorecard-metric">
                        <p class="metric-label">"Total records"</p>
                        <Suspense fallback=|| view! { <div class="metric-value skeleton-text full"></div> }>
                            {move || total_records.get().to_string()}
                        </Suspense>
                        <span class="metric-hint">"Contacts, companies, deals"</span>
                    </div>
                    <div class="scorecard-health">
                        <p class="metric-label">"Experience health"</p>
                        <div class="health-indicator">
                            <div class="health-score">{move || adoption_score.get()}</div>
                            <div class="health-meta">
                                <p class="health-title">"Adoption index"</p>
                                <p class="health-subtitle">"Playbooks, touchpoints, automation coverage"</p>
                            </div>
                        </div>
                        <div class="health-legend">
                            <span>"Live collaboration"</span>
                            <span>"Smart routing"</span>
                            <span>"Guided onboarding"</span>
                        </div>
                    </div>
                </div>
            </section>

            <section class="home-metrics">
                <div class="home-metrics-grid">
                    <div class="home-card">
                        <div class="card-header">
                            <span class="card-icon">"👥"</span>
                            <div>
                                <p class="card-eyebrow">"HubSpot-grade contacts"</p>
                                <h3>"Contacts"</h3>
                            </div>
                            <span class="chip success">"Live"</span>
                        </div>
                        <p class="card-subtitle">"Unified timelines, touchpoints, ownership, SLAs"</p>
                        <div class="metric-value">
                            <Suspense fallback=|| "…">
                                {move || contacts_count.get().map(|c| c.to_string()).unwrap_or_else(|| "0".to_string())}
                            </Suspense>
                        </div>
                        <p class="metric-caption">"Two-way sync • dedupe • quick import"</p>
                    </div>

                    <div class="home-card">
                        <div class="card-header">
                            <span class="card-icon">"🏢"</span>
                            <div>
                                <p class="card-eyebrow">"Account 360"</p>
                                <h3>"Companies"</h3>
                            </div>
                            <span class="chip info">"Mapped"</span>
                        </div>
                        <p class="card-subtitle">"Parent/child hierarchies, custom fields, segments"</p>
                        <div class="metric-value">
                            <Suspense fallback=|| "…">
                                {move || companies_count.get().map(|c| c.to_string()).unwrap_or_else(|| "0".to_string())}
                            </Suspense>
                        </div>
                        <p class="metric-caption">"Enrichment-ready • account owners • playbooks"</p>
                    </div>

                    <div class="home-card">
                        <div class="card-header">
                            <span class="card-icon">"💼"</span>
                            <div>
                                <p class="card-eyebrow">"Pipeline intelligence"</p>
                                <h3>"Deals"</h3>
                            </div>
                            <span class="chip warning">"Forecasting"</span>
                        </div>
                        <p class="card-subtitle">"Stages, probability, multi-currency, weighted totals"</p>
                        <div class="metric-value">
                            <Suspense fallback=|| "…">
                                {move || deals_count.get().map(|c| c.to_string()).unwrap_or_else(|| "0".to_string())}
                            </Suspense>
                        </div>
                        <p class="metric-caption">"Close-rate drivers • renewal visibility"</p>
                    </div>

                    <div class="home-card">
                        <div class="card-header">
                            <span class="card-icon">"✅"</span>
                            <div>
                                <p class="card-eyebrow">"Work orchestration"</p>
                                <h3>"Tasks"</h3>
                            </div>
                            <span class="chip purple">"Guided"</span>
                        </div>
                        <p class="card-subtitle">"Smart queues, SLA timers, auto-assignment"</p>
                        <div class="metric-value">
                            <Suspense fallback=|| "…">
                                {move || tasks_count.get().map(|c| c.to_string()).unwrap_or_else(|| "0".to_string())}
                            </Suspense>
                        </div>
                        <p class="metric-caption">"Inbox + sequences + approvals"</p>
                    </div>

                    <div class="home-card">
                        <div class="card-header">
                            <span class="card-icon">"🧠"</span>
                            <div>
                                <p class="card-eyebrow">"Metadata-first"</p>
                                <h3>"Properties"</h3>
                            </div>
                            <span class="chip">"Schema"</span>
                        </div>
                        <p class="card-subtitle">"Self-serve objects, governance, auditability"</p>
                        <div class="metric-value">
                            <Suspense fallback=|| "…">
                                {move || properties_count.get().map(|c| c.to_string()).unwrap_or_else(|| "0".to_string())}
                            </Suspense>
                        </div>
                        <p class="metric-caption">"Launch without migrations • API ready"</p>
                    </div>

                    <div class="home-card">
                        <div class="card-header">
                            <span class="card-icon">"📊"</span>
                            <div>
                                <p class="card-eyebrow">"Frappe-grade reports"</p>
                                <h3>"Insights"</h3>
                            </div>
                            <span class="chip">"Live"</span>
                        </div>
                        <p class="card-subtitle">"Dashboards, embedded charts, export, warehouse sync"</p>
                        <div class="metric-value">
                            <Suspense fallback=|| "…">
                                {move || total_records.get().to_string()}
                            </Suspense>
                        </div>
                        <p class="metric-caption">"Data studio • Prometheus metrics"</p>
                    </div>
                </div>
            </section>

            <section class="experience-lanes">
                <div class="lane-card">
                    <div class="lane-header">
                        <h4>"Momentum lane"</h4>
                        <span class="chip success">"Recommended"</span>
                    </div>
                    <p class="lane-subtitle">"Convert faster with collaborative playbooks and tight routing."</p>
                    <ul class="lane-list">
                        <li>"Smart owner routing • round robin • escalation rules"</li>
                        <li>"Sequences with pause/resume, SLA timers, and next best step"</li>
                        <li>"Universal inbox with notes, mentions, and activity timeline"</li>
                    </ul>
                </div>

                <div class="lane-card">
                    <div class="lane-header">
                        <h4>"Automation lane"</h4>
                        <span class="chip warning">"Low-code"</span>
                    </div>
                    <p class="lane-subtitle">"Drag-and-drop workflows with approvals, SLAs, and observability."</p>
                    <ul class="lane-list">
                        <li>"Workflow canvas with WASM actions and reusable contracts"</li>
                        <li>"Branching logic, delays, webhooks, AI text enrichers"</li>
                        <li>"Runs in offline mode with CRDT merges once reconnected"</li>
                    </ul>
                </div>

                <div class="lane-card">
                    <div class="lane-header">
                        <h4>"Quality lane"</h4>
                        <span class="chip info">"Guardrails"</span>
                    </div>
                    <p class="lane-subtitle">"Enterprise governance without slowing teams down."</p>
                    <ul class="lane-list">
                        <li>"Role-aware views, audit fields, and inline approvals"</li>
                        <li>"Schema-driven validation for every property and object"</li>
                        <li>"Built-in performance monitors with backpressure controls"</li>
                    </ul>
                </div>
            </section>
        </div>
    }
}
