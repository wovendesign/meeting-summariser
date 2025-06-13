use serde_json::json;
use tauri_plugin_http::reqwest::Client;
use std::time::Instant;

use crate::llm::{
    config::{DEFAULT_CONTEXT_SIZE, API_GENERATE_ENDPOINT},
    error::{LlmError, LlmResult, IntoLlmError},
    models::OllamaResponse,
    progress::ProgressTracker,
};

pub struct LlmService {
    client: Client,
    base_url: String,
    model: String,
}

impl LlmService {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            model,
        }
    }

    pub async fn generate_text(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        structure: Option<schemars::Schema>,
        progress_tracker: Option<&ProgressTracker>,
    ) -> LlmResult<String> {
        let start_time = Instant::now();
        println!("🚀 Starting LLM text generation...");

        if let Some(tracker) = progress_tracker {
            tracker.emit_api_status("🔄 Trying external API...")
                .map_err(|e| LlmError::NetworkError(e))?;
        }

        let api_start = Instant::now();
        match self.try_external_api(system_prompt, user_prompt, structure).await {
            Ok(response) => {
                let api_duration = api_start.elapsed();
                let total_duration = start_time.elapsed();
                println!(
                    "✅ API successful! API time: {:.2}s, Total time: {:.2}s",
                    api_duration.as_secs_f64(),
                    total_duration.as_secs_f64()
                );
                
                if let Some(tracker) = progress_tracker {
                    tracker.emit_api_status("✅ External API successful")
                        .map_err(|e| LlmError::NetworkError(e))?;
                }
                
                Ok(response)
            }
            Err(e) => {
                let api_duration = api_start.elapsed();
                println!(
                    "❌ API failed after {:.2}s: {}, falling back to local model",
                    api_duration.as_secs_f64(),
                    e
                );
                
                if let Some(tracker) = progress_tracker {
                    tracker.emit_api_status("❌ External API failed, switching to local model...")
                        .map_err(|e| LlmError::NetworkError(e))?;
                }
                
                Err(e)
            }
        }
    }

    async fn try_external_api(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        structure: Option<schemars::Schema>,
    ) -> LlmResult<String> {
        println!("Trying external Ollama API");

        // Merge system and user prompts into one string
        let full_prompt = format!("System: {}\nUser: {}", system_prompt, user_prompt);

        let mut json = json!({
            "model": self.model,
            "prompt": full_prompt,
            "stream": false,
            "num_ctx": DEFAULT_CONTEXT_SIZE,
        });

        if let Some(schema) = structure {
            json.as_object_mut().unwrap().insert(
                "format".to_string(),
                serde_json::Value::from(schema),
            );
        }

        let url = format!("{}{}", self.base_url, API_GENERATE_ENDPOINT);
        
        let response = self
            .client
            .post(&url)
            .json(&json)
            .send()
            .await
            .map_network_err("Failed to send request to Ollama")?
            .json::<OllamaResponse>()
            .await
            .map_parse_err("Failed to parse Ollama response")?;

        Ok(response.response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_service_creation() {
        let service = LlmService::new(
            "http://localhost:11434".to_string(),
            "llama3.1".to_string(),
        );
        assert_eq!(service.base_url, "http://localhost:11434");
        assert_eq!(service.model, "llama3.1");
    }
}
