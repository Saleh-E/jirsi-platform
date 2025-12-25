//! CRDT Sync API - Real-time collaborative text editing endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
    routing::post,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use core_models::crdt::{CrdtText, CrdtError};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CrdtUpdateRequest {
    /// Field name (e.g., "description", "notes")
    pub field: String,
    
    /// Client's current state vector
    pub state_vector: Vec<u8>,
    
    /// Update to apply (Yrs encoded)
    #[serde(default)]
    pub update: Option<Vec<u8>>,
}

#[derive(Debug, Serialize)]
pub struct CrdtUpdateResponse {
    /// Server's update since client's state
    pub update: Vec<u8>,
    
    /// New state vector
    pub state_vector: Vec<u8>,
    
    /// Current text (for verification)
    pub text: Option<String>,
}

/// Sync CRDT text field
async fn sync_crdt_field(
    State(_state): State<AppState>,
    Path((entity_id, field)): Path<(Uuid, String)>,
    Json(req): Json<CrdtUpdateRequest>,
) -> Result<Json<CrdtUpdateResponse>, (StatusCode, String)> {
    // TODO: Load from database
    let mut server_crdt = CrdtText::new(); // Placeholder
    
    // Apply client's update if provided  
    if let Some(update) = req.update {
        server_crdt.apply_update(&update)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        
        // TODO: Save to database
    }
    
    // Get update since client's state
    let update = server_crdt.get_update_since(&req.state_vector)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(CrdtUpdateResponse {
        update,
        state_vector: server_crdt.state_vector.clone(),
        text: server_crdt.get_text().map(|s| s.to_string()),
    }))
}

/// Router for CRDT sync endpoints
pub fn crdt_sync_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/crdt/:entity_id/:field", post(sync_crdt_field))
}
