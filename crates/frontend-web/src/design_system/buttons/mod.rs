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
        // Base classes + Variant classes
        // Base: inline-flex items-center justify-center rounded-lg font-medium transition-all focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed
        match self {
            ButtonVariant::Primary => "inline-flex items-center justify-center rounded-lg font-medium transition-all focus:outline-none focus:ring-2 focus:ring-offset-1 disabled:opacity-50 disabled:cursor-not-allowed bg-indigo-600 text-white hover:bg-indigo-500 shadow-lg shadow-indigo-500/20 focus:ring-indigo-500",
            ButtonVariant::Secondary => "inline-flex items-center justify-center rounded-lg font-medium transition-all focus:outline-none focus:ring-2 focus:ring-offset-1 disabled:opacity-50 disabled:cursor-not-allowed bg-white/5 text-slate-200 border border-white/10 hover:bg-white/10 focus:ring-slate-500 backdrop-blur-sm",
            ButtonVariant::Ghost => "inline-flex items-center justify-center rounded-lg font-medium transition-all focus:outline-none focus:ring-2 focus:ring-offset-1 disabled:opacity-50 disabled:cursor-not-allowed bg-transparent text-slate-400 hover:text-white hover:bg-white/5 focus:ring-slate-500",
            ButtonVariant::Danger => "inline-flex items-center justify-center rounded-lg font-medium transition-all focus:outline-none focus:ring-2 focus:ring-offset-1 disabled:opacity-50 disabled:cursor-not-allowed bg-red-600 text-white hover:bg-red-500 focus:ring-red-500",
            ButtonVariant::Success => "inline-flex items-center justify-center rounded-lg font-medium transition-all focus:outline-none focus:ring-2 focus:ring-offset-1 disabled:opacity-50 disabled:cursor-not-allowed bg-green-600 text-white hover:bg-green-500 focus:ring-green-500",
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
            ButtonSize::Sm => "h-8 px-3 text-sm gap-1.5",
            ButtonSize::Md => "h-10 px-4 py-2 text-sm gap-2",
            ButtonSize::Lg => "h-12 px-6 text-base gap-2.5",
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
            class=format!("{} h-10 w-10 p-0 shrink-0", variant.class())
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
    let size = ButtonSize::Md; // Default size for links
    
    view! {
        <a
            href=href
            class=format!("{} {}", variant.class(), size.class())
        >
            {icon.map(|i| view! {
                <i class=format!("fa-solid {} mr-2", i)></i>
            })}
            {children()}
        </a>
    }
}
