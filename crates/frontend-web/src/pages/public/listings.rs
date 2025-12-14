//! Public Listings Page - World-class property gallery with hero section

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
    #[serde(default)]
    pub is_new: Option<bool>,
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

/// Public Listings Page Component - World Class Design
#[component]
pub fn PublicListingsPage() -> impl IntoView {
    let navigate = use_navigate();
    let (search_query, set_search_query) = create_signal(String::new());
    let (property_type_filter, set_property_type_filter) = create_signal("all".to_string());
    
    // Get tenant name from localStorage or use default
    let tenant_name = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item("tenant_name").ok())
        .flatten()
        .unwrap_or_else(|| "Demo Real Estate".to_string());
    
    let listings = create_resource(
        || (),
        |_| async move {
            let url = "http://localhost:3000/public/listings?tenant_slug=demo";
            fetch_json::<ListingsResponse>(url).await.ok()
        }
    );
    
    view! {
        <div class="public-listings">
            // Hero Section
            <section class="hero-section">
                <div class="hero-background">
                    <div class="hero-overlay"></div>
                </div>
                <div class="hero-content">
                    <h1 class="hero-title">"Find Your Dream Home"</h1>
                    <p class="hero-subtitle">"Discover exceptional properties from " {tenant_name.clone()}</p>
                    
                    // Search Bar
                    <div class="hero-search">
                        <div class="search-container">
                            <span class="search-icon">"🔍"</span>
                            <input
                                type="text"
                                class="search-input"
                                placeholder="Search by location, property type..."
                                prop:value=search_query
                                on:input=move |ev| set_search_query.set(event_target_value(&ev))
                            />
                        </div>
                        <select
                            class="type-filter"
                            on:change=move |ev| set_property_type_filter.set(event_target_value(&ev))
                        >
                            <option value="all">"All Types"</option>
                            <option value="apartment">"Apartments"</option>
                            <option value="villa">"Villas"</option>
                            <option value="townhouse">"Townhouses"</option>
                            <option value="penthouse">"Penthouses"</option>
                            <option value="land">"Land"</option>
                        </select>
                        <button class="search-btn">"Search"</button>
                    </div>
                    
                    // Stats
                    {move || listings.get().flatten().map(|r| view! {
                        <div class="hero-stats">
                            <div class="stat">
                                <span class="stat-number">{r.total}</span>
                                <span class="stat-label">"Properties"</span>
                            </div>
                            <div class="stat-divider"></div>
                            <div class="stat">
                                <span class="stat-number">"50+"</span>
                                <span class="stat-label">"Locations"</span>
                            </div>
                            <div class="stat-divider"></div>
                            <div class="stat">
                                <span class="stat-number">"100%"</span>
                                <span class="stat-label">"Verified"</span>
                            </div>
                        </div>
                    })}
                </div>
            </section>
            
            // Main Content
            <main class="listings-main">
                <div class="listings-header-bar">
                    <h2 class="listings-section-title">"Featured Properties"</h2>
                    {move || listings.get().flatten().map(|r| view! {
                        <span class="listings-count">{r.total}" properties found"</span>
                    })}
                </div>
                
                {move || match listings.get() {
                    None => view! {
                        <div class="listings-loading">
                            <div class="loading-spinner"></div>
                            <p>"Loading properties..."</p>
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
                        let filtered: Vec<_> = response.data.into_iter()
                            .filter(|l| {
                                let q = search_query.get().to_lowercase();
                                let pt = property_type_filter.get();
                                
                                let matches_search = q.is_empty() || 
                                    l.title.to_lowercase().contains(&q) ||
                                    l.city.as_ref().map(|c| c.to_lowercase().contains(&q)).unwrap_or(false);
                                
                                let matches_type = pt == "all" || 
                                    l.property_type.as_ref().map(|t| t.to_lowercase() == pt).unwrap_or(false);
                                
                                matches_search && matches_type
                            })
                            .collect();
                        
                        if filtered.is_empty() {
                            view! {
                                <div class="listings-empty">
                                    <div class="listings-empty__icon">"🔍"</div>
                                    <h2 class="listings-empty__title">"No Properties Found"</h2>
                                    <p class="listings-empty__text">
                                        "Try adjusting your search criteria or browse all properties."
                                    </p>
                                    <button 
                                        class="btn btn-primary"
                                        on:click=move |_| {
                                            set_search_query.set(String::new());
                                            set_property_type_filter.set("all".to_string());
                                        }
                                    >
                                        "Clear Filters"
                                    </button>
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <div class="listings-grid">
                                    {filtered.into_iter().map(|listing| {
                                        let id = listing.id.clone();
                                        let nav = navigate.clone();
                                        let cover = get_cover_photo(listing.photos.as_ref());
                                        let is_rent = listing.rent_amount.is_some() && listing.price.is_none();
                                        let display_price = if is_rent { listing.rent_amount } else { listing.price };
                                        let price_str = format_price(display_price, listing.currency.clone());
                                        let status = listing.status.clone().unwrap_or_else(|| "active".to_string());
                                        let listing_type = if is_rent { "rent" } else { "sale" };
                                        let is_new = listing.is_new.unwrap_or(false);
                                        
                                        let on_click = move |_| {
                                            nav(&format!("/listings/{}", id), Default::default());
                                        };
                                        
                                        view! {
                                            <article class="listing-card" on:click=on_click tabindex="0">
                                                // Image Container with 16:9 aspect ratio
                                                <div class="listing-card__image-container">
                                                    <img
                                                        src=cover.clone()
                                                        alt=listing.title.clone()
                                                        class="listing-card__image"
                                                        loading="lazy"
                                                    />
                                                    <div class="listing-card__overlay"></div>
                                                    
                                                    // Badges
                                                    <div class="listing-card__badges">
                                                        {is_new.then(|| view! {
                                                            <span class="listing-card__badge listing-card__badge--new">"New"</span>
                                                        })}
                                                        
                                                        <span class=format!("listing-card__badge listing-card__badge--{}", status.to_lowercase())>
                                                            {match status.as_str() {
                                                                "active" => "Active",
                                                                "sold" => "Sold",
                                                                "pending" => "Pending",
                                                                "rented" => "Rented",
                                                                _ => "Active"
                                                            }}
                                                        </span>
                                                    </div>
                                                    
                                                    // Type Badge (top right)
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
                                                    
                                                    // Specs Bar with icons
                                                    <div class="listing-card__specs">
                                                        {listing.bedrooms.map(|b| view! {
                                                            <div class="listing-card__spec">
                                                                <span class="listing-card__spec-icon">"🛏️"</span>
                                                                <span class="listing-card__spec-value">{b}</span>
                                                            </div>
                                                        })}
                                                        
                                                        {listing.bathrooms.map(|b| view! {
                                                            <div class="listing-card__spec">
                                                                <span class="listing-card__spec-icon">"🚿"</span>
                                                                <span class="listing-card__spec-value">{b}</span>
                                                            </div>
                                                        })}
                                                        
                                                        {listing.size_sqm.map(|s| view! {
                                                            <div class="listing-card__spec">
                                                                <span class="listing-card__spec-icon">"📐"</span>
                                                                <span class="listing-card__spec-value">{format!("{:.0} sqm", s)}</span>
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
