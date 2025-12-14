//! Public API Routes
//! 
//! Unauthenticated endpoints for public-facing tenant websites.
//! These endpoints require tenant resolution via middleware.

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::tenant::{ResolvedTenant, TenantBranding};
use crate::state::AppState;

/// Build public routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tenant", get(get_tenant_info))
        .route("/listings", get(get_listings))
        .route("/listings/:id", get(get_listing_detail))
        .route("/inquire", post(submit_inquiry))
}

// ============================================================================
// DTOs (Data Transfer Objects)
// ============================================================================

/// Public tenant information response
#[derive(Debug, Serialize)]
pub struct PublicTenantResponse {
    pub id: Uuid,
    pub name: String,
    #[serde(flatten)]
    pub branding: TenantBranding,
}

/// Public property listing (safe subset of data)
#[derive(Debug, Serialize, FromRow)]
pub struct PublicListing {
    pub id: Uuid,
    pub reference: Option<String>,
    pub title: String,
    pub property_type: Option<String>,
    pub usage: Option<String>,
    pub status: Option<String>,
    pub city: Option<String>,
    pub area: Option<String>,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub size_sqm: Option<f64>,
    pub price: Option<f64>,
    pub rent_amount: Option<f64>,
    pub currency: Option<String>,
    pub photos: Option<serde_json::Value>,
    pub listed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Full property detail for public view
#[derive(Debug, Serialize, FromRow)]
pub struct PublicListingDetail {
    pub id: Uuid,
    pub reference: Option<String>,
    pub title: String,
    pub property_type: Option<String>,
    pub usage: Option<String>,
    pub status: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub area: Option<String>,
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub size_sqm: Option<f64>,
    pub floor: Option<i32>,
    pub total_floors: Option<i32>,
    pub year_built: Option<i32>,
    pub price: Option<f64>,
    pub rent_amount: Option<f64>,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub amenities: Option<serde_json::Value>,
    pub photos: Option<serde_json::Value>,
    pub listed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Listing query parameters
#[derive(Debug, Deserialize)]
pub struct ListingQuery {
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub property_type: Option<String>,
    pub city: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

/// Inquiry submission request
#[derive(Debug, Deserialize)]
pub struct InquiryRequest {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub message: Option<String>,
    pub listing_id: Option<Uuid>,
}

/// Inquiry submission response
#[derive(Debug, Serialize)]
pub struct InquiryResponse {
    pub success: bool,
    pub message: String,
    pub inquiry_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
}

/// Paginated listings response
#[derive(Debug, Serialize)]
pub struct ListingsResponse {
    pub data: Vec<PublicListing>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /public/tenant - Get tenant branding information
async fn get_tenant_info(
    Extension(tenant): Extension<ResolvedTenant>,
) -> impl IntoResponse {
    let response = PublicTenantResponse {
        id: tenant.id,
        name: tenant.name.clone(),
        branding: tenant.get_branding(),
    };
    Json(response)
}

/// GET /public/listings - Get published property listings
async fn get_listings(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<ResolvedTenant>,
    Query(params): Query<ListingQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;
    
    // Build dynamic query
    let mut conditions = vec!["tenant_id = $1", "status = 'active'", "is_published = true", "deleted_at IS NULL"];
    let mut param_idx = 2;
    
    // Build base query with filters
    let base_query = format!(
        r#"
        SELECT id, reference, title, property_type, usage, status,
               city, area, bedrooms, bathrooms, 
               CAST(size_sqm AS FLOAT8) as size_sqm,
               CAST(price AS FLOAT8) as price, 
               CAST(rent_amount AS FLOAT8) as rent_amount, 
               currency, photos, listed_at
        FROM properties
        WHERE {}
        ORDER BY listed_at DESC NULLS LAST, created_at DESC
        LIMIT {} OFFSET {}
        "#,
        conditions.join(" AND "),
        per_page,
        offset
    );
    
    let count_query = format!(
        "SELECT COUNT(*) FROM properties WHERE {}",
        conditions.join(" AND ")
    );
    
    // Execute queries
    let listings: Vec<PublicListing> = match sqlx::query_as(&base_query)
        .bind(tenant.id)
        .fetch_all(&state.pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Failed to fetch listings: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch listings"}))
            ).into_response();
        }
    };
    
    let total: i64 = match sqlx::query_scalar(&count_query)
        .bind(tenant.id)
        .fetch_one(&state.pool)
        .await
    {
        Ok(count) => count,
        Err(_) => 0,
    };
    
