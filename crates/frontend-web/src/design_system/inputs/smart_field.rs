//! SmartField: The "Morphing" Input Component
//! Appears as static label, transforms to input on click.

use leptos::*;
use wasm_bindgen::JsCast;


#[component]
pub fn SmartField(
    /// The reactive value to bind to
    value: RwSignal<String>,
    /// Display label above the field
    #[prop(optional, into)]
    label: String,
    /// Callback when value is saved (on blur)
    #[prop(optional)]
    on_save: Option<Callback<String>>,
) -> impl IntoView {
    let (is_editing, set_editing) = create_signal(false);
    let (show_saved, set_show_saved) = create_signal(false);
    let original_value = create_rw_signal(value.get_untracked());

    let label_store: StoredValue<String> = store_value(label);

    let handle_blur = move |_| {
        set_editing.set(false);
        if let Some(callback) = on_save {
            callback.call(value.get());
            set_show_saved.set(true);
            set_timeout(
                move || set_show_saved.set(false),
                std::time::Duration::from_millis(2000),
            );
        }
    };

    view! {
        <div 
            class="group relative flex flex-col gap-1 min-h-[40px] px-3 py-2 rounded-xl transition-all duration-300 hover:bg-white/5 cursor-text focus-within:bg-white/5"
            on:click=move |_| {
                original_value.set(value.get());
                set_editing.set(true);
            }
        >
            <Show when=move || label_store.with_value(|l| !l.is_empty())>
                <span class="text-[10px] uppercase tracking-widest text-zinc-500 font-bold group-hover:text-zinc-300 transition-colors">
                    {label_store.get_value()}
                </span>
            </Show>

            <Show 
                when=move || is_editing.get()
                fallback=move || view! { 
                    <span class="text-sm font-medium text-zinc-100 truncate">
                        {move || {
                            let v = value.get();
                            if v.is_empty() { "—".to_string() } else { v }
                        }}
                    </span> 
                }
            >
                <input 
                    type="text"
                    autofocus=true
                    class="bg-transparent border-none outline-none ring-0 text-sm font-medium text-white w-full animate-spring-up"
                    prop:value=value
                    on:input=move |ev| value.set(event_target_value(&ev))
                    on:blur=handle_blur
                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                        if ev.key() == "Enter" {
                            ev.prevent_default();
                            if let Some(target) = ev.target() {
                                if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                                    let _ = input.blur();
                                }
                            }
                        } else if ev.key() == "Escape" {
                            ev.prevent_default();
                            value.set(original_value.get());
                            set_editing.set(false);
                        }
                    }
                />
            </Show>

            // The "Sync" indicator: A tiny green pulse when data saves
            <Show when=move || show_saved.get()>
                <div class="absolute right-2 bottom-2 w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse shadow-[0_0_8px_var(--success-glow)]" />
            </Show>
        </div>
    }
}
