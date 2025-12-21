//! Log Activity Modal - Log calls, emails, meetings, or notes
//! Metadata-driven activity logging

use leptos::*;
use crate::api::create_interaction;

#[component]
pub fn LogActivityModal(
    entity_type: String,
    record_id: String,
    set_show_modal: WriteSignal<bool>,
    on_success: Callback<()>,
) -> impl IntoView {
    let (interaction_type, set_interaction_type) = create_signal("note".to_string());
    let (title, set_title) = create_signal(String::new());
    let (content, set_content) = create_signal(String::new());
    let (saving, set_saving) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let et = entity_type.clone();
        let rid = record_id.clone();
        let itype = interaction_type.get();
        let t = title.get();
        let c = content.get();
        
        if t.is_empty() {
            set_error.set(Some("Title is required".to_string()));
            return;
        }

        spawn_local(async move {
            set_saving.set(true);
            set_error.set(None);
            
            // Hardcoding created_by for now (TODO: get from auth context)
            let user_id = "0e147821-5cbc-4a23-8bd8-3f5b1a784ea5"; 

            match create_interaction(&et, &rid, &itype, &t, Some(&c), user_id).await {
                Ok(_) => {
                    set_show_modal.set(false);
                    on_success.call(());
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_saving.set(false);
        });
    };

    view! {
        <div class="modal-overlay" on:click=move |_| set_show_modal.set(false)>
            <div class="modal activity-modal" on:click=move |ev| ev.stop_propagation()>
                <h2>"Log Activity"</h2>
                
                {move || error.get().map(|e| view! {
                    <div class="error-banner">{e}</div>
                })}
                
                <form on:submit=handle_submit>
                    <div class="form-group">
                        <label>"Activity Type"</label>
                        <select 
                            class="form-input" 
                            on:change=move |ev| set_interaction_type.set(event_target_value(&ev))
                        >
                            <option value="note">"Note"</option>
                            <option value="call">"Call"</option>
                            <option value="email">"Email"</option>
                            <option value="meeting">"Meeting"</option>
                        </select>
                    </div>
                    
                    <div class="form-group">
                        <label>"Title"</label>
                        <input 
                            type="text" 
                            class="form-input" 
                            placeholder="Brief summary..."
                            on:input=move |ev| set_title.set(event_target_value(&ev))
                            prop:value=title
                            required
                        />
                    </div>
                    
                    <div class="form-group">
                        <label>"Details"</label>
                        <textarea 
                            class="form-input" 
                            rows="4"
                            placeholder="Describe what happened..."
                            on:input=move |ev| set_content.set(event_target_value(&ev))
                            prop:value=content
                        ></textarea>
                    </div>
                    
                    <div class="form-actions">
                        <button 
                            type="button" 
                            class="btn" 
                            on:click=move |_| set_show_modal.set(false)
                            disabled=saving
                        >
                            "Cancel"
                        </button>
                        <button 
                            type="submit" 
                            class="btn btn-primary"
                            disabled=saving
                        >
                            {move || if saving.get() { "Saving..." } else { "Log Activity" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
