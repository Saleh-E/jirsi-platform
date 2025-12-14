//! Public Listings Page - Premium property gallery for public website

use leptos::*;
use leptos_router::*;
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
    pub country: Option<String>,
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

/// Format price with currency symbol
fn format_price(price: Option<f64>, currency: Option<String>) -> String {
    match price {
        Some(p) => {
            let curr = currency.unwrap_or_else(|| "USD".to_string());
            let symbol = match curr.as_str() {
                "USD" => "$",
                "EUR" => "€",
                "GBP" => "£",
                "AED" => "AED ",
                "SAR" => "SAR ",
                _ => "$",
            };
            format!("{}{}", symbol, format_number(p))
        }
        None => "Price on Request".to_string(),
    }
}

/// Format number with thousands separator
fn format_number(n: f64) -> String {
    let s = (n as i64).to_string();
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
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
        .unwrap_or_else(|| "https://images.unsplash.com/photo-1564013799919-ab600027ffc6?w=800&q=80".to_string())
}

/// Public Listings Page Component - Premium Design
#[component]
pub fn PublicListingsPage() -> impl IntoView {
    let navigate = use_navigate();
    
    let listings = create_resource(
        || (),
        |_| async move {
            let url = "http://localhost:3000/public/listings?tenant_slug=demo";
            fetch_json::<ListingsResponse>(url).await.ok()
        }
    );
    
    view! {
        <div class="public-listings">
            // Header Section
            <header class="listings-header">
                <h1 class="listings-title">"Discover Your Dream Property"</h1>
                <p class="listings-subtitle">
                    "Browse our exclusive collection of premium real estate"
                    {move || listings.get().flatten().map(|r| view! {
                        <span class="listings-count">" • "{r.total}" properties available"</span>
                    })}
                </p>
            </header>
            
            // Main Content
            <main>
                {move || match listings.get() {
                    None => view! {
                        <div class="listings-loading">
                            <div class="loading-spinner"></div>
                        </div>
                    }.into_view(),
                    
                    Some(None) => view! {
                        <div class="listings-empty">
                            <div class="listings-empty__icon">"🏠"</div>
                            <h2 class="listings-empty__title">"Unable to Load Properties"</h2>
                            <p class="listings-empty__text">
                                "We're having trouble connecting to our server. Please try again later."
                            </p>
                        </div>
                    }.into_view(),
                    
                    Some(Some(response)) => {
                        if response.data.is_empty() {
                            view! {
                                <div class="listings-empty">
                                    <div class="listings-empty__icon">"🔍"</div>
                                    <h2 class="listings-empty__title">"No Properties Found"</h2>
                                    <p class="listings-empty__text">
                                        "We don't have any properties matching your criteria right now. Check back soon for new listings!"
                                    </p>
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <div class="listings-grid">
                                    {response.data.into_iter().map(|listing| {
                                        let id = listing.id.clone();
                                        let nav = navigate.clone();
                                        let cover = get_cover_photo(listing.photos.as_ref());
                                        let is_rent = listing.rent_amount.is_some() && listing.price.is_none();
                                        let display_price = if is_rent { listing.rent_amount } else { listing.price };
                                        let price_str = format_price(display_price, listing.currency.clone());
                                        let status = listing.status.clone().unwrap_or_else(|| "active".to_string());
                                        let listing_type = if is_rent { "rent" } else { "sale" };
                                        
                                        let on_click = move |_| {
                                            nav(&format!("/listings/{}", id), Default::default());
                                        };
                                        
                                        view! {
                                            <article class="listing-card" on:click=on_click tabindex="0">
                                                // Image Container
                                                <div class="listing-card__image-container">
                                                    <img
                                                        src=cover.clone()
                                                        alt=listing.title.clone()
                                                        class="listing-card__image"
                                                        loading="lazy"
                                                    />
                                                    <div class="listing-card__overlay"></div>
                                                    
                                                    // Status Badge
                                                    <span class=format!("listing-card__badge listing-card__badge--status listing-card__badge--{}", status.to_lowercase())>
                                                        {match status.as_str() {
                                                            "active" => "Active",
                                                            "sold" => "Sold",
                                                            "pending" => "Pending",
                                                            "rented" => "Rented",
                                                            _ => "Active"
                                                        }}
                                                    </span>
                                                    
                                                    // Type Badge
                                                    <span class=format!("listing-card__badge listing-card__badge--type listing-card__badge--{}", listing_type)>
                                                        {if is_rent { "For Rent" } else { "For Sale" }}
                                                    </span>
                                                    
                                                    // Price Tag
                                                    <div class="listing-card__price">
                                                        {price_str}
                                                        {is_rent.then(|| view! { <span class="listing-card__price-suffix">"/mo"</span> })}
                                                    </div>
                                                </div>
                                                
                                                // Content
                                                <div class="listing-card__content">
                                                    <h3 class="listing-card__title">{listing.title.clone()}</h3>
                                                    
                                                    <div class="listing-card__location">
                                                        <span class="listing-card__location-icon">"📍"</span>
                                                        {listing.city.clone().unwrap_or_default()}
                                                        {listing.country.as_ref().map(|c| format!(", {}", c))}
                                                    </div>
                                                    
                                                    // Property Type
                                                    {listing.property_type.clone().map(|pt| view! {
                                                        <div class="listing-card__type">{pt}</div>
                                                    })}
                                                    
                                                    // Specs Bar
                                                    <div class="listing-card__specs">
                                                        {listing.bedrooms.map(|b| view! {
                                                            <div class="listing-card__spec">
                                                                <span class="listing-card__spec-icon">"🛏️"</span>
                                                                <span class="listing-card__spec-value">{b}</span>
                                                                <span class="listing-card__spec-label">"Beds"</span>
                                                            </div>
                                                        })}
                                                        
                                                        {listing.bathrooms.map(|b| view! {
                                                            <div class="listing-card__spec">
                                                                <span class="listing-card__spec-icon">"🚿"</span>
                                                                <span class="listing-card__spec-value">{b}</span>
                                                                <span class="listing-card__spec-label">"Baths"</span>
                                                            </div>
                                                        })}
                                                        
                                                        {listing.size_sqm.map(|s| view! {
                                                            <div class="listing-card__spec">
                                                                <span class="listing-card__spec-icon">"📐"</span>
                                                                <span class="listing-card__spec-value">{format!("{:.0}", s)}</span>
                                                                <span class="listing-card__spec-label">"m²"</span>
                                                            </div>
                                                        })}
                                                    </div>
                                                </div>
                                            </article>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_view()
                        }
                    }
                }}
            </main>
        </div>
    }
}
