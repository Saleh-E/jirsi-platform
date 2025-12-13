//! Properties API routes

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, postgres::PgRow};
use std::sync::Arc;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/properties", get(list_properties).post(create_property))
        .route("/properties/:id", get(get_property).put(update_property).delete(delete_property))
        .route("/properties/:id/viewings", get(list_viewings).post(create_viewing))
        .route("/properties/:id/offers", get(list_offers).post(create_offer))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PropertyQuery {
    pub tenant_id: Uuid,
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
    pub status: Option<String>,
    pub city: Option<String>,
}

fn default_limit() -> i32 { 50 }

#[derive(Debug, Serialize)]
pub struct PropertyListResponse {
    pub data: Vec<serde_json::Value>,
    pub total: i64,
}

async fn list_properties(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PropertyQuery>,
) -> Result<Json<PropertyListResponse>, ApiError> {
    let count_row: PgRow = sqlx::query("SELECT COUNT(*) as count FROM properties WHERE tenant_id = $1 AND deleted_at IS NULL")
        .bind(query.tenant_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e: sqlx::Error| ApiError::Internal(e.to_string()))?;
    let total: i64 = count_row.try_get("count").unwrap_or(0);

    let rows: Vec<PgRow> = sqlx::query(
        r#"
        SELECT id, reference, title, property_type, status, address, city, country, 
               price, currency, bedrooms, bathrooms, area_sqm, created_at
        FROM properties 
        WHERE tenant_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(query.tenant_id)
    .bind(query.limit as i64)
    .bind(query.offset as i64)
    .fetch_all(&state.pool)
    .await
    .map_err(|e: sqlx::Error| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|row: &PgRow| {
        serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
            "reference": row.try_get::<String, _>("reference").unwrap_or_default(),
            "title": row.try_get::<String, _>("title").unwrap_or_default(),
            "property_type": row.try_get::<String, _>("property_type").unwrap_or_default(),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "address": row.try_get::<String, _>("address").unwrap_or_default(),
            "city": row.try_get::<String, _>("city").unwrap_or_default(),
            "country": row.try_get::<String, _>("country").unwrap_or_default(),
            "price": row.try_get::<Option<i64>, _>("price").ok().flatten(),
            "currency": row.try_get::<String, _>("currency").unwrap_or_default(),
            "bedrooms": row.try_get::<Option<i32>, _>("bedrooms").ok().flatten(),
            "bathrooms": row.try_get::<Option<i32>, _>("bathrooms").ok().flatten(),
            "area_sqm": row.try_get::<Option<f64>, _>("area_sqm").ok().flatten(),
        })
    }).collect();

    Ok(Json(PropertyListResponse { data, total }))
}

#[derive(Debug, Deserialize)]
pub struct CreatePropertyRequest {
    pub reference: String,
    pub title: String,
    pub description: Option<String>,
    pub property_type: Option<String>,
    pub address: String,
    pub city: String,
    pub country: Option<String>,
    pub price: Option<i64>,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub area_sqm: Option<f64>,
}

async fn create_property(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PropertyQuery>,
    Json(body): Json<CreatePropertyRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    
    sqlx::query(
        r#"INSERT INTO properties (id, tenant_id, reference, title, description, property_type, 
           address, city, country, price, bedrooms, bathrooms, area_sqm, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"#
    )
    .bind(id)
    .bind(query.tenant_id)
    .bind(&body.reference)
    .bind(&body.title)
    .bind(&body.description)
    .bind(body.property_type.as_deref().unwrap_or("apartment"))
    .bind(&body.address)
    .bind(&body.city)
    .bind(body.country.as_deref().unwrap_or("USA"))
    .bind(body.price)
    .bind(body.bedrooms)
    .bind(body.bathrooms)
    .bind(body.area_sqm)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e: sqlx::Error| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "id": id, "created": true })))
}

async fn get_property(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<PropertyQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let row: PgRow = sqlx::query(
        r#"SELECT * FROM properties WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"#
    )
    .bind(id)
    .bind(query.tenant_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e: sqlx::Error| ApiError::Internal(e.to_string()))?
    .ok_or(ApiError::NotFound("Property not found".to_string()))?;

    Ok(Json(serde_json::json!({
        "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
        "reference": row.try_get::<String, _>("reference").unwrap_or_default(),
        "title": row.try_get::<String, _>("title").unwrap_or_default(),
        "description": row.try_get::<Option<String>, _>("description").ok().flatten(),
        "property_type": row.try_get::<String, _>("property_type").unwrap_or_default(),
        "status": row.try_get::<String, _>("status").unwrap_or_default(),
        "address": row.try_get::<String, _>("address").unwrap_or_default(),
        "city": row.try_get::<String, _>("city").unwrap_or_default(),
        "country": row.try_get::<String, _>("country").unwrap_or_default(),
        "price": row.try_get::<Option<i64>, _>("price").ok().flatten(),
        "currency": row.try_get::<String, _>("currency").unwrap_or_default(),
        "bedrooms": row.try_get::<Option<i32>, _>("bedrooms").ok().flatten(),
        "bathrooms": row.try_get::<Option<i32>, _>("bathrooms").ok().flatten(),
        "area_sqm": row.try_get::<Option<f64>, _>("area_sqm").ok().flatten(),
    })))
}

