//! StatCard Component - Reusable KPI display box

use leptos::*;

/// Props for trend direction
#[derive(Clone, Copy, PartialEq)]
pub enum TrendDirection {
    Up,
    Down,
    Neutral,
}

/// StatCard component for displaying KPIs
#[component]
pub fn StatCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(optional)] trend_value: Option<f64>,
    #[prop(optional)] trend_direction: Option<TrendDirection>,
    #[prop(optional, into)] icon: Option<String>,
    #[prop(optional, into)] color: Option<String>,
) -> impl IntoView {
    let color = color.unwrap_or_else(|| "primary".to_string());
    let icon = icon.unwrap_or_else(|| "📊".to_string());
    
    view! {
        <div class=format!("stat-card stat-card--{}", color)>
            <div class="stat-card__header">
                <span class="stat-card__icon">{icon}</span>
                <span class="stat-card__title">{title}</span>
            </div>
            <div class="stat-card__value">{value}</div>
            {trend_value.map(|trend| {
                let direction = trend_direction.unwrap_or_else(|| {
                    if trend > 0.0 { TrendDirection::Up }
                    else if trend < 0.0 { TrendDirection::Down }
                    else { TrendDirection::Neutral }
                });
                
                let (arrow, class) = match direction {
                    TrendDirection::Up => ("↑", "trend--up"),
                    TrendDirection::Down => ("↓", "trend--down"),
                    TrendDirection::Neutral => ("→", "trend--neutral"),
                };
                
                view! {
                    <div class=format!("stat-card__trend {}", class)>
                        <span class="trend__arrow">{arrow}</span>
                        <span class="trend__value">{format!("{:.1}%", trend.abs())}</span>
                        <span class="trend__period">"vs last month"</span>
                    </div>
                }
            })}
        </div>
    }
}

/// Mini stat for compact displays
#[component]
pub fn MiniStat(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
) -> impl IntoView {
    view! {
        <div class="mini-stat">
            <span class="mini-stat__value">{value}</span>
            <span class="mini-stat__label">{label}</span>
        </div>
    }
}
