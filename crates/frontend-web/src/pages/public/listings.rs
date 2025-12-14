//! Public Listings Page - Property gallery for public website

use leptos::*;
use serde::{Deserialize, Serialize};
use crate::api::fetch_json;

/// Public property listing from API
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicListing {
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
    pub city: Option<String>,
    #[serde(default)]
    pub area: Option<String>,
    #[serde(default)]
    pub bedrooms: Option<i32>,
    #[serde(default)]
    pub bathrooms: Option<i32>,
    #[serde(default)]
    pub size_sqm: Option<f64>,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub rent_amount: Option<f64>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub photos: Option<serde_json::Value>,
    #[serde(default)]
    pub listed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListingsResponse {
    pub data: Vec<PublicListing>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
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

/// Format number with commas
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

/// Get first photo URL from photos JSON
fn get_cover_photo(photos: Option<&serde_json::Value>) -> String {
    photos
        .and_then(|p| p.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str().or_else(|| v.get("url").and_then(|u| u.as_str())))
        .map(String::from)
        .unwrap_or_else(|| "https://via.placeholder.com/400x300?text=No+Image".to_string())
}

/// Public Listings Page Component
#[component]
pub fn PublicListingsPage() -> impl IntoView {
    let listings = create_resource(
        || (),
        |_| async move {
            let url = "http://localhost:3000/public/listings?tenant_slug=demo";
            fetch_json::<ListingsResponse>(url).await.ok()
        }
    );
    
    view! {
        <div class="listings-page">
            <div class="listings-header">
                <h1 class="listings-title">"Our Properties"</h1>
                <p class="listings-subtitle">"Discover your perfect property from our curated selection"</p>
            </div>
            
            // Listings Grid
            <div class="listings-grid">
                {move || match listings.get() {
                    None => view! { <div class="listings-loading">"Loading properties..."</div> }.into_view(),
                    Some(None) => view! { <div class="listings-error">"Failed to load properties"</div> }.into_view(),
                    Some(Some(response)) => {
                        if response.data.is_empty() {
                            view! { <div class="listings-empty">"No properties available at this time"</div> }.into_view()
                        } else {
                            response.data.into_iter().map(|listing| {
                                let id = listing.id.clone();
                                let cover = get_cover_photo(listing.photos.as_ref());
                                let price = format_price(listing.price.or(listing.rent_amount), listing.currency.clone());
                                let is_rent = listing.rent_amount.is_some() && listing.price.is_none();
                                
                                view! {
                                    <a href=format!("/listings/{}", id) class="listing-card">
                                        <div class="listing-card__image">
                                            <img src=cover alt=listing.title.clone() />
                                            {listing.property_type.map(|pt| view! {
                                                <span class="listing-card__badge">{pt}</span>
                                            })}
                                        </div>
                                        <div class="listing-card__content">
                                            <h3 class="listing-card__title">{listing.title}</h3>
                                            <div class="listing-card__location">
                                                {listing.city.clone()}
                                                {listing.area.as_ref().map(|a| format!(", {}", a))}
                                            </div>
                                            <div class="listing-card__price">
                                                {price}
                                                {is_rent.then(|| view! { <span class="listing-card__price-suffix">"/month"</span> })}
                                            </div>
                                            <div class="listing-card__specs">
                                                {listing.bedrooms.map(|b| view! {
                                                    <span class="listing-card__spec">{b}" Beds"</span>
                                                })}
                                                {listing.bathrooms.map(|b| view! {
                                                    <span class="listing-card__spec">{b}" Baths"</span>
                                                })}
                                                {listing.size_sqm.map(|s| view! {
                                                    <span class="listing-card__spec">{format!("{:.0} sqm", s)}</span>
                                                })}
                                            </div>
                                        </div>
                                    </a>
                                }
                            }).collect_view()
                        }
                    }
                }}
            </div>
        </div>
        
        // CSS
        <style>
        {r#"
.listings-page { padding: 1rem 0; }
.listings-header { text-align: center; margin-bottom: 2.5rem; }
.listings-title { font-size: 2.25rem; font-weight: 700; color: var(--text-primary, #f0f0f5); margin-bottom: 0.5rem; }
.listings-subtitle { color: var(--text-muted, #888); font-size: 1.125rem; }
.listings-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 1.5rem; }
.listings-loading, .listings-error, .listings-empty { grid-column: 1 / -1; text-align: center; padding: 3rem; color: var(--text-muted, #888); }
.listing-card { background: var(--bg-secondary, #1a1a24); border-radius: 12px; overflow: hidden; text-decoration: none; color: inherit; transition: transform 0.2s; border: 1px solid var(--border-color, #2a2a3a); }
.listing-card:hover { transform: translateY(-4px); box-shadow: 0 12px 40px rgba(0, 0, 0, 0.3); }
.listing-card__image { position: relative; aspect-ratio: 16 / 10; overflow: hidden; }
.listing-card__image img { width: 100%; height: 100%; object-fit: cover; }
.listing-card__badge { position: absolute; top: 0.75rem; left: 0.75rem; padding: 0.25rem 0.75rem; background: var(--primary-color, #7c3aed); color: white; font-size: 0.75rem; font-weight: 600; border-radius: 4px; }
.listing-card__content { padding: 1.25rem; }
.listing-card__title { font-size: 1.125rem; font-weight: 600; color: var(--text-primary, #f0f0f5); margin-bottom: 0.375rem; }
.listing-card__location { color: var(--text-muted, #888); font-size: 0.875rem; margin-bottom: 0.75rem; }
.listing-card__price { font-size: 1.375rem; font-weight: 700; color: var(--primary-color, #7c3aed); margin-bottom: 0.75rem; }
.listing-card__price-suffix { font-size: 0.875rem; font-weight: 400; color: var(--text-muted, #888); }
.listing-card__specs { display: flex; gap: 1rem; flex-wrap: wrap; }
.listing-card__spec { color: var(--text-secondary, #a0a0b0); font-size: 0.875rem; }
        "#}
        </style>
    }
}
