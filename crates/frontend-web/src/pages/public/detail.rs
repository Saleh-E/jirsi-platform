//! Public Property Detail Page - Single property view with inquiry form

use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use crate::api::fetch_json;

/// Public property detail from API
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicListingDetail {
    pub id: String,
    #[serde(default)]
    pub reference: Option<String>,
    pub title: String,
    #[serde(default)]
    pub property_type: Option<String>,
    #[serde(default)]
    pub usage: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub area: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub latitude: Option<f64>,
    #[serde(default)]
    pub longitude: Option<f64>,
    #[serde(default)]
    pub bedrooms: Option<i32>,
    #[serde(default)]
    pub bathrooms: Option<i32>,
    #[serde(default)]
    pub size_sqm: Option<f64>,
    #[serde(default)]
    pub floor: Option<i32>,
    #[serde(default)]
    pub total_floors: Option<i32>,
    #[serde(default)]
    pub year_built: Option<i32>,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub rent_amount: Option<f64>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub amenities: Option<serde_json::Value>,
    #[serde(default)]
    pub photos: Option<serde_json::Value>,
    #[serde(default)]
    pub listed_at: Option<String>,
}

/// Format price with currency
fn format_price(price: Option<f64>, currency: Option<String>) -> String {
    match price {
        Some(p) => {
            let curr = currency.unwrap_or_else(|| "USD".to_string());
            let symbol = match curr.as_str() {
                "USD" => "$",
                "EUR" => "€",
                "GBP" => "£",
                "AED" => "AED ",
                _ => "",
            };
            format!("{}{}", symbol, format_number(p))
        }
        None => "Price on Request".to_string(),
    }
}

