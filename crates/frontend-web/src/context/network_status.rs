//! Network Status Context
//!
//! Global signal for tracking network/sync status across the app.
//! Used by fetch_engine to communicate retry state to UI components.

use leptos::*;

/// Network status states
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NetworkStatus {
    /// All requests successful, network healthy
    Online,
    /// Currently retrying a failed request
    Syncing,
    /// All retries failed, network unavailable
    Offline,
}

/// Global network status context
#[derive(Clone, Copy)]
pub struct NetworkStatusContext {
    pub status: RwSignal<NetworkStatus>,
    pub retry_count: RwSignal<u32>,
}

impl NetworkStatusContext {
    /// Set status to syncing with retry count
    pub fn set_syncing(&self, attempt: u32) {
        self.status.set(NetworkStatus::Syncing);
        self.retry_count.set(attempt);
    }
    
    /// Set status to online (success)
    pub fn set_online(&self) {
        self.status.set(NetworkStatus::Online);
        self.retry_count.set(0);
    }
    
    /// Set status to offline (all retries failed)
    pub fn set_offline(&self) {
        self.status.set(NetworkStatus::Offline);
    }
}

/// Provide network status context at app root
pub fn provide_network_status() {
    let status = create_rw_signal(NetworkStatus::Online);
    let retry_count = create_rw_signal(0u32);
    provide_context(NetworkStatusContext { status, retry_count });
}

/// Get network status context
pub fn use_network_status() -> Option<NetworkStatusContext> {
    use_context::<NetworkStatusContext>()
}

/// Simple status badge component that shows network status
#[component]
pub fn NetworkStatusBadge() -> impl IntoView {
    let ctx = use_network_status();
    
    view! {
        {move || {
            let Some(ctx) = ctx else {
                return view! { <span></span> }.into_view();
            };
            
            let status = ctx.status.get();
            let retry = ctx.retry_count.get();
            
            match status {
                NetworkStatus::Online => view! {
                    <span class="network-badge online" title="Connected">
                        <span class="badge-dot"></span>
                    </span>
                }.into_view(),
                NetworkStatus::Syncing => view! {
                    <span class="network-badge syncing" title=format!("Saving... (attempt {}/3)", retry)>
                        <span class="badge-dot pulse"></span>
                        <span class="badge-text">"Saving..."</span>
                    </span>
                }.into_view(),
                NetworkStatus::Offline => view! {
                    <span class="network-badge offline" title="Offline - Changes saved locally">
                        <span class="badge-dot"></span>
                        <span class="badge-text">"Offline"</span>
                    </span>
                }.into_view(),
            }
        }}
    }
}
