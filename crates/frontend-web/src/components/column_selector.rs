//! Column Selector Component
//! 
//! Dropdown component to show/hide columns in entity list views.

use leptos::*;

/// Column visibility state
#[derive(Clone, Debug)]
pub struct ColumnConfig {
    pub field: String,
    pub label: String,
    pub visible: bool,
}

/// Column Selector Dropdown Component
#[component]
pub fn ColumnSelector(
    /// Available columns
    columns: Signal<Vec<ColumnConfig>>,
    /// Callback when column visibility changes
    on_toggle: Callback<String>,
    /// Controls dropdown visibility
    show_dropdown: RwSignal<bool>,
) -> impl IntoView {
    // Toggle dropdown
    let toggle_dropdown = move |_| {
        show_dropdown.update(|v| *v = !*v);
    };
    
    // Close dropdown when clicking outside
    let close_dropdown = move |_| {
        show_dropdown.set(false);
    };

    view! {
        <div class="column-selector-wrapper">
            <button 
                class="column-selector-btn"
                on:click=toggle_dropdown
                title="Manage columns"
            >
                <svg class="icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="3" y="3" width="7" height="7"></rect>
                    <rect x="14" y="3" width="7" height="7"></rect>
                    <rect x="14" y="14" width="7" height="7"></rect>
                    <rect x="3" y="14" width="7" height="7"></rect>
                </svg>
                <span>"Columns"</span>
            </button>
            
            <Show when=move || show_dropdown.get()>
                <div class="column-selector-backdrop" on:click=close_dropdown></div>
                <div class="column-selector-dropdown">
                    <div class="dropdown-header">
                        <span class="dropdown-title">"Show/Hide Columns"</span>
                    </div>
                    <div class="dropdown-body">
                        {move || columns.get().into_iter().map(|col| {
                            let field = col.field.clone();
                            let field_for_toggle = field.clone();
                            let on_toggle = on_toggle.clone();
                            
                            view! {
                                <label class="column-option">
                                    <input 
                                        type="checkbox"
                                        checked=col.visible
                                        on:change=move |_| {
                                            on_toggle.call(field_for_toggle.clone());
                                        }
                                    />
                                    <span class="column-label">{col.label}</span>
                                </label>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </Show>
        </div>
    }
}