fn format_number(n: f64) -> String {
    let s = (n as i64).to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

/// Get photos as Vec<String>
fn get_photos(photos: Option<&serde_json::Value>) -> Vec<String> {
    photos
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    v.as_str()
                        .or_else(|| v.get("url").and_then(|u| u.as_str()))
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Get amenities as Vec<String>
fn get_amenities(amenities: Option<&serde_json::Value>) -> Vec<String> {
    amenities
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Public Property Detail Page Component
#[component]
pub fn PublicDetailPage() -> impl IntoView {
    let params = use_params_map();
    
    // Form state
    let (name, set_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (phone, set_phone) = create_signal(String::new());
    let (message, set_message) = create_signal(String::new());
    let (submitted, set_submitted) = create_signal(false);
    let (submitting, set_submitting) = create_signal(false);
    let (error, _set_error) = create_signal::<Option<String>>(None);
    
    // Selected photo for gallery
    let (selected_photo, set_selected_photo) = create_signal::<usize>(0);
    
    // Fetch listing
    let listing = create_resource(
        move || params.with(|p| p.get("id").cloned().unwrap_or_default()),
        |id| async move {
            if id.is_empty() { return None; }
            let url = format!("http://localhost:3000/public/listings/{}?tenant_slug=demo", id);
            fetch_json::<PublicListingDetail>(&url).await.ok()
        }
    );
    
    view! {
        <div class="detail-page">
            {move || match listing.get() {
                None => view! { <div class="detail-loading">"Loading property..."</div> }.into_view(),
                Some(None) => view! { <div class="detail-error">"Property not found"</div> }.into_view(),
                Some(Some(property)) => {
                    let photos = get_photos(property.photos.as_ref());
                    let amenities = get_amenities(property.amenities.as_ref());
                    let price = format_price(property.price.or(property.rent_amount), property.currency.clone());
                    let is_rent = property.rent_amount.is_some() && property.price.is_none();
                    let photos_len = photos.len();
                    let photos_for_gallery = photos.clone();
                    let photos_for_thumbs = photos.clone();
                    
                    view! {
                        <div class="detail-content">
                            // Left Column - Property Info
                            <div class="detail-main">
                                // Photo Gallery
                                <div class="detail-gallery">
                                    <div class="detail-gallery__main">
                                        {move || {
                                            let idx = selected_photo.get().min(photos_len.saturating_sub(1));
                                            let photo = photos_for_gallery.get(idx).cloned()
                                                .unwrap_or_else(|| "https://via.placeholder.com/800x600?text=No+Image".to_string());
                                            view! { <img src=photo alt="Property" /> }
                                        }}
                                    </div>
                                    {(photos_len > 1).then(|| {
                                        view! {
                                            <div class="detail-gallery__thumbs">
                                                {photos_for_thumbs.into_iter().enumerate().map(|(i, photo)| {
                                                    view! {
                                                        <button
                                                            class="detail-gallery__thumb"
                                                            on:click=move |_| set_selected_photo.set(i)
                                                        >
                                                            <img src=photo.clone() alt="" />
                                                        </button>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }
                                    })}
                                </div>
                                
                                // Property Info
                                <div class="detail-info">
                                    <div class="detail-header">
                                        <div>
                                            <h1 class="detail-title">{property.title.clone()}</h1>
                                            <div class="detail-location">
                                                {property.city.clone()}
                                                {property.area.as_ref().map(|a| format!(", {}", a))}
                                            </div>
                                        </div>
                                        <div class="detail-price">
                                            {price}
                                            {is_rent.then(|| view! { <span class="detail-price__suffix">"/month"</span> })}
                                        </div>
                                    </div>
                                    
                                    // Specs
                                    <div class="detail-specs">
                                        {property.bedrooms.map(|b| view! {
                                            <div class="detail-spec">
                                                <span class="detail-spec__label">"Bedrooms"</span>
                                                <span class="detail-spec__value">{b}</span>
                                            </div>
                                        })}
                                        {property.bathrooms.map(|b| view! {
                                            <div class="detail-spec">
                                                <span class="detail-spec__label">"Bathrooms"</span>
                                                <span class="detail-spec__value">{b}</span>
                                            </div>
                                        })}
                                        {property.size_sqm.map(|s| view! {
                                            <div class="detail-spec">
                                                <span class="detail-spec__label">"Size"</span>
                                                <span class="detail-spec__value">{format!("{:.0} sqm", s)}</span>
                                            </div>
                                        })}
                                    </div>
                                    
                                    // Description
                                    {property.description.as_ref().map(|desc| view! {
                                        <div class="detail-section">
                                            <h2 class="detail-section__title">"Description"</h2>
                                            <p class="detail-description">{desc.clone()}</p>
                                        </div>
                                    })}
                                    
                                    // Amenities
                                    {(!amenities.is_empty()).then(|| view! {
                                        <div class="detail-section">
                                            <h2 class="detail-section__title">"Amenities"</h2>
                                            <div class="detail-amenities">
                                                {amenities.iter().cloned().map(|amenity| view! {
                                                    <span class="detail-amenity">"✓ "{amenity}</span>
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    })}
                                </div>
                            </div>
                            
                            // Right Column - Inquiry Form
                            <div class="detail-sidebar">
                                <div class="inquiry-form" id="contact">
                                    {move || if submitted.get() {
                                        view! {
                                            <div class="inquiry-success">
                                                <div class="inquiry-success__icon">"✓"</div>
                                                <h3 class="inquiry-success__title">"Thank You!"</h3>
                                                <p class="inquiry-success__message">
                                                    "Your inquiry has been sent. We'll be in touch soon."
                                                </p>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <div>
                                                <h3 class="inquiry-form__title">"Contact Agent"</h3>
                                                <p class="inquiry-form__subtitle">"Interested in this property? Send us a message."</p>
                                                
                                                <div class="inquiry-form__field">
                                                    <label>"Name"</label>
                                                    <input
                                                        type="text"
                                                        required
                                                        prop:value=move || name.get()
                                                        on:input=move |ev| set_name.set(event_target_value(&ev))
                                                        placeholder="Your name"
                                                    />
                                                </div>
                                                
                                                <div class="inquiry-form__field">
                                                    <label>"Email"</label>
                                                    <input
                                                        type="email"
                                                        required
                                                        prop:value=move || email.get()
                                                        on:input=move |ev| set_email.set(event_target_value(&ev))
                                                        placeholder="your@email.com"
                                                    />
                                                </div>
                                                
                                                <div class="inquiry-form__field">
                                                    <label>"Phone"</label>
                                                    <input
                                                        type="tel"
                                                        prop:value=move || phone.get()
                                                        on:input=move |ev| set_phone.set(event_target_value(&ev))
                                                        placeholder="+1 234 567 8900"
                                                    />
                                                </div>
                                                
                                                <div class="inquiry-form__field">
                                                    <label>"Message"</label>
                                                    <textarea
                                                        rows="4"
                                                        prop:value=move || message.get()
                                                        on:input=move |ev| set_message.set(event_target_value(&ev))
                                                        placeholder="I'm interested in this property..."
                                                    ></textarea>
                                                </div>
                                                
                                                {move || error.get().map(|e| view! {
                                                    <div class="inquiry-form__error">{e}</div>
                                                })}
                                                
                                                <button 
                                                    type="button"
                                                    class="inquiry-form__submit"
                                                    disabled=move || submitting.get()
                                                    on:click=move |_| {
                                                        set_submitting.set(true);
                                                        // TODO: Submit inquiry via API
                                                        set_timeout(move || {
                                                            set_submitted.set(true);
                                                            set_submitting.set(false);
                                                        }, std::time::Duration::from_millis(500));
                                                    }
                                                >
                                                    {move || if submitting.get() { "Sending..." } else { "Send Inquiry" }}
                                                </button>
                                            </div>
                                        }.into_view()
                                    }}
                                </div>
                            </div>
                        </div>
                    }.into_view()
                }
            }}
        </div>
        
        // CSS
        <style>
        {r#"
.detail-page { padding: 1rem 0; }
.detail-loading, .detail-error { text-align: center; padding: 3rem; color: var(--text-muted, #888); }
.detail-content { display: grid; grid-template-columns: 1fr 380px; gap: 2rem; align-items: start; }
@media (max-width: 968px) { .detail-content { grid-template-columns: 1fr; } }
.detail-gallery { margin-bottom: 2rem; }
.detail-gallery__main { aspect-ratio: 16 / 9; border-radius: 12px; overflow: hidden; background: var(--bg-tertiary, #252530); margin-bottom: 0.75rem; }
.detail-gallery__main img { width: 100%; height: 100%; object-fit: cover; }
.detail-gallery__thumbs { display: flex; gap: 0.5rem; overflow-x: auto; }
.detail-gallery__thumb { flex-shrink: 0; width: 80px; height: 60px; border-radius: 6px; overflow: hidden; border: 2px solid transparent; padding: 0; cursor: pointer; }
.detail-gallery__thumb img { width: 100%; height: 100%; object-fit: cover; }
.detail-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 1rem; margin-bottom: 1.5rem; }
.detail-title { font-size: 1.75rem; font-weight: 700; color: var(--text-primary, #f0f0f5); margin-bottom: 0.375rem; }
.detail-location { color: var(--text-muted, #888); }
.detail-price { font-size: 1.75rem; font-weight: 700; color: var(--primary-color, #7c3aed); white-space: nowrap; }
.detail-price__suffix { font-size: 1rem; font-weight: 400; color: var(--text-muted, #888); }
.detail-specs { display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 1rem; padding: 1.25rem; background: var(--bg-secondary, #1a1a24); border-radius: 12px; margin-bottom: 2rem; }
.detail-spec { text-align: center; }
.detail-spec__label { display: block; font-size: 0.75rem; color: var(--text-muted, #888); text-transform: uppercase; margin-bottom: 0.25rem; }
.detail-spec__value { font-size: 1.125rem; font-weight: 600; color: var(--text-primary, #f0f0f5); }
.detail-section { margin-bottom: 2rem; }
.detail-section__title { font-size: 1.25rem; font-weight: 600; color: var(--text-primary, #f0f0f5); margin-bottom: 1rem; }
.detail-description { color: var(--text-secondary, #a0a0b0); line-height: 1.7; white-space: pre-wrap; }
.detail-amenities { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 0.75rem; }
.detail-amenity { color: var(--text-secondary, #a0a0b0); }
.detail-sidebar { position: sticky; top: 100px; }
.inquiry-form { background: var(--bg-secondary, #1a1a24); border: 1px solid var(--border-color, #2a2a3a); border-radius: 12px; padding: 1.5rem; }
.inquiry-form__title { font-size: 1.25rem; font-weight: 600; color: var(--text-primary, #f0f0f5); margin-bottom: 0.375rem; }
.inquiry-form__subtitle { color: var(--text-muted, #888); font-size: 0.875rem; margin-bottom: 1.5rem; }
.inquiry-form__field { margin-bottom: 1rem; }
.inquiry-form__field label { display: block; font-size: 0.875rem; font-weight: 500; color: var(--text-secondary, #a0a0b0); margin-bottom: 0.375rem; }
.inquiry-form__field input, .inquiry-form__field textarea { width: 100%; padding: 0.75rem; background: var(--bg-tertiary, #252530); border: 1px solid var(--border-color, #3a3a4a); border-radius: 8px; color: var(--text-primary, #f0f0f5); font-size: 1rem; }
.inquiry-form__field input:focus, .inquiry-form__field textarea:focus { outline: none; border-color: var(--primary-color, #7c3aed); }
.inquiry-form__field textarea { resize: vertical; min-height: 100px; }
.inquiry-form__error { color: #ef4444; font-size: 0.875rem; margin-bottom: 1rem; }
.inquiry-form__submit { width: 100%; padding: 0.875rem; background: var(--primary-color, #7c3aed); border: none; border-radius: 8px; color: white; font-size: 1rem; font-weight: 600; cursor: pointer; transition: opacity 0.2s; }
.inquiry-form__submit:hover:not(:disabled) { opacity: 0.9; }
.inquiry-form__submit:disabled { opacity: 0.6; cursor: not-allowed; }
.inquiry-success { text-align: center; padding: 2rem 0; }
.inquiry-success__icon { width: 64px; height: 64px; margin: 0 auto 1rem; background: #22c55e; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 2rem; color: white; }
.inquiry-success__title { font-size: 1.5rem; font-weight: 600; color: var(--text-primary, #f0f0f5); margin-bottom: 0.5rem; }
.inquiry-success__message { color: var(--text-muted, #888); }
        "#}
        </style>
    }
}
