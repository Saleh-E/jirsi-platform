//! Dashboard Page - Analytics overview with KPIs and charts

use leptos::*;
use serde::{Deserialize, Serialize};
use crate::api::{fetch_json, API_BASE, TENANT_ID};
use crate::components::stat_card::StatCard;
use crate::components::chart::{FunnelChart, FunnelItem, LineChart, LineChartPoint};

/// Dashboard data from API
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct DashboardData {
    #[serde(default)]
    pub total_leads: i64,
    #[serde(default)]
    pub total_deals: i64,
    #[serde(default)]
    pub ongoing_deals: i64,
    #[serde(default)]
    pub forecasted_revenue: f64,
    #[serde(default)]
    pub win_rate: f64,
    #[serde(default)]
    pub leads_trend: f64,
    #[serde(default)]
    pub deals_trend: f64,
    #[serde(default)]
    pub sales_trend: Vec<SalesTrendPoint>,
    #[serde(default)]
    pub funnel_data: Vec<FunnelStage>,
    #[serde(default)]
    pub recent_activities: Vec<Activity>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SalesTrendPoint {
    pub date: String,
    pub leads: i64,
    pub deals: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FunnelStage {
    pub stage: String,
    pub count: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Activity {
    pub id: String,
    pub action: String,
    pub entity: String,
    pub entity_name: String,
    pub user: String,
    pub timestamp: String,
}

/// Dashboard Page Component
#[component]
pub fn DashboardPage() -> impl IntoView {
    let (loading, set_loading) = create_signal(true);
    let (data, set_data) = create_signal(DashboardData::default());
    
    // Mock data for demo (in production, fetch from /analytics/dashboard)
    create_effect(move |_| {
        set_loading.set(true);
        
        // Simulate API call with mock data
        set_timeout(move || {
            let mock_data = DashboardData {
                total_leads: 156,
                total_deals: 42,
                ongoing_deals: 28,
                forecasted_revenue: 1_250_000.0,
                win_rate: 68.5,
                leads_trend: 12.5,
                deals_trend: 8.3,
                sales_trend: vec![
                    SalesTrendPoint { date: "Jan".to_string(), leads: 45, deals: 12 },
                    SalesTrendPoint { date: "Feb".to_string(), leads: 52, deals: 15 },
                    SalesTrendPoint { date: "Mar".to_string(), leads: 48, deals: 14 },
                    SalesTrendPoint { date: "Apr".to_string(), leads: 61, deals: 18 },
                    SalesTrendPoint { date: "May".to_string(), leads: 55, deals: 16 },
                    SalesTrendPoint { date: "Jun".to_string(), leads: 72, deals: 22 },
                ],
                funnel_data: vec![
                    FunnelStage { stage: "New".to_string(), count: 45 },
                    FunnelStage { stage: "Qualified".to_string(), count: 32 },
                    FunnelStage { stage: "Proposal".to_string(), count: 18 },
                    FunnelStage { stage: "Negotiation".to_string(), count: 12 },
                    FunnelStage { stage: "Won".to_string(), count: 8 },
                ],
                recent_activities: vec![
                    Activity {
                        id: "1".to_string(),
                        action: "Created".to_string(),
                        entity: "Deal".to_string(),
                        entity_name: "Luxury Villa Sale".to_string(),
                        user: "John Doe".to_string(),
                        timestamp: "2 hours ago".to_string(),
                    },
                    Activity {
                        id: "2".to_string(),
                        action: "Updated".to_string(),
                        entity: "Contact".to_string(),
                        entity_name: "Sarah Smith".to_string(),
                        user: "Jane Admin".to_string(),
                        timestamp: "4 hours ago".to_string(),
                    },
                    Activity {
                        id: "3".to_string(),
                        action: "Won".to_string(),
                        entity: "Deal".to_string(),
                        entity_name: "Downtown Penthouse".to_string(),
                        user: "Mike Sales".to_string(),
                        timestamp: "Yesterday".to_string(),
                    },
                    Activity {
                        id: "4".to_string(),
                        action: "Scheduled".to_string(),
                        entity: "Viewing".to_string(),
                        entity_name: "Beach House Tour".to_string(),
                        user: "John Doe".to_string(),
                        timestamp: "Yesterday".to_string(),
                    },
                ],
            };
            set_data.set(mock_data);
            set_loading.set(false);
        }, std::time::Duration::from_millis(300));
    });

    view! {
        <div class="dashboard-page">
            <div class="dashboard-header">
                <div>
                    <h1 class="dashboard-title">"Dashboard"</h1>
                    <p class="dashboard-subtitle">"Welcome back! Here's your business overview."</p>
                </div>
                <div class="dashboard-date-range">
                    <select class="form-input form-select">
                        <option value="today">"Today"</option>
                        <option value="week">"This Week"</option>
                        <option value="month" selected=true>"This Month"</option>
                        <option value="quarter">"This Quarter"</option>
                        <option value="year">"This Year"</option>
                    </select>
                </div>
            </div>

            {move || if loading.get() {
                view! {
                    <div class="dashboard-loading">
                        <div class="loading-spinner"></div>
                        <p>"Loading dashboard..."</p>
                    </div>
                }.into_view()
            } else {
                let d = data.get();
                
                view! {
                    <>
                        // KPI Cards Row
                        <div class="dashboard-kpis">
                            <StatCard
                                title="Total Leads"
                                value=d.total_leads.to_string()
                                trend_value=d.leads_trend
                                icon="👥"
                                color="blue"
                            />
                            <StatCard
                                title="Active Deals"
                                value=d.ongoing_deals.to_string()
                                trend_value=d.deals_trend
                                icon="📈"
                                color="purple"
                            />
                            <StatCard
                                title="Forecasted Revenue"
                                value=format_currency(d.forecasted_revenue)
                                trend_value=15.2
                                icon="💰"
                                color="green"
                            />
                            <StatCard
                                title="Win Rate"
                                value=format!("{:.1}%", d.win_rate)
                                trend_value=3.5
                                icon="🏆"
                                color="orange"
                            />
                        </div>

                        // Charts Row
                        <div class="dashboard-charts">
                            <div class="dashboard-chart">
                                <LineChart
                                    title="Sales Trend"
                                    points={d.sales_trend.iter().map(|p| {
                                        LineChartPoint { x: p.date.clone(), y: p.leads as f64 }
                                    }).collect::<Vec<_>>()}
                                    color="#4f46e5"
                                />
                            </div>
                            <div class="dashboard-chart">
                                <FunnelChart
                                    title="Sales Funnel"
                                    items={d.funnel_data.iter().enumerate().map(|(i, f)| {
                                        let colors = ["#4f46e5", "#7c3aed", "#a855f7", "#d946ef", "#22c55e"];
                                        FunnelItem {
                                            label: f.stage.clone(),
                                            value: f.count,
                                            color: colors.get(i).unwrap_or(&"#4f46e5").to_string(),
                                        }
                                    }).collect::<Vec<_>>()}
                                />
                            </div>
                        </div>

                        // Recent Activity
                        <div class="dashboard-activity">
                            <h3 class="activity-title">"Recent Activity"</h3>
                            <div class="activity-list">
                                {d.recent_activities.iter().map(|a| {
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
                        </div>
                    </>
                }.into_view()
            }}
        </div>
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
