//! Glass Toasts: Physics-based notification stack

use leptos::*;
use std::collections::VecDeque;

/// Toast types for different feedback scenarios
#[derive(Clone, Copy, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Info,
    Warning,
}

/// Individual toast message
#[derive(Clone)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub toast_type: ToastType,
}

/// Toast context for global access
#[derive(Clone, Copy)]
pub struct ToastContext {
    pub show_toast: Callback<(String, ToastType)>,
}

/// Provide toast context at app root
pub fn provide_toast_context() {
    let (toasts, set_toasts) = create_signal(VecDeque::<Toast>::new());
    let next_id = create_rw_signal(0u64);

    let show_toast = Callback::new(move |(message, toast_type): (String, ToastType)| {
        let id = next_id.get();
        next_id.set(id + 1);
        
        set_toasts.update(|t| {
            t.push_back(Toast { id, message, toast_type });
            while t.len() > 5 {
                t.pop_front();
            }
        });
        
        // Auto-dismiss after 3 seconds
        set_timeout(move || {
            set_toasts.update(|t| {
                t.retain(|toast| toast.id != id);
            });
        }, std::time::Duration::from_secs(3));
    });

    provide_context(ToastContext { show_toast });
    provide_context(toasts);
}

/// Hook to use toasts
pub fn use_toast() -> ToastContext {
    use_context::<ToastContext>().expect("ToastContext not provided. Call provide_toast_context() in App.")
}

/// Toast container component - place in app root
#[component]
pub fn ToastContainer() -> impl IntoView {
    let toasts = use_context::<ReadSignal<VecDeque<Toast>>>()
        .expect("Toast signal not provided");

    view! {
        <div class="fixed bottom-4 right-4 z-[200] flex flex-col gap-2">
            {move || {
                toasts.get().iter().cloned().collect::<Vec<_>>().into_iter().map(|toast| {
                    view! { <GlassToast toast=toast /> }
                }).collect_view()
            }}
        </div>
    }
}

/// Individual glass toast component
#[component]
fn GlassToast(toast: Toast) -> impl IntoView {
    let (color_class, icon) = match toast.toast_type {
        ToastType::Success => ("border-emerald-500/50 shadow-emerald-900/30", "fa-check"),
        ToastType::Error => ("border-red-500/50 shadow-red-900/30", "fa-xmark"),
        ToastType::Warning => ("border-amber-500/50 shadow-amber-900/30", "fa-triangle-exclamation"),
        ToastType::Info => ("border-blue-500/50 shadow-blue-900/30", "fa-info"),
    };
    
    let dot_color = match toast.toast_type {
        ToastType::Success => "bg-emerald-500",
        ToastType::Error => "bg-red-500",
        ToastType::Warning => "bg-amber-500",
        ToastType::Info => "bg-blue-500",
    };

    view! {
        <div class=format!(
            "flex items-center gap-3 px-4 py-3 rounded-xl glass-morphism border {} shadow-lg animate-spring-up min-w-[280px]",
            color_class
        )>
            <div class=format!("w-2 h-2 rounded-full {} animate-pulse", dot_color) />
            <span class="text-sm font-medium text-white flex-1">{toast.message}</span>
            <i class=format!("fa-solid {} text-zinc-500 text-xs", icon) />
        </div>
    }
}
