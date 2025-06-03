use std::path::Path;
use tokio::process::Command;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioInfo {
    pub duration_seconds: f64,
    pub needs_splitting: bool,
    pub chunk_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioChunk {
    pub chunk_index: usize,
    pub start_time: f64,
    pub end_time: f64,
    pub file_path: String,
}

/// Check if FFmpeg is available on the system
pub async fn check_ffmpeg_installation() -> Result<(), String> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .await
        .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("ffmpeg not found or returned error: {}", stderr.trim()))
    }
}

/// Get audio duration using ffprobe
pub async fn get_audio_duration<P: AsRef<Path>>(audio_path: P) -> Result<f64, String> {
    check_ffmpeg_installation().await?;

    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("csv=p=0")
        .arg(audio_path.as_ref())
        .output()
        .await
        .map_err(|e| format!("Failed to execute ffprobe: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffprobe failed: {}", stderr.trim()));
    }

    let duration_str = String::from_utf8_lossy(&output.stdout);
    let duration: f64 = duration_str
        .trim()
        .parse()
        .map_err(|e| format!("Failed to parse duration '{}': {}", duration_str.trim(), e))?;

    Ok(duration)
}

/// Check audio length and determine if splitting is needed
pub async fn analyze_audio<P: AsRef<Path>>(audio_path: P) -> Result<AudioInfo, String> {
    let duration_seconds = get_audio_duration(&audio_path).await?;
    
    // 30 minutes = 1800 seconds
    const MAX_CHUNK_DURATION: f64 = 1800.0;
    
    let needs_splitting = duration_seconds > MAX_CHUNK_DURATION;
    let chunk_count = if needs_splitting {
        (duration_seconds / MAX_CHUNK_DURATION).ceil() as usize
    } else {
        1
    };

    Ok(AudioInfo {
        duration_seconds,
        needs_splitting,
        chunk_count,
    })
}

/// Split audio into chunks of maximum 30 minutes each
pub async fn split_audio_into_chunks<P: AsRef<Path>>(
    audio_path: P,
    output_dir: P,
    meeting_id: &str,
) -> Result<Vec<AudioChunk>, String> {
    let audio_info = analyze_audio(&audio_path).await?;
    
    if !audio_info.needs_splitting {
        // Return single chunk info for the original file
        return Ok(vec![AudioChunk {
            chunk_index: 0,
            start_time: 0.0,
            end_time: audio_info.duration_seconds,
            file_path: audio_path.as_ref().to_string_lossy().to_string(),
        }]);
    }

    check_ffmpeg_installation().await?;

    let mut chunks = Vec::new();
    const CHUNK_DURATION: f64 = 1800.0; // 30 minutes in seconds

    for i in 0..audio_info.chunk_count {
        let start_time = i as f64 * CHUNK_DURATION;
        let end_time = ((i + 1) as f64 * CHUNK_DURATION).min(audio_info.duration_seconds);
        let chunk_duration = end_time - start_time;

        let chunk_filename = format!("{}_chunk_{:02}.ogg", meeting_id, i);
        let chunk_path = output_dir.as_ref().join(&chunk_filename);

        println!("Creating chunk {}: {:.2}s to {:.2}s ({:.2}s duration)", 
                 i, start_time, end_time, chunk_duration);

        let output = Command::new("ffmpeg")
            .arg("-i")
            .arg(audio_path.as_ref())
            .arg("-ss")
            .arg(format!("{:.2}", start_time))
            .arg("-t")
            .arg(format!("{:.2}", chunk_duration))
            .arg("-c")
            .arg("copy")
            .arg("-y") // Overwrite output files
            .arg(&chunk_path)
            .output()
            .await
            .map_err(|e| format!("Failed to execute ffmpeg for chunk {}: {}", i, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ffmpeg failed for chunk {}: {}", i, stderr.trim()));
        }

        chunks.push(AudioChunk {
            chunk_index: i,
            start_time,
            end_time,
            file_path: chunk_path.to_string_lossy().to_string(),
        });
    }

    println!("Successfully created {} audio chunks", chunks.len());
    Ok(chunks)
}

/// Tauri command wrapper for check_ffmpeg_installation
#[tauri::command]
pub async fn check_ffmpeg_installation_command() -> Result<(), String> {
    check_ffmpeg_installation().await
}

/// Tauri command wrapper for get_audio_duration
#[tauri::command]
pub async fn get_audio_duration_command(app: AppHandle, meeting_id: &str) -> Result<f64, String> {
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let file_name = format!("{}.ogg", meeting_id);
    let audio_path = base_dir.join(file_name);

    get_audio_duration(audio_path).await
}

/// Tauri command wrapper for analyze_audio
#[tauri::command]
pub async fn analyze_audio_command(app: AppHandle, meeting_id: &str) -> Result<AudioInfo, String> {
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let file_name = format!("{}.ogg", meeting_id);
    let audio_path = base_dir.join(file_name);

    analyze_audio(audio_path).await
}

/// Tauri command wrapper for split_audio_into_chunks
#[tauri::command]
pub async fn split_audio_into_chunks_command(
    app: AppHandle,
    meeting_id: &str,
) -> Result<Vec<AudioChunk>, String> {
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let file_name = format!("{}.ogg", meeting_id);
    let audio_path = base_dir.join(file_name);

    split_audio_into_chunks(audio_path, base_dir, meeting_id).await
}