//! Holographic Sidebar: The floating navigation pill

use leptos::*;
use leptos_router::*;
use crate::context::theme::ThemeToggle;

pub mod neural_status;
pub use neural_status::NeuralStatus;

#[component]
pub fn HolographicSidebar() -> impl IntoView {
    view! {
        <aside class="main-sidebar w-72 h-full glass-morphism rounded-[2.5rem] flex flex-col p-4 animate-slide-in-left transition-all duration-300">
            // Brand Mark
            <div class="flex items-center gap-3 px-4 py-4 mb-4 border-b border-white/5">
                <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-violet-500 to-fuchsia-500 flex items-center justify-center shadow-[0_0_20px_rgba(139,92,246,0.3)]">
                    <span class="text-white font-bold text-lg">"J"</span>
                </div>
                <div class="flex flex-col">
                    <span class="text-white font-bold text-sm">"Jirsi"</span>
                    <span class="text-zinc-500 text--[10px] uppercase tracking-widest">"Neural OS"</span>
                </div>
            </div>

            // Navigation
            <nav class="flex-1 flex flex-col gap-1 overflow-y-auto custom-scrollbar">
                <NavItem href="/app/dashboard" icon="fa-chart-line" label="Command Center" />
                
                <div class="h-px bg-white/5 my-2 mx-2" />
                
                <h3 class="px-4 py-2 text-[10px] uppercase tracking-widest text-zinc-500 font-bold">"CRM"</h3>
                <NavItem href="/app/crm/entity/contact" icon="fa-users" label="Contacts" />
                <NavItem href="/app/crm/entity/company" icon="fa-briefcase" label="Companies" />
                <NavItem href="/app/crm/entity/deal" icon="fa-handshake" label="Deals" />
                <NavItem href="/app/crm/entity/task" icon="fa-check-circle" label="Tasks" />
                
                <div class="h-px bg-white/5 my-2 mx-2" />
                
                <h3 class="px-4 py-2 text-[10px] uppercase tracking-widest text-zinc-500 font-bold">"Real Estate"</h3>
                <NavItem href="/app/crm/entity/property" icon="fa-building" label="Properties" />
                <NavItem href="/app/crm/entity/listing" icon="fa-bullhorn" label="Listings" />
                <NavItem href="/app/crm/entity/viewing" icon="fa-calendar" label="Viewings" />
                <NavItem href="/app/crm/entity/offer" icon="fa-file-alt" label="Offers" />
                <NavItem href="/app/crm/entity/contract" icon="fa-file-signature" label="Contracts" />
                
                <div class="h-px bg-white/5 my-2 mx-2" />
                
                <h3 class="px-4 py-2 text-[10px] uppercase tracking-widest text-zinc-500 font-bold">"System"</h3>
                <NavItem href="/app/inbox" icon="fa-inbox" label="Inbox" />
                <NavItem href="/app/reports" icon="fa-chart-pie" label="Reports" />
                // Public Listings Link
                <NavItem href="/listings" icon="fa-globe" label="Public Site" />
                
                <div class="h-px bg-white/5 my-2 mx-2" />
                
                <NavItem href="/app/automation" icon="fa-robot" label="Automation" />
                <NavItem href="/app/settings/workflows" icon="fa-network-wired" label="Workflows" />
                <NavItem href="/app/settings" icon="fa-cog" label="Settings" />
            </nav>

            // Neural Status (Heartbeat)
            <NeuralStatus />
            
            // Theme Toggle & Footer
            <div class="flex items-center justify-between px-4 pt-2 -mb-2">
                 <ThemeToggle />
            </div>

            // User Profile
            <div class="pt-4 mt-2 border-t border-white/5">
                <div class="flex items-center gap-3 px-4 py-2 rounded-xl hover:bg-white/5 transition-colors cursor-pointer group">
                    <div class="w-8 h-8 rounded-full bg-gradient-to-br from-zinc-700 to-zinc-800 border border-white/10 group-hover:border-violet-500/50 transition-colors" />
                    <div class="flex flex-col">
                        <span class="text-zinc-200 text-sm font-medium group-hover:text-white transition-colors">"Saleh"</span>
                        <span class="text-zinc-500 text-[10px]">"Admin"</span>
                    </div>
                </div>
            </div>
        </aside>
    }
}

#[component]
fn NavItem(href: &'static str, icon: &'static str, label: &'static str) -> impl IntoView {
    let location = use_location();
    let is_active = move || location.pathname.get().starts_with(href);
    
    view! {
        <a 
            href=href 
            class="group relative flex items-center gap-3 px-4 py-3 rounded-2xl transition-all duration-500 ease-spring"
        >
            // Active State Background
            <Show when=is_active>
                <div class="absolute inset-0 bg-white/10 rounded-2xl light-edge animate-scale-in" />
            </Show>
            
            // Icon
            <i class=move || format!(
                "fa-solid {} relative z-10 text-sm transition-colors duration-300 {}",
                icon,
                if is_active() { "text-violet-400" } else { "text-zinc-500 group-hover:text-zinc-300" }
            ) />
            
            // Label
            <span class=move || format!(
                "relative z-10 text-sm font-medium transition-colors duration-300 {}",
                if is_active() { "text-white" } else { "text-zinc-400 group-hover:text-zinc-200" }
            )>
                {label}
            </span>
            
            // Active Glow Dot
            <Show when=is_active>
                <div class="absolute right-4 w-1.5 h-1.5 rounded-full bg-violet-500 shadow-[0_0_10px_#7C3AED] animate-pulse-glow" />
            </Show>
        </a>
    }
}
