//! Badge components for status indicators
//!
//! Badges use semantic status colors from design_tokens.css
//! for consistent theming across light/dark modes.

use leptos::*;

/// Badge color variants
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BadgeColor {
    #[default]
    Neutral,
    Success,
    Warning,
    Error,
    Info,
    Blue,
    Green,
    Amber,
    Red,
    Purple,
    Cyan,
}

impl BadgeColor {
    pub fn class(&self) -> &'static str {
        match self {
            BadgeColor::Neutral => "ui-badge-neutral",
            BadgeColor::Success => "ui-badge-success",
            BadgeColor::Warning => "ui-badge-warning",
            BadgeColor::Error => "ui-badge-error",
            BadgeColor::Info => "ui-badge-info",
            BadgeColor::Blue => "ui-badge-info",
            BadgeColor::Green => "ui-badge-success",
            BadgeColor::Amber => "ui-badge-warning",
            BadgeColor::Red => "ui-badge-error",
            BadgeColor::Purple => "ui-badge-purple",
            BadgeColor::Cyan => "ui-badge-cyan",
        }
    }
    
    /// Convert color string from entity registry to BadgeColor
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "success" | "green" => BadgeColor::Green,
            "warning" | "amber" => BadgeColor::Amber,
            "error" | "red" | "danger" => BadgeColor::Red,
            "info" | "blue" => BadgeColor::Blue,
            "purple" | "violet" => BadgeColor::Purple,
            "cyan" | "teal" => BadgeColor::Cyan,
            _ => BadgeColor::Neutral,
        }
    }
}

/// Badge component for status indicators
#[component]
pub fn Badge(
    #[prop(into)] label: String,
    #[prop(into, optional)] color: Option<BadgeColor>,
    #[prop(into, optional)] icon: Option<String>,
) -> impl IntoView {
    let color = color.unwrap_or_default();
    
    view! {
        <span class=format!("ui-badge {}", color.class())>
            {icon.map(|i| view! {
                <i class=format!("fa-solid {} mr-1 text-xs", i)></i>
            })}
            {label}
        </span>
    }
}

/// Status badge with dot indicator
#[component]
pub fn StatusBadge(
    #[prop(into)] status: String,
    #[prop(into, optional)] color: Option<String>,
) -> impl IntoView {
    let badge_color = color
        .as_ref()
        .map(|c| BadgeColor::from_str(c))
        .unwrap_or(BadgeColor::Neutral);
    
    // Format status for display (e.g., "in_progress" -> "In Progress")
    let display_status = status
        .replace("_", " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    
    view! {
        <span class=format!("ui-badge {}", badge_color.class())>
            <span class="ui-badge-dot"></span>
            {display_status}
        </span>
    }
}

/// Priority badge with color coding
#[component]
pub fn PriorityBadge(
    #[prop(into)] priority: String,
) -> impl IntoView {
    let (color, icon) = match priority.to_lowercase().as_str() {
        "high" | "urgent" => (BadgeColor::Red, "fa-arrow-up"),
        "medium" => (BadgeColor::Amber, "fa-minus"),
        "low" => (BadgeColor::Neutral, "fa-arrow-down"),
        _ => (BadgeColor::Neutral, "fa-circle"),
    };
    
    let display = priority
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default() + &priority[1..];
    
    view! {
        <span class=format!("ui-badge {}", color.class())>
            <i class=format!("fa-solid {} mr-1 text-xs", icon)></i>
            {display}
        </span>
    }
}
