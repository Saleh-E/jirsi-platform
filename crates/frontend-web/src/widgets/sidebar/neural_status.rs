//! Neural Status: Live Heartbeat Indicator
//! Shows real-time connection state with pulsing animation.

use leptos::*;
use crate::context::network_status::{use_network_status, NetworkStatus};

#[component]
pub fn NeuralStatus() -> impl IntoView {
    // Get real network status from context
    let network_ctx = use_network_status();
    
    // Derive status text and colors from network state
    let status_text = move || {
        if let Some(ctx) = network_ctx {
            match ctx.status.get() {
                NetworkStatus::Online => "ONLINE",
                NetworkStatus::Syncing => "SYNCING",
                NetworkStatus::Offline => "OFFLINE",
            }
        } else {
            "STANDBY"
        }
    };
    
    let is_online = move || {
        network_ctx.map(|ctx| ctx.status.get() == NetworkStatus::Online).unwrap_or(true)
    };
    
    let is_syncing = move || {
        network_ctx.map(|ctx| ctx.status.get() == NetworkStatus::Syncing).unwrap_or(false)
    };
    
    let dot_color = move || {
        if is_syncing() {
            ("bg-amber-500", "bg-amber-400", "#fbbf24")
        } else if is_online() {
            ("bg-emerald-500", "bg-emerald-400", "#34d399")
        } else {
            ("bg-red-500", "bg-red-400", "#f87171")
        }
    };
    
    let text_color = move || {
        if is_syncing() {
            "text-amber-500/80"
        } else if is_online() {
            "text-emerald-500/80"
        } else {
            "text-red-500/80"
        }
    };

    view! {
        <div class="px-4 py-4 mt-auto border-t border-white/5">
            <div class="flex items-center justify-between group cursor-pointer hover:bg-white/5 rounded-xl p-2 -m-2 transition-all duration-300">
                <div class="flex items-center gap-3">
                    // The Heartbeat Dot (Pulsing Animation)
                    <div class="relative flex items-center justify-center w-2 h-2">
                        {move || {
                            let (ping_color, dot_color, shadow) = dot_color();
                            view! {
                                <div class=format!("absolute w-full h-full rounded-full {} animate-ping opacity-75", ping_color)></div>
                                <div 
                                    class=format!("relative w-2 h-2 rounded-full {}", dot_color)
                                    style=format!("box-shadow: 0 0 10px {}", shadow)
                                />
                            }
                        }}
                    </div>
                    
                    <div class="flex flex-col">
                        <span class="text-[10px] font-bold text-zinc-500 tracking-widest group-hover:text-zinc-300 transition-colors">
                            "NEURAL CORE"
                        </span>
                        <span class=move || format!("text-[10px] font-mono {}", text_color())>
                            {status_text}
                        </span>
                    </div>
                </div>

                // Version / Latency
                <div class="flex flex-col items-end">
                    <span class="text-[9px] font-mono text-zinc-600 group-hover:text-emerald-400 transition-colors">
                        "v3.0"
                    </span>
                    <span class="text-[9px] font-mono text-zinc-700">
                        "12ms"
                    </span>
                </div>
            </div>
        </div>
    }
}
