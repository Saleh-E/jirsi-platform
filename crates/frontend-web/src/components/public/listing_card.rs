//! Enhanced Listing Card Component - World-class property card design

use leptos::*;
use serde::{Deserialize, Serialize};

/// Public listing data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicListing {
    pub id: String,
    pub title: String,
    pub property_type: Option<String>,
    pub listing_type: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub area_sqm: Option<f64>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub status: Option<String>,
    pub photos: Option<Vec<String>>,
    pub description: Option<String>,
}

/// Premium listing card with hover effects, badges, and responsive design
#[component]
pub fn ListingCard(
    #[prop(into)] listing: PublicListing,
    #[prop(optional)] on_click: Option<Callback<String>>,
) -> impl IntoView {
    let _listing_id = listing.id.clone();
    let listing_for_click = listing.clone();

    let handle_click = move |_| {
        if let Some(callback) = &on_click {
            callback.call(listing_for_click.id.clone());
        }
    };

    // Format price with currency
    let format_price = move || {
        match listing.price {
            Some(p) => {
                let currency = listing.currency.clone().unwrap_or_else(|| "USD".to_string());
                let symbol = match currency.as_str() {
                    "USD" => "$",
                    "EUR" => "€",
                    "GBP" => "£",
                    "AED" => "AED ",
                    "SAR" => "SAR ",
                    _ => "$",
                };
                format!("{}{}", symbol, format_number(p as i64))
            }
            None => "Price on Request".to_string(),
        }
    };

    // Get first photo or placeholder
    let photo_url = listing.photos
        .as_ref()
        .and_then(|p| p.first().cloned())
        .unwrap_or_else(|| "https://images.unsplash.com/photo-1564013799919-ab600027ffc6?w=800&q=80".to_string());

    let status = listing.status.clone().unwrap_or_else(|| "active".to_string());
    let listing_type = listing.listing_type.clone().unwrap_or_else(|| "sale".to_string());
    let property_type = listing.property_type.clone().unwrap_or_else(|| "Property".to_string());
    let city = listing.city.clone().unwrap_or_default();
    let country = listing.country.clone().unwrap_or_default();

    view! {
        <article
            class="listing-card"
            on:click=handle_click
            tabindex="0"
        >
            // Image Container
            <div class="listing-card__image-container">
                <img
                    src=photo_url
                    alt=listing.title.clone()
                    class="listing-card__image"
                    loading="lazy"
                />
                
                // Gradient Overlay
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
                <span class=format!("listing-card__badge listing-card__badge--type listing-card__badge--{}", listing_type.to_lowercase())>
                    {match listing_type.as_str() {
                        "sale" => "For Sale",
                        "rent" => "For Rent",
                        "lease" => "For Lease",
                        _ => "For Sale"
                    }}
                </span>
                
                // Price Tag
                <div class="listing-card__price">
                    {format_price}
                    {match listing_type.as_str() {
                        "rent" | "lease" => view! { <span class="listing-card__price-suffix">"/mo"</span> }.into_view(),
                        _ => view! {}.into_view(),
                    }}
                </div>
            </div>
            
            // Content
            <div class="listing-card__content">
                <h3 class="listing-card__title">{listing.title.clone()}</h3>
                
                <div class="listing-card__location">
                    <span class="listing-card__location-icon">"📍"</span>
                    {format!("{}{}", city, if !country.is_empty() { format!(", {}", country) } else { String::new() })}
                </div>
                
                // Property Type
                <div class="listing-card__type">{property_type}</div>
                
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
                    
                    {listing.area_sqm.map(|a| view! {
                        <div class="listing-card__spec">
                            <span class="listing-card__spec-icon">"📐"</span>
                            <span class="listing-card__spec-value">{format!("{:.0}", a)}</span>
                            <span class="listing-card__spec-label">"m²"</span>
                        </div>
                    })}
                </div>
            </div>
        </article>
    }
}

/// Format number with thousands separator
fn format_number(n: i64) -> String {
    let s = n.to_string();
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
