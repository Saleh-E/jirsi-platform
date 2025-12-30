//! Landlord Portal - ROI Dashboard and Property Management
//!
//! Provides landlords with:
//! - Financial overview (ROI, income, expenses)
//! - Property portfolio at-a-glance
//! - Contract status
//! - Maintenance requests

use leptos::*;
use uuid::Uuid;

/// Financial summary for landlord
#[derive(Clone, Debug)]
pub struct LandlordFinancials {
    pub total_income: f64,
    pub total_expenses: f64,
    pub net_profit: f64,
    pub roi_percentage: f64,
    pub pending_payments: f64,
    pub occupancy_rate: f64,
}

/// Property summary
#[derive(Clone, Debug)]
pub struct PropertySummary {
    pub id: Uuid,
    pub title: String,
    pub address: String,
    pub status: String,
    pub monthly_rent: f64,
    pub tenant_name: Option<String>,
    pub next_payment_due: Option<String>,
}

/// Maintenance request
#[derive(Clone, Debug)]
pub struct MaintenanceRequest {
    pub id: Uuid,
    pub property_title: String,
    pub description: String,
    pub priority: String,
    pub status: String,
    pub created_at: String,
}

/// Landlord Portal Dashboard
#[component]
pub fn LandlordPortal() -> impl IntoView {
    // Mock data
    let financials = LandlordFinancials {
        total_income: 45000.0,
        total_expenses: 8500.0,
        net_profit: 36500.0,
        roi_percentage: 12.5,
        pending_payments: 3000.0,
        occupancy_rate: 92.0,
    };
    
    let properties = vec![
        PropertySummary {
            id: Uuid::new_v4(),
            title: "Marina Towers #1205".to_string(),
            address: "Dubai Marina, Dubai".to_string(),
            status: "Rented".to_string(),
            monthly_rent: 12000.0,
            tenant_name: Some("John Smith".to_string()),
            next_payment_due: Some("2024-01-05".to_string()),
        },
        PropertySummary {
            id: Uuid::new_v4(),
            title: "Business Bay Office".to_string(),
            address: "Business Bay, Dubai".to_string(),
            status: "Available".to_string(),
            monthly_rent: 25000.0,
            tenant_name: None,
            next_payment_due: None,
        },
    ];
    
    let maintenance_requests = vec![
        MaintenanceRequest {
            id: Uuid::new_v4(),
            property_title: "Marina Towers #1205".to_string(),
            description: "AC not cooling properly".to_string(),
            priority: "High".to_string(),
            status: "In Progress".to_string(),
            created_at: "2024-01-02".to_string(),
        },
    ];
    
    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 p-6">
            // Header
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-white">"🏠 Landlord Portal"</h1>
                <p class="text-slate-400">"Welcome back! Here\"s your property portfolio overview."</p>
            </div>
            
            // Financial KPIs
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                <FinancialCard 
                    title="Net Income"
                    value=format!("${:.0}", financials.net_profit)
                    subtitle="This month"
                    icon="💰"
                    color="green"
                />
                <FinancialCard 
                    title="ROI"
                    value=format!("{:.1}%", financials.roi_percentage)
                    subtitle="Annual return"
                    icon="📈"
                    color="blue"
                />
                <FinancialCard 
                    title="Occupancy Rate"
                    value=format!("{:.0}%", financials.occupancy_rate)
                    subtitle="Across portfolio"
                    icon="🏢"
                    color="purple"
                />
                <FinancialCard 
                    title="Pending"
                    value=format!("${:.0}", financials.pending_payments)
                    subtitle="To collect"
                    icon="⏳"
                    color="yellow"
                />
            </div>
            
            // Main content grid
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Properties list
                <div class="lg:col-span-2 bg-white/5 border border-white/10 rounded-2xl p-6">
                    <h2 class="text-xl font-bold text-white mb-4">"My Properties"</h2>
                    <div class="space-y-4">
                        {properties.into_iter().map(|prop| view! {
                            <PropertyCard property=prop />
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
                
                // Maintenance requests
                <div class="bg-white/5 border border-white/10 rounded-2xl p-6">
                    <h2 class="text-xl font-bold text-white mb-4">"🔧 Maintenance"</h2>
                    <div class="space-y-4">
                        {if maintenance_requests.is_empty() {
                            view! {
                                <div class="text-center text-slate-500 py-8">
                                    "No pending requests"
                                </div>
                            }.into_view()
                        } else {
                            maintenance_requests.into_iter().map(|req| view! {
                                <MaintenanceCard request=req />
                            }).collect::<Vec<_>>().into_view()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn FinancialCard(
    title: &'static str,
    value: String,
    subtitle: &'static str,
    icon: &'static str,
    color: &'static str,
) -> impl IntoView {
    let bg_color = match color {
        "green" => "bg-green-500/20",
        "blue" => "bg-blue-500/20",
        "purple" => "bg-purple-500/20",
        "yellow" => "bg-yellow-500/20",
        _ => "bg-slate-500/20",
    };
    
    let text_color = match color {
        "green" => "text-green-400",
        "blue" => "text-blue-400",
        "purple" => "text-purple-400",
        "yellow" => "text-yellow-400",
        _ => "text-slate-400",
    };
    
    view! {
        <div class=format!("p-6 rounded-2xl border border-white/10 {}", bg_color)>
            <div class="flex items-center justify-between mb-2">
                <span class="text-3xl">{icon}</span>
            </div>
            <div class=format!("text-2xl font-bold {}", text_color)>{value}</div>
            <div class="text-sm text-slate-400">{title}</div>
            <div class="text-xs text-slate-500 mt-1">{subtitle}</div>
        </div>
    }
}

#[component]
fn PropertyCard(property: PropertySummary) -> impl IntoView {
    let status_color = match property.status.as_str() {
        "Rented" => "bg-green-500/20 text-green-400",
        "Available" => "bg-blue-500/20 text-blue-400",
        _ => "bg-slate-500/20 text-slate-400",
    };
    
    view! {
        <div class="p-4 bg-white/5 rounded-xl border border-white/10 hover:bg-white/10 transition-all">
            <div class="flex items-center justify-between mb-2">
                <div>
                    <h3 class="font-semibold text-white">{&property.title}</h3>
                    <p class="text-sm text-slate-400">{&property.address}</p>
                </div>
                <span class=format!("px-3 py-1 rounded-full text-xs font-medium {}", status_color)>
                    {&property.status}
                </span>
            </div>
            <div class="flex items-center justify-between text-sm">
                <span class="text-slate-400">
                    {property.tenant_name.as_ref().map(|t| format!("👤 {}", t)).unwrap_or_else(|| "No tenant".to_string())}
                </span>
                <span class="text-green-400 font-medium">
                    {format!("${:.0}/mo", property.monthly_rent)}
                </span>
            </div>
        </div>
    }
}

#[component]
fn MaintenanceCard(request: MaintenanceRequest) -> impl IntoView {
    let priority_color = match request.priority.as_str() {
        "High" => "text-red-400",
        "Medium" => "text-yellow-400",
        _ => "text-slate-400",
    };
    
    view! {
        <div class="p-4 bg-white/5 rounded-xl border border-white/10">
            <div class="flex items-center justify-between mb-2">
                <span class=format!("text-sm font-medium {}", priority_color)>
                    {&request.priority}
                </span>
                <span class="text-xs text-slate-500">{&request.created_at}</span>
            </div>
            <p class="text-white text-sm mb-1">{&request.description}</p>
            <p class="text-slate-400 text-xs">{&request.property_title}</p>
        </div>
    }
}
