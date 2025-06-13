# LLM Module Usage Examples

This document provides practical examples of how to use the refactored LLM module.

## Basic Usage

### 1. Simple Summary Generation

```rust
use crate::llm::{generate_summary};

#[tauri::command]
async fn summarize_meeting(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    generate_summary(app, meeting_id).await
}
```

### 2. Custom Configuration

```rust
use crate::llm::{LlmConfig, LlmSession, Language};

async fn custom_summary_generation(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    // Create custom configuration
    let config = LlmConfig::default()
        .with_chunk_size(15_000)  // Larger chunks for better context
        .with_timeout(300)        // 5-minute timeout
        .with_retries(5);         // More retries for reliability

    // Validate configuration
    config.validate().map_err(|e| format!("Config error: {}", e))?;

    // Create session with performance tracking
    let mut session = LlmSession::new(app, config)
        .map_err(|e| e.to_string())?
        .with_performance_tracking();

    // Generate summary
    let result = session.generate_summary(meeting_id).await;

    // Print performance metrics
    if let Some(metrics) = session.get_performance_metrics() {
        println!("Processing took {:.2}s", metrics.total_duration.as_secs_f64());
        println!("Processed {} characters at {:.1} chars/sec",
                 metrics.total_characters_processed,
                 metrics.characters_per_second);
    }

    result.map_err(|e| e.to_string())
}
```

### 3. Health Check and Diagnostics

```rust
use crate::llm::{LlmConfig, LlmUtils};

#[tauri::command]
async fn check_llm_health(config: LlmConfig) -> Result<bool, String> {
    LlmUtils::health_check(&config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_processing_estimate(text: String, config: LlmConfig) -> Result<String, String> {
    let chunks = LlmUtils::estimate_chunks(&text, config.chunk_size);
    let time = LlmUtils::estimate_processing_time(&text, &config);
    let memory = LlmUtils::estimate_memory_usage(&text, &config);

    Ok(format!(
        "Estimated processing: {} chunks, {:.1} minutes, {:.1} MB memory",
        chunks,
        time.as_secs_f64() / 60.0,
        memory as f64 / (1024.0 * 1024.0)
    ))
}
```

### 4. Builder Pattern Usage

```rust
use crate::llm::{SummaryGeneratorBuilder, Language};

async fn advanced_summary_generation(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    let generator = SummaryGeneratorBuilder::new(app)
        .with_language(Language::German)
        .with_config(
            LlmConfig::default()
                .with_chunk_size(12_000)
                .with_timeout(240)
        )
        .with_performance_tracking(true)
        .build()
        .map_err(|e| e.to_string())?;

    generator.generate_summary(meeting_id).await
        .map_err(|e| e.to_string())
}
```

### 5. Error Handling Examples

```rust
use crate::llm::{LlmError, LlmResult, generate_summary};

async fn robust_summary_generation(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    match generate_summary(app, meeting_id).await {
        Ok(summary) => Ok(summary),
        Err(e) => {
            // Convert LlmError to user-friendly message
            let user_message = match e.parse::<LlmError>() {
                Ok(LlmError::NetworkError(_)) => {
                    "Network connection failed. Please check your internet connection and try again.".to_string()
                }
                Ok(LlmError::ParseError(_)) => {
                    "Failed to process the response. The model might be overloaded.".to_string()
                }
                Ok(LlmError::FileError(_)) => {
                    "Failed to access meeting files. Please ensure the meeting exists.".to_string()
                }
                Ok(LlmError::ConfigError(_)) => {
                    "Configuration error. Please check your LLM settings.".to_string()
                }
                Ok(LlmError::TimeoutError(_)) => {
                    "Request timed out. The meeting might be too long or the server too busy.".to_string()
                }
                Ok(LlmError::SerializationError(_)) => {
                    "Data format error. Please try again or contact support.".to_string()
                }
                _ => e, // Fallback to original error message
            };
            Err(user_message)
        }
    }
}
```

