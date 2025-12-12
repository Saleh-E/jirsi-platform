//! Properties entity definitions (Phase 3)

use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Property status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyStatus {
    Draft,
    Available,
    Reserved,
    UnderOffer,
    Sold,
    Leased,
    Withdrawn,
}

/// Property type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    Apartment,
    House,
    Villa,
    Land,
    Commercial,
    Office,
    Retail,
    Industrial,
    Other,
}

/// Property unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyUnit {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub reference: String,
    pub title: String,
    pub description: Option<String>,
    pub property_type: PropertyType,
    pub status: PropertyStatus,
    pub address: String,
    pub city: String,
    pub country: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub price: Option<i64>, // cents
    pub currency: String,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub area_sqm: Option<f64>,
    pub year_built: Option<i32>,
    pub features: Vec<String>,
    pub images: Vec<Uuid>,
    pub owner_contact_id: Option<Uuid>,
    pub agent_user_id: Option<Uuid>,
    pub custom_fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Viewing (property showing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewing {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub property_id: Uuid,
    pub contact_id: Uuid,
    pub agent_id: Uuid,
    pub scheduled_at: DateTime<Utc>,
    pub duration_minutes: i32,
    pub status: ViewingStatus,
    pub notes: Option<String>,
    pub feedback: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewingStatus {
    Scheduled,
    Completed,
    Cancelled,
    NoShow,
}

/// Offer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Offer {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub property_id: Uuid,
    pub contact_id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub status: OfferStatus,
    pub conditions: Option<String>,
    pub valid_until: Option<NaiveDate>,
    pub submitted_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfferStatus {
    Pending,
    Accepted,
    Rejected,
    Countered,
    Withdrawn,
    Expired,
}
