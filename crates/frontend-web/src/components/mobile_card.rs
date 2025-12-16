//! Mobile Card Component for Entity Lists

use leptos::*;

/// Props for mobile card
#[derive(Clone)]
pub struct MobileCardData {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub status: Option<String>,
    pub image_url: Option<String>,
    pub phone: Option<String>,
}

/// Mobile card component for entity list items
#[component]
pub fn MobileCard(
    data: MobileCardData,
    #[prop(into)] on_click: Callback<String>,
) -> impl IntoView {
    let id = data.id.clone();
    let phone = data.phone.clone();
    
    view! {
        <div class="mobile-card" on:click=move |_| on_click.call(id.clone())>
            <div class="card-avatar">
                {match &data.image_url {
                    Some(url) => view! { <img src=url.clone() alt="avatar"/> }.into_view(),
                    None => view! { 
                        <div class="avatar-placeholder">
                            {data.title.chars().next().unwrap_or('?').to_string()}
                        </div>
                    }.into_view(),
                }}
            </div>
            
            <div class="card-content">
                <div class="card-title">{data.title}</div>
                <div class="card-subtitle">{data.subtitle}</div>
                {data.status.map(|s| view! {
                    <span class="card-status" data-status=s.to_lowercase()>{s}</span>
                })}
            </div>
            
            <div class="card-actions">
                {phone.map(|p| {
                    let phone_url = format!("tel:{}", p);
                    view! {
                        <a href=phone_url class="action-btn call-btn" on:click=|e| e.stop_propagation()>
                            "📞"
                        </a>
                    }
                })}
            </div>
        </div>
    }
}

/// Mobile card list wrapper
#[component]
pub fn MobileCardList(
    items: Vec<MobileCardData>,
    #[prop(into)] on_item_click: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="mobile-card-list">
            <For
                each=move || items.clone()
                key=|item| item.id.clone()
                children=move |item| {
                    let on_click = on_item_click.clone();
                    view! { <MobileCard data=item on_click=on_click/> }
                }
            />
        </div>
    }
}
