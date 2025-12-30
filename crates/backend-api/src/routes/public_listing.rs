//! Public Property Listing Page
//!
//! SEO-optimized public property listing with:
//! - Dynamic OpenGraph meta tags
//! - Schema.org PropertyListing markup
//! - Book Viewing widget (no login required)
//! - Lead capture form

use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::state::AppState;

/// Property details for public display
#[derive(Debug, Serialize, Deserialize)]
pub struct PublicPropertyListing {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub price: Option<f64>,
    pub currency: String,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub area_sqm: Option<f64>,
    pub property_type: String,
    pub status: String,
    pub images: Vec<String>,
    pub amenities: Vec<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub agent_name: Option<String>,
    pub agent_phone: Option<String>,
    pub published_at: Option<String>,
}

/// Viewing request form data
#[derive(Debug, Serialize, Deserialize)]
pub struct ViewingRequest {
    pub property_id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub preferred_date: Option<String>,
    pub preferred_time: Option<String>,
    pub message: Option<String>,
    pub consent_to_contact: bool,
}

/// Build public listing routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/p/:slug", get(public_listing_page))
        .route("/p/:slug/request-viewing", axum::routing::post(submit_viewing_request))
        .route("/sitemap-properties.xml", get(property_sitemap))
}

/// Render public property listing page with SSR
async fn public_listing_page(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    // Parse slug - format: "property-title-uuid"
    let property_id = extract_uuid_from_slug(&slug);
    
    // Fetch property from database
    let property = match fetch_property(&state.pool, property_id).await {
        Ok(p) => p,
        Err(_) => return Html(render_404_page()),
    };
    
    // Generate SEO-optimized HTML
    let html = render_listing_page(&property, &state);
    
    Html(html)
}

/// Extract UUID from slug (last segment after last -)
fn extract_uuid_from_slug(slug: &str) -> Option<Uuid> {
    let parts: Vec<&str> = slug.rsplitn(6, '-').collect();
    if parts.len() >= 5 {
        let uuid_str = parts.iter().take(5).rev().cloned().collect::<Vec<_>>().join("-");
        Uuid::parse_str(&uuid_str).ok()
    } else {
        None
    }
}

