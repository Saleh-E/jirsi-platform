//! Tenant Portal - Home, Payments, and Requests
//!
//! Provides tenants with:
//! - My home overview
//! - Payment history and upcoming
//! - Maintenance request submission
//! - Document access

use leptos::*;
use uuid::Uuid;

/// Tenant portal main view
#[component]
pub fn TenantPortal() -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal("home");
    
    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 p-6">
            // Header
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-white">"🏠 My Home"</h1>
                <p class="text-slate-400">"Welcome to your tenant portal"</p>
            </div>
            
            // Tab navigation
            <div class="flex gap-2 mb-8">
                {["home", "payments", "requests", "documents"].into_iter().map(|tab| {
                    let is_active = move || active_tab.get() == tab;
                    let not_active = move || !is_active();
                    view! {
                        <button
                            class="px-4 py-2 rounded-lg font-medium transition-all"
                            class=("bg-indigo-500", is_active)
                            class=("text-white", is_active)
                            class=("bg-white/5", not_active)
                            class=("text-slate-400", not_active)
                            on:click=move |_| set_active_tab.set(tab)
                        >
                            {match tab {
                                "home" => "🏠 Home",
                                "payments" => "💳 Payments",
                                "requests" => "🔧 Requests",
                                "documents" => "📄 Documents",
                                _ => tab,
                            }}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
            
            // Tab content
            {move || match active_tab.get() {
                "home" => view! { <HomeTab /> }.into_view(),
                "payments" => view! { <PaymentsTab /> }.into_view(),
                "requests" => view! { <RequestsTab /> }.into_view(),
                "documents" => view! { <DocumentsTab /> }.into_view(),
                _ => view! { <HomeTab /> }.into_view(),
            }}
        </div>
    }
}

#[component]
fn HomeTab() -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            // Property info
            <div class="bg-white/5 border border-white/10 rounded-2xl p-6">
                <h2 class="text-xl font-bold text-white mb-4">"My Rental"</h2>
                <div class="space-y-4">
                    <div class="flex items-center gap-4">
                        <div class="w-24 h-24 bg-indigo-500/20 rounded-xl flex items-center justify-center text-4xl">
                            "🏢"
                        </div>
                        <div>
                            <h3 class="text-lg font-semibold text-white">"Marina Towers #1205"</h3>
                            <p class="text-slate-400">"Dubai Marina, UAE"</p>
                            <p class="text-sm text-slate-500">"2 Bed • 2 Bath • 1,200 sqft"</p>
                        </div>
                    </div>
                    <div class="border-t border-white/10 pt-4">
                        <div class="flex justify-between text-sm mb-2">
                            <span class="text-slate-400">"Monthly Rent"</span>
                            <span class="text-white font-medium">"$12,000"</span>
                        </div>
                        <div class="flex justify-between text-sm mb-2">
                            <span class="text-slate-400">"Lease Ends"</span>
                            <span class="text-white">"Dec 31, 2024"</span>
                        </div>
                        <div class="flex justify-between text-sm">
                            <span class="text-slate-400">"Landlord"</span>
                            <span class="text-white">"Ahmed Properties LLC"</span>
                        </div>
                    </div>
                </div>
            </div>
            
            // Quick actions
            <div class="bg-white/5 border border-white/10 rounded-2xl p-6">
                <h2 class="text-xl font-bold text-white mb-4">"Quick Actions"</h2>
                <div class="grid grid-cols-2 gap-4">
                    <button class="p-4 bg-indigo-500/20 rounded-xl text-center hover:bg-indigo-500/30 transition-all">
                        <div class="text-3xl mb-2">"💳"</div>
                        <div class="text-sm text-white">"Pay Rent"</div>
                    </button>
                    <button class="p-4 bg-orange-500/20 rounded-xl text-center hover:bg-orange-500/30 transition-all">
                        <div class="text-3xl mb-2">"🔧"</div>
                        <div class="text-sm text-white">"Report Issue"</div>
                    </button>
                    <button class="p-4 bg-green-500/20 rounded-xl text-center hover:bg-green-500/30 transition-all">
                        <div class="text-3xl mb-2">"📞"</div>
                        <div class="text-sm text-white">"Contact"</div>
                    </button>
                    <button class="p-4 bg-purple-500/20 rounded-xl text-center hover:bg-purple-500/30 transition-all">
                        <div class="text-3xl mb-2">"📄"</div>
                        <div class="text-sm text-white">"View Lease"</div>
                    </button>
                </div>
            </div>
            
            // Next payment
            <div class="lg:col-span-2 bg-gradient-to-r from-indigo-500/20 to-purple-500/20 border border-white/10 rounded-2xl p-6">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-slate-400 text-sm">"Next Payment Due"</p>
                        <p class="text-2xl font-bold text-white">"$12,000"</p>
                        <p class="text-slate-400">"Due on January 5, 2024"</p>
                    </div>
                    <button class="px-6 py-3 bg-indigo-500 text-white rounded-lg font-medium hover:bg-indigo-600 transition-all">
                        "Pay Now"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn PaymentsTab() -> impl IntoView {
    view! {
        <div class="bg-white/5 border border-white/10 rounded-2xl p-6">
            <h2 class="text-xl font-bold text-white mb-4">"Payment History"</h2>
            <div class="space-y-3">
                {["Dec 2023", "Nov 2023", "Oct 2023", "Sep 2023"].into_iter().enumerate().map(|(i, month)| {
                    let status = if i == 0 { "Pending" } else { "Paid" };
                    let status_color = if i == 0 { "text-yellow-400" } else { "text-green-400" };
                    view! {
                        <div class="flex items-center justify-between p-4 bg-white/5 rounded-xl">
                            <div>
                                <p class="text-white font-medium">{format!("Rent - {}", month)}</p>
                                <p class="text-sm text-slate-400">"Monthly rent payment"</p>
                            </div>
                            <div class="text-right">
                                <p class="text-white font-medium">"$12,000"</p>
                                <p class=format!("text-sm {}", status_color)>{status}</p>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn RequestsTab() -> impl IntoView {
    let (show_form, set_show_form) = create_signal(false);
    
    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <h2 class="text-xl font-bold text-white">"Maintenance Requests"</h2>
                <button 
                    class="px-4 py-2 bg-indigo-500 text-white rounded-lg font-medium"
                    on:click=move |_| set_show_form.set(true)
                >
                    "+ New Request"
                </button>
            </div>
            
            // Request form modal
            {move || show_form.get().then(|| view! {
                <div class="bg-white/5 border border-white/10 rounded-2xl p-6">
                    <h3 class="text-lg font-bold text-white mb-4">"Submit Request"</h3>
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm text-slate-400 mb-2">"Category"</label>
                            <select class="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white">
                                <option>"Plumbing"</option>
                                <option>"Electrical"</option>
                                <option>"HVAC"</option>
                                <option>"Appliance"</option>
                                <option>"Other"</option>
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm text-slate-400 mb-2">"Description"</label>
                            <textarea class="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white min-h-[100px]" placeholder="Describe the issue..."></textarea>
                        </div>
                        <div class="flex gap-3">
                            <button class="flex-1 py-3 bg-indigo-500 text-white rounded-lg font-medium">
                                "Submit"
                            </button>
                            <button 
                                class="px-6 py-3 bg-white/5 text-slate-400 rounded-lg"
                                on:click=move |_| set_show_form.set(false)
                            >
                                "Cancel"
                            </button>
                        </div>
                    </div>
                </div>
            })}
            
            // Existing requests
            <div class="bg-white/5 border border-white/10 rounded-2xl p-6">
                <div class="space-y-3">
                    <div class="flex items-center justify-between p-4 bg-white/5 rounded-xl">
                        <div>
                            <p class="text-white font-medium">"AC not cooling"</p>
                            <p class="text-sm text-slate-400">"Submitted 2 days ago"</p>
                        </div>
                        <span class="px-3 py-1 bg-yellow-500/20 text-yellow-400 rounded-full text-sm">
                            "In Progress"
                        </span>
                    </div>
                    <div class="flex items-center justify-between p-4 bg-white/5 rounded-xl">
                        <div>
                            <p class="text-white font-medium">"Leaky faucet in bathroom"</p>
                            <p class="text-sm text-slate-400">"Submitted 1 week ago"</p>
                        </div>
                        <span class="px-3 py-1 bg-green-500/20 text-green-400 rounded-full text-sm">
                            "Completed"
                        </span>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn DocumentsTab() -> impl IntoView {
    view! {
        <div class="bg-white/5 border border-white/10 rounded-2xl p-6">
            <h2 class="text-xl font-bold text-white mb-4">"Documents"</h2>
            <div class="space-y-3">
                {["Lease Agreement", "House Rules", "Move-in Checklist", "Emergency Contacts"].into_iter().map(|doc| {
                    view! {
                        <div class="flex items-center justify-between p-4 bg-white/5 rounded-xl hover:bg-white/10 transition-all cursor-pointer">
                            <div class="flex items-center gap-3">
                                <span class="text-2xl">"📄"</span>
                                <div>
                                    <p class="text-white font-medium">{doc}</p>
                                    <p class="text-sm text-slate-400">"PDF • Last updated Dec 2023"</p>
                                </div>
                            </div>
                            <button class="text-indigo-400 hover:text-indigo-300">
                                "Download"
                            </button>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
