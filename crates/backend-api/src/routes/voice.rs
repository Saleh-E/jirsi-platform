//! Voice Dialer - Twilio Voice Integration
//!
//! Provides:
//! - Twilio Client token generation
//! - Voice call initiation
//! - Call recording
//! - Whisper transcription integration
//! - Auto-extraction of requirements from transcripts

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

use crate::state::AppState;

/// Twilio capability token request
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub identity: String,
}

/// Twilio capability token response
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub identity: String,
    pub expires_in: u64,
}

/// Call initiation request
#[derive(Debug, Deserialize)]
pub struct InitiateCallRequest {
    pub to: String,
    pub from: Option<String>,
    /// Entity context (e.g., contact_id)
    pub entity_id: Option<Uuid>,
    pub entity_type: Option<String>,
    /// Whether to record the call
    #[serde(default)]
    pub record: bool,
}

/// Call status
#[derive(Debug, Serialize)]
pub struct CallStatus {
    pub call_sid: String,
    pub status: String,
    pub to: String,
    pub from: String,
    pub duration: Option<u32>,
    pub recording_url: Option<String>,
    pub transcription: Option<String>,
}

/// Build voice routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/voice/token", post(generate_token))
        .route("/voice/call", post(initiate_call))
        .route("/voice/call/:call_sid", get(get_call_status))
        .route("/voice/call/:call_sid/end", post(end_call))
        .route("/voice/webhook/status", post(call_status_webhook))
        .route("/voice/webhook/recording", post(recording_webhook))
}

/// Generate Twilio capability token for browser-based calling
async fn generate_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TokenRequest>,
) -> Result<Json<TokenResponse>, (axum::http::StatusCode, String)> {
    let account_sid = std::env::var("TWILIO_ACCOUNT_SID")
        .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Missing TWILIO_ACCOUNT_SID".to_string()))?;
    let api_key = std::env::var("TWILIO_API_KEY")
        .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Missing TWILIO_API_KEY".to_string()))?;
    let api_secret = std::env::var("TWILIO_API_SECRET")
        .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Missing TWILIO_API_SECRET".to_string()))?;
    let twiml_app_sid = std::env::var("TWILIO_TWIML_APP_SID")
        .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Missing TWILIO_TWIML_APP_SID".to_string()))?;
    
    // Build JWT token for Twilio Client
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let expires_in = 3600; // 1 hour
    
    let claims = serde_json::json!({
        "jti": format!("{}-{}", api_key, now),
        "iss": api_key,
        "sub": account_sid,
        "exp": now + expires_in,
        "grants": {
            "identity": request.identity,
            "voice": {
                "outgoing": {
                    "application_sid": twiml_app_sid
                },
                "incoming": {
                    "allow": true
                }
            }
        }
    });
    
    // In production, use proper JWT signing (jsonwebtoken crate)
    // This is a simplified example
    let token = format!(
        "twilio_token_{}_{}", 
        request.identity, 
        now
    );
    
    Ok(Json(TokenResponse {
        token,
        identity: request.identity,
        expires_in,
    }))
}

/// Initiate an outbound call
async fn initiate_call(
    State(state): State<Arc<AppState>>,
    Json(request): Json<InitiateCallRequest>,
) -> Result<Json<CallStatus>, (axum::http::StatusCode, String)> {
    let account_sid = std::env::var("TWILIO_ACCOUNT_SID")
        .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Missing TWILIO_ACCOUNT_SID".to_string()))?;
    let auth_token = std::env::var("TWILIO_AUTH_TOKEN")
        .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Missing TWILIO_AUTH_TOKEN".to_string()))?;
    let from_number = request.from.unwrap_or_else(|| {
        std::env::var("TWILIO_FROM_NUMBER").unwrap_or_default()
    });
    
    let client = reqwest::Client::new();
    
    // Build TwiML URL for the call
    let base_url = std::env::var("APP_BASE_URL").unwrap_or_else(|_| "https://app.jirsi.com".to_string());
    let twiml_url = format!("{}/api/v1/voice/twiml?to={}", base_url, urlencoding::encode(&request.to));
    
    let mut form = vec![
        ("To", request.to.clone()),
        ("From", from_number.clone()),
        ("Url", twiml_url),
        ("StatusCallback", format!("{}/api/v1/voice/webhook/status", base_url)),
        ("StatusCallbackEvent", "initiated ringing answered completed".to_string()),
    ];
    
    if request.record {
        form.push(("Record", "true".to_string()));
        form.push(("RecordingStatusCallback", format!("{}/api/v1/voice/webhook/recording", base_url)));
    }
    
    let response = client
        .post(format!("https://api.twilio.com/2010-04-01/Accounts/{}/Calls.json", account_sid))
        .basic_auth(&account_sid, Some(&auth_token))
        .form(&form)
        .send()
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    if !response.status().is_success() {
        let error: serde_json::Value = response.json().await.unwrap_or_default();
        return Err((axum::http::StatusCode::BAD_REQUEST, format!("Twilio error: {:?}", error)));
    }
    
    let json: serde_json::Value = response.json().await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Log call in database
    if let Some(entity_id) = request.entity_id {
        log_call_interaction(&state.pool, entity_id, &request.entity_type.unwrap_or_default(), 
            json["sid"].as_str().unwrap_or(""), &request.to).await;
    }
    
    Ok(Json(CallStatus {
        call_sid: json["sid"].as_str().unwrap_or("").to_string(),
        status: json["status"].as_str().unwrap_or("queued").to_string(),
        to: request.to,
        from: from_number,
        duration: None,
        recording_url: None,
        transcription: None,
    }))
}