/// Fetch property from database
async fn fetch_property(pool: &sqlx::PgPool, id: Option<Uuid>) -> Result<PublicPropertyListing, String> {
    let id = id.ok_or("Invalid property ID")?;
    
    let row = sqlx::query_as::<_, (
        Uuid, String, Option<String>, Option<String>, Option<String>, Option<String>,
        Option<f64>, Option<i32>, Option<i32>, Option<f64>, String, String,
        Option<f64>, Option<f64>
    )>(
        r#"
        SELECT 
            id, 
            data->>'title' as title,
            data->>'description' as description,
            data->>'address' as address,
            data->>'city' as city,
            data->>'country' as country,
            (data->>'price')::float as price,
            (data->>'bedrooms')::int as bedrooms,
            (data->>'bathrooms')::int as bathrooms,
            (data->>'area_sqm')::float as area_sqm,
            COALESCE(data->>'property_type', 'property') as property_type,
            COALESCE(data->>'status', 'available') as status,
            (data->>'latitude')::float as latitude,
            (data->>'longitude')::float as longitude
        FROM entities
        WHERE id = $1 AND is_deleted = FALSE
        "#
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(PublicPropertyListing {
        id: row.0,
        title: row.1,
        description: row.2,
        address: row.3,
        city: row.4,
        country: row.5,
        price: row.6,
        currency: "AED".to_string(),
        bedrooms: row.7,
        bathrooms: row.8,
        area_sqm: row.9,
        property_type: row.10,
        status: row.11,
        images: vec![], // TODO: fetch images
        amenities: vec![],
        latitude: row.12,
        longitude: row.13,
        agent_name: None,
        agent_phone: None,
        published_at: None,
    })
}

/// Render the listing page with OpenGraph and Schema.org
fn render_listing_page(property: &PublicPropertyListing, state: &AppState) -> String {
    let og_image = property.images.first()
        .cloned()
        .unwrap_or_else(|| "https://jirsi.com/default-property.jpg".to_string());
    
    let price_text = property.price
        .map(|p| format!("{} {}", property.currency, format_price(p)))
        .unwrap_or_else(|| "Price on request".to_string());
    
    let location = [&property.city, &property.country]
        .iter()
        .filter_map(|o| o.as_ref().map(|s| s.as_str()))
        .collect::<Vec<_>>()
        .join(", ");
    
    let schema_json = render_schema_org(property);
    
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} | {location} - Jirsi Properties</title>
    <meta name="description" content="{description}">
    
    <!-- OpenGraph Tags -->
    <meta property="og:type" content="website">
    <meta property="og:title" content="{title}">
    <meta property="og:description" content="{og_description}">
    <meta property="og:image" content="{og_image}">
    <meta property="og:url" content="https://jirsi.com/p/{slug}">
    <meta property="og:site_name" content="Jirsi Properties">
    
    <!-- Twitter Card -->
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="{title}">
    <meta name="twitter:description" content="{og_description}">
    <meta name="twitter:image" content="{og_image}">
    
    <!-- Schema.org Structured Data -->
    <script type="application/ld+json">
    {schema_json}
    </script>
    
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <style>
        :root {{
            --primary: #6366f1;
            --primary-dark: #4f46e5;
            --bg: #0f172a;
            --surface: #1e293b;
            --text: #f8fafc;
            --text-muted: #94a3b8;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ 
            font-family: 'Inter', system-ui, sans-serif; 
            background: var(--bg);
            color: var(--text);
            line-height: 1.6;
        }}
        .container {{ max-width: 1200px; margin: 0 auto; padding: 0 20px; }}
        .header {{ 
            padding: 20px 0; 
            background: rgba(15, 23, 42, 0.9);
            backdrop-filter: blur(10px);
            position: sticky;
            top: 0;
            z-index: 100;
            border-bottom: 1px solid rgba(255,255,255,0.1);
        }}
        .logo {{ font-size: 1.5rem; font-weight: 700; color: var(--primary); text-decoration: none; }}
        .hero {{ position: relative; height: 60vh; min-height: 400px; }}
        .hero-image {{ width: 100%; height: 100%; object-fit: cover; }}
        .hero-overlay {{ 
            position: absolute; 
            bottom: 0; 
            left: 0; 
            right: 0; 
            padding: 40px 20px;
            background: linear-gradient(transparent, rgba(0,0,0,0.8));
        }}
        .badge {{ 
            display: inline-block;
            padding: 6px 12px;
            background: var(--primary);
            color: white;
            border-radius: 20px;
            font-size: 0.875rem;
            font-weight: 500;
            margin-bottom: 16px;
        }}
        .title {{ font-size: 2.5rem; font-weight: 700; margin-bottom: 8px; }}
        .location {{ color: var(--text-muted); font-size: 1.125rem; }}
        .content {{ display: grid; grid-template-columns: 2fr 1fr; gap: 40px; padding: 40px 0; }}
        @media (max-width: 768px) {{ .content {{ grid-template-columns: 1fr; }} }}
        .details {{ }}
        .price {{ font-size: 2rem; font-weight: 700; color: var(--primary); margin-bottom: 24px; }}
        .specs {{ display: flex; gap: 24px; margin-bottom: 24px; flex-wrap: wrap; }}
        .spec {{ display: flex; align-items: center; gap: 8px; color: var(--text-muted); }}
        .spec-icon {{ font-size: 1.25rem; }}
        .description {{ color: var(--text-muted); margin-bottom: 32px; }}
        .section-title {{ font-size: 1.25rem; font-weight: 600; margin-bottom: 16px; }}
        .amenities {{ display: flex; flex-wrap: wrap; gap: 8px; }}
        .amenity {{ padding: 8px 16px; background: var(--surface); border-radius: 8px; font-size: 0.875rem; }}
        .sidebar {{ }}
        .card {{ 
            background: var(--surface); 
            border-radius: 16px; 
            padding: 24px;
            border: 1px solid rgba(255,255,255,0.1);
        }}
        .card-title {{ font-size: 1.25rem; font-weight: 600; margin-bottom: 16px; }}
        .form-group {{ margin-bottom: 16px; }}
        .form-label {{ display: block; font-size: 0.875rem; margin-bottom: 6px; color: var(--text-muted); }}
        .form-input {{ 
            width: 100%; 
            padding: 12px 16px; 
            background: rgba(255,255,255,0.05);
            border: 1px solid rgba(255,255,255,0.1);
            border-radius: 8px;
            color: var(--text);
            font-size: 1rem;
        }}
        .form-input:focus {{ outline: none; border-color: var(--primary); }}
        .form-textarea {{ min-height: 80px; resize: vertical; }}
        .btn {{ 
            width: 100%;
            padding: 14px 24px;
            background: var(--primary);
            color: white;
            border: none;
            border-radius: 8px;
            font-size: 1rem;
            font-weight: 600;
            cursor: pointer;
            transition: background 0.2s;
        }}
        .btn:hover {{ background: var(--primary-dark); }}
        .consent {{ display: flex; align-items: start; gap: 8px; margin: 16px 0; font-size: 0.875rem; color: var(--text-muted); }}
        .consent input {{ margin-top: 4px; }}
        .agent {{ display: flex; align-items: center; gap: 12px; margin-top: 24px; padding-top: 24px; border-top: 1px solid rgba(255,255,255,0.1); }}
        .agent-avatar {{ width: 48px; height: 48px; border-radius: 50%; background: var(--primary); display: flex; align-items: center; justify-content: center; }}
        .agent-info {{ }}
        .agent-name {{ font-weight: 600; }}
        .agent-phone {{ color: var(--text-muted); font-size: 0.875rem; }}
    </style>
</head>
<body>
    <header class="header">
        <div class="container">
            <a href="/" class="logo">Jirsi</a>
        </div>
    </header>
    
    <div class="hero">
        <img src="{og_image}" alt="{title}" class="hero-image">
        <div class="hero-overlay">
            <div class="container">
                <span class="badge">{property_type}</span>
                <h1 class="title">{title}</h1>
                <p class="location">📍 {location}</p>
            </div>
        </div>
    </div>
    
    <main class="container">
        <div class="content">
            <div class="details">
                <div class="price">{price_text}</div>
                
                <div class="specs">
                    {bedrooms_html}
                    {bathrooms_html}
                    {area_html}
                </div>
                
                <p class="description">{description}</p>
                
                <h3 class="section-title">Amenities</h3>
                <div class="amenities">
                    <span class="amenity">Air Conditioning</span>
                    <span class="amenity">Parking</span>
                    <span class="amenity">Security</span>
                    <span class="amenity">Gym</span>
                    <span class="amenity">Pool</span>
                </div>
            </div>
            
            <aside class="sidebar">
                <div class="card">
                    <h3 class="card-title">📅 Book a Viewing</h3>
                    <form action="/p/{slug}/request-viewing" method="POST" id="viewing-form">
                        <input type="hidden" name="property_id" value="{property_id}">
                        
                        <div class="form-group">
                            <label class="form-label">Your Name *</label>
                            <input type="text" name="name" class="form-input" required>
                        </div>
                        
                        <div class="form-group">
                            <label class="form-label">Email *</label>
                            <input type="email" name="email" class="form-input" required>
                        </div>
                        
                        <div class="form-group">
                            <label class="form-label">Phone *</label>
                            <input type="tel" name="phone" class="form-input" required>
                        </div>
                        
                        <div class="form-group">
                            <label class="form-label">Preferred Date</label>
                            <input type="date" name="preferred_date" class="form-input">
                        </div>
                        
                        <div class="form-group">
                            <label class="form-label">Message</label>
                            <textarea name="message" class="form-input form-textarea" placeholder="Any questions or special requests?"></textarea>
                        </div>
                        
                        <label class="consent">
                            <input type="checkbox" name="consent_to_contact" required>
                            <span>I agree to be contacted by the agent regarding this property.</span>
                        </label>
                        
                        <button type="submit" class="btn">Request Viewing</button>
                    </form>
                    
                    <div class="agent">
                        <div class="agent-avatar">👤</div>
                        <div class="agent-info">
                            <div class="agent-name">{agent_name}</div>
                            <div class="agent-phone">{agent_phone}</div>
                        </div>
                    </div>
                </div>
            </aside>
        </div>
    </main>
    
    <script>
        document.getElementById('viewing-form').addEventListener('submit', async (e) => {{
            e.preventDefault();
            const form = e.target;
            const data = new FormData(form);
            
            try {{
                const response = await fetch(form.action, {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(Object.fromEntries(data))
                }});
                
                if (response.ok) {{
                    alert('Thank you! We will contact you shortly to arrange your viewing.');
                    form.reset();
                }} else {{
                    alert('Something went wrong. Please try again.');
                }}
            }} catch (err) {{
                alert('Network error. Please try again.');
            }}
        }});
    </script>
