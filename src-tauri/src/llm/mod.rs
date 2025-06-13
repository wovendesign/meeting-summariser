pub mod config;
pub mod error;
pub mod file_manager;
pub mod models;
pub mod performance;
pub mod progress;
pub mod prompts;
pub mod service;
pub mod summary;
pub mod text_processing;
pub mod utils;

// Re-export commonly used items
pub use config::LlmConfig;
#[allow(unused_imports)]
pub use error::{LlmError, LlmResult};
#[allow(unused_imports)]
pub use models::*;
#[allow(unused_imports)]
pub use prompts::{Language, PromptManager};
#[allow(unused_imports)]
pub use service::LlmService;
#[allow(unused_imports)]
pub use summary::SummaryGenerator;

// Re-export the public API tauri commands
pub use summary::{generate_summary, get_meeting_summary, is_summarizing, test_llm_connection};

// Make sure the tauri command macros are available
pub use summary::{
    __cmd__generate_summary, 
    __cmd__get_meeting_summary, 
    __cmd__is_summarizing, 
    __cmd__test_llm_connection
};