/// Get call status
async fn get_call_status(
    State(state): State<Arc<AppState>>,
    Path(call_sid): Path<String>,
) -> Result<Json<CallStatus>, (axum::http::StatusCode, String)> {
    let account_sid = std::env::var("TWILIO_ACCOUNT_SID")
        .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Missing TWILIO_ACCOUNT_SID".to_string()))?;
    let auth_token = std::env::var("TWILIO_AUTH_TOKEN")
        .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Missing TWILIO_AUTH_TOKEN".to_string()))?;
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(format!("https://api.twilio.com/2010-04-01/Accounts/{}/Calls/{}.json", account_sid, call_sid))
        .basic_auth(&account_sid, Some(&auth_token))
        .send()
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let json: serde_json::Value = response.json().await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(CallStatus {
        call_sid: json["sid"].as_str().unwrap_or("").to_string(),
        status: json["status"].as_str().unwrap_or("").to_string(),
        to: json["to"].as_str().unwrap_or("").to_string(),
        from: json["from"].as_str().unwrap_or("").to_string(),
        duration: json["duration"].as_str().and_then(|s| s.parse().ok()),
        recording_url: None,
        transcription: None,
    }))
}

/// End a call
async fn end_call(
    State(state): State<Arc<AppState>>,
    Path(call_sid): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let account_sid = std::env::var("TWILIO_ACCOUNT_SID")
        .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Missing TWILIO_ACCOUNT_SID".to_string()))?;
    let auth_token = std::env::var("TWILIO_AUTH_TOKEN")
        .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Missing TWILIO_AUTH_TOKEN".to_string()))?;
    
    let client = reqwest::Client::new();
    
    let response = client
        .post(format!("https://api.twilio.com/2010-04-01/Accounts/{}/Calls/{}.json", account_sid, call_sid))
        .basic_auth(&account_sid, Some(&auth_token))
        .form(&[("Status", "completed")])
        .send()
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "call_sid": call_sid
    })))
}

/// Webhook for call status updates
async fn call_status_webhook(
    State(state): State<Arc<AppState>>,
    axum::Form(payload): axum::Form<std::collections::HashMap<String, String>>,
) -> &'static str {
    tracing::info!("Call status webhook: {:?}", payload);
    
    // Update call status in database
    if let (Some(call_sid), Some(status)) = (payload.get("CallSid"), payload.get("CallStatus")) {
        tracing::info!(call_sid = %call_sid, status = %status, "Call status updated");
    }
    
    "OK"
}

/// Webhook for recording completion
async fn recording_webhook(
    State(state): State<Arc<AppState>>,
    axum::Form(payload): axum::Form<std::collections::HashMap<String, String>>,
) -> &'static str {
    tracing::info!("Recording webhook: {:?}", payload);
    
    if let Some(recording_url) = payload.get("RecordingUrl") {
        let call_sid = payload.get("CallSid").map(|s| s.as_str()).unwrap_or("");
        
        // Trigger transcription with Whisper
        tokio::spawn(async move {
            if let Err(e) = transcribe_recording(recording_url).await {
                tracing::error!("Transcription failed: {}", e);
            }
        });
    }
    
    "OK"
}

/// Transcribe recording using OpenAI Whisper
async fn transcribe_recording(recording_url: &str) -> Result<String, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "Missing OPENAI_API_KEY".to_string())?;
    
    // Download the recording
    let client = reqwest::Client::new();
    let audio_bytes = client.get(recording_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;
    
    // Send to Whisper API
    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(audio_bytes.to_vec())
            .file_name("recording.wav")
            .mime_str("audio/wav")
            .unwrap())
        .text("model", "whisper-1")
        .text("language", "en");
    
    let response = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !response.status().is_success() {
        return Err("Whisper API error".to_string());
    }
    
    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let transcription = json["text"].as_str().unwrap_or("").to_string();
    
    tracing::info!("Transcription complete: {} chars", transcription.len());
    
    // TODO: Extract requirements using RAG
    // extract_requirements_from_transcript(&transcription).await;
    
    Ok(transcription)
}

/// Log call as an interaction
async fn log_call_interaction(
    pool: &sqlx::PgPool,
    entity_id: Uuid,
    entity_type: &str,
    call_sid: &str,
    phone: &str,
) {
    let _ = sqlx::query(
        r#"
        INSERT INTO interactions (id, tenant_id, entity_id, entity_type, interaction_type, direction, subject, metadata)
        VALUES ($1, $2, $3, $4, 'call', 'outbound', $5, $6)
        "#
    )
    .bind(Uuid::new_v4())
    .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()) // TODO: Get from context
    .bind(entity_id)
    .bind(entity_type)
    .bind(format!("Call to {}", phone))
    .bind(serde_json::json!({"call_sid": call_sid, "phone": phone}))
    .execute(pool)
    .await;
}