</body>
</html>"#,
        title = html_escape(&property.title),
        location = html_escape(&location),
        description = html_escape(&property.description.clone().unwrap_or_default()),
        og_description = html_escape(&property.description.clone().unwrap_or_else(|| format!("{} in {}", property.property_type, location))),
        og_image = html_escape(&og_image),
        slug = slugify(&property.title, property.id),
        schema_json = schema_json,
        property_type = html_escape(&property.property_type),
        price_text = html_escape(&price_text),
        bedrooms_html = property.bedrooms.map(|b| format!(r#"<span class="spec"><span class="spec-icon">🛏️</span> {} Bedrooms</span>"#, b)).unwrap_or_default(),
        bathrooms_html = property.bathrooms.map(|b| format!(r#"<span class="spec"><span class="spec-icon">🚿</span> {} Bathrooms</span>"#, b)).unwrap_or_default(),
        area_html = property.area_sqm.map(|a| format!(r#"<span class="spec"><span class="spec-icon">📐</span> {} sqm</span>"#, a)).unwrap_or_default(),
        property_id = property.id,
        agent_name = property.agent_name.clone().unwrap_or_else(|| "Property Agent".to_string()),
        agent_phone = property.agent_phone.clone().unwrap_or_else(|| "Contact for details".to_string()),
    )
}

/// Render Schema.org PropertyListing structured data
fn render_schema_org(property: &PublicPropertyListing) -> String {
    let images: Vec<String> = property.images.iter()
        .map(|i| format!(r#""{}""#, i))
        .collect();
    
    let geo = property.latitude.zip(property.longitude)
        .map(|(lat, lng)| format!(r#",
        "geo": {{
            "@type": "GeoCoordinates",
            "latitude": {},
            "longitude": {}
        }}"#, lat, lng))
        .unwrap_or_default();
    
    format!(r#"{{
        "@context": "https://schema.org",
        "@type": "RealEstateListing",
        "name": "{}",
        "description": "{}",
        "url": "https://jirsi.com/p/{}",
        "image": [{}],
        "address": {{
            "@type": "PostalAddress",
            "streetAddress": "{}",
            "addressLocality": "{}",
            "addressCountry": "{}"
        }}{},
        "offers": {{
            "@type": "Offer",
            "price": "{}",
            "priceCurrency": "{}"
        }},
        "numberOfRooms": {},
        "floorSize": {{
            "@type": "QuantitativeValue",
            "value": {},
            "unitCode": "MTK"
        }}
    }}"#,
        property.title,
        property.description.clone().unwrap_or_default(),
        slugify(&property.title, property.id),
        images.join(","),
        property.address.clone().unwrap_or_default(),
        property.city.clone().unwrap_or_default(),
        property.country.clone().unwrap_or_default(),
        geo,
        property.price.unwrap_or(0.0),
        property.currency,
        property.bedrooms.unwrap_or(0),
        property.area_sqm.unwrap_or(0.0),
    )
}

/// Submit viewing request and trigger workflow
async fn submit_viewing_request(
    State(state): State<Arc<AppState>>,
    axum::Json(request): axum::Json<ViewingRequest>,
) -> impl IntoResponse {
    // Create lead in database
    let lead_id = Uuid::new_v4();
    
    let _ = sqlx::query(
        r#"
        INSERT INTO entities (id, tenant_id, entity_type_id, data, is_deleted)
        SELECT $1, tenant_id, et.id, $3, FALSE
        FROM entity_types et
        WHERE et.code = 'lead'
        LIMIT 1
        "#
    )
    .bind(lead_id)
    .bind(serde_json::json!({
        "name": request.name,
        "email": request.email,
        "phone": request.phone,
        "source": "website_viewing_request",
        "property_id": request.property_id,
        "preferred_date": request.preferred_date,
        "preferred_time": request.preferred_time,
        "message": request.message,
        "status": "new",
    }))
    .execute(&state.pool)
    .await;
    
    // Trigger inbound lead workflow
    // This would be handled by the workflow engine
    
    axum::Json(serde_json::json!({
        "success": true,
        "lead_id": lead_id,
        "message": "Viewing request submitted successfully"
    }))
}

/// Generate property sitemap XML
async fn property_sitemap(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let properties: Vec<(Uuid, String, String)> = sqlx::query_as(
        r#"
        SELECT id, data->>'title' as title, updated_at::text
        FROM entities e
        JOIN entity_types et ON e.entity_type_id = et.id
        WHERE et.code = 'property' AND e.is_deleted = FALSE
        ORDER BY updated_at DESC
        LIMIT 1000
        "#
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    
    let urls: String = properties.iter()
        .map(|(id, title, updated)| format!(
            r#"  <url>
    <loc>https://jirsi.com/p/{}</loc>
    <lastmod>{}</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.8</priority>
  </url>"#,
            slugify(title, *id),
            &updated[..10]
        ))
        .collect::<Vec<_>>()
        .join("\n");
    
    let xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{urls}
</urlset>"#);
    
    (
        [("Content-Type", "application/xml")],
        xml
    )
}

/// Generate URL-friendly slug
fn slugify(title: &str, id: Uuid) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    
    format!("{}-{}", slug, id)
}

/// Format price with thousands separator
fn format_price(price: f64) -> String {
    let formatted: String = price.round().to_string()
        .chars()
        .rev()
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(",")
        .chars()
        .rev()
        .collect();
    formatted
}

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&#39;")
}

/// Render 404 page
fn render_404_page() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <title>Property Not Found - Jirsi</title>
    <style>
        body { font-family: system-ui; background: #0f172a; color: #f8fafc; display: flex; align-items: center; justify-content: center; min-height: 100vh; margin: 0; }
        .content { text-align: center; }
        h1 { font-size: 4rem; margin: 0; }
        p { color: #94a3b8; margin: 16px 0 32px; }
        a { color: #6366f1; text-decoration: none; }
    </style>
</head>
<body>
    <div class="content">
        <h1>404</h1>
        <p>This property is no longer available.</p>
        <a href="/">← Browse all properties</a>
    </div>
</body>
</html>"#.to_string()
}
