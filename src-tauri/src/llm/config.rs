use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub use_external_api: bool,
    pub external_endpoint: String,
    pub external_model: String,
    pub chunk_size: usize,
    pub max_retries: u32,
    pub timeout_seconds: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            use_external_api: true,
            external_endpoint: "http://localhost:11434".to_string(),
            external_model: "llama3.1".to_string(),
            chunk_size: 10_000,
            max_retries: 3,
            timeout_seconds: 120,
        }
    }
}

impl LlmConfig {
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), String> {
        if self.external_endpoint.is_empty() {
            return Err("External endpoint cannot be empty".to_string());
        }

        if self.external_model.is_empty() {
            return Err("External model cannot be empty".to_string());
        }

        if self.chunk_size == 0 {
            return Err("Chunk size must be greater than 0".to_string());
        }

        if self.chunk_size > 50_000 {
            return Err("Chunk size too large (max 50,000 characters)".to_string());
        }

        if self.timeout_seconds == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        if self.timeout_seconds > 3600 {
            return Err("Timeout too large (max 1 hour)".to_string());
        }

        // Validate URL format
        if !self.external_endpoint.starts_with("http://")
            && !self.external_endpoint.starts_with("https://")
        {
            return Err("External endpoint must be a valid HTTP/HTTPS URL".to_string());
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    #[allow(dead_code)]
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    #[allow(dead_code)]
    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}

pub const DEFAULT_CONTEXT_SIZE: usize = 8096;
pub const API_GENERATE_ENDPOINT: &str = "/api/generate";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = LlmConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = LlmConfig::default();

        // Test empty endpoint
        config.external_endpoint = String::new();
        assert!(config.validate().is_err());

        // Test invalid URL
        config.external_endpoint = "not-a-url".to_string();
        assert!(config.validate().is_err());

        // Test valid URL
        config.external_endpoint = "http://localhost:11434".to_string();
        assert!(config.validate().is_ok());

        // Test chunk size validation
        config.chunk_size = 0;
        assert!(config.validate().is_err());

        config.chunk_size = 100_000;
        assert!(config.validate().is_err());

        config.chunk_size = 10_000;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_builder_pattern() {
        let config = LlmConfig::default()
            .with_chunk_size(15_000)
            .with_timeout(300)
            .with_retries(5);

        assert_eq!(config.chunk_size, 15_000);
        assert_eq!(config.timeout_seconds, 300);
        assert_eq!(config.max_retries, 5);
        assert!(config.validate().is_ok());
    }
}