async fn update_property(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<PropertyQuery>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let now = chrono::Utc::now();
    
    sqlx::query(
        r#"UPDATE properties SET 
           title = COALESCE($3, title),
           status = COALESCE($4, status),
           price = COALESCE($5, price),
           updated_at = $6
           WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"#
    )
    .bind(id)
    .bind(query.tenant_id)
    .bind(body.get("title").and_then(|v| v.as_str()))
    .bind(body.get("status").and_then(|v| v.as_str()))
    .bind(body.get("price").and_then(|v| v.as_i64()))
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e: sqlx::Error| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "id": id, "updated": true })))
}

async fn delete_property(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<PropertyQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let now = chrono::Utc::now();
    
    sqlx::query("UPDATE properties SET deleted_at = $3 WHERE id = $1 AND tenant_id = $2")
        .bind(id)
        .bind(query.tenant_id)
        .bind(now)
        .execute(&state.pool)
        .await
        .map_err(|e: sqlx::Error| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "id": id, "deleted": true })))
}

// Viewings
async fn list_viewings(
    State(state): State<Arc<AppState>>,
    Path(property_id): Path<Uuid>,
    Query(query): Query<PropertyQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let rows: Vec<PgRow> = sqlx::query(
        r#"SELECT v.*, c.first_name, c.last_name 
           FROM viewings v 
           LEFT JOIN contacts c ON v.contact_id = c.id
           WHERE v.property_id = $1 AND v.tenant_id = $2
           ORDER BY v.scheduled_at DESC"#
    )
    .bind(property_id)
    .bind(query.tenant_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e: sqlx::Error| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|row: &PgRow| {
        serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
            "contact_name": format!("{} {}", 
                row.try_get::<Option<String>, _>("first_name").ok().flatten().unwrap_or_default(),
                row.try_get::<Option<String>, _>("last_name").ok().flatten().unwrap_or_default()
            ),
            "scheduled_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("scheduled_at").ok(),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
        })
    }).collect();

    Ok(Json(serde_json::json!({ "data": data })))
}

#[derive(Debug, Deserialize)]
pub struct CreateViewingRequest {
    pub contact_id: Uuid,
    pub scheduled_at: chrono::DateTime<chrono::Utc>,
    pub notes: Option<String>,
}

async fn create_viewing(
    State(state): State<Arc<AppState>>,
    Path(property_id): Path<Uuid>,
    Query(query): Query<PropertyQuery>,
    Json(body): Json<CreateViewingRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = Uuid::new_v4();
    
    // Get any agent - simplified assignment
    let agent_id: Uuid = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE tenant_id = $1 LIMIT 1")
        .bind(query.tenant_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e: sqlx::Error| ApiError::Internal(e.to_string()))?;

    sqlx::query(
        r#"INSERT INTO viewings (id, tenant_id, property_id, contact_id, agent_id, scheduled_at, notes, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())"#
    )
    .bind(id)
    .bind(query.tenant_id)
    .bind(property_id)
    .bind(body.contact_id)
    .bind(agent_id)
    .bind(body.scheduled_at)
    .bind(&body.notes)
    .execute(&state.pool)
    .await
    .map_err(|e: sqlx::Error| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "id": id, "created": true })))
}

// Offers
async fn list_offers(
    State(state): State<Arc<AppState>>,
    Path(property_id): Path<Uuid>,
    Query(query): Query<PropertyQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let rows: Vec<PgRow> = sqlx::query(
        r#"SELECT o.*, c.first_name, c.last_name 
           FROM offers o 
           LEFT JOIN contacts c ON o.contact_id = c.id
           WHERE o.property_id = $1 AND o.tenant_id = $2
           ORDER BY o.submitted_at DESC"#
    )
    .bind(property_id)
    .bind(query.tenant_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e: sqlx::Error| ApiError::Internal(e.to_string()))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|row: &PgRow| {
        serde_json::json!({
            "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
            "contact_name": format!("{} {}", 
                row.try_get::<Option<String>, _>("first_name").ok().flatten().unwrap_or_default(),
                row.try_get::<Option<String>, _>("last_name").ok().flatten().unwrap_or_default()
            ),
            "amount": row.try_get::<i64, _>("amount").unwrap_or(0),
            "currency": row.try_get::<String, _>("currency").unwrap_or_default(),
            "status": row.try_get::<String, _>("status").unwrap_or_default(),
            "submitted_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("submitted_at").ok(),
        })
    }).collect();

    Ok(Json(serde_json::json!({ "data": data })))
}

#[derive(Debug, Deserialize)]
pub struct CreateOfferRequest {
    pub contact_id: Uuid,
    pub amount: i64,
    pub conditions: Option<String>,
}

async fn create_offer(
    State(state): State<Arc<AppState>>,
    Path(property_id): Path<Uuid>,
    Query(query): Query<PropertyQuery>,
    Json(body): Json<CreateOfferRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let id = Uuid::new_v4();
    
    sqlx::query(
        r#"INSERT INTO offers (id, tenant_id, property_id, contact_id, amount, conditions, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())"#
    )
    .bind(id)
    .bind(query.tenant_id)
    .bind(property_id)
    .bind(body.contact_id)
    .bind(body.amount)
    .bind(&body.conditions)
    .execute(&state.pool)
    .await
    .map_err(|e: sqlx::Error| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "id": id, "created": true })))
}
