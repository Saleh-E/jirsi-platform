//! Agent Command Center - Lead Management and Dialer
//!
//! Provides agents with:
//! - Lead command center
//! - Click-to-dial interface
//! - Quick property matching
//! - Calendar overview

use leptos::*;
use uuid::Uuid;

/// Lead data for agent view
#[derive(Clone, Debug)]
pub struct Lead {
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub email: Option<String>,
    pub source: String,
    pub status: String,
    pub budget: Option<f64>,
    pub preferred_areas: Vec<String>,
    pub last_contact: Option<String>,
}

/// Agent Command Center
#[component]
pub fn AgentCommandCenter() -> impl IntoView {
    let (selected_lead, set_selected_lead) = create_signal(Option::<Lead>::None);
    let (dialer_active, set_dialer_active) = create_signal(false);
    
    let leads = vec![
        Lead {
            id: Uuid::new_v4(),
            name: "Sarah Johnson".to_string(),
            phone: "+971501234567".to_string(),
            email: Some("sarah@email.com".to_string()),
            source: "Website".to_string(),
            status: "Hot".to_string(),
            budget: Some(2000000.0),
            preferred_areas: vec!["Dubai Marina".to_string(), "JBR".to_string()],
            last_contact: Some("1 hour ago".to_string()),
        },
        Lead {
            id: Uuid::new_v4(),
            name: "Ahmed Al-Maktoum".to_string(),
            phone: "+971509876543".to_string(),
            email: Some("ahmed@company.ae".to_string()),
            source: "Referral".to_string(),
            status: "Warm".to_string(),
            budget: Some(5000000.0),
            preferred_areas: vec!["Palm Jumeirah".to_string()],
            last_contact: Some("2 days ago".to_string()),
        },
        Lead {
            id: Uuid::new_v4(),
            name: "Emma Wilson".to_string(),
            phone: "+971507654321".to_string(),
            email: None,
            source: "Dubizzle".to_string(),
            status: "New".to_string(),
            budget: Some(1500000.0),
            preferred_areas: vec!["Downtown".to_string()],
            last_contact: None,
        },
    ];
    
    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 p-6">
            // Header
            <div class="flex items-center justify-between mb-8">
                <div>
                    <h1 class="text-3xl font-bold text-white">"🎯 Agent Command Center"</h1>
                    <p class="text-slate-400">"Your leads, calls, and closings"</p>
                </div>
                <div class="flex gap-3">
                    <button class="px-4 py-2 bg-green-500 text-white rounded-lg font-medium flex items-center gap-2">
                        <span>"📞"</span> "Start Dialer"
                    </button>
                    <button class="px-4 py-2 bg-indigo-500 text-white rounded-lg font-medium flex items-center gap-2">
                        <span>"+"</span> "Add Lead"
                    </button>
                </div>
            </div>
            
            // Stats row
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
                <StatCard icon="🔥" label="Hot Leads" value="12" trend="+3 today" color="red" />
                <StatCard icon="📞" label="Calls Today" value="8" trend="32 this week" color="green" />
                <StatCard icon="🏠" label="Viewings" value="5" trend="2 scheduled" color="blue" />
                <StatCard icon="🎉" label="Closings" value="2" trend="$3.5M value" color="purple" />
            </div>
            
            // Main content
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Leads list
                <div class="lg:col-span-2 bg-white/5 border border-white/10 rounded-2xl p-6">
                    <div class="flex items-center justify-between mb-4">
                        <h2 class="text-xl font-bold text-white">"Active Leads"</h2>
                        <div class="flex gap-2">
                            {["All", "Hot", "Warm", "New"].into_iter().map(|filter| {
                                view! {
                                    <button class="px-3 py-1 bg-white/5 text-slate-400 rounded-lg text-sm hover:bg-white/10">
                                        {filter}
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                    <div class="space-y-3">
                        {leads.clone().into_iter().map(|lead| {
                            let lead_for_click = lead.clone();
                            view! {
                                <LeadCard 
                                    lead=lead 
                                    on_select=move |l| set_selected_lead.set(Some(l.clone()))
                                    on_call=move |_| set_dialer_active.set(true)
                                />
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
                
                // Quick actions / Selected lead
                <div class="space-y-6">
                    // Today's schedule
                    <div class="bg-white/5 border border-white/10 rounded-2xl p-6">
                        <h2 class="text-lg font-bold text-white mb-4">"📅 Today"</h2>
                        <div class="space-y-3">
                            <div class="flex items-center gap-3 p-3 bg-blue-500/10 rounded-lg">
                                <span class="text-blue-400">"10:00"</span>
                                <div>
                                    <p class="text-white text-sm">"Viewing - Marina Towers"</p>
                                    <p class="text-slate-400 text-xs">"Sarah Johnson"</p>
                                </div>
                            </div>
                            <div class="flex items-center gap-3 p-3 bg-purple-500/10 rounded-lg">
                                <span class="text-purple-400">"14:00"</span>
                                <div>
                                    <p class="text-white text-sm">"Listing Presentation"</p>
                                    <p class="text-slate-400 text-xs">"New client - Palm Villa"</p>
                                </div>
                            </div>
                            <div class="flex items-center gap-3 p-3 bg-green-500/10 rounded-lg">
                                <span class="text-green-400">"16:30"</span>
                                <div>
                                    <p class="text-white text-sm">"Contract Signing"</p>
                                    <p class="text-slate-400 text-xs">"Ahmed - JBR Apartment"</p>
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    // AI Suggestions
                    <div class="bg-gradient-to-br from-indigo-500/20 to-purple-500/20 border border-white/10 rounded-2xl p-6">
                        <h2 class="text-lg font-bold text-white mb-4">"🧠 AI Suggestions"</h2>
                        <div class="space-y-3">
                            <div class="p-3 bg-white/5 rounded-lg">
                                <p class="text-sm text-white">"Call Sarah Johnson - She viewed 3 properties this week"</p>
                            </div>
                            <div class="p-3 bg-white/5 rounded-lg">
                                <p class="text-sm text-white">"New listing matches Ahmed\"s requirements"</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StatCard(
    icon: &'static str,
    label: &'static str,
    value: &'static str,
    trend: &'static str,
    color: &'static str,
) -> impl IntoView {
    let bg = match color {
        "red" => "bg-red-500/10",
        "green" => "bg-green-500/10",
        "blue" => "bg-blue-500/10",
        "purple" => "bg-purple-500/10",
        _ => "bg-slate-500/10",
    };
    
    view! {
        <div class=format!("p-4 rounded-xl border border-white/10 {}", bg)>
            <div class="flex items-center gap-2 mb-2">
                <span class="text-xl">{icon}</span>
                <span class="text-slate-400 text-sm">{label}</span>
            </div>
            <div class="text-2xl font-bold text-white">{value}</div>
            <div class="text-xs text-slate-500">{trend}</div>
        </div>
    }
}

#[component]
fn LeadCard(
    lead: Lead,
    on_select: impl Fn(Lead) + 'static,
    on_call: impl Fn(()) + 'static,
) -> impl IntoView {
    let status_color = match lead.status.as_str() {
        "Hot" => "bg-red-500/20 text-red-400",
        "Warm" => "bg-orange-500/20 text-orange-400",
        "New" => "bg-blue-500/20 text-blue-400",
        _ => "bg-slate-500/20 text-slate-400",
    };
    
    let lead_clone = lead.clone();
    
    view! {
        <div class="flex items-center justify-between p-4 bg-white/5 rounded-xl hover:bg-white/10 transition-all cursor-pointer"
             on:click=move |_| on_select(lead_clone.clone())>
            <div class="flex items-center gap-4">
                <div class="w-10 h-10 bg-indigo-500/20 rounded-full flex items-center justify-center text-lg">
                    "👤"
                </div>
                <div>
                    <div class="flex items-center gap-2">
                        <span class="text-white font-medium">{&lead.name}</span>
                        <span class=format!("px-2 py-0.5 rounded-full text-xs {}", status_color)>
                            {&lead.status}
                        </span>
                    </div>
                    <div class="flex items-center gap-3 text-sm text-slate-400">
                        <span>{&lead.phone}</span>
                        <span>"•"</span>
                        <span>{lead.source.clone()}</span>
                        {lead.last_contact.as_ref().map(|t| view! {
                            <span class="text-slate-500">{format!("• {}", t)}</span>
                        })}
                    </div>
                </div>
            </div>
            <div class="flex items-center gap-2">
                <button 
                    class="p-2 bg-green-500/20 text-green-400 rounded-lg hover:bg-green-500/30"
                    on:click=move |e| {
                        e.stop_propagation();
                        on_call(());
                    }
                >
                    "📞"
                </button>
                <button class="p-2 bg-blue-500/20 text-blue-400 rounded-lg hover:bg-blue-500/30">
                    "💬"
                </button>
            </div>
        </div>
    }
}
