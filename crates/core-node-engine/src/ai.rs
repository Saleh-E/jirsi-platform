use async_trait::async_trait;
use serde_json::Value;
use std::fmt::Debug;

#[async_trait]
pub trait AiService: Send + Sync + Debug {
    async fn generate(&self, prompt: &str, system: Option<&str>) -> Result<String, String>;
}
