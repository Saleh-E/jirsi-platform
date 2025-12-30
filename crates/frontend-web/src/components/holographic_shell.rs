use leptos::*;

/// Holographic background and ambient effects
#[component]
fn AmbientBackground() -> impl IntoView {
    view! {
        <div class="fixed inset-0 z-0 pointer-events-none overflow-hidden bg-void">
            // Grid Pattern Overlay
            <div class="absolute inset-0 bg-grid-white/[0.02] bg-[length:32px_32px]"></div>
            
            // Primary Glow Orb (Top Right)
            <div class="absolute -top-[20%] -right-[10%] w-[800px] h-[800px] rounded-full bg-indigo-600/20 blur-[120px] animate-pulse-glow"></div>
            
            // Secondary Glow Orb (Bottom Left)
            <div class="absolute -bottom-[20%] -left-[10%] w-[600px] h-[600px] rounded-full bg-purple-600/10 blur-[100px] animate-float"></div>
            
            // Subtle Noise Texture (Optional, adds grit)
            <div class="absolute inset-0 opacity-[0.03] mix-blend-overlay" style="background-image: url('/assets/noise.png')"></div>
        </div>
    }
}

/// Holographic Shell Wrapper
/// Wraps the entire application to provide the premium "glass" feel and ambient background.
#[component]
pub fn HolographicShell(children: Children) -> impl IntoView {
    view! {
        <div class="relative min-h-screen font-sans text-slate-200 selection:bg-indigo-500/30">
            <AmbientBackground />
            
            // Main Content Layer (Z-10 to sit above background)
            <div class="relative z-10 flex flex-col min-h-screen">
                {children()}
            </div>
        </div>
    }
}
