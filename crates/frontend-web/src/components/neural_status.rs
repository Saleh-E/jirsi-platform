//! Neural Status Widget - Real-time system activity indicator
//!
//! Displays a "Neural Core" status with animated pulses showing:
//! - API activity
//! - Sync status
//! - Connection health
//! - Recent actions counter

use leptos::*;

/// Activity pulse animation states
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActivityLevel {
    Idle,
    Low,
    Medium,
    High,
}

impl ActivityLevel {
    pub fn pulse_class(&self) -> &'static str {
        match self {
            ActivityLevel::Idle => "opacity-30",
            ActivityLevel::Low => "opacity-50 animate-pulse",
            ActivityLevel::Medium => "opacity-70 animate-pulse",
            ActivityLevel::High => "opacity-100 animate-ping",
        }
    }
    
    pub fn color_class(&self) -> &'static str {
        match self {
            ActivityLevel::Idle => "bg-slate-500",
            ActivityLevel::Low => "bg-green-500",
            ActivityLevel::Medium => "bg-amber-500",
            ActivityLevel::High => "bg-indigo-500",
        }
    }
}

/// Neural Status Widget - Shows system activity
#[component]
pub fn NeuralStatus() -> impl IntoView {
    // Simulated activity level (would connect to real API activity in production)
    let (activity, set_activity) = create_signal(ActivityLevel::Low);
    let (actions_count, set_actions_count) = create_signal(0u32);
    let (is_online, _set_online) = create_signal(true);
    
    // Simulate activity changes (demo purposes)
    create_effect(move |_| {
        use gloo_timers::callback::Interval;
        
        let interval = Interval::new(3000, move || {
            set_actions_count.update(|c| *c = (*c + 1) % 100);
            
            // Cycle through activity levels for demo
            set_activity.update(|a| {
                *a = match *a {
                    ActivityLevel::Idle => ActivityLevel::Low,
                    ActivityLevel::Low => ActivityLevel::Medium,
                    ActivityLevel::Medium => ActivityLevel::High,
                    ActivityLevel::High => ActivityLevel::Idle,
                };
            });
        });
        
        // Keep interval alive
        interval.forget();
    });
    
    view! {
        <div class="flex items-center gap-3 px-4 py-3 border-t border-white/10 bg-white/5">
            // Neural Core Icon with Activity Pulse
            <div class="relative">
                <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-lg shadow-indigo-500/30">
                    <span class="text-white text-sm">{"⚡"}</span>
                </div>
                // Activity pulse ring
                <div 
                    class=move || format!(
                        "absolute inset-0 rounded-lg {} {}",
                        activity.get().color_class(),
                        activity.get().pulse_class()
                    )
                ></div>
            </div>
            
            // Status Text
            <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                    <span class="text-xs font-bold text-white uppercase tracking-wider truncate">
                        "Neural Core"
                    </span>
                    // Online indicator dot
                    <span 
                        class=move || format!(
                            "w-1.5 h-1.5 rounded-full {}",
                            if is_online.get() { "bg-green-500 animate-pulse" } else { "bg-red-500" }
                        )
                    ></span>
                </div>
                <div class="text-[10px] text-slate-500 uppercase tracking-widest">
                    {move || if is_online.get() { "Online" } else { "Offline" }}
                </div>
            </div>
            
            // Actions Counter
            <div class="text-right">
                <div class="text-sm font-mono font-bold text-indigo-400">
                    {move || format!("{:02}", actions_count.get())}
                </div>
                <div class="text-[9px] text-slate-600 uppercase">
                    "ops"
                </div>
            </div>
        </div>
    }
}

/// Compact Neural Status for collapsed sidebar
#[component]
pub fn NeuralStatusCompact() -> impl IntoView {
    let (is_online, _set_online) = create_signal(true);
    
    view! {
        <div class="flex justify-center py-3 border-t border-white/10">
            <div class="relative">
                <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-lg shadow-indigo-500/30">
                    <span class="text-white text-sm">{"⚡"}</span>
                </div>
                <span 
                    class=move || format!(
                        "absolute -top-0.5 -right-0.5 w-2.5 h-2.5 rounded-full border-2 border-surface {}",
                        if is_online.get() { "bg-green-500 animate-pulse" } else { "bg-red-500" }
                    )
                ></span>
            </div>
        </div>
    }
}

/// Live Counter Widget for dashboard metrics
#[component]
pub fn LiveCounter(
    #[prop(into)] label: String,
    #[prop(into)] value: Signal<i32>,
    #[prop(into, optional)] icon: Option<String>,
    #[prop(into, optional)] trend: Option<Signal<i32>>,
) -> impl IntoView {
    view! {
        <div class="glass-surface p-4 flex items-center gap-4">
            {icon.map(|i| view! {
                <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-indigo-500/20 to-purple-500/20 flex items-center justify-center text-2xl">
                    {i}
                </div>
            })}
            <div class="flex-1">
                <div class="text-2xl font-bold text-white font-mono">
                    {move || value.get()}
                </div>
                <div class="text-xs text-slate-400 uppercase tracking-wider">
                    {label}
                </div>
            </div>
            {trend.map(|t| view! {
                <div class=move || format!(
                    "flex items-center gap-1 text-sm {}",
                    if t.get() >= 0 { "text-green-400" } else { "text-red-400" }
                )>
                    <span>{move || if t.get() >= 0 { "↑" } else { "↓" }}</span>
                    <span>{move || format!("{}", t.get().abs())}</span>
                </div>
            })}
        </div>
    }
}