### 6. Testing Examples

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{LlmConfig, LlmUtils, text_processing::split_text_into_chunks};

    #[test]
    fn test_config_validation() {
        let config = LlmConfig::default();
        assert!(config.validate().is_ok());

        let invalid_config = LlmConfig {
            external_endpoint: "invalid-url".to_string(),
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_text_chunking() {
        let text = "This is a test. It has sentences. Multiple sentences actually.";
        let chunks = split_text_into_chunks(text, 25);

        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.len() <= 25 || chunk.ends_with('.'));
        }
    }

    #[test]
    fn test_processing_estimates() {
        let config = LlmConfig::default();
        let text = "a".repeat(50_000);

        let chunks = LlmUtils::estimate_chunks(&text, config.chunk_size);
        let time = LlmUtils::estimate_processing_time(&text, &config);
        let memory = LlmUtils::estimate_memory_usage(&text, &config);

        assert!(chunks >= 5);
        assert!(time.as_secs() > 0);
        assert!(memory > 50 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = LlmConfig::default();

        // This would require a running LLM service
        // In real tests, you might use a mock server
        match LlmUtils::health_check(&config).await {
            Ok(healthy) => println!("Health check result: {}", healthy),
            Err(e) => println!("Health check failed: {}", e),
        }
    }
}
```

### 7. Integration with Frontend

```typescript
// Frontend TypeScript example
import { invoke } from "@tauri-apps/api/tauri";

interface ProcessingEstimate {
  chunks: number;
  estimatedTime: number;
  estimatedMemory: number;
}

class LlmService {
  async generateSummary(meetingId: string): Promise<string> {
    try {
      return await invoke("generate_summary", { meetingId });
    } catch (error) {
      console.error("Summary generation failed:", error);
      throw new Error(`Failed to generate summary: ${error}`);
    }
  }

  async checkHealth(): Promise<boolean> {
    try {
      return await invoke("check_llm_health");
    } catch (error) {
      console.error("Health check failed:", error);
      return false;
    }
  }

  async getProcessingEstimate(text: string): Promise<ProcessingEstimate> {
    try {
      const estimate = await invoke("get_processing_estimate", { text });
      return JSON.parse(estimate);
    } catch (error) {
      console.error("Failed to get estimate:", error);
      throw error;
    }
  }

  async testConnection(): Promise<boolean> {
    try {
      const result = await invoke("test_llm_connection");
      return result.includes("successful");
    } catch (error) {
      console.error("Connection test failed:", error);
      return false;
    }
  }
}

// Usage in a Svelte component
export class MeetingSummaryManager {
  private llmService = new LlmService();

  async generateSummaryWithProgress(
    meetingId: string,
    onProgress: (message: string) => void,
  ): Promise<string> {
    // Check health first
    const isHealthy = await this.llmService.checkHealth();
    if (!isHealthy) {
      throw new Error("LLM service is not available");
    }

    onProgress("Starting summary generation...");

    try {
      const summary = await this.llmService.generateSummary(meetingId);
      onProgress("Summary generated successfully!");
      return summary;
    } catch (error) {
      onProgress(`Error: ${error.message}`);
      throw error;
    }
  }
}
```

## Performance Optimization Tips

### 1. Chunk Size Optimization

```rust
// For shorter meetings (< 30 minutes)
let config = LlmConfig::default().with_chunk_size(8_000);

// For longer meetings (> 1 hour)
let config = LlmConfig::default().with_chunk_size(15_000);

// For very long meetings (> 3 hours)
let config = LlmConfig::default().with_chunk_size(20_000);
```

### 2. Memory Management

```rust
// Monitor memory usage for large meetings
let memory_estimate = LlmUtils::estimate_memory_usage(&transcript, &config);
if memory_estimate > 500 * 1024 * 1024 { // 500MB
    println!("Warning: Large memory usage expected: {:.1} MB",
             memory_estimate as f64 / (1024.0 * 1024.0));
}
```

### 3. Error Recovery

```rust
// Implement retry logic for failed chunks
let mut retries = 0;
let max_retries = config.max_retries;

while retries < max_retries {
    match process_chunk(&chunk).await {
        Ok(result) => return Ok(result),
        Err(e) if retries < max_retries - 1 => {
            retries += 1;
            println!("Retry {}/{}: {}", retries, max_retries, e);
            tokio::time::sleep(Duration::from_secs(retries as u64 * 2)).await;
        }
        Err(e) => return Err(e),
    }
}
```

This refactored structure provides a solid foundation for maintainable, testable, and extensible LLM functionality.
