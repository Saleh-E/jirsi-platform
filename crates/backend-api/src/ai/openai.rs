use async_trait::async_trait;
use core_node_engine::ai::AiService;
use serde_json::Value; // Added import for Value if needed, though this file uses reqwest/serde mainly

#[derive(Debug, Clone)]
pub struct OpenAiService {
    api_key: String,
    model: String,
}

impl OpenAiService {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "gpt-3.5-turbo".to_string(), // Default model
        }
    }
}

#[async_trait]
impl AiService for OpenAiService {
    async fn generate(&self, prompt: &str, system: Option<&str>) -> Result<String, String> {
        let client = reqwest::Client::new();
        
        let mut messages = Vec::new();
        if let Some(sys) = system {
            messages.push(serde_json::json!({
                "role": "system",
                "content": sys
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt
        }));
        
        // Mock response if API key is "mock"
        if self.api_key == "mock" {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            return Ok(format!("Mock AI Response for: {}", prompt));
        }

        let resp = client.post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": messages,
                "temperature": 0.7
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
            
        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(format!("OpenAI API Error: {}", error_text));
        }
        
        let json: serde_json::Value = resp.json().await.map_err(|e: reqwest::Error| e.to_string())?;
        
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
            
        Ok(content)
    }
}
