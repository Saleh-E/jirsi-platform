//! ChartContainer - CSS-based chart components (no JS dependencies)

use leptos::*;

/// Bar chart data point
#[derive(Clone, Debug)]
pub struct BarChartItem {
    pub label: String,
    pub value: f64,
    pub color: Option<String>,
}

/// Simple CSS-based horizontal bar chart
#[component]
pub fn BarChart(
    #[prop(into)] title: String,
    items: Vec<BarChartItem>,
) -> impl IntoView {
    let max_value = items.iter().map(|i| i.value).fold(0.0f64, f64::max);
    
    view! {
        <div class="chart-container">
            <h3 class="chart-title">{title}</h3>
            <div class="bar-chart">
                {items.into_iter().map(|item| {
                    let percentage = if max_value > 0.0 { (item.value / max_value) * 100.0 } else { 0.0 };
                    let color = item.color.unwrap_or_else(|| "var(--primary)".to_string());
                    
                    view! {
                        <div class="bar-chart__row">
                            <span class="bar-chart__label">{&item.label}</span>
                            <div class="bar-chart__bar-container">
                                <div 
                                    class="bar-chart__bar"
                                    style=format!("width: {}%; background-color: {}", percentage, color)
                                ></div>
                            </div>
                            <span class="bar-chart__value">{format!("{:.0}", item.value)}</span>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

/// Funnel chart data point
#[derive(Clone, Debug)]
pub struct FunnelItem {
    pub label: String,
    pub value: i64,
    pub color: String,
}

/// CSS-based funnel chart
#[component]
pub fn FunnelChart(
    #[prop(into)] title: String,
    items: Vec<FunnelItem>,
) -> impl IntoView {
    let max_value = items.iter().map(|i| i.value).max().unwrap_or(1) as f64;
    
    view! {
        <div class="chart-container">
            <h3 class="chart-title">{title}</h3>
            <div class="funnel-chart">
                {items.into_iter().enumerate().map(|(idx, item)| {
                    let width_percentage = if max_value > 0.0 { 
                        ((item.value as f64 / max_value) * 80.0) + 20.0 // Min 20%, max 100%
                    } else { 50.0 };
                    
                    view! {
                        <div class="funnel-chart__stage">
                            <div 
                                class="funnel-chart__bar"
                                style=format!("width: {}%; background-color: {}", width_percentage, item.color)
                            >
                                <span class="funnel-chart__label">{&item.label}</span>
                                <span class="funnel-chart__value">{item.value}</span>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

/// Line chart point
#[derive(Clone, Debug)]
pub struct LineChartPoint {
    pub x: String,
    pub y: f64,
}

/// SVG-based line chart 
#[component]
pub fn LineChart(
    #[prop(into)] title: String,
    points: Vec<LineChartPoint>,
    #[prop(optional, into)] color: Option<String>,
) -> impl IntoView {
    let color = color.unwrap_or_else(|| "var(--primary)".to_string());
    let width = 400.0;
    let height = 150.0;
    let padding = 40.0;
    
    let chart_width = width - padding * 2.0;
    let chart_height = height - padding * 2.0;
    
    let max_y = points.iter().map(|p| p.y).fold(0.0f64, f64::max);
    let min_y = 0.0f64; // Start from 0
    let y_range = if max_y > min_y { max_y - min_y } else { 1.0 };
    
    let point_spacing = if points.len() > 1 { chart_width / (points.len() - 1) as f64 } else { 0.0 };
    
    // Generate SVG path
    let path = points.iter().enumerate().map(|(i, p)| {
        let x = padding + (i as f64 * point_spacing);
        let y = padding + chart_height - ((p.y - min_y) / y_range * chart_height);
        if i == 0 { format!("M{},{}", x, y) } else { format!("L{},{}", x, y) }
    }).collect::<Vec<_>>().join(" ");
    
    view! {
        <div class="chart-container">
            <h3 class="chart-title">{title}</h3>
            <div class="line-chart">
                <svg viewBox=format!("0 0 {} {}", width, height) class="line-chart__svg">
                    // Grid lines
                    {(0..5).map(|i| {
                        let y = padding + (i as f64 * chart_height / 4.0);
                        view! {
                            <line 
                                x1=padding 
                                y1=y 
                                x2=width - padding 
                                y2=y 
                                class="line-chart__grid"
                            />
                        }
                    }).collect_view()}
                    
                    // Line path
                    <path 
                        d=path.clone() 
                        fill="none" 
                        stroke=color.clone()
                        stroke-width="2"
                        class="line-chart__line"
                    />
                    
                    // Data points
                    {points.iter().enumerate().map(|(i, p)| {
                        let x = padding + (i as f64 * point_spacing);
                        let y = padding + chart_height - ((p.y - min_y) / y_range * chart_height);
                        view! {
                            <circle 
                                cx=x 
                                cy=y 
                                r="4" 
                                fill=color.clone()
                                class="line-chart__point"
                            />
                        }
                    }).collect_view()}
                </svg>
                
                // X-axis labels
                <div class="line-chart__x-labels">
                    {points.iter().enumerate().filter(|(i, _)| {
                        // Show every Nth label to avoid crowding
                        let step = if points.len() > 7 { points.len() / 7 } else { 1 };
                        i % step == 0
                    }).map(|(_, p)| {
                        view! { <span class="line-chart__x-label">{&p.x}</span> }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
