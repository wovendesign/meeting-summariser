use std::time::Instant;
use schemars::schema_for;
use serde_json::json;
use tauri::{AppHandle, Manager, Emitter};
use tokio::sync::Mutex;

use crate::{get_meeting_transcript, AppState};
use crate::llm::{
    config::LlmConfig,
    error::{LlmError, LlmResult},
    file_manager::FileManager,
    models::{FirstSummaryFormat, FinalSummaryFormat, KeyFact, MeetingToMarkdown},
    progress::ProgressTracker,
    prompts::{Language, PromptManager},
    service::LlmService,
    text_processing::split_text_into_chunks,
};

pub struct SummaryGenerator {
    app_handle: AppHandle,
    file_manager: FileManager,
    language: Language,
}

impl SummaryGenerator {
    pub fn new(app_handle: AppHandle, language: Language) -> Self {
        let file_manager = FileManager::new(app_handle.clone());
        Self {
            app_handle,
            file_manager,
            language,
        }
    }

    pub async fn generate_summary(&self, meeting_id: &str) -> LlmResult<String> {
        let summary_start_time = Instant::now();
        println!("🚀 Starting full meeting summary generation...");

        // Check if another summarization is running
        self.check_and_set_summarization_state(meeting_id).await?;

        let transcript = get_meeting_transcript(self.app_handle.clone(), meeting_id)
            .await
            .map_err(|e| LlmError::FileError(format!("Failed to get transcript: {}", e)))?;

        if transcript.is_empty() {
            return Err(LlmError::FileError("No transcript to summarize".to_string()));
        }

        let content = if transcript.len() > 10_000 {
            self.summarize_long_transcript(&transcript, meeting_id).await?
        } else {
            return Err(LlmError::ConfigError("Direct summarization not implemented yet".to_string()));
        };

        // Save the summary
        self.file_manager.save_final_summary(meeting_id, &content).await
            .map_err(|e| LlmError::FileError(e))?;

        self.file_manager.save_meeting_metadata(meeting_id, content.title.to_string())
            .map_err(|e| LlmError::FileError(e))?;

        let total_duration = summary_start_time.elapsed();
        println!("🎉 Full meeting summary completed!");
        println!(
            "⏱️  Total summary generation time: {:.2}s",
            total_duration.as_secs_f64()
        );

        self.app_handle
            .emit(
                "llm-progress",
                &format!(
                    "✅ Summary completed in {:.1}s",
                    total_duration.as_secs_f64()
                ),
            )
            .map_err(|e| LlmError::NetworkError(format!("Failed to emit progress: {}", e)))?;

        Ok(content.to_markdown())
    }

    /// Regenerate only the final summary using existing chunk summaries
    pub async fn regenerate_final_summary(&self, meeting_id: &str) -> LlmResult<String> {
        let summary_start_time = Instant::now();
        println!("🔄 Starting final summary regeneration from existing chunks...");

        // Check if another summarization is running
        self.check_and_set_summarization_state(meeting_id).await?;

        // Read existing chunk summaries from disk
        let chunk_summaries = self.file_manager.read_chunk_summaries(meeting_id).await
            .map_err(|e| LlmError::FileError(format!("Failed to read chunk summaries: {}", e)))?;

        println!("📦 Found {} saved chunk summaries", chunk_summaries.len());

        // Get LLM config
        let config = self.get_llm_config().await?;
        let llm_service = LlmService::new(config.external_endpoint, config.external_model);

        // Generate final summary from existing chunk summaries
        let mut progress_tracker = ProgressTracker::new(self.app_handle.clone(), 1);
        progress_tracker.start_summarization(meeting_id)
            .map_err(|e| LlmError::NetworkError(e))?;

        let content = self.generate_final_summary(chunk_summaries, &llm_service, &mut progress_tracker).await?;

        // Save the regenerated summary
        self.file_manager.save_final_summary(meeting_id, &content).await
            .map_err(|e| LlmError::FileError(e))?;

        self.file_manager.save_meeting_metadata(meeting_id, content.title.to_string())
            .map_err(|e| LlmError::FileError(e))?;

        // Reset summarization state
        {
            let state = self.app_handle.state::<Mutex<AppState>>();
            let mut state = state.lock().await;
            state.currently_summarizing = None;
        }

        let total_duration = summary_start_time.elapsed();
        println!("🎉 Final summary regeneration completed!");
        println!(
            "⏱️  Total regeneration time: {:.2}s",
            total_duration.as_secs_f64()
        );

        self.app_handle
            .emit(
                "llm-progress",
                &format!(
                    "✅ Final summary regenerated in {:.1}s",
                    total_duration.as_secs_f64()
                ),
            )
            .map_err(|e| LlmError::NetworkError(format!("Failed to emit progress: {}", e)))?;

        Ok(content.to_markdown())
    }

