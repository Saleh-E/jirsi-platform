//! Holographic Shell: The main application layout
//! Features floating sidebar and glassmorphism effects.

use leptos::*;
use leptos_router::*;
use crate::widgets::sidebar::HolographicSidebar;
use crate::widgets::command_center::CommandCenter;
use crate::design_system::feedback::toast::ToastContainer;

#[component]
pub fn HolographicShell() -> impl IntoView {
    view! {
        <div class="relative flex h-screen w-screen bg-void p-4 gap-4 overflow-hidden">
            // The Floating Sidebar
            <HolographicSidebar />
            
            // Main Content Area
            <main class="relative flex-1 h-full glass-morphism rounded-[2.5rem] overflow-hidden animate-spring-up">
                <div class="h-full w-full overflow-y-auto p-8 custom-scrollbar">
                    <Outlet />
                </div>
            </main>
            
            // Global Command Center (Cmd+K)
            <CommandCenter />
            
            // Global Toast Container
            <ToastContainer />
        </div>
    }
}
