use std::time::Instant;
use tauri::{AppHandle, Emitter};

pub struct ProgressTracker {
    app_handle: AppHandle,
    #[allow(dead_code)]
    start_time: Instant,
    total_steps: usize,
    current_step: usize,
}

impl ProgressTracker {
    pub fn new(app_handle: AppHandle, total_steps: usize) -> Self {
        Self {
            app_handle,
            start_time: Instant::now(),
            total_steps,
            current_step: 0,
        }
    }

    pub fn start_summarization(&self, meeting_id: &str) -> Result<(), String> {
        self.app_handle
            .emit("summarization-started", meeting_id)
            .map_err(|e| format!("Failed to emit summarization-started: {}", e))?;
        
        self.app_handle
            .emit("summarization-chunk-start", self.total_steps)
            .map_err(|e| format!("Failed to emit summarization-chunk-start: {}", e))?;

        Ok(())
    }

    pub fn update_progress(&mut self, message: &str) -> Result<(), String> {
        self.current_step += 1;
        
        self.app_handle
            .emit("summarization-chunk-progress", self.current_step - 1)
            .map_err(|e| format!("Failed to emit chunk progress: {}", e))?;

        let progress_message = format!(
            "Step {}/{}: {}",
            self.current_step, self.total_steps, message
        );
        
        self.app_handle
            .emit("llm-progress", &progress_message)
            .map_err(|e| format!("Failed to emit progress: {}", e))?;

        Ok(())
    }

    pub fn log_chunk_completed(&self, chunk_index: usize, duration: std::time::Duration) {
        println!(
            "✅ Chunk {} completed in {:.2}s",
            chunk_index + 1,
            duration.as_secs_f64()
        );
    }

    pub fn log_timing_stats(&self, chunk_times: &[std::time::Duration]) -> Result<(), String> {
        if chunk_times.is_empty() {
            return Ok(());
        }

        let total_chunk_time: std::time::Duration = chunk_times.iter().sum();
        let average_chunk_time = total_chunk_time / chunk_times.len() as u32;
        let min_chunk_time = chunk_times.iter().min().unwrap();
        let max_chunk_time = chunk_times.iter().max().unwrap();

        println!("📊 Chunk timing statistics:");
        println!(
            "   Total chunk processing time: {:.2}s",
            total_chunk_time.as_secs_f64()
        );
        println!(
            "   Average chunk time: {:.2}s",
            average_chunk_time.as_secs_f64()
        );
        println!("   Fastest chunk: {:.2}s", min_chunk_time.as_secs_f64());
        println!("   Slowest chunk: {:.2}s", max_chunk_time.as_secs_f64());

        let stats_message = format!(
            "📊 Chunk stats: Avg {:.1}s/chunk, Total {:.1}s for {} chunks",
            average_chunk_time.as_secs_f64(),
            total_chunk_time.as_secs_f64(),
            chunk_times.len()
        );

        self.app_handle
            .emit("llm-progress", &stats_message)
            .map_err(|e| format!("Failed to emit timing stats: {}", e))?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn complete(&self, message: &str) -> Result<(), String> {
        let total_duration = self.start_time.elapsed();
        
        println!("🎉 {}", message);
        println!(
            "⏱️  Total time: {:.2}s",
            total_duration.as_secs_f64()
        );

        let completion_message = format!(
            "✅ {} in {:.1}s",
            message,
            total_duration.as_secs_f64()
        );

        self.app_handle
            .emit("llm-progress", &completion_message)
            .map_err(|e| format!("Failed to emit completion: {}", e))?;

        Ok(())
    }

    pub fn emit_api_status(&self, message: &str) -> Result<(), String> {
        self.app_handle
            .emit("llm-progress", message)
            .map_err(|e| format!("Failed to emit API status: {}", e))
    }
}