    async fn check_and_set_summarization_state(&self, meeting_id: &str) -> LlmResult<()> {
        let state = self.app_handle.state::<Mutex<AppState>>();
        let mut state = state.lock().await;

        if state.currently_summarizing.is_some() {
            return Err(LlmError::ConfigError("Another summarization is running".to_string()));
        }

        state.currently_summarizing = Some(meeting_id.to_string());
        
        self.app_handle
            .emit("summarization-started", meeting_id)
            .map_err(|e| LlmError::NetworkError(format!("Failed to emit summarization-started: {}", e)))?;

        Ok(())
    }

    async fn summarize_long_transcript(&self, transcript: &str, meeting_id: &str) -> LlmResult<FinalSummaryFormat> {
        self.app_handle
            .emit(
                "llm-progress",
                "📄 Transcript is long, splitting into chunks for processing...",
            )
            .map_err(|e| LlmError::NetworkError(format!("Failed to emit progress: {}", e)))?;

        // Get LLM config
        let config = self.get_llm_config().await?;
        let llm_service = LlmService::new(config.external_endpoint, config.external_model);

        // Split transcript into manageable chunks
        let chunks = split_text_into_chunks(transcript, config.chunk_size);
        println!("📦 Split transcript into {} chunks", chunks.len());

        // Summarize chunks and combine
        self.summarize_chunks(chunks, meeting_id, &llm_service).await
    }

    async fn get_llm_config(&self) -> LlmResult<LlmConfig> {
        let state = self.app_handle.state::<Mutex<AppState>>();
        let state = state.lock().await;
        Ok(state.llm_config.clone())
    }

    async fn summarize_chunks(
        &self,
        chunks: Vec<String>,
        meeting_id: &str,
        llm_service: &LlmService,
    ) -> LlmResult<FinalSummaryFormat> {
        let mut chunk_summaries = Vec::new();
        let mut chunk_times = Vec::new();
        let mut key_facts = KeyFact {
            responisible_for_moderation: None,
            responisible_for_protocol: None,
            responisible_for_timekeeping: None,
            attendees: None,
        };

        let total_steps = chunks.len() + 1;
        let mut progress_tracker = ProgressTracker::new(self.app_handle.clone(), total_steps);
        progress_tracker.start_summarization(meeting_id)
            .map_err(|e| LlmError::NetworkError(e))?;

        // Process each chunk
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_start_time = Instant::now();
            
            progress_tracker.update_progress(&format!(
                "Summarizing chunk {} of {}",
                i + 1,
                chunks.len()
            )).map_err(|e| LlmError::NetworkError(e))?;

            let chunk_summary = self.process_chunk(chunk, &key_facts, llm_service, &progress_tracker).await?;
            
            let chunk_duration = chunk_start_time.elapsed();
            chunk_times.push(chunk_duration);
            progress_tracker.log_chunk_completed(i, chunk_duration);

            // Update key facts from chunk summary
            self.update_key_facts(&mut key_facts, &chunk_summary);

            // Save chunk and summary
            self.file_manager.save_chunk(meeting_id, i, chunk).await
                .map_err(|e| LlmError::FileError(e))?;
            
            let chunk_summary_json = serde_json::to_string_pretty(&chunk_summary)
                .map_err(|e| LlmError::SerializationError(format!("Failed to serialize chunk summary: {}", e)))?;
            
            self.file_manager.save_chunk_summary(meeting_id, i, &chunk_summary_json).await
                .map_err(|e| LlmError::FileError(e))?;

            chunk_summaries.push(chunk_summary);
        }

