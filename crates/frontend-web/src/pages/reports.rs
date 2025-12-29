//! Reports Page - Agent Leaderboard and Analytics Reports

use leptos::*;

/// Available report types
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    AgentLeaderboard,
    DealPerformance,
    ActivityLog,
    LeadSources,
}

impl ReportType {
    fn name(&self) -> &'static str {
        match self {
            ReportType::AgentLeaderboard => "Agent Leaderboard",
            ReportType::DealPerformance => "Deal Performance",
            ReportType::ActivityLog => "Activity Log",
            ReportType::LeadSources => "Lead Sources",
        }
    }
    
    fn icon(&self) -> &'static str {
        match self {
            ReportType::AgentLeaderboard => "🏆",
            ReportType::DealPerformance => "💰",
            ReportType::ActivityLog => "📋",
            ReportType::LeadSources => "📊",
        }
    }
}

/// Agent data for leaderboard
#[derive(Clone)]
struct AgentData {
    name: String,
    email: String,
    revenue: f64,
    revenue_target: Option<f64>,
    deals_won: i32,
    deals_target: Option<i32>,
    conversion_rate: f64,
}

/// Reports Page Component
#[component]
pub fn ReportsPage() -> impl IntoView {
    let (selected_report, set_selected_report) = create_signal(ReportType::AgentLeaderboard);
    
    // Mock agent data (would come from API in production)
    let agents = vec![
        AgentData {
            name: "Sarah Johnson".to_string(),
            email: "sarah@demo.com".to_string(),
            revenue: 45000.0,
            revenue_target: Some(50000.0),
            deals_won: 8,
            deals_target: Some(10),
            conversion_rate: 32.0,
        },
        AgentData {
            name: "Michael Chen".to_string(),
            email: "michael@demo.com".to_string(),
            revenue: 38500.0,
            revenue_target: Some(50000.0),
            deals_won: 6,
            deals_target: Some(10),
            conversion_rate: 28.0,
        },
        AgentData {
            name: "Emily Davis".to_string(),
            email: "emily@demo.com".to_string(),
            revenue: 32000.0,
            revenue_target: Some(50000.0),
            deals_won: 5,
            deals_target: Some(10),
            conversion_rate: 25.0,
        },
        AgentData {
            name: "James Wilson".to_string(),
            email: "james@demo.com".to_string(),
            revenue: 28000.0,
            revenue_target: Some(50000.0),
            deals_won: 4,
            deals_target: Some(10),
            conversion_rate: 22.0,
        },
    ];
    
    let report_types = vec![
        ReportType::AgentLeaderboard,
        ReportType::DealPerformance,
        ReportType::ActivityLog,
        ReportType::LeadSources,
    ];
    
    view! {
        <div class="reports-page">
            // Sidebar
            <aside class="reports-sidebar">
                <h2 class="reports-sidebar__title">"📈 Reports"</h2>
                <nav class="reports-nav">
                    {report_types.into_iter().map(|report| {
                        let is_active = move || selected_report.get() == report;
                        view! {
                            <div 
                                class=move || format!(
                                    "reports-nav__item{}",
                                    if is_active() { " active" } else { "" }
                                )
                                on:click=move |_| set_selected_report.set(report)
                            >
                                <span class="reports-nav__icon">{report.icon()}</span>
                                {report.name()}
                            </div>
                        }
                    }).collect_view()}
                </nav>
            </aside>
            
            // Main Content
            <main class="reports-content">
                <div class="reports-header">
                    <h1 class="reports-header__title">
                        {move || selected_report.get().name()}
                    </h1>
                    <div class="reports-actions">
                        <button class="ui-btn ui-btn-secondary" on:click=move |_| export_to_csv()>
                            "📥 Export CSV"
                        </button>
                    </div>
                </div>
                
                // Report Content
                {move || match selected_report.get() {
                    ReportType::AgentLeaderboard => view! {
                        <AgentLeaderboard agents=agents.clone() />
                    }.into_view(),
                    _ => view! {
                        <div class="chart-empty">
                            <div class="chart-empty-icon">"🚧"</div>
                            <p>"This report is coming soon!"</p>
                        </div>
                    }.into_view(),
                }}
            </main>
        </div>
    }
}

/// Agent Leaderboard Table
#[component]
fn AgentLeaderboard(agents: Vec<AgentData>) -> impl IntoView {
    view! {
        <table class="leaderboard-table">
            <thead>
                <tr>
                    <th class="leaderboard-rank">"#"</th>
                    <th>"Agent"</th>
                    <th>"Revenue"</th>
                    <th>"Progress"</th>
                    <th>"Deals Won"</th>
                    <th>"Conversion"</th>
                </tr>
            </thead>
            <tbody>
                {agents.into_iter().enumerate().map(|(idx, agent)| {
                    let rank = idx + 1;
                    let rank_class = match rank {
                        1 => "leaderboard-rank--1",
                        2 => "leaderboard-rank--2",
                        3 => "leaderboard-rank--3",
                        _ => "",
                    };
                    let initials = agent.name.split_whitespace()
                        .take(2)
                        .filter_map(|s| s.chars().next())
                        .collect::<String>();
                    let progress = agent.revenue_target
                        .map(|t| (agent.revenue / t * 100.0).min(100.0))
                        .unwrap_or(0.0);
                    
                    view! {
                        <tr>
                            <td class=format!("leaderboard-rank {}", rank_class)>
                                {if rank <= 3 {
                                    match rank {
                                        1 => "🥇".to_string(),
                                        2 => "🥈".to_string(),
                                        3 => "🥉".to_string(),
                                        _ => rank.to_string(),
                                    }
                                } else {
                                    rank.to_string()
                                }}
                            </td>
                            <td>
                                <div class="leaderboard-agent">
                                    <div class="leaderboard-avatar">{initials}</div>
                                    <div>
                                        <div class="leaderboard-name">{&agent.name}</div>
                                        <div class="leaderboard-email">{&agent.email}</div>
                                    </div>
                                </div>
                            </td>
                            <td class="leaderboard-value">
                                {format!("${:.0}", agent.revenue)}
                            </td>
                            <td>
                                <div class="leaderboard-progress">
                                    <div class="leaderboard-progress__bar">
                                        <div 
                                            class="leaderboard-progress__fill"
                                            style=format!("width: {}%", progress)
                                        ></div>
                                    </div>
                                    <span class="leaderboard-progress__text">
                                        {format!("{:.0}%", progress)}
                                    </span>
                                </div>
                            </td>
                            <td class="leaderboard-value">{agent.deals_won}</td>
                            <td class="leaderboard-value">
                                {format!("{:.1}%", agent.conversion_rate)}
                            </td>
                        </tr>
                    }
                }).collect_view()}
            </tbody>
        </table>
    }
}

/// Export current report to CSV
fn export_to_csv() {
    // For now, just show the CSV content in a simple way
    // In production, you'd use a proper file download API
    if let Some(window) = web_sys::window() {
        let _ = window.alert_with_message(
            "CSV Export:\n\n\
            Agent,Email,Revenue,Target,Deals Won,Conversion Rate\n\
            Sarah Johnson,sarah@demo.com,45000,50000,8,32%\n\
            Michael Chen,michael@demo.com,38500,50000,6,28%\n\
            Emily Davis,emily@demo.com,32000,50000,5,25%\n\
            James Wilson,james@demo.com,28000,50000,4,22%\n\n\
            (Copy this data to a .csv file)"
        );
    }
}

