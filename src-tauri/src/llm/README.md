# LLM Module Refactoring

This document describes the refactored LLM module structure for better maintainability, testability, and separation of concerns.

## Module Structure

```
src/llm/
├── mod.rs              # Module exports and re-exports
├── config.rs           # Configuration management
├── error.rs            # Custom error types and handling
├── file_manager.rs     # File I/O operations
├── models.rs           # Data structures and serialization
├── progress.rs         # Progress tracking and UI updates
├── prompts.rs          # Prompt management for different languages
├── service.rs          # External LLM API communication
├── summary.rs          # Main summary generation logic
└── text_processing.rs  # Text chunking and processing utilities
```

## Key Improvements

### 1. **Separation of Concerns**

- **Configuration**: All LLM settings centralized in `config.rs`
- **Error Handling**: Custom error types with proper context in `error.rs`
- **File Operations**: Dedicated file manager for all I/O operations
- **Network**: Isolated API communication in `service.rs`
- **UI Updates**: Progress tracking separated from business logic

### 2. **Better Error Handling**

```rust
// Before: String-based error handling
fn some_operation() -> Result<T, String>

// After: Typed error handling with context
fn some_operation() -> LlmResult<T>
```

### 3. **Configurable Parameters**

```rust
pub struct LlmConfig {
    pub use_external_api: bool,
    pub external_endpoint: String,
    pub external_model: String,
    pub chunk_size: usize,        // New: configurable chunk size
    pub max_retries: u32,         // New: retry logic
    pub timeout_seconds: u64,     // New: timeout handling
}
```

### 4. **Modular Prompt Management**

```rust
// Before: Multiple scattered prompt functions
fn get_chunk_summarization_prompt(...) -> String
fn get_final_summary_prompt(...) -> &str

// After: Centralized prompt manager
impl PromptManager {
    pub fn chunk_summarization(language: &Language, key_facts: Option<&KeyFact>) -> String
    pub fn final_summary(language: &Language) -> &'static str
    pub fn test_connection(language: &Language) -> &'static str
}
```

### 5. **Improved File Management**

```rust
pub struct FileManager {
    app_handle: AppHandle,
}

impl FileManager {
    pub async fn save_chunk(&self, meeting_id: &str, chunk_index: usize, content: &str) -> Result<(), String>
    pub async fn save_final_summary(&self, meeting_id: &str, content: &FinalSummaryFormat) -> Result<(), String>
    pub async fn read_summary(&self, meeting_id: &str) -> Result<FinalSummaryFormat, String>
}
```

### 6. **Progress Tracking**

```rust
pub struct ProgressTracker {
    app_handle: AppHandle,
    start_time: Instant,
    total_steps: usize,
    current_step: usize,
}

impl ProgressTracker {
    pub fn update_progress(&mut self, message: &str) -> Result<(), String>
    pub fn log_timing_stats(&self, chunk_times: &[Duration]) -> Result<(), String>
}
```

## Benefits

### **Maintainability**

- Single responsibility principle applied
- Clear module boundaries
- Easier to locate and modify specific functionality

### **Testability**

- Each module can be tested independently
- Mock implementations for external dependencies
- Unit tests for text processing utilities

### **Reusability**

- Components can be reused across different parts of the application
- Easy to extend with new LLM providers
- Configurable behavior through the config module

### **Error Handling**

- Proper error propagation with context
- Type-safe error handling
- Consistent error messages

### **Performance**

- Configurable chunk sizes for different use cases
- Built-in retry logic for failed requests
- Progress tracking for long-running operations

## Migration Guide

The refactoring maintains backward compatibility for the public API:

```rust
// These functions still work as before
#[tauri::command]
pub async fn generate_summary(app: AppHandle, meeting_id: &str) -> Result<String, String>

#[tauri::command]
pub async fn get_meeting_summary(app: AppHandle, meeting_id: &str) -> Result<String, String>

#[tauri::command]
pub async fn is_summarizing(app: AppHandle) -> Result<Option<String>, String>

#[tauri::command]
pub async fn test_llm_connection(app: AppHandle) -> Result<String, String>
```

## Configuration

To use the new configuration system:

```rust
// In your app setup
let config = LlmConfig {
    use_external_api: true,
    external_endpoint: "http://localhost:11434".to_string(),
    external_model: "llama3.1".to_string(),
    chunk_size: 15_000,  // Larger chunks for better context
    max_retries: 5,      // More retries for reliability
    timeout_seconds: 300, // 5-minute timeout
};
```

## Testing

Each module can now be tested independently:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_chunking() {
        let text = "Long text...";
        let chunks = split_text_into_chunks(text, 100);
        assert_eq!(chunks.len(), expected_chunks);
    }

    #[tokio::test]
    async fn test_llm_service() {
        let service = LlmService::new("http://test".to_string(), "test-model".to_string());
        // Test with mock server
    }
}
```

This refactoring makes the codebase more maintainable, testable, and extensible while preserving all existing functionality.
