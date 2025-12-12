//! Form component

use leptos::*;
use crate::models::FieldDef;

/// Generic entity form based on FieldDefs
#[component]
pub fn EntityForm(
    fields: Vec<FieldDef>,
    #[prop(optional)] _initial_data: Option<serde_json::Value>,
    on_submit: Callback<serde_json::Value>,
) -> impl IntoView {
    let sorted_fields: Vec<_> = {
        let mut fields = fields;
        fields.sort_by_key(|f| f.sort_order);
        fields
    };

    view! {
        <form class="entity-form" on:submit=move |ev| {
            ev.prevent_default();
            // TODO: Collect form data and call on_submit
            on_submit.call(serde_json::json!({}));
        }>
            {sorted_fields.iter().map(|field| {
                let field = field.clone();
                view! {
                    <div class="form-field">
                        <label class="form-label">
                            {&field.label}
                            {field.is_required.then(|| view! { <span class="required">"*"</span> })}
                        </label>
                        <input
                            type="text"
                            name=&field.name
                            class="form-input"
                            required=field.is_required
                            placeholder=field.placeholder.unwrap_or_default()
                        />
                    </div>
                }
            }).collect::<Vec<_>>()}
            <div class="form-actions">
                <button type="submit" class="btn btn-primary">"Save"</button>
            </div>
        </form>
    }
}
