//! Dialer Widget - Floating Click-to-Dial Component
//!
//! A floating dialer widget that enables:
//! - One-click calling from any page
//! - Call status display
//! - Quick access to recent calls
//! - Integration with contact records

use leptos::*;
use uuid::Uuid;

/// Dialer state
#[derive(Clone, Copy, PartialEq)]
pub enum DialerState {
    Idle,
    Dialing,
    Ringing,
    Connected,
    Ended,
    Error,
}

impl DialerState {
    fn icon(&self) -> &'static str {
        match self {
            Self::Idle => "📞",
            Self::Dialing => "⏳",
            Self::Ringing => "🔔",
            Self::Connected => "🟢",
            Self::Ended => "📵",
            Self::Error => "❌",
        }
    }
    
    fn label(&self) -> &'static str {
        match self {
            Self::Idle => "Ready",
            Self::Dialing => "Dialing...",
            Self::Ringing => "Ringing...",
            Self::Connected => "Connected",
            Self::Ended => "Call Ended",
            Self::Error => "Error",
        }
    }
}

/// Recent call entry
#[derive(Clone)]
pub struct RecentCall {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub duration: Option<u32>,
    pub timestamp: String,
    pub entity_id: Option<Uuid>,
}

/// Dialer Widget Component
#[component]
pub fn DialerWidget() -> impl IntoView {
    let (is_expanded, set_expanded) = create_signal(false);
    let (state, set_state) = create_signal(DialerState::Idle);
    let (phone_input, set_phone_input) = create_signal(String::new());
    let (current_call_sid, set_current_call_sid) = create_signal(Option::<String>::None);
    let (call_duration, set_call_duration) = create_signal(0u32);
    let (error_message, set_error) = create_signal(Option::<String>::None);
    let (recording_enabled, set_recording) = create_signal(true);
    
    let (recent_calls, set_recent_calls) = create_signal(vec![
        RecentCall {
            id: "1".to_string(),
            name: "John Doe".to_string(),
            phone: "+1 555 123 4567".to_string(),
            duration: Some(245),
            timestamp: "2 min ago".to_string(),
            entity_id: None,
        },
        RecentCall {
            id: "2".to_string(),
            name: "Property Inquiry".to_string(),
            phone: "+971 50 123 4567".to_string(),
            duration: Some(180),
            timestamp: "15 min ago".to_string(),
            entity_id: None,
        },
    ]);
    
    // Duration timer
    create_effect(move |_| {
        if state.get() == DialerState::Connected {
            let handle = set_interval_with_handle(
                move || set_call_duration.update(|d| *d += 1),
                std::time::Duration::from_secs(1),
            );
            
            on_cleanup(move || {
                if let Ok(h) = handle {
                    h.clear();
                }
            });
        }
    });
    
    let format_duration = move || {
        let secs = call_duration.get();
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{:02}:{:02}", mins, secs)
    };
    
    let initiate_call = move |phone: String| {
        if phone.is_empty() {
            set_error.set(Some("Please enter a phone number".to_string()));
            return;
        }
        
        set_state.set(DialerState::Dialing);
        set_error.set(None);
        set_call_duration.set(0);
        
        spawn_local(async move {
            let result = call_api(&phone, recording_enabled.get_untracked()).await;
            
            match result {
                Ok(call_sid) => {
                    set_current_call_sid.set(Some(call_sid));
                    // Simulate state progression (in production, use webhooks)
                    gloo_timers::future::TimeoutFuture::new(1500).await;
                    set_state.set(DialerState::Ringing);
                    gloo_timers::future::TimeoutFuture::new(2000).await;
                    set_state.set(DialerState::Connected);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_state.set(DialerState::Error);
                }
            }
        });
    };
    
    let end_call = move |_| {
        if let Some(call_sid) = current_call_sid.get() {
            spawn_local(async move {
                let _ = end_call_api(&call_sid).await;
            });
        }
        
        set_state.set(DialerState::Ended);
        set_current_call_sid.set(None);
        
        // Auto-reset after 3 seconds
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(3000).await;
            set_state.set(DialerState::Idle);
        });
    };
    
    let on_keypress = move |e: web_sys::KeyboardEvent| {
        if e.key() == "Enter" {
            initiate_call(phone_input.get());
        }
    };
    
    view! {
        // Floating widget container
        <div class="fixed bottom-6 right-6 z-50">
            // Collapsed state - just the button
            {move || if !is_expanded.get() {
                view! {
                    <button
                        class="w-14 h-14 bg-gradient-to-r from-green-500 to-emerald-500 rounded-full shadow-lg shadow-green-500/30 flex items-center justify-center text-white text-2xl hover:scale-110 transition-all duration-300"
                        on:click=move |_| set_expanded.set(true)
                    >
                        "📞"
                    </button>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
            
            // Expanded state - full dialer
            {move || if is_expanded.get() {
                view! {
                    <div class="w-80 bg-slate-900/95 backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl overflow-hidden">
                        // Header
                        <div class="bg-gradient-to-r from-green-500/20 to-emerald-500/20 px-4 py-3 flex items-center justify-between border-b border-white/10">
                            <div class="flex items-center gap-2">
                                <span class="text-xl">{move || state.get().icon()}</span>
                                <span class="text-white font-medium">{move || state.get().label()}</span>
                            </div>
                            <div class="flex items-center gap-2">
                                {move || if state.get() == DialerState::Connected {
                                    view! {
                                        <span class="text-green-400 font-mono text-sm">{format_duration}</span>
                                    }.into_view()
                                } else {
                                    view! {}.into_view()
                                }}
                                <button
                                    class="text-slate-400 hover:text-white transition-colors"
                                    on:click=move |_| set_expanded.set(false)
                                >
                                    "✕"
                                </button>
                            </div>
                        </div>
                        
                        // Dialer body
                        <div class="p-4">
                            // Error display
                            {move || error_message.get().map(|e| view! {
                                <div class="bg-red-500/10 border border-red-500/30 text-red-400 text-sm px-3 py-2 rounded-lg mb-3">
                                    {e}
                                </div>
                            })}
                            
                            // Phone input (only when idle)
                            {move || if matches!(state.get(), DialerState::Idle | DialerState::Ended | DialerState::Error) {
                                view! {
                                    <div class="mb-4">
                                        <input
                                            type="tel"
                                            class="w-full bg-white/5 border border-white/10 rounded-lg px-4 py-3 text-white text-lg font-mono placeholder-slate-500 focus:border-green-500 focus:outline-none"
                                            placeholder="+1 555 123 4567"
                                            prop:value=phone_input
                                            on:input=move |e| set_phone_input.set(event_target_value(&e))
                                            on:keypress=on_keypress
                                        />
                                    </div>
                                    
                                    // Recording toggle
                                    <div class="flex items-center justify-between mb-4 text-sm">
                                        <span class="text-slate-400">"Record call"</span>
                                        <button
                                            class="relative w-12 h-6 rounded-full transition-colors"
                                            class:bg-green-500=recording_enabled
                                            class:bg-slate-600=move || !recording_enabled.get()
                                            on:click=move |_| set_recording.update(|v| *v = !*v)
                                        >
                                            <div 
                                                class="absolute top-1 w-4 h-4 bg-white rounded-full transition-all"
                                                class:left-1=move || !recording_enabled.get()
                                                class:left-7=recording_enabled
                                            />
                                        </button>
                                    </div>
                                    
                                    // Call button
                                    <button
                                        class="w-full py-3 bg-gradient-to-r from-green-500 to-emerald-500 text-white rounded-lg font-medium hover:shadow-lg hover:shadow-green-500/25 transition-all"
                                        on:click=move |_| initiate_call(phone_input.get())
                                    >
                                        "📞 Call"
                                    </button>
                                }.into_view()
                            } else if matches!(state.get(), DialerState::Dialing | DialerState::Ringing | DialerState::Connected) {
                                view! {
                                    // Active call UI
                                    <div class="text-center py-4">
                                        <div class="text-2xl font-mono text-white mb-2">
                                            {phone_input}
                                        </div>
                                        {move || if state.get() == DialerState::Connected {
                                            view! {
                                                <div class="text-4xl font-mono text-green-400 mb-4">
                                                    {format_duration}
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <div class="flex items-center justify-center gap-1 mb-4">
                                                    <span class="w-2 h-2 bg-green-400 rounded-full animate-bounce" style="animation-delay: 0ms" />
                                                    <span class="w-2 h-2 bg-green-400 rounded-full animate-bounce" style="animation-delay: 150ms" />
                                                    <span class="w-2 h-2 bg-green-400 rounded-full animate-bounce" style="animation-delay: 300ms" />
                                                </div>
                                            }.into_view()
                                        }}
                                        
                                        // End call button
                                        <button
                                            class="w-16 h-16 bg-red-500 rounded-full text-white text-2xl hover:bg-red-600 transition-colors mx-auto"
                                            on:click=end_call
                                        >
                                            "📵"
                                        </button>
                                    </div>
                                }.into_view()
                            } else {
                                view! {}.into_view()
                            }}
                        </div>
                        
                        // Recent calls (only when idle)
                        {move || if matches!(state.get(), DialerState::Idle | DialerState::Ended | DialerState::Error) {
                            view! {
                                <div class="border-t border-white/10 max-h-48 overflow-y-auto">
                                    <div class="px-4 py-2 text-xs text-slate-500 uppercase tracking-wide">"Recent"</div>
                                    {move || recent_calls.get().into_iter().map(|call| {
                                        let phone = call.phone.clone();
                                        view! {
                                            <button
                                                class="w-full px-4 py-2 flex items-center gap-3 hover:bg-white/5 transition-colors"
                                                on:click=move |_| {
                                                    set_phone_input.set(phone.clone());
                                                    initiate_call(phone.clone());
                                                }
                                            >
                                                <div class="w-8 h-8 bg-slate-700 rounded-full flex items-center justify-center text-sm">
                                                    "👤"
                                                </div>
                                                <div class="flex-1 text-left">
                                                    <div class="text-white text-sm">{call.name}</div>
                                                    <div class="text-slate-500 text-xs">{call.phone}</div>
                                                </div>
                                                <div class="text-slate-500 text-xs">{call.timestamp}</div>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }}
                    </div>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}

/// Call the API to initiate a call
async fn call_api(phone: &str, record: bool) -> Result<String, String> {
    use gloo_net::http::Request;
    
    let body = serde_json::json!({
        "to": phone,
        "record": record
    });
    
    let response = Request::post("/api/v1/voice/call")
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !response.ok() {
        return Err("Failed to initiate call".to_string());
    }
    
    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    
    Ok(json["call_sid"].as_str().unwrap_or("").to_string())
}

/// Call the API to end a call
async fn end_call_api(call_sid: &str) -> Result<(), String> {
    use gloo_net::http::Request;
    
    Request::post(&format!("/api/v1/voice/call/{}/end", call_sid))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Click-to-dial hook - can be used from entity detail pages
pub fn use_click_to_dial() -> impl Fn(String) {
    move |phone: String| {
        // Dispatch event to open dialer with pre-filled number
        if let Some(window) = web_sys::window() {
            let event = web_sys::CustomEvent::new_with_event_init_dict(
                "jirsi:dial",
                web_sys::CustomEventInit::new()
                    .detail(&wasm_bindgen::JsValue::from_str(&phone)),
            ).ok();
            
            if let Some(e) = event {
                let _ = window.dispatch_event(&e);
            }
        }
    }
}

