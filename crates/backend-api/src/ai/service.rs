use std::sync::Arc;
use core_node_engine::ai::AiService;
use crate::ai::openai::OpenAiService;

pub fn create_ai_service() -> Arc<dyn AiService> {
    // Check environment variable for API key
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "mock".to_string());
    
    Arc::new(OpenAiService::new(api_key))
}
