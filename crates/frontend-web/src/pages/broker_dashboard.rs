//! Broker Dashboard - Team & Commission Management
//!
//! Provides brokers with:
//! - Agent performance overview
//! - Commission tracking
//! - Team deals pipeline
//! - Revenue analytics

use leptos::*;
use uuid::Uuid;

/// Agent performance data
#[derive(Clone, Debug)]
pub struct AgentPerformance {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub deals_closed: i32,
    pub revenue: f64,
    pub commission: f64,
    pub leads_assigned: i32,
    pub conversion_rate: f64,
}

/// Broker Dashboard
#[component]
pub fn BrokerDashboard() -> impl IntoView {
    let agents = vec![
        AgentPerformance {
            id: Uuid::new_v4(),
            name: "Sarah Ahmed".to_string(),
            avatar: None,
            deals_closed: 12,
            revenue: 2500000.0,
            commission: 75000.0,
            leads_assigned: 45,
            conversion_rate: 26.7,
        },
        AgentPerformance {
            id: Uuid::new_v4(),
            name: "Omar Hassan".to_string(),
            avatar: None,
            deals_closed: 8,
            revenue: 1800000.0,
            commission: 54000.0,
            leads_assigned: 38,
            conversion_rate: 21.0,
        },
        AgentPerformance {
            id: Uuid::new_v4(),
            name: "Fatima Al-Rashid".to_string(),
            avatar: None,
            deals_closed: 15,
            revenue: 3200000.0,
            commission: 96000.0,
            leads_assigned: 52,
            conversion_rate: 28.8,
        },
    ];
    
    let total_revenue: f64 = agents.iter().map(|a| a.revenue).sum();
    let total_commission: f64 = agents.iter().map(|a| a.commission).sum();
    let total_deals: i32 = agents.iter().map(|a| a.deals_closed).sum();
    
    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 p-6">
            // Header
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-white">"📊 Broker Dashboard"</h1>
                <p class="text-slate-400">"Team performance and commission tracking"</p>
            </div>
            
            // KPI Cards
            <div class="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
                <div class="p-6 bg-gradient-to-br from-green-500/20 to-emerald-500/20 rounded-2xl border border-white/10">
                    <div class="text-3xl mb-2">"💰"</div>
                    <div class="text-2xl font-bold text-green-400">{format!("${:.0}K", total_revenue / 1000.0)}</div>
                    <div class="text-sm text-slate-400">"Total Revenue"</div>
                </div>
                <div class="p-6 bg-gradient-to-br from-purple-500/20 to-pink-500/20 rounded-2xl border border-white/10">
                    <div class="text-3xl mb-2">"💎"</div>
                    <div class="text-2xl font-bold text-purple-400">{format!("${:.0}K", total_commission / 1000.0)}</div>
                    <div class="text-sm text-slate-400">"Total Commissions"</div>
                </div>
                <div class="p-6 bg-gradient-to-br from-blue-500/20 to-cyan-500/20 rounded-2xl border border-white/10">
                    <div class="text-3xl mb-2">"🎯"</div>
                    <div class="text-2xl font-bold text-blue-400">{total_deals}</div>
                    <div class="text-sm text-slate-400">"Deals Closed"</div>
                </div>
                <div class="p-6 bg-gradient-to-br from-orange-500/20 to-amber-500/20 rounded-2xl border border-white/10">
                    <div class="text-3xl mb-2">"👥"</div>
                    <div class="text-2xl font-bold text-orange-400">{agents.len()}</div>
                    <div class="text-sm text-slate-400">"Active Agents"</div>
                </div>
            </div>
            
            // Agent leaderboard
            <div class="bg-white/5 border border-white/10 rounded-2xl p-6 mb-8">
                <h2 class="text-xl font-bold text-white mb-4">"🏆 Agent Leaderboard"</h2>
                <div class="overflow-x-auto">
                    <table class="w-full">
                        <thead>
                            <tr class="text-left text-slate-400 border-b border-white/10">
                                <th class="pb-3 font-medium">"Rank"</th>
                                <th class="pb-3 font-medium">"Agent"</th>
                                <th class="pb-3 font-medium text-right">"Deals"</th>
                                <th class="pb-3 font-medium text-right">"Revenue"</th>
                                <th class="pb-3 font-medium text-right">"Commission"</th>
                                <th class="pb-3 font-medium text-right">"Conversion"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {agents.clone().into_iter().enumerate().map(|(i, agent)| {
                                let rank_badge = match i {
                                    0 => "🥇",
                                    1 => "🥈",
                                    2 => "🥉",
                                    _ => "",
                                };
                                view! {
                                    <tr class="border-b border-white/5">
                                        <td class="py-4 text-2xl">{rank_badge}</td>
                                        <td class="py-4">
                                            <div class="flex items-center gap-3">
                                                <div class="w-10 h-10 bg-indigo-500/20 rounded-full flex items-center justify-center">
                                                    "👤"
                                                </div>
                                                <span class="text-white font-medium">{&agent.name}</span>
                                            </div>
                                        </td>
                                        <td class="py-4 text-right text-white">{agent.deals_closed}</td>
                                        <td class="py-4 text-right text-green-400">{format!("${:.0}K", agent.revenue / 1000.0)}</td>
                                        <td class="py-4 text-right text-purple-400">{format!("${:.0}K", agent.commission / 1000.0)}</td>
                                        <td class="py-4 text-right">
                                            <span class="px-2 py-1 bg-blue-500/20 text-blue-400 rounded-lg text-sm">
                                                {format!("{:.1}%", agent.conversion_rate)}
                                            </span>
                                        </td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>()}
                        </tbody>
                    </table>
                </div>
            </div>
            
            // Commission payouts
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div class="bg-white/5 border border-white/10 rounded-2xl p-6">
                    <h2 class="text-xl font-bold text-white mb-4">"💳 Pending Payouts"</h2>
                    <div class="space-y-3">
                        {agents.into_iter().map(|agent| {
                            view! {
                                <div class="flex items-center justify-between p-4 bg-white/5 rounded-xl">
                                    <div class="flex items-center gap-3">
                                        <div class="w-8 h-8 bg-purple-500/20 rounded-full flex items-center justify-center text-sm">
                                            "👤"
                                        </div>
                                        <span class="text-white">{&agent.name}</span>
                                    </div>
                                    <div class="flex items-center gap-4">
                                        <span class="text-purple-400 font-medium">{format!("${:.0}", agent.commission)}</span>
                                        <button class="px-3 py-1 bg-green-500 text-white rounded-lg text-sm">
                                            "Pay"
                                        </button>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
                
                <div class="bg-white/5 border border-white/10 rounded-2xl p-6">
                    <h2 class="text-xl font-bold text-white mb-4">"📈 This Month"</h2>
                    <div class="space-y-4">
                        <div class="flex items-center justify-between">
                            <span class="text-slate-400">"Target"</span>
                            <span class="text-white font-medium">"$10,000,000"</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-slate-400">"Achieved"</span>
                            <span class="text-green-400 font-medium">{format!("${:.0}", total_revenue)}</span>
                        </div>
                        <div class="w-full bg-white/10 rounded-full h-3">
                            <div 
                                class="bg-gradient-to-r from-indigo-500 to-purple-500 h-3 rounded-full"
                                style=format!("width: {}%", (total_revenue / 10000000.0 * 100.0).min(100.0))
                            ></div>
                        </div>
                        <div class="text-center text-slate-400 text-sm">
                            {format!("{:.1}% of monthly target", total_revenue / 10000000.0 * 100.0)}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
