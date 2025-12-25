//! Command Handlers API
//! 
//! REST endpoints for executing Deal commands

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
    routing::{post, get},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

use crate::{
    cqrs::{*, commands::*, events::*, aggregates::*},
    state::AppState,
};

/// Create Deal endpoint
#[derive(Debug, Deserialize)]
pub struct CreateDealRequest {
    pub title: String,
    pub value: Option<Decimal>,
    pub stage: Option<String>,
    pub contact_id: Option<Uuid>,
    pub property_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CreateDealResponse {
    pub deal_id: Uuid,
    pub version: u64,
}

async fn create_deal(
    State(state): State<AppState>,
    Json(req): Json<CreateDealRequest>,
) -> Result<Json<CreateDealResponse>, (StatusCode, String)> {
    // Get tenant and user from auth (simplified for now)
    let tenant_id = Uuid::new_v4(); // TODO: Extract from JWT
    let user_id = Uuid::new_v4();   // TODO: Extract from JWT
    
    // Build command
    let mut cmd = CreateDealCommand::new(tenant_id, req.title, user_id);
    
    if let Some(value) = req.value {
        cmd = cmd.with_value(value);
    }
    
    if let Some(stage) = req.stage {
        cmd = cmd.with_stage(stage);
    }
    
    if let Some(contact_id) = req.contact_id {
        cmd = cmd.with_contact(contact_id);
    }
    
    if let Some(property_id) = req.property_id {
        cmd = cmd.with_property(property_id);
    }
    
    // Execute command
    let event = Deal::create(cmd)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    
    let deal_id = event.deal_id();
    
    // Save event to event store
    state.event_store
        .save_event(deal_id, "Deal", 0, &event, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Update read model via projection
    state.deal_projection
        .project(&event)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    
    // Cache invalidation skipped - not available in current AppState
    
    Ok(Json(CreateDealResponse {
        deal_id,
        version: 1,
    }))
}

/// Update Deal Stage endpoint
#[derive(Debug, Deserialize)]
pub struct UpdateStageRequest {
    pub new_stage: String,
    pub reason: Option<String>,
}

async fn update_deal_stage(
    State(state): State<AppState>,
    Path(deal_id): Path<Uuid>,
    Json(req): Json<UpdateStageRequest>,
) -> Result<Json<()>, (StatusCode, String)> {
    let user_id = Uuid::new_v4(); // TODO: Extract from JWT
    
    // Load aggregate from event store
    let deal = state.event_store
        .load_aggregate(deal_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    
    // Execute command
    let mut cmd = UpdateDealStageCommand::new(deal_id, req.new_stage, user_id);
    if let Some(reason) = req.reason {
        cmd = cmd.with_reason(reason);
    }
    
    let event = deal.update_stage(cmd)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    
    // Save event
    state.event_store
        .save_event(deal_id, "Deal", deal.version, &event, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Update read model
    state.deal_projection
        .project(&event)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    
    // Cache invalidation skipped - not available in current AppState
    
    Ok(Json(()))
}

/// Get Deal event history (time travel!)
#[derive(Debug, Serialize)]
pub struct DealEventResponse {
    pub event_type: String,
    pub version: i64,
    pub data: serde_json::Value,
    pub timestamp: String,
}

async fn get_deal_history(
    State(state): State<AppState>,
    Path(deal_id): Path<Uuid>,
) -> Result<Json<Vec<DealEventResponse>>, (StatusCode, String)> {
    let events = state.event_store
        .get_event_stream(deal_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    
    let response: Vec<DealEventResponse> = events
        .into_iter()
        .map(|(event, version)| {
            let event_type = match &event {
                DealEvent::Created { .. } => "Created",
                DealEvent::StageUpdated { .. } => "StageUpdated",
                DealEvent::ValueAdded { .. } => "ValueAdded",
                DealEvent::ContactAssigned { .. } => "ContactAssigned",  
                DealEvent::PropertyAssigned { .. } => "PropertyAssigned",
                DealEvent::Closed { .. } => "Closed",
            };
            
            DealEventResponse {
                event_type: event_type.to_string(),
                version,
                data: serde_json::to_value(&event).unwrap(),
                timestamp: event.timestamp().to_rfc3339(),
            }
        })
        .collect();
    
    Ok(Json(response))
}

/// Router for Deal commands
pub fn deal_command_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/deals", post(create_deal))
        .route("/api/v1/deals/:id/stage", post(update_deal_stage))
        .route("/api/v1/deals/:id/history", get(get_deal_history))
}
