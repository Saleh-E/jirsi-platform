//! Entity list page with data table, columns, and Add New functionality

use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};
use crate::api::{API_BASE, TENANT_ID};

#[component]
pub fn EntityListPage() -> impl IntoView {
    let params = use_params_map();
    let entity = move || params.with(|p| p.get("entity").cloned().unwrap_or_default());
    
    // Signals
    let (data, set_data) = create_signal(Vec::<serde_json::Value>::new());
    let (loading, set_loading) = create_signal(true);
    let (show_form, set_show_form) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);
    
    // Form fields
    let (first_name, set_first_name) = create_signal(String::new());
    let (last_name, set_last_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (name, set_name) = create_signal(String::new());
    let (amount, set_amount) = create_signal(String::new());
    let (title, set_title) = create_signal(String::new());
    let (description, set_description) = create_signal(String::new());
    let (reference, set_reference) = create_signal(String::new());
    let (address, set_address) = create_signal(String::new());
    let (city, set_city) = create_signal(String::new());
    let (price, set_price) = create_signal(String::new());
    let (bedrooms, set_bedrooms) = create_signal(String::new());

    // Fetch data on load
    let entity_for_fetch = entity.clone();
    create_effect(move |_| {
        let entity_type = entity_for_fetch();
        spawn_local(async move {
            set_loading.set(true);
            match fetch_entity_data(&entity_type).await {
                Ok(items) => {
                    set_data.set(items);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    });

    // Handle Add New
    let entity_for_submit = entity.clone();
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let entity_type = entity_for_submit();
        
        let body = match entity_type.as_str() {
            "contact" => serde_json::json!({
                "first_name": first_name.get(),
                "last_name": last_name.get(),
                "email": email.get()
            }),
            "company" => serde_json::json!({
                "name": name.get()
            }),
            "deal" => serde_json::json!({
                "name": name.get(),
                "amount": amount.get().parse::<f64>().unwrap_or(0.0),
                "stage": "prospecting"
            }),
            "task" => serde_json::json!({
                "tenant_id": TENANT_ID,
                "created_by": "adbd25ef-fe37-43e7-bebe-60dab610903b",
                "title": title.get(),
                "description": description.get(),
                "status": "open",
                "priority": "medium"
            }),
            "property" => serde_json::json!({
                "reference": reference.get(),
                "title": title.get(),
                "address": address.get(),
                "city": city.get(),
                "price": price.get().parse::<i64>().unwrap_or(0),
                "bedrooms": bedrooms.get().parse::<i32>().ok()
            }),
            _ => serde_json::json!({})
        };

        spawn_local(async move {
            match create_entity(&entity_type, body).await {
                Ok(_) => {
                    set_show_form.set(false);
                    // Refresh data
                    if let Ok(items) = fetch_entity_data(&entity_type).await {
                        set_data.set(items);
                    }
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    };

    view! {
        <div class="entity-list-page">
            <header class="page-header">
                <h1>{move || entity().to_uppercase()}</h1>
                <button class="btn btn-primary" on:click=move |_| set_show_form.set(true)>
                    "+ New"
                </button>
            </header>

            // Add New Form Modal
            {move || show_form.get().then(|| {
                let etype = entity();
                view! {
                    <div class="modal-overlay" on:click=move |_| set_show_form.set(false)>
                        <div class="modal" on:click=move |ev| ev.stop_propagation()>
                            <h2>"Add New " {etype.clone()}</h2>
                            <form on:submit=on_submit.clone()>
                                {if entity() == "contact" {
                                    view! {
                                        <div class="form-group">
                                            <label>"First Name"</label>
                                            <input type="text" on:input=move |ev| set_first_name.set(event_target_value(&ev)) required />
                                        </div>
                                        <div class="form-group">
                                            <label>"Last Name"</label>
                                            <input type="text" on:input=move |ev| set_last_name.set(event_target_value(&ev)) required />
                                        </div>
                                        <div class="form-group">
                                            <label>"Email"</label>
                                            <input type="email" on:input=move |ev| set_email.set(event_target_value(&ev)) />
                                        </div>
                                    }.into_view()
                                } else if entity() == "company" {
                                    view! {
                                        <div class="form-group">
                                            <label>"Company Name"</label>
                                            <input type="text" on:input=move |ev| set_name.set(event_target_value(&ev)) required />
                                        </div>
                                    }.into_view()
                                } else if entity() == "deal" {
                                    view! {
                                        <div class="form-group">
                                            <label>"Deal Name"</label>
                                            <input type="text" on:input=move |ev| set_name.set(event_target_value(&ev)) required />
                                        </div>
                                        <div class="form-group">
                                            <label>"Amount"</label>
                                            <input type="number" on:input=move |ev| set_amount.set(event_target_value(&ev)) />
                                        </div>
                                    }.into_view()
                                } else if entity() == "task" {
                                    view! {
                                        <div class="form-group">
                                            <label>"Title"</label>
                                            <input type="text" on:input=move |ev| set_title.set(event_target_value(&ev)) required />
                                        </div>
                                        <div class="form-group">
                                            <label>"Description"</label>
                                            <textarea on:input=move |ev| set_description.set(event_target_value(&ev)) />
                                        </div>
                                    }.into_view()
                                } else if entity() == "property" {
                                    view! {
                                        <div class="form-group">
                                            <label>"Reference"</label>
                                            <input type="text" on:input=move |ev| set_reference.set(event_target_value(&ev)) required placeholder="PROP-001" />
                                        </div>
                                        <div class="form-group">
                                            <label>"Title"</label>
                                            <input type="text" on:input=move |ev| set_title.set(event_target_value(&ev)) required placeholder="Beach House Villa" />
                                        </div>
                                        <div class="form-group">
                                            <label>"Address"</label>
                                            <input type="text" on:input=move |ev| set_address.set(event_target_value(&ev)) required />
                                        </div>
                                        <div class="form-group">
                                            <label>"City"</label>
                                            <input type="text" on:input=move |ev| set_city.set(event_target_value(&ev)) required />
                                        </div>
                                        <div class="form-group">
                                            <label>"Price"</label>
                                            <input type="number" on:input=move |ev| set_price.set(event_target_value(&ev)) />
                                        </div>
                                        <div class="form-group">
                                            <label>"Bedrooms"</label>
                                            <input type="number" on:input=move |ev| set_bedrooms.set(event_target_value(&ev)) />
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <p>"Unknown entity"</p> }.into_view()
                                }}
                                <div class="form-actions">
                                    <button type="button" class="btn" on:click=move |_| set_show_form.set(false)>"Cancel"</button>
                                    <button type="submit" class="btn btn-primary">"Save"</button>
                                </div>
                            </form>
                        </div>
                    </div>
                }
            })}

            // Error display
            {move || error.get().map(|e| view! { <div class="error-message">{e}</div> })}

            // Data table
            <div class="page-content">
                {move || {
                    if loading.get() {
                        view! { <p class="loading">"Loading..."</p> }.into_view()
                    } else if data.get().is_empty() {
                        view! { <p>"No records found. Click '+ New' to create one."</p> }.into_view()
                    } else {
                        let etype = entity();
                        view! {
                            <table class="data-table">
                                <thead>
                                    <tr>
                                        {if etype == "contact" {
                                            view! {
                                                <th>"First Name"</th>
                                                <th>"Last Name"</th>
                                                <th>"Email"</th>
                                                <th>"Phone"</th>
                                            }.into_view()
                                        } else if etype == "company" {
                                            view! {
                                                <th>"Name"</th>
                                                <th>"Domain"</th>
                                                <th>"Industry"</th>
                                            }.into_view()
                                        } else if etype == "deal" {
                                            view! {
                                                <th>"Name"</th>
                                                <th>"Amount"</th>
                                                <th>"Stage"</th>
                                            }.into_view()
                                        } else if etype == "task" {
                                            view! {
                                                <th>"Title"</th>
                                                <th>"Status"</th>
                                                <th>"Priority"</th>
                                            }.into_view()
                                        } else if etype == "property" {
                                            view! {
                                                <th>"Reference"</th>
                                                <th>"Title"</th>
                                                <th>"City"</th>
                                                <th>"Price"</th>
                                                <th>"Bedrooms"</th>
                                            }.into_view()
                                        } else {
                                            view! { <th>"Data"</th> }.into_view()
                                        }}
                                    </tr>
                                </thead>
                                <tbody>
                                    <For
                                        each=move || data.get()
                                        key=|item| item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string()
                                        children=move |item| {
                                            let etype = entity();
                                            let first = item.get("first_name").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                            let last = item.get("last_name").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                            let email = item.get("email").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                            let phone = item.get("phone").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                            let domain = item.get("domain").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                            let industry = item.get("industry").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                            let amount = format!("${:.0}", item.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0));
                                            let stage = item.get("stage").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                            let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                            let priority = item.get("priority").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                            
                                            view! {
                                                <tr>
                                                    {if etype == "contact" {
                                                        view! {
                                                            <td>{first}</td>
                                                            <td>{last}</td>
                                                            <td>{email}</td>
                                                            <td>{phone}</td>
                                                        }.into_view()
                                                    } else if etype == "company" {
                                                        view! {
                                                            <td>{name.clone()}</td>
                                                            <td>{domain}</td>
                                                            <td>{industry}</td>
                                                        }.into_view()
                                                    } else if etype == "deal" {
                                                        view! {
                                                            <td>{name}</td>
                                                            <td>{amount}</td>
                                                            <td>{stage}</td>
                                                        }.into_view()
                                                    } else if etype == "task" {
                                                        view! {
                                                            <td>{title}</td>
                                                            <td>{status}</td>
                                                            <td>{priority}</td>
                                                        }.into_view()
                                                    } else if etype == "property" {
                                                        let reference = item.get("reference").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                                        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                                        let city = item.get("city").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                                                        let price = item.get("price").and_then(|v| v.as_i64()).map(|p| format!("${}", p)).unwrap_or("-".to_string());
                                                        let bedrooms = item.get("bedrooms").and_then(|v| v.as_i64()).map(|b| b.to_string()).unwrap_or("-".to_string());
                                                        view! {
                                                            <td>{reference}</td>
                                                            <td>{title}</td>
                                                            <td>{city}</td>
                                                            <td>{price}</td>
                                                            <td>{bedrooms}</td>
                                                        }.into_view()
                                                    } else {
                                                        view! { <td>"-"</td> }.into_view()
                                                    }}
                                                </tr>
                                            }
                                        }
                                    />
                                </tbody>
                            </table>
                        }.into_view()
                    }
                }}
            </div>
        </div>
    }
}

async fn fetch_entity_data(entity_type: &str) -> Result<Vec<serde_json::Value>, String> {
    let window = web_sys::window().ok_or("no window")?;
    
    // Task and Property use different endpoints
    let url = if entity_type == "task" {
        format!("{}/tasks?tenant_id={}", API_BASE, TENANT_ID)
    } else if entity_type == "property" {
        format!("{}/properties?tenant_id={}", API_BASE, TENANT_ID)
    } else {
        format!("{}/entities/{}?tenant_id={}", API_BASE, entity_type, TENANT_ID)
    };
    
    let opts = RequestInit::new();
    opts.set_method("GET");

    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Request error: {:?}", e))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "response conversion error")?;

    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let json = JsFuture::from(resp.json().map_err(|_| "JSON error")?)
        .await
        .map_err(|e| format!("JSON await error: {:?}", e))?;

    let result: serde_json::Value = serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Parse error: {:?}", e))?;

    result.get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .ok_or("No data field".to_string())
}

async fn create_entity(entity_type: &str, body: serde_json::Value) -> Result<(), String> {
    let window = web_sys::window().ok_or("no window")?;
    
    // Task and Property use different endpoints
    let url = if entity_type == "task" {
        format!("{}/tasks?tenant_id={}", API_BASE, TENANT_ID)
    } else if entity_type == "property" {
        format!("{}/properties?tenant_id={}", API_BASE, TENANT_ID)
    } else {
        format!("{}/entities/{}?tenant_id={}", API_BASE, entity_type, TENANT_ID)
    };
    
    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&JsValue::from_str(&body.to_string()));

    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Request error: {:?}", e))?;

    request.headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Header error: {:?}", e))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "response conversion error")?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Failed to create: {}", resp.status()))
    }
}
