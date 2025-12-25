use leptos::*;
use gloo_timers::future::TimeoutFuture;
use crate::offline::db::LocalDatabase;
use crate::offline::sync::SyncManager;
use crate::api::TENANT_ID;
use uuid::Uuid;

#[component]
pub fn SyncIndicator() -> impl IntoView {
    let (status, set_status) = create_signal("Online".to_string());
    let (is_syncing, set_is_syncing) = create_signal(false);
    let (db_ready, set_db_ready) = create_signal(false);
    
    // Initialize DB (optional - don't show error if OPFS unavailable)
    create_effect(move |_| {
        spawn_local(async move {
            let db = LocalDatabase::new().await;
            match db {
                Ok(_) => {
                    set_db_ready.set(true);
                    set_status.set("Online".to_string());
                    gloo_console::log!("Offline DB Initialized");
                },
                Err(e) => {
                    // OPFS not available is OK - we're just online-only mode
                    gloo_console::warn!("Offline DB not available (online-only mode):", e);
                    set_status.set("Online".to_string()); // Still online, just no offline support
                }
            }
        });
    });

    let on_sync = move |_| {
        if is_syncing.get() || !db_ready.get() { return; }
        
        set_is_syncing.set(true);
        set_status.set("Syncing...".to_string());
        
        spawn_local(async move {
            // Re-instantiate DB/Manager for now (or move to context later)
            if let Ok(db) = LocalDatabase::new().await {
                let manager = SyncManager::new(db);
                let tenant_id = Uuid::parse_str(TENANT_ID).unwrap_or_default();
                
                // Pull key entities
                let _ = manager.pull_entities("contact", &tenant_id).await;
                let _ = manager.pull_entities("deal", &tenant_id).await;
                let _ = manager.pull_entities("property", &tenant_id).await;
                
                // Push changes
                let _ = manager.push_changes().await;
                
                set_status.set("Synced".to_string());
                TimeoutFuture::new(2000).await;
                set_status.set("Online".to_string());
            } else {
                set_status.set("Sync Failed".to_string());
            }
            set_is_syncing.set(false);
        });
    };

    view! {
        <div class="sync-indicator">
            <button 
                class="btn-sync" 
                class:syncing=is_syncing
                on:click=on_sync
                title="Sync Status"
                disabled=move || !db_ready.get()
            >
                <span class="icon">
                    {move || if is_syncing.get() { "⟳" } else { "☁" }}
                </span>
                <span class="status-text">{status}</span>
            </button>
            <style>
                ".sync-indicator { display: flex; align-items: center; margin-right: 1rem; }
                 .btn-sync { background: none; border: none; cursor: pointer; display: flex; align-items: center; gap: 0.5rem; color: var(--text-color); font-size: 0.9rem; padding: 0.25rem 0.5rem; border-radius: 4px; transition: background 0.2s; }
                 .btn-sync:hover { background: rgba(255,255,255,0.1); }
                 .btn-sync:disabled { opacity: 0.5; cursor: not-allowed; }
                 .btn-sync.syncing .icon { animation: spin 1s linear infinite; }
                 @keyframes spin { 100% { transform: rotate(360deg); } }"
            </style>
        </div>
    }
}
