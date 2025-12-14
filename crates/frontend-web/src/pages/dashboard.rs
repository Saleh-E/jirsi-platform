//! Dashboard Page - Analytics overview with KPIs and charts
//! Connected to real backend API

use leptos::*;
use crate::api::{fetch_dashboard, DashboardResponse};
use crate::components::stat_card::StatCard;
use crate::components::chart::{FunnelChart, FunnelItem, LineChart, LineChartPoint};

/// Dashboard Page Component - Real Data Integration
#[component]
pub fn DashboardPage() -> impl IntoView {
    // Date range state
    let (date_range, set_date_range) = create_signal("this_month".to_string());
    
    // Get user name from localStorage
    let user_name = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item("user_email").ok())
        .flatten()
        .map(|email| {
            let name = email.split('@').next().unwrap_or("User");
            let parts: Vec<&str> = name.split('.').collect();
            if parts.len() >= 2 {
                format!("{} {}", capitalize(parts[0]), capitalize(parts[1]))
            } else {
                capitalize(name)
            }
        })
        .unwrap_or_else(|| "User".to_string());
    
    // Fetch dashboard data using create_resource
    let dashboard_data = create_resource(
        move || date_range.get(),
        |range| async move {
            fetch_dashboard(&range).await
        }
    );

    view! {
        <div class="dashboard-page">
            // Header with greeting and date picker
            <div class="dashboard-header">
                <div>
                    <h1 class="dashboard-title">
                        {get_greeting()}", "{user_name}"!"
                    </h1>
                    <p class="dashboard-subtitle">"Here's your sales command center"</p>
                </div>
                <div class="dashboard-date-range">
                    <select 
                        class="form-input form-select"
                        on:change=move |ev| set_date_range.set(event_target_value(&ev))
                    >
                        <option value="today">"Today"</option>
                        <option value="this_week">"This Week"</option>
                        <option value="this_month" selected=true>"This Month"</option>
                        <option value="this_quarter">"This Quarter"</option>
                        <option value="this_year">"This Year"</option>
                    </select>
                </div>
            </div>

            // Main content with loading states
            {move || {
                match dashboard_data.get() {
                    None => view! { <DashboardSkeleton/> }.into_view(),
                    Some(Err(err)) => view! { <DashboardError error=err/> }.into_view(),
                    Some(Ok(data)) => view! { <DashboardContent data=data/> }.into_view(),
                }
            }}
        </div>
    }
}

/// Get greeting
fn get_greeting() -> &'static str {
    "Welcome"
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

/// Skeleton loader while data is loading
#[component]
fn DashboardSkeleton() -> impl IntoView {
    view! {
        <div class="dashboard-skeleton">
            <div class="dashboard-kpis">
                <div class="skeleton-card"><div class="skeleton-shimmer"></div></div>
                <div class="skeleton-card"><div class="skeleton-shimmer"></div></div>
                <div class="skeleton-card"><div class="skeleton-shimmer"></div></div>
                <div class="skeleton-card"><div class="skeleton-shimmer"></div></div>
            </div>
            <div class="dashboard-charts">
                <div class="skeleton-chart"><div class="skeleton-shimmer"></div></div>
                <div class="skeleton-chart skeleton-chart--small"><div class="skeleton-shimmer"></div></div>
            </div>
            <div class="skeleton-activity"><div class="skeleton-shimmer"></div></div>
        </div>
    }
}

/// Error state display
#[component]
fn DashboardError(error: String) -> impl IntoView {
    view! {
        <div class="dashboard-error">
            <div class="error-icon">"⚠️"</div>
            <h3>"Unable to load dashboard"</h3>
            <p class="error-message">{error}</p>
            <button 
                class="btn btn-primary"
                on:click=move |_| {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().reload();
                    }
                }
            >
                "Try Again"
            </button>
        </div>
    }
}

