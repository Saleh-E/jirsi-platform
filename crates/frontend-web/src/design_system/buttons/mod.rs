//! Button components using semantic design tokens
//!
//! All buttons use CSS classes defined in design_tokens.css
//! for automatic light/dark theme adaptation.

use leptos::*;

/// Button variant determines styling
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    Danger,
    Success,
}

impl ButtonVariant {
    pub fn class(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "ui-btn ui-btn-primary",
            ButtonVariant::Secondary => "ui-btn ui-btn-secondary",
            ButtonVariant::Ghost => "ui-btn ui-btn-ghost",
            ButtonVariant::Danger => "ui-btn ui-btn-danger",
            ButtonVariant::Success => "ui-btn ui-btn-success",
        }
    }
}

/// Button size
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl ButtonSize {
    pub fn class(&self) -> &'static str {
        match self {
            ButtonSize::Sm => "ui-btn-sm",
            ButtonSize::Md => "ui-btn-md",
            ButtonSize::Lg => "ui-btn-lg",
        }
    }
}

/// Primary button component with semantic tokens
#[component]
pub fn Button(
    #[prop(into, optional)] variant: Option<ButtonVariant>,
    #[prop(into, optional)] size: Option<ButtonSize>,
    #[prop(into, optional)] icon: Option<String>,
    #[prop(into, optional)] loading: Option<Signal<bool>>,
    #[prop(into, optional)] disabled: Option<Signal<bool>>,
    #[prop(into, optional)] class: Option<String>,
    #[prop(optional)] on_click: Option<Callback<ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or_default();
    let size = size.unwrap_or_default();
    let extra_class = class.unwrap_or_default();
    
    let is_loading = loading.map(|s| s.get()).unwrap_or(false);
    let is_disabled = disabled.map(|s| s.get()).unwrap_or(false);
    
    let classes = format!(
        "{} {} {}",
        variant.class(),
        size.class(),
        extra_class
    );

    view! {
        <button
            class=classes
            disabled=is_disabled || is_loading
            on:click=move |ev| {
                if let Some(handler) = on_click {
                    handler.call(ev);
                }
            }
        >
            {move || {
                if is_loading {
                    view! {
                        <i class="fa-solid fa-spinner fa-spin mr-2"></i>
                    }.into_view()
                } else if let Some(ref icon_class) = icon {
                    view! {
                        <i class=format!("fa-solid {} mr-2", icon_class)></i>
                    }.into_view()
                } else {
                    ().into_view()
                }
            }}
            {children()}
        </button>
    }
}

/// Icon-only button
#[component]
pub fn IconButton(
    #[prop(into)] icon: String,
    #[prop(into, optional)] variant: Option<ButtonVariant>,
    #[prop(into, optional)] tooltip: Option<String>,
    #[prop(optional)] on_click: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView {
    let variant = variant.unwrap_or(ButtonVariant::Ghost);
    
    view! {
        <button
            class=format!("{} ui-btn-icon", variant.class())
            title=tooltip.unwrap_or_default()
            on:click=move |ev| {
                if let Some(handler) = on_click {
                    handler.call(ev);
                }
            }
        >
            <i class=format!("fa-solid {}", icon)></i>
        </button>
    }
}

/// Link styled as button
#[component]
pub fn LinkButton(
    #[prop(into)] href: String,
    #[prop(into, optional)] variant: Option<ButtonVariant>,
    #[prop(into, optional)] icon: Option<String>,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or(ButtonVariant::Primary);
    
    view! {
        <a
            href=href
            class=variant.class()
        >
            {icon.map(|i| view! {
                <i class=format!("fa-solid {} mr-2", i)></i>
            })}
            {children()}
        </a>
    }
}