        // Log timing statistics
        progress_tracker.log_timing_stats(&chunk_times)
            .map_err(|e| LlmError::NetworkError(e))?;

        // Save all chunk summaries
        let summary_strings: Vec<String> = chunk_summaries
            .iter()
            .map(|s| serde_json::to_string_pretty(s).unwrap_or_default())
            .collect();
        
        self.file_manager.save_all_chunk_summaries(meeting_id, &summary_strings).await
            .map_err(|e| LlmError::FileError(e))?;

        // Generate final summary
        self.generate_final_summary(chunk_summaries, llm_service, &mut progress_tracker).await
    }

    async fn process_chunk(
        &self,
        chunk: &str,
        key_facts: &KeyFact,
        llm_service: &LlmService,
        progress_tracker: &ProgressTracker,
    ) -> LlmResult<FirstSummaryFormat> {
        let chunk_system_prompt = PromptManager::chunk_summarization(&self.language, Some(key_facts));

        let chunk_summary_json = llm_service
            .generate_text(
                &chunk_system_prompt,
                chunk,
                Some(schema_for!(FirstSummaryFormat)),
                Some(progress_tracker),
            )
            .await?;

        serde_json::from_str(&chunk_summary_json)
            .map_err(|e| LlmError::ParseError(format!("Failed to parse chunk summary JSON: {}", e)))
    }

    fn update_key_facts(&self, key_facts: &mut KeyFact, chunk_summary: &FirstSummaryFormat) {
        if let Some(moderation) = &chunk_summary.key_facts.responisible_for_moderation {
            key_facts.responisible_for_moderation = Some(moderation.clone());
        }
        if let Some(protocol) = &chunk_summary.key_facts.responisible_for_protocol {
            key_facts.responisible_for_protocol = Some(protocol.clone());
        }
        if let Some(timekeeping) = &chunk_summary.key_facts.responisible_for_timekeeping {
            key_facts.responisible_for_timekeeping = Some(timekeeping.clone());
        }
        if let Some(attendees) = &chunk_summary.key_facts.attendees {
            if key_facts.attendees.is_none() {
                key_facts.attendees = Some(attendees.clone());
            } else {
                // Merge attendees, avoiding duplicates
                let mut existing_ids: Vec<usize> = key_facts
                    .attendees
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|a| a.id)
                    .collect();
                for attendee in attendees {
                    if !existing_ids.contains(&attendee.id) {
                        existing_ids.push(attendee.id);
                        key_facts.attendees.as_mut().unwrap().push(attendee.clone());
                    }
                }
            }
        }
    }

    async fn generate_final_summary(
        &self,
        chunk_summaries: Vec<FirstSummaryFormat>,
        llm_service: &LlmService,
        progress_tracker: &mut ProgressTracker,
    ) -> LlmResult<FinalSummaryFormat> {
        let final_summary_start_time = Instant::now();

        progress_tracker.update_progress("Combining chunk summaries into final summary...")
            .map_err(|e| LlmError::NetworkError(e))?;

        let final_system_prompt = PromptManager::final_summary(&self.language);
        let combined_summaries = self.combine_structured_first_summaries(chunk_summaries);

        let final_string = llm_service
            .generate_text(
                final_system_prompt,
                &json!(combined_summaries).to_string(),
                Some(schema_for!(FinalSummaryFormat)),
                Some(progress_tracker),
            )
            .await?;

        let final_summary: FinalSummaryFormat = serde_json::from_str(&final_string)
            .map_err(|e| LlmError::ParseError(format!("Failed to parse final summary JSON: {}", e)))?;

        let final_summary_duration = final_summary_start_time.elapsed();
        println!(
            "✅ Final summary generation completed in {:.2}s",
            final_summary_duration.as_secs_f64()
        );

        Ok(final_summary)
    }

    fn combine_structured_first_summaries(&self, summaries: Vec<FirstSummaryFormat>) -> FirstSummaryFormat {
        let mut combined = FirstSummaryFormat {
            key_facts: KeyFact {
                responisible_for_moderation: None,
                responisible_for_protocol: None,
                responisible_for_timekeeping: None,
                attendees: None,
            },
            topics: Vec::new(),
            todos: None,
        };

        for summary in summaries {
            // Combine key facts
            if let Some(moderation) = summary.key_facts.responisible_for_moderation {
                combined.key_facts.responisible_for_moderation = Some(moderation);
            }
            if let Some(protocol) = summary.key_facts.responisible_for_protocol {
                combined.key_facts.responisible_for_protocol = Some(protocol);
            }
            if let Some(timekeeping) = summary.key_facts.responisible_for_timekeeping {
                combined.key_facts.responisible_for_timekeeping = Some(timekeeping);
            }
            if let Some(attendees) = summary.key_facts.attendees {
                if combined.key_facts.attendees.is_none() {
                    combined.key_facts.attendees = Some(attendees);
                } else {
                    // Merge attendees, avoiding duplicates
                    let existing_ids: Vec<usize> = combined
                        .key_facts
                        .attendees
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|a| a.id)
                        .collect();
                    for attendee in attendees {
                        if !existing_ids.contains(&attendee.id) {
                            combined
                                .key_facts
                                .attendees
                                .as_mut()
                                .unwrap()
                                .push(attendee);
                        }
                    }
                }
            }

            // Combine topics
            combined.topics.extend(summary.topics);

            // Combine todos
            if let Some(todos) = summary.todos {
                if combined.todos.is_none() {
                    combined.todos = Some(todos);
                } else {
                    combined.todos.as_mut().unwrap().extend(todos);
                }
            }
        }

        combined
    }
}

