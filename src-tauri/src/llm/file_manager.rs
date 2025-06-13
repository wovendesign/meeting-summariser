use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tokio::fs;
use crate::MeetingMetadata;
use crate::llm::models::{FinalSummaryFormat, MeetingToMarkdown};

pub struct FileManager {
    app_handle: AppHandle,
}

impl FileManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    pub fn get_meeting_dir(&self, meeting_id: &str) -> Result<PathBuf, String> {
        let app_dir = self
            .app_handle
            .path()
            .app_local_data_dir()
            .map_err(|e| format!("Failed to get app local data directory: {}", e))?;
        Ok(app_dir.join("uploads").join(meeting_id))
    }

    pub fn get_chunks_dir(&self, meeting_id: &str) -> Result<PathBuf, String> {
        Ok(self.get_meeting_dir(meeting_id)?.join("chunks"))
    }

    pub async fn ensure_chunks_dir_exists(&self, meeting_id: &str) -> Result<(), String> {
        let chunks_dir = self.get_chunks_dir(meeting_id)?;
        fs::create_dir_all(&chunks_dir)
            .await
            .map_err(|e| format!("Failed to create chunks directory: {}", e))
    }

    pub async fn save_chunk(&self, meeting_id: &str, chunk_index: usize, content: &str) -> Result<(), String> {
        self.ensure_chunks_dir_exists(meeting_id).await?;
        let chunks_dir = self.get_chunks_dir(meeting_id)?;
        let chunk_file = chunks_dir.join(format!("chunk_{:03}.txt", chunk_index + 1));
        
        fs::write(&chunk_file, content)
            .await
            .map_err(|e| format!("Failed to save chunk {}: {}", chunk_index + 1, e))
    }

    pub async fn save_chunk_summary(&self, meeting_id: &str, chunk_index: usize, summary: &str) -> Result<(), String> {
        let chunks_dir = self.get_chunks_dir(meeting_id)?;
        let summary_file = chunks_dir.join(format!("chunk_{:03}_summary.json", chunk_index + 1));
        
        fs::write(&summary_file, summary)
            .await
            .map_err(|e| format!("Failed to save chunk summary {}: {}", chunk_index + 1, e))
    }

    pub async fn save_all_chunk_summaries(&self, meeting_id: &str, summaries: &[String]) -> Result<(), String> {
        let chunks_dir = self.get_chunks_dir(meeting_id)?;
        let all_chunks_summary_file = chunks_dir.join("all_chunk_summaries.md");
        
        let all_summaries_content = summaries
            .iter()
            .enumerate()
            .map(|(i, summary)| format!("# Chunk {} Summary\n\n{}", i + 1, summary))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        fs::write(&all_chunks_summary_file, &all_summaries_content)
            .await
            .map_err(|e| format!("Failed to save all chunk summaries: {}", e))
    }

    pub async fn save_final_summary(&self, meeting_id: &str, content: &FinalSummaryFormat) -> Result<(), String> {
        let meeting_dir = self.get_meeting_dir(meeting_id)?;
        let summary_path = meeting_dir.join("summary.md");
        let summary_json_path = meeting_dir.join("summary.json");

        let markdown = content.to_markdown();
        fs::write(summary_path, markdown)
            .await
            .map_err(|e| format!("Failed to save summary markdown: {}", e))?;

        let json = serde_json::to_string(content)
            .map_err(|e| format!("Failed to serialize summary: {}", e))?;
        fs::write(summary_json_path, json)
            .await
            .map_err(|e| format!("Failed to save summary JSON: {}", e))?;

        Ok(())
    }

    pub async fn read_summary(&self, meeting_id: &str) -> Result<FinalSummaryFormat, String> {
        let meeting_dir = self.get_meeting_dir(meeting_id)?;
        let summary_path = meeting_dir.join("summary.json");

        let summary_json = fs::read_to_string(summary_path)
            .await
            .map_err(|e| format!("Failed to read summary file: {}", e))?;

        serde_json::from_str(&summary_json)
            .map_err(|e| format!("Failed to parse summary JSON: {}", e))
    }

    pub fn save_meeting_metadata(&self, meeting_id: &str, name: String) -> Result<(), String> {
        let meeting_dir = self.get_meeting_dir(meeting_id)?;
        let metadata_path = meeting_dir.join("meeting.json");

        // Try to read existing metadata to preserve created_at
        let created_at = if let Ok(content) = std::fs::read_to_string(&metadata_path) {
            if let Ok(existing_metadata) = serde_json::from_str::<MeetingMetadata>(&content) {
                existing_metadata.created_at
            } else {
                Some(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
            }
        } else {
            Some(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
        };

        let metadata = MeetingMetadata {
            id: meeting_id.to_string(),
            name: Some(name),
            created_at,
        };

        let json = serde_json::to_string(&metadata)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
        std::fs::write(metadata_path, json)
            .map_err(|e| format!("Failed to write metadata: {}", e))?;

        Ok(())
    }
}
