//! Data table component

use leptos::*;
use crate::models::{FieldDef, ViewColumn};

/// Generic data table based on ViewDef columns
#[component]
pub fn DataTable(
    columns: Vec<ViewColumn>,
    fields: Vec<FieldDef>,
    data: Vec<serde_json::Value>,
    #[prop(optional)] _on_row_click: Option<Callback<serde_json::Value>>,
) -> impl IntoView {
    // Build field lookup
    let field_map: std::collections::HashMap<String, FieldDef> = fields
        .into_iter()
        .map(|f| (f.name.clone(), f))
        .collect();

    let visible_columns: Vec<_> = columns
        .into_iter()
        .filter(|c| c.visible)
        .collect();

    view! {
        <div class="data-table-wrapper">
            <table class="data-table">
                <thead>
                    <tr>
                        {visible_columns.iter().map(|col| {
                            let label = field_map
                                .get(&col.field)
                                .map(|f| f.label.clone())
                                .unwrap_or_else(|| col.field.clone());
                            view! {
                                <th class="table-header">{label}</th>
                            }
                        }).collect::<Vec<_>>()}
                    </tr>
                </thead>
                <tbody>
                    {data.iter().map(|row| {
                        view! {
                            <tr class="table-row">
                                {visible_columns.iter().map(|col| {
                                    let value = row.get(&col.field)
                                        .cloned()
                                        .unwrap_or(serde_json::Value::Null);
                                    view! {
                                        <td class="table-cell">{value.to_string()}</td>
                                    }
                                }).collect::<Vec<_>>()}
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}