    Json(ListingsResponse {
        data: listings,
        total,
        page,
        per_page,
    }).into_response()
}

/// GET /public/listings/:id - Get single property detail
async fn get_listing_detail(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<ResolvedTenant>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let query = r#"
        SELECT id, reference, title, property_type, usage, status,
               country, city, area, address,
               CAST(latitude AS FLOAT8) as latitude,
               CAST(longitude AS FLOAT8) as longitude,
               bedrooms, bathrooms, 
               CAST(size_sqm AS FLOAT8) as size_sqm,
               floor, total_floors, year_built,
               CAST(price AS FLOAT8) as price, 
               CAST(rent_amount AS FLOAT8) as rent_amount, 
               currency, description, amenities, photos, listed_at
        FROM properties
        WHERE id = $1 AND tenant_id = $2 AND is_published = true AND deleted_at IS NULL
    "#;
    
    match sqlx::query_as::<_, PublicListingDetail>(query)
        .bind(id)
        .bind(tenant.id)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(listing)) => Json(listing).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Property not found"}))
        ).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch property: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch property"}))
            ).into_response()
        }
    }
}

/// POST /public/inquire - Submit inquiry form
async fn submit_inquiry(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<ResolvedTenant>,
    Json(payload): Json<InquiryRequest>,
) -> impl IntoResponse {
    // Validate email
    if !payload.email.contains('@') {
        return (
            StatusCode::BAD_REQUEST,
            Json(InquiryResponse {
                success: false,
                message: "Invalid email address".to_string(),
                inquiry_id: None,
                contact_id: None,
            })
        ).into_response();
    }
    
    // 1. Check if contact exists by email
    let existing_contact: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM contacts WHERE tenant_id = $1 AND email = $2 AND deleted_at IS NULL"
    )
    .bind(tenant.id)
    .bind(&payload.email)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    
    let contact_id = match existing_contact {
        Some(id) => id,
        None => {
            // Create new contact
            let new_id = Uuid::new_v4();
            let names: Vec<&str> = payload.name.splitn(2, ' ').collect();
            let first_name = names.get(0).unwrap_or(&"");
            let last_name = names.get(1).unwrap_or(&"");
            
            match sqlx::query(
                r#"
                INSERT INTO contacts (id, tenant_id, first_name, last_name, email, phone, lead_source, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, 'Website Inquiry', NOW(), NOW())
                "#
            )
            .bind(new_id)
            .bind(tenant.id)
            .bind(first_name)
            .bind(last_name)
            .bind(&payload.email)
            .bind(&payload.phone)
            .execute(&state.pool)
            .await
            {
                Ok(_) => new_id,
                Err(e) => {
                    tracing::error!("Failed to create contact: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(InquiryResponse {
                            success: false,
                            message: "Failed to process inquiry".to_string(),
                            inquiry_id: None,
                            contact_id: None,
                        })
                    ).into_response();
                }
            }
        }
    };
    
    // 2. Create interaction (inquiry)
    let interaction_id = Uuid::new_v4();
    let interaction_data = serde_json::json!({
        "type": "inquiry",
        "message": payload.message,
        "listing_id": payload.listing_id,
        "source": "public_website"
    });
    
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO interactions (id, tenant_id, entity_type, entity_id, interaction_type, data, created_by, created_at)
        VALUES ($1, $2, 'contact', $3, 'inquiry', $4, NULL, NOW())
        "#
    )
    .bind(interaction_id)
    .bind(tenant.id)
    .bind(contact_id)
    .bind(&interaction_data)
    .execute(&state.pool)
    .await
    {
        tracing::error!("Failed to create interaction: {}", e);
        // Continue anyway - don't fail the whole request
    }
    
    // 3. Create deal if listing_id provided
    if let Some(listing_id) = payload.listing_id {
        let deal_id = Uuid::new_v4();
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO deals (id, tenant_id, name, stage, contact_id, property_id, created_at, updated_at)
            VALUES ($1, $2, $3, 'new', $4, $5, NOW(), NOW())
            "#
        )
        .bind(deal_id)
        .bind(tenant.id)
        .bind(format!("Inquiry: {}", payload.name))
        .bind(contact_id)
        .bind(listing_id)
        .execute(&state.pool)
        .await
        {
            tracing::error!("Failed to create deal: {}", e);
            // Continue anyway
        }
    }
    
    // 4. TODO: Trigger workflow
    // The workflow system should pick this up from the interaction creation
    
    Json(InquiryResponse {
        success: true,
        message: "Thank you for your inquiry! We'll be in touch soon.".to_string(),
        inquiry_id: Some(interaction_id),
        contact_id: Some(contact_id),
    }).into_response()
}
