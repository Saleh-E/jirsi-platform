//! Glass Stat Card: Crystal-like dashboard statistics

use leptos::*;

#[component]
pub fn GlassStatCard(
    #[prop(into)]
    label: String,
    #[prop(into)]
    value: String,
    #[prop(optional, into)]
    trend: Option<String>,
    #[prop(optional, into)]
    icon: Option<String>,
    #[prop(optional, into)]
    color: Option<String>,
    #[prop(default = 0)]
    delay_index: u32,
) -> impl IntoView {
    let is_positive = trend.as_ref().map(|t| t.starts_with('+')).unwrap_or(false);
    let animation_delay = format!("animation-delay: {}ms", delay_index * 100);
    let _bg_color = color.unwrap_or_else(|| "violet".to_string()); // Could use for dynamic gradients

    view! {
        <div 
            class="glass-morphism rounded-2xl p-6 relative overflow-hidden group hover:scale-[1.02] transition-transform duration-300 ease-spring animate-spring-up"
            style=animation_delay
        >
            // Decorative gradient
            <div class="absolute -top-10 -right-10 w-24 h-24 bg-white/5 blur-2xl rounded-full pointer-events-none group-hover:bg-white/10 transition-colors duration-500" />
            
            <div class="relative z-10 flex flex-col gap-2">
                // Header
                <div class="flex items-center justify-between">
                    <span class="text-[10px] uppercase tracking-widest text-zinc-500 font-bold">{label}</span>
                    {icon.map(|i| view! {
                        <i class=format!("fa-solid {} text-zinc-600 text-sm", i) />
                    })}
                </div>
                
                // Value
                <div class="flex items-end gap-2">
                    <span class="text-3xl font-bold text-white tracking-tight">{value}</span>
                    {trend.map(|t| view! {
                        <span class=format!(
                            "text-xs font-bold mb-1 {}",
                            if is_positive { "text-emerald-400" } else { "text-red-400" }
                        )>
                            {t}
                        </span>
                    })}
                </div>
            </div>
        </div>
    }
}
