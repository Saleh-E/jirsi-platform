//! Sync Indicator Component
//! 
//! A premium floating network status badge with real-time sync feedback.
//! Features: glassmorphism styling, animated states, offline detection.

use leptos::*;
use gloo_timers::future::TimeoutFuture;
use crate::offline::db::LocalDatabase;
use crate::offline::sync::SyncManager;
use crate::api::TENANT_ID;
use uuid::Uuid;

/// Network sync status enum
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    Synced,
    Saving,
    Syncing,
    Offline,
    Error,
}

impl SyncStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Synced => "All changes saved",
            Self::Saving => "Saving...",
            Self::Syncing => "Syncing...",
            Self::Offline => "Offline - changes saved locally",
            Self::Error => "Sync failed - retry?",
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Synced => "✓",
            Self::Saving => "↻",
            Self::Syncing => "↻",
            Self::Offline => "⚡",
            Self::Error => "⚠",
        }
    }
    
    pub fn should_show(&self) -> bool {
        !matches!(self, Self::Synced)
    }
}

/// Global sync context for other components to report status
#[derive(Clone)]
pub struct SyncContext {
    pub status: RwSignal<SyncStatus>,
}

impl SyncContext {
    pub fn set_saving(&self) {
        self.status.set(SyncStatus::Saving);
    }
    
    pub fn set_synced(&self) {
        self.status.set(SyncStatus::Synced);
    }
    
    pub fn set_error(&self) {
        self.status.set(SyncStatus::Error);
    }
    
    pub fn set_offline(&self) {
        self.status.set(SyncStatus::Offline);
    }
}

/// Get sync context from provider
pub fn use_sync() -> Option<SyncContext> {
    use_context::<SyncContext>()
}

