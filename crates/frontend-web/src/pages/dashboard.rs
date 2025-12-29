//! Cinematic Dashboard: The Command Center
//! The heartbeat of the Neural OS.

use leptos::*;
use crate::widgets::cards::glass_stat::GlassStatCard;
use crate::design_system::tables::phantom_row::PhantomRow;
use crate::api::{fetch_dashboard, DashboardResponse};

#[component]
pub fn Dashboard() -> impl IntoView {
    // Data Resource
    let dashboard_data = create_local_resource(
        || "this_month",
        |range| async move {
            fetch_dashboard(range).await
        }
    );

    view! {
        <div class="h-full flex flex-col gap-8 p-8 overflow-y-auto custom-scrollbar animate-fade-in">
            // Header
            <header class="flex items-center justify-between">
                <div>
                    <h1 class="text-4xl font-bold text-white tracking-tight mb-2">
                        "Command Center"
                    </h1>
                    <div class="flex items-center gap-2 text-zinc-400 font-mono text-sm">
                        <span class="text-emerald-400">"Neural Core v3.0"</span>
                        "•"
                        "All Systems Online"
                    </div>
                </div>
                
                // Live Status
                <div class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-emerald-500/10 border border-emerald-500/20">
                    <div class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></div>
                    <span class="text-xs font-medium text-emerald-400">"Live"</span>
                </div>
            </header>

            // Stats Grid (Staggered Animation)
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <Suspense fallback=move || view! { <div class="col-span-4 text-center text-zinc-500 animate-pulse">"Loading Neural Metrics..."</div> }>
                    {move || {
                        dashboard_data.get().map(|res| {
                            match res {
                                Ok(data) => {
                                    let kpis = data.kpis;
                                    view! {
                                        <div class="animate-spring-up" style="animation-delay: 0ms">
                                            <GlassStatCard 
                                                label="REVENUE (FCST)" 
                                                value=format!("${:.0}", kpis.forecasted_revenue) 
                                                trend=format!("+{:.1}%", kpis.revenue_trend) 
                                                icon="fa-chart-line" 
                                                color="violet"
                                            />
                                        </div>
                                        <div class="animate-spring-up" style="animation-delay: 100ms">
                                            <GlassStatCard 
                                                label="ACTIVE DEALS" 
                                                value=kpis.ongoing_deals.to_string()
                                                trend=format!("{:.1}%", kpis.deals_trend)
                                                icon="fa-handshake" 
                                                color="emerald" 
                                            />
                                        </div>
                                        <div class="animate-spring-up" style="animation-delay: 200ms">
                                            <GlassStatCard 
                                                label="TOTAL LEADS" 
                                                value=kpis.total_leads.to_string()
                                                trend=format!("+{:.1}%", kpis.leads_trend) 
                                                icon="fa-users" 
                                                color="blue" 
                                            />
                                        </div>
                                        <div class="animate-spring-up" style="animation-delay: 300ms">
                                            <GlassStatCard 
                                                label="WIN RATE" 
                                                value=format!("{:.1}%", kpis.win_rate)
                                                trend=format!("{:.1}%", kpis.win_rate_trend)
                                                icon="fa-trophy" 
                                                color="rose" 
                                            />
                                        </div>
                                    }.into_view()
                                }
                                Err(e) => view! { <div class="col-span-4 text-red-500">"Error loading dashboard: " {e}</div> }.into_view()
                            }
                        })
                    }}
                </Suspense>
            </div>

            // Recent Transmissions (Static for now, but PhantomRow ready)
            <section class="animate-slide-in-left" style="animation-delay: 400ms">
                <div class="flex items-center justify-between mb-6">
                    <h2 class="text-sm font-bold text-zinc-500 tracking-widest uppercase">
                        "Recent Activity"
                    </h2>
                </div>

                <div class="flex flex-col gap-1">
                     <Suspense fallback=move || view! { <div>"Loading activity..."</div> }>
                        {move || {
                            dashboard_data.get().map(|res| {
                                if let Ok(data) = res {
                                    data.recent_activities.iter().take(5).map(|activity| {
                                        view! {
                                            <PhantomRow
                                                primary_text=activity.entity_name.clone()
                                                secondary_text=format!("{} - {}", activity.action, activity.entity)
                                                status=activity.action.clone()
                                                status_color="zinc" 
                                                meta_text=activity.timestamp.clone()
                                            />
                                        }
                                    }).collect_view()
                                } else {
                                    view! {}.into_view()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </section>
        </div>
    }
}
