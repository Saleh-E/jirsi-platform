//! Analytics API routes for dashboard

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::state::AppState;
use core_analytics::{DashboardResponse, get_dashboard};

/// Analytics routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/dashboard", get(get_dashboard_handler))
}

/// Query params for dashboard
#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    #[serde(default)]
    pub range: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<String>,
}

/// Dashboard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// GET /analytics/dashboard
async fn get_dashboard_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DashboardQuery>,
) -> Result<Json<ApiResponse<DashboardResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Get tenant_id from query or use default
    let tenant_id = params.tenant_id.unwrap_or_else(|| "demo".to_string());
    let range = params.range.unwrap_or_else(|| "this_month".to_string());
    
    match get_dashboard(&state.pool, &tenant_id, &range).await {
        Ok(data) => Ok(Json(ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Dashboard query error: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to fetch dashboard: {}", e)),
                }),
            ))
        }
    }
}