/// Dashboard content with real data
#[component]
fn DashboardContent(data: DashboardResponse) -> impl IntoView {
    let kpis = data.kpis.clone();
    
    view! {
        <>
            // KPI Cards Row
            <div class="dashboard-kpis">
                <StatCard
                    title="Total Leads"
                    value=kpis.total_leads.to_string()
                    trend_value=kpis.leads_trend
                    icon="👥"
                    color="blue"
                />
                <StatCard
                    title="Active Deals"
                    value=kpis.ongoing_deals.to_string()
                    trend_value=kpis.deals_trend
                    icon="📈"
                    color="purple"
                />
                <StatCard
                    title="Forecasted Revenue"
                    value=format_currency(kpis.forecasted_revenue)
                    trend_value=kpis.revenue_trend
                    icon="💰"
                    color="green"
                />
                <StatCard
                    title="Win Rate"
                    value=format!("{:.1}%", kpis.win_rate)
                    trend_value=kpis.win_rate_trend
                    icon="🏆"
                    color="orange"
                />
            </div>

            // Charts Row
            <div class="dashboard-charts">
                <div class="dashboard-chart dashboard-chart--wide">
                    {if data.sales_trend.is_empty() {
                        view! {
                            <div class="chart-empty">
                                <span class="chart-empty-icon">"📊"</span>
                                <p>"No sales data yet"</p>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <LineChart
                                title="Sales Trend"
                                points={data.sales_trend.iter().map(|p| {
                                    LineChartPoint { x: p.date.clone(), y: p.leads as f64 }
                                }).collect::<Vec<_>>()}
                                color="#4f46e5"
                            />
                        }.into_view()
                    }}
                </div>
                <div class="dashboard-chart">
                    {if data.funnel_data.is_empty() {
                        view! {
                            <div class="chart-empty">
                                <span class="chart-empty-icon">"🔻"</span>
                                <p>"No funnel data yet"</p>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <FunnelChart
                                title="Sales Funnel"
                                items={data.funnel_data.iter().enumerate().map(|(i, f)| {
                                    let colors = ["#4f46e5", "#7c3aed", "#a855f7", "#d946ef", "#22c55e"];
                                    FunnelItem {
                                        label: f.stage.clone(),
                                        value: f.count,
                                        color: colors.get(i).unwrap_or(&"#4f46e5").to_string(),
                                    }
                                }).collect::<Vec<_>>()}
                            />
                        }.into_view()
                    }}
                </div>
            </div>

            // Recent Activity
            <div class="dashboard-activity">
                <h3 class="activity-title">"Recent Activity"</h3>
                {if data.recent_activities.is_empty() {
                    view! {
                        <div class="activity-empty">
                            <span class="activity-empty-icon">"📝"</span>
                            <p>"No recent activity"</p>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="activity-list">
                            {data.recent_activities.iter().map(|a| {
                                let action_class = match a.action.as_str() {
                                    "Created" => "activity-action--created",
                                    "Updated" => "activity-action--updated",
                                    "Won" => "activity-action--won",
                                    "Scheduled" => "activity-action--scheduled",
                                    _ => "activity-action--default",
                                };
                                
                                view! {
                                    <div class="activity-item">
                                        <div class=format!("activity-action {}", action_class)>
                                            {match a.action.as_str() {
                                                "Created" => "✨",
                                                "Updated" => "📝",
                                                "Won" => "🎉",
                                                "Scheduled" => "📅",
                                                _ => "📌",
                                            }}
                                        </div>
                                        <div class="activity-content">
                                            <p class="activity-text">
                                                <span class="activity-user">{&a.user}</span>
                                                " "
                                                <span class="activity-verb">{a.action.to_lowercase()}</span>
                                                " "
                                                <span class="activity-entity">{&a.entity}</span>
                                                ": "
                                                <span class="activity-name">{&a.entity_name}</span>
                                            </p>
                                            <span class="activity-time">{&a.timestamp}</span>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_view()
                }}
            </div>
        </>
    }
}

/// Format number as currency
fn format_currency(n: f64) -> String {
    if n >= 1_000_000.0 {
        format!("${:.1}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("${:.0}K", n / 1_000.0)
    } else {
        format!("${:.0}", n)
    }
}
