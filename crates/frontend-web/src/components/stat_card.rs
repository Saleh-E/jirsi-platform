//! StatCard Component - Reusable KPI display box with optional progress bars

use leptos::*;

/// Props for trend direction
#[derive(Clone, Copy, PartialEq)]
pub enum TrendDirection {
    Up,
    Down,
    Neutral,
}

/// StatCard component for displaying KPIs with optional targets
#[component]
pub fn StatCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(optional)] trend_value: Option<f64>,
    #[prop(optional)] trend_direction: Option<TrendDirection>,
    #[prop(optional)] target: Option<f64>,
    #[prop(optional)] progress: Option<f64>,
    #[prop(optional, into)] icon: Option<String>,
    #[prop(optional, into)] color: Option<String>,
) -> impl IntoView {
    let color = color.unwrap_or_else(|| "primary".to_string());
    let icon = icon.unwrap_or_else(|| "📊".to_string());
    
    // Determine progress bar color based on percentage
    let get_progress_class = move |p: f64| -> &'static str {
        if p >= 100.0 { "progress--green" }
        else if p >= 75.0 { "progress--blue" }
        else if p >= 50.0 { "progress--orange" }
        else { "progress--yellow" }
    };
    
    // Format target value for display
    let format_target = |t: f64| -> String {
        if t >= 1_000_000.0 {
            format!("${:.1}M", t / 1_000_000.0)
        } else if t >= 1_000.0 {
            format!("${:.0}K", t / 1_000.0)
        } else {
            format!("{:.0}", t)
        }
    };
    
    view! {
        <div class=format!("stat-card stat-card--{}", color)>
            <div class="stat-card__header">
                <span class="stat-card__icon">{icon}</span>
                <span class="stat-card__title">{title}</span>
            </div>
            <div class="stat-card__value">{value}</div>
            
            // Progress bar (if target exists)
            {progress.map(|p| {
                let target_val = target.unwrap_or(0.0);
                let progress_class = get_progress_class(p);
                let bar_width = p.min(100.0);
                let exceeds = p > 100.0;
                
                view! {
                    <div class="stat-card__progress">
                        <div class="progress-bar">
                            <div 
                                class=format!("progress-bar__fill {}", progress_class)
                                style=format!("width: {}%", bar_width)
                            ></div>
                            {exceeds.then(|| view! {
                                <div class="progress-bar__excess"></div>
                            })}
                        </div>
                        <div class="progress-bar__label">
                            <span class=progress_class>
                                {format!("{:.0}%", p)}
                            </span>
                            " of "
                            <span class="progress-bar__target">
                                {format_target(target_val)}
                            </span>
                            " target"
                        </div>
                    </div>
                }
            })}
            
            // Trend indicator
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
                        <span class="trend__period">"vs last period"</span>
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
