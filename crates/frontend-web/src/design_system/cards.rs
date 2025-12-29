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
        <div class=format!("ui-card {} {}", padding_class, extra)>
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
        <div class="ui-card">
            <div class="ui-card-header">
                <div>
                    <h3 class="ui-card-title">{title}</h3>
                    {subtitle.map(|s| view! {
                        <p class="ui-card-subtitle">{s}</p>
                    })}
                </div>
                {actions.map(|a| view! {
                    <div class="ui-card-actions">
                        {a()}
                    </div>
                })}
            </div>
            <div class="ui-card-body">
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
        Some(c) if c > 0.0 => "text-success",
        Some(c) if c < 0.0 => "text-error",
        _ => "text-muted",
    };
    
    let change_icon = match change {
        Some(c) if c > 0.0 => "fa-arrow-up",
        Some(c) if c < 0.0 => "fa-arrow-down",
        _ => "fa-minus",
    };
    
    view! {
        <div class="ui-card ui-stat-card">
            <div class="flex justify-between items-start">
                <div>
                    <p class="text-secondary text-sm">{label}</p>
                    <p class="text-primary text-2xl font-bold mt-1">{value}</p>
                    {change.map(|c| view! {
                        <div class=format!("flex items-center gap-1 mt-2 text-sm {}", change_color)>
                            <i class=format!("fa-solid {}", change_icon)></i>
                            <span>{format!("{:.1}%", c.abs())}</span>
                            {change_label.clone().map(|l| view! {
                                <span class="text-muted">{l}</span>
                            })}
                        </div>
                    })}
                </div>
                {icon.map(|i| view! {
                    <div class="ui-stat-icon">
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
        <div class="ui-empty-state">
            {icon.map(|i| view! {
                <div class="ui-empty-state-icon">
                    <i class=format!("fa-solid {}", i)></i>
                </div>
            })}
            <h3 class="ui-empty-state-title">{title}</h3>
            {description.map(|d| view! {
                <p class="ui-empty-state-description">{d}</p>
            })}
            {action.map(|a| view! {
                <div class="mt-4">
                    {a()}
                </div>
            })}
        </div>
    }
}
