use tauri::AppHandle;
use crate::llm::{
    config::LlmConfig,
    error::{LlmError, LlmResult},
    file_manager::FileManager,
    performance::PerformanceTracker,
    prompts::{Language, PromptManager},
    service::LlmService,
    summary::SummaryGenerator,
};

/// Builder pattern for creating summary generators with custom configuration
#[allow(dead_code)]
pub struct SummaryGeneratorBuilder {
    app_handle: AppHandle,
    language: Language,
    config: Option<LlmConfig>,
    performance_tracking: bool,
}

impl SummaryGeneratorBuilder {
    #[allow(dead_code)]
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            language: Language::default(),
            config: None,
            performance_tracking: false,
        }
    }

    #[allow(dead_code)]
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    #[allow(dead_code)]
    pub fn with_config(mut self, config: LlmConfig) -> Self {
        self.config = Some(config);
        self
    }

    #[allow(dead_code)]
    pub fn with_performance_tracking(mut self, enabled: bool) -> Self {
        self.performance_tracking = enabled;
        self
    }

    #[allow(dead_code)]
    pub fn build(self) -> LlmResult<SummaryGenerator> {
        // Validate configuration if provided
        if let Some(ref config) = self.config {
            config.validate().map_err(|e| LlmError::ConfigError(e))?;
        }

        Ok(SummaryGenerator::new(self.app_handle, self.language))
    }
}

/// Utility functions for common LLM operations
#[allow(dead_code)]
pub struct LlmUtils;

impl LlmUtils {
    /// Quick test to check if the LLM service is available
    #[allow(dead_code)]
    pub async fn health_check(config: &LlmConfig) -> LlmResult<bool> {
        let service = LlmService::new(
            config.external_endpoint.clone(),
            config.external_model.clone(),
        );

        let test_prompt = PromptManager::test_connection(&Language::English);
        let test_message = PromptManager::test_user_message(&Language::English);

        match service.generate_text(test_prompt, test_message, None, None).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Estimate the number of chunks a text will be split into
    #[allow(dead_code)]
    pub fn estimate_chunks(text: &str, chunk_size: usize) -> usize {
        if text.is_empty() {
            return 0;
        }
        
        let char_count = text.chars().count();
        if char_count <= chunk_size {
            1
        } else {
            // Rough estimate accounting for optimal break points
            ((char_count as f64 / chunk_size as f64) * 1.1).ceil() as usize
        }
    }

    /// Estimate processing time based on text length and configuration
    #[allow(dead_code)]
    pub fn estimate_processing_time(text: &str, config: &LlmConfig) -> std::time::Duration {
        let chunks = Self::estimate_chunks(text, config.chunk_size);
        
        // Base estimates (these would be calibrated based on actual performance data)
        let base_chunk_time = std::time::Duration::from_secs(30); // 30 seconds per chunk
        let final_summary_time = std::time::Duration::from_secs(60); // 1 minute for final summary
        
        base_chunk_time * chunks as u32 + final_summary_time
    }

    /// Get memory usage estimate for processing a text
    #[allow(dead_code)]
    pub fn estimate_memory_usage(text: &str, config: &LlmConfig) -> usize {
        let chunks = Self::estimate_chunks(text, config.chunk_size);
        
        // Rough estimates in bytes
        let base_memory = 50 * 1024 * 1024; // 50MB base
        let per_chunk_memory = 10 * 1024 * 1024; // 10MB per chunk
        
        base_memory + (per_chunk_memory * chunks)
    }
}

/// Helper for managing LLM sessions with automatic cleanup
#[allow(dead_code)]
pub struct LlmSession {
    pub app_handle: AppHandle,
    pub config: LlmConfig,
    pub performance_tracker: Option<PerformanceTracker>,
    pub file_manager: FileManager,
}

impl LlmSession {
    #[allow(dead_code)]
    pub fn new(app_handle: AppHandle, config: LlmConfig) -> LlmResult<Self> {
        config.validate().map_err(|e| LlmError::ConfigError(e))?;
        
        let file_manager = FileManager::new(app_handle.clone());
        
        Ok(Self {
            app_handle,
            config,
            performance_tracker: None,
            file_manager,
        })
    }

    #[allow(dead_code)]
    pub fn with_performance_tracking(mut self) -> Self {
        self.performance_tracker = Some(PerformanceTracker::new());
        self
    }

    #[allow(dead_code)]
    pub async fn generate_summary(&mut self, meeting_id: &str) -> LlmResult<String> {
        let generator = SummaryGenerator::new(self.app_handle.clone(), Language::default());
        
        let result = generator.generate_summary(meeting_id).await;
        
        // Print performance summary if tracking is enabled
        if let Some(ref tracker) = self.performance_tracker {
            tracker.print_summary();
        }
        
        result
    }

    #[allow(dead_code)]
    pub fn get_performance_metrics(&self) -> Option<crate::llm::performance::PerformanceMetrics> {
        self.performance_tracker.as_ref().map(|t| t.get_metrics())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_chunks() {
        assert_eq!(LlmUtils::estimate_chunks("", 1000), 0);
        assert_eq!(LlmUtils::estimate_chunks("short", 1000), 1);
        
        let long_text = "a".repeat(5000);
        let chunks = LlmUtils::estimate_chunks(&long_text, 1000);
        assert!(chunks >= 5 && chunks <= 6); // Should be around 5-6 chunks
    }

    #[test]
    fn test_estimate_processing_time() {
        let config = LlmConfig::default();
        let short_text = "short text";
        let time = LlmUtils::estimate_processing_time(short_text, &config);
        
        // Should be base time for 1 chunk + final summary
        assert!(time.as_secs() >= 90); // 30 + 60 seconds minimum
    }

    #[test]
    fn test_estimate_memory_usage() {
        let config = LlmConfig::default();
        let text = "test";
        let usage = LlmUtils::estimate_memory_usage(text, &config);
        
        assert!(usage > 50 * 1024 * 1024); // Should be more than base memory
    }
}
