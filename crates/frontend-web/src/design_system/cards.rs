//! Card components for content containers
//!
//! Cards use semantic tokens for consistent theming.

use leptos::*;

/// Card component - base container
#[component]
pub fn Card(
    #[prop(into, optional)] class: Option<String>,
    #[prop(into, optional)] padding: Option<bool>,
    children: Children,
) -> impl IntoView {
    let padding_class = if padding.unwrap_or(true) { "p-4" } else { "" };
    let extra = class.unwrap_or_default();
    
    view! {
        <div class=format!("glass-surface {} {}", padding_class, extra)>
            {children()}
        </div>
    }
}

/// Card with header
#[component]
pub fn CardWithHeader(
    #[prop(into)] title: String,
    #[prop(into, optional)] subtitle: Option<String>,
    #[prop(into, optional)] actions: Option<Children>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="glass-surface">
            <div class="flex items-center justify-between p-4 border-b border-white/10">
                <div>
                    <h3 class="text-lg font-semibold text-white">{title}</h3>
                    {subtitle.map(|s| view! {
                        <p class="text-sm text-slate-400">{s}</p>
                    })}
                </div>
                {actions.map(|a| view! {
                    <div class="flex items-center gap-2">
                        {a()}
                    </div>
                })}
            </div>
            <div class="p-4">
                {children()}
            </div>
        </div>
    }
}

/// Stat card for dashboard metrics
#[component]
pub fn StatCard(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(into, optional)] icon: Option<String>,
    #[prop(into, optional)] change: Option<f64>,
    #[prop(into, optional)] change_label: Option<String>,
) -> impl IntoView {
    let change_color = match change {
        Some(c) if c > 0.0 => "text-green-500",
        Some(c) if c < 0.0 => "text-red-500",
        _ => "text-slate-500",
    };
    
    let change_icon = match change {
        Some(c) if c > 0.0 => "fa-arrow-up",
        Some(c) if c < 0.0 => "fa-arrow-down",
        _ => "fa-minus",
    };
    
    view! {
        <div class="glass-surface p-4">
            <div class="flex justify-between items-start">
                <div>
                    <p class="text-slate-400 text-sm">{label}</p>
                    <p class="text-white text-2xl font-bold mt-1">{value}</p>
                    {change.map(|c| view! {
                        <div class=format!("flex items-center gap-1 mt-2 text-sm {}", change_color)>
                            <i class=format!("fa-solid {}", change_icon)></i>
                            <span>{format!("{:.1}%", c.abs())}</span>
                            {change_label.clone().map(|l| view! {
                                <span class="text-slate-500 ml-1">{l}</span>
                            })}
                        </div>
                    })}
                </div>
                {icon.map(|i| view! {
                    <div class="p-3 rounded-lg bg-indigo-500/10 text-indigo-400">
                        <i class=format!("fa-solid {}", i)></i>
                    </div>
                })}
            </div>
        </div>
    }
}

/// Empty state card
#[component]
pub fn EmptyState(
    #[prop(into)] title: String,
    #[prop(into, optional)] description: Option<String>,
    #[prop(into, optional)] icon: Option<String>,
    #[prop(optional)] action: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center p-8 text-center rounded-xl border border-dashed border-slate-700 bg-white/5">
            {icon.map(|i| view! {
                <div class="text-4xl text-slate-500 mb-4">
                    <i class=format!("fa-solid {}", i)></i>
                </div>
            })}
            <h3 class="text-lg font-medium text-white mb-2">{title}</h3>
            {description.map(|d| view! {
                <p class="text-sm text-slate-400 max-w-sm">{d}</p>
            })}
            {action.map(|a| view! {
                <div class="mt-4">
                    {a()}
                </div>
            })}
        </div>
    }
}