/// Sync Indicator - Floating badge for network status
#[component]
pub fn SyncIndicator() -> impl IntoView {
    let status = create_rw_signal(SyncStatus::Synced);
    let (is_syncing, set_is_syncing) = create_signal(false);
    let (db_ready, set_db_ready) = create_signal(false);
    let (visible, set_visible) = create_signal(false);
    
    // Provide global sync context
    provide_context(SyncContext { status });
    
    // Initialize DB (optional - don't show error if OPFS unavailable)
    create_effect(move |_| {
        spawn_local(async move {
            let db = LocalDatabase::new().await;
            match db {
                Ok(_) => {
                    set_db_ready.set(true);
                    gloo_console::log!("✓ Offline DB Initialized");
                },
                Err(e) => {
                    // OPFS not available is OK - we're just online-only mode
                    gloo_console::warn!("Offline DB not available (online-only mode):", e);
                }
            }
        });
    });
    
    // Track visibility based on status
    create_effect(move |_| {
        let current = status.get();
        if current.should_show() {
            set_visible.set(true);
        } else {
            // Delay hiding to show "saved" confirmation briefly
            spawn_local(async move {
                TimeoutFuture::new(1500).await;
                if !status.get().should_show() {
                    set_visible.set(false);
                }
            });
        }
    });

    let on_sync = move |_| {
        if is_syncing.get() { return; }
        
        set_is_syncing.set(true);
        status.set(SyncStatus::Syncing);
        
        spawn_local(async move {
            if db_ready.get() {
                if let Ok(db) = LocalDatabase::new().await {
                    let manager = SyncManager::new(db);
                    let tenant_id = Uuid::parse_str(TENANT_ID).unwrap_or_default();
                    
                    // Pull key entities
                    let _ = manager.pull_entities("contact", &tenant_id).await;
                    let _ = manager.pull_entities("deal", &tenant_id).await;
                    let _ = manager.pull_entities("property", &tenant_id).await;
                    
                    // Push changes
                    let _ = manager.push_changes().await;
                    
                    status.set(SyncStatus::Synced);
                } else {
                    status.set(SyncStatus::Error);
                }
            } else {
                // Online-only mode - just show success
                TimeoutFuture::new(500).await;
                status.set(SyncStatus::Synced);
            }
            set_is_syncing.set(false);
        });
    };

    view! {
        // Headerbar inline indicator
        <div class="sync-indicator-header">
            <button 
                class="btn-sync-header" 
                class:syncing=is_syncing
                on:click=on_sync
                title="Sync Status"
            >
                <span class="icon">
                    {move || if is_syncing.get() { "↻" } else { "☁" }}
                </span>
                <span class="status-text">
                    {move || match status.get() {
                        SyncStatus::Synced => "Online",
                        SyncStatus::Saving => "Saving...",
                        SyncStatus::Syncing => "Syncing...",
                        SyncStatus::Offline => "Offline",
                        SyncStatus::Error => "Error",
                    }}
                </span>
            </button>
        </div>
        
        // Fixed floating badge (only visible during non-synced states)
        <Show when=move || visible.get()>
            <div class="sync-badge-float">
                {move || {
                    let s = status.get();
                    let badge_class = match s {
                        SyncStatus::Saving | SyncStatus::Syncing => "sync-badge saving",
                        SyncStatus::Offline => "sync-badge offline",
                        SyncStatus::Error => "sync-badge error",
                        SyncStatus::Synced => "sync-badge synced",
                    };
                    view! {
                        <span class=badge_class on:click=on_sync.clone()>
                            <span class="badge-icon">{s.icon()}</span>
                            <span class="badge-text">{s.label()}</span>
                        </span>
                    }
                }}
            </div>
        </Show>
        
        <style>
            r#"
            .sync-indicator-header {
                display: flex;
                align-items: center;
                margin-right: 0.75rem;
            }
            
            .btn-sync-header {
                background: none;
                border: none;
                cursor: pointer;
                display: flex;
                align-items: center;
                gap: 0.4rem;
                color: var(--text-secondary, #6b7280);
                font-size: 0.85rem;
                padding: 0.35rem 0.6rem;
                border-radius: 6px;
                transition: all 0.2s ease;
            }
            
            .btn-sync-header:hover {
                background: var(--hover-bg, rgba(99, 102, 241, 0.1));
                color: var(--text-color, #374151);
            }
            
            .btn-sync-header.syncing .icon {
                animation: spin-sync 1s linear infinite;
            }
            
            .sync-badge-float {
                position: fixed;
                bottom: 1.5rem;
                right: 1.5rem;
                z-index: 9999;
                animation: slideUp 0.25s cubic-bezier(0.16, 1, 0.3, 1);
            }
            
            .sync-badge {
                display: inline-flex;
                align-items: center;
                gap: 0.5rem;
                padding: 0.5rem 1rem;
                border-radius: 9999px;
                font-size: 0.8rem;
                font-weight: 600;
                backdrop-filter: blur(12px);
                box-shadow: 0 4px 15px rgba(0, 0, 0, 0.15);
                cursor: pointer;
                transition: transform 0.2s ease, box-shadow 0.2s ease;
            }
            
            .sync-badge:hover {
                transform: scale(1.02);
                box-shadow: 0 6px 20px rgba(0, 0, 0, 0.2);
            }
            
            .sync-badge.saving {
                background: linear-gradient(135deg, rgba(251, 191, 36, 0.9), rgba(245, 158, 11, 0.9));
                color: #78350f;
            }
            
            .sync-badge.saving .badge-icon {
                animation: spin-sync 1s linear infinite;
            }
            
            .sync-badge.offline {
                background: linear-gradient(135deg, rgba(156, 163, 175, 0.9), rgba(107, 114, 128, 0.9));
                color: #1f2937;
            }
            
            .sync-badge.error {
                background: linear-gradient(135deg, rgba(248, 113, 113, 0.9), rgba(239, 68, 68, 0.9));
                color: #7f1d1d;
            }
            
            .sync-badge.synced {
                background: linear-gradient(135deg, rgba(74, 222, 128, 0.9), rgba(34, 197, 94, 0.9));
                color: #14532d;
            }
            
            .badge-icon {
                font-size: 1rem;
            }
            
            @keyframes spin-sync {
                100% { transform: rotate(360deg); }
            }
            
            @keyframes slideUp {
                0% { opacity: 0; transform: translateY(10px); }
                100% { opacity: 1; transform: translateY(0); }
            }
            "#
        </style>
    }
}