// Public API functions
#[tauri::command]
pub async fn generate_summary(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    let generator = SummaryGenerator::new(app, Language::default());
    generator.generate_summary(meeting_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_summarizing(app: AppHandle) -> Result<Option<String>, String> {
    let state = app.state::<Mutex<AppState>>();
    let state = state.lock().await;
    Ok(state.currently_summarizing.clone())
}

#[tauri::command]
pub async fn get_meeting_summary(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    let file_manager = FileManager::new(app);
    let summary = file_manager.read_summary(meeting_id).await.map_err(|e| e.to_string())?;
    Ok(summary.to_markdown())
}

#[tauri::command]
pub async fn regenerate_final_summary(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    let generator = SummaryGenerator::new(app, Language::default());
    generator.regenerate_final_summary(meeting_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_llm_connection(app: AppHandle) -> Result<String, String> {
    let language = Language::default();
    let test_system_prompt = PromptManager::test_connection(&language);
    let test_user_prompt = PromptManager::test_user_message(&language);

    // Get LLM config
    let state = app.state::<Mutex<AppState>>();
    let config = {
        let state = state.lock().await;
        state.llm_config.clone()
    };

    let llm_service = LlmService::new(config.external_endpoint, config.external_model);
    let progress_tracker = ProgressTracker::new(app.clone(), 1);

    progress_tracker.emit_api_status("Starting LLM connection test...")
        .map_err(|e| format!("Failed to emit progress: {}", e))?;

    // Reset progress indicators
    app.emit("llm-download-progress", 0).map_err(|e| e.to_string())?;
    app.emit("llm-loading-progress", 0).map_err(|e| e.to_string())?;

    match llm_service
        .generate_text(
            test_system_prompt,
            test_user_prompt,
            None,
            Some(&progress_tracker),
        )
        .await
    {
        Ok(response) => {
            progress_tracker.emit_api_status("LLM test completed successfully!")
                .map_err(|e| format!("Failed to emit progress: {}", e))?;
            Ok(format!("Test successful! Response: {}", response.trim()))
        }
        Err(e) => {
            progress_tracker.emit_api_status(&format!("LLM test failed: {}", e))
                .map_err(|e| format!("Failed to emit progress: {}", e))?;
            Err(format!("Test failed: {}", e))
        }
    }
}
