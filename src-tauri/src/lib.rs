use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::ipc::Response;
use tauri::{AppHandle, Manager};
use tokio::fs;
use tokio::sync::Mutex;

mod audio;
mod llm;
mod meeting;
mod whisperx;

#[derive(Default)]
struct AppState {
    currently_transcribing: Option<String>,
    currently_summarizing: Option<String>,
    llm_config: LlmConfig,
}

#[derive(Clone, Serialize, Deserialize)]
struct LlmConfig {
    use_external_api: bool,
    external_endpoint: String,
    external_model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            use_external_api: true, // Try external first
            external_endpoint: "http://localhost:11434/v1".to_string(),
            external_model: "llama3".to_string(),
        }
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[derive(Serialize, Deserialize)]
struct MeetingMetadata {
    id: String,
    name: Option<String>,
    created_at: Option<String>, // ISO 8601 date string
}
#[tauri::command]
async fn get_meetings(app: AppHandle) -> Result<Vec<MeetingMetadata>, String> {
    // resolve <app>/uploads
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let uploads = app_dir.join("uploads");

    // read directory
    let mut rd = fs::read_dir(uploads).await.map_err(|e| e.to_string())?;

    let mut folders = Vec::new();
    while let Some(entry) = rd.next_entry().await.map_err(|e| e.to_string())? {
        let ft = entry.file_type().await.map_err(|e| e.to_string())?;
        if ft.is_dir() {
            folders.push(entry.file_name().to_string_lossy().into_owned());
        }
    }

    // fetch metadata for each folder
    let mut meetings = Vec::new();
    for id in folders {
        let metadata = get_meeting_metadata(app.clone(), &id).await?;

        meetings.push(metadata);
    }

    Ok(meetings)
}

#[tauri::command]
async fn add_meeting(app: AppHandle, name: &str) -> Result<(), String> {
    // resolve <app>/uploads
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let uploads = app_dir.join("uploads");

    // ensure uploads directory exists
    fs::create_dir_all(&uploads)
        .await
        .map_err(|e| e.to_string())?;

    // create new meeting folder
    let meeting_dir = uploads.join(name);
    fs::create_dir(&meeting_dir)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_meeting_transcript(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    println!("Getting meeting transcript for {}", meeting_id);

    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let file_name = format!("{}.txt", meeting_id);
    let transcript_path = base_dir.join(file_name);

    println!("Path: {}", transcript_path.display());

    // read transcript file
    fs::read_to_string(transcript_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_meeting_transcript_json(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    // resolve <app>/uploads/<name>/transcript.txt
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let file_name = format!("{}.json", meeting_id);
    let transcript_path = base_dir.join(file_name);

    // read transcript file
    fs::read_to_string(transcript_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_llm_config(app: AppHandle) -> Result<LlmConfig, String> {
    let state = app.state::<Mutex<AppState>>();
    let state = state.lock().await;
    Ok(state.llm_config.clone())
}

#[tauri::command]
async fn set_llm_config(
    app: AppHandle,
    use_external_api: bool,
    external_endpoint: String,
    external_model: String,
) -> Result<(), String> {
    let state = app.state::<Mutex<AppState>>();
    let mut state = state.lock().await;
    state.llm_config = LlmConfig {
        use_external_api,
        external_endpoint,
        external_model,
    };
    Ok(())
}

#[tauri::command]
async fn get_meeting_metadata(app: AppHandle, meeting_id: &str) -> Result<MeetingMetadata, String> {
    // resolve <app>/uploads/<meeting_id>/meeting.json
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let metadata_path = app_dir
        .join("uploads")
        .join(meeting_id)
        .join("meeting.json");

    // read and parse JSON
    let content = fs::read_to_string(&metadata_path)
        .await
        .map_err(|e| e.to_string());

    if let Ok(content) = content {
        let mut metadata: MeetingMetadata =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;

        // If created_at is missing, try to get it from file creation time or meeting_id
        if metadata.created_at.is_none() {
            metadata.created_at = get_fallback_date(&metadata_path, meeting_id).await;
        }

        Ok(metadata)
    } else {
        // Create new metadata with current date
        let created_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string());

        Ok(MeetingMetadata {
            id: meeting_id.to_string(),
            name: None,
            created_at,
        })
    }
}

#[tauri::command]
async fn get_meeting_audio(app: AppHandle, meeting_id: &str) -> Result<Response, String> {
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let file_name = format!("{}.ogg", meeting_id);
    let audio_path = base_dir.join(file_name);

    let data = fs::read(audio_path);
    return match data.await {
        Ok(audio_data) => {
            // Create a response with the audio data
            let response = Response::new(audio_data);
            Ok(response)
        }
        Err(e) => Err(e.to_string()),
    };
}

#[tauri::command]
async fn rename_meeting(app: AppHandle, meeting_id: &str, new_name: &str) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let meeting_dir = app_dir.join("uploads").join(meeting_id);
    let metadata_path = meeting_dir.join("meeting.json");

    // Get existing metadata or create new one
    let mut metadata = if metadata_path.exists() {
        let content = fs::read_to_string(&metadata_path)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::from_str::<MeetingMetadata>(&content).map_err(|e| e.to_string())?
    } else {
        MeetingMetadata {
            id: meeting_id.to_string(),
            name: None,
            created_at: Some(Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
        }
    };

    // Update the name
    metadata.name = Some(new_name.to_string());

    // Write back to file
    let json_content = serde_json::to_string_pretty(&metadata).map_err(|e| e.to_string())?;
    fs::write(&metadata_path, json_content)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Helper function to get fallback date from file creation time or meeting_id
async fn get_fallback_date(metadata_path: &Path, meeting_id: &str) -> Option<String> {
    // Try to get file creation time from the parent directory (meeting directory)
    if let Ok(metadata) = fs::metadata(metadata_path.parent()?).await {
        if let Ok(created) = metadata.created() {
            if let Ok(duration) = created.duration_since(UNIX_EPOCH) {
                if let Some(dt) = DateTime::from_timestamp(duration.as_secs() as i64, 0) {
                    return Some(dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string());
                }
            }
        }
    }

    // Fallback: try to parse timestamp from meeting_id (format: recording-{timestamp})
    if meeting_id.starts_with("recording-") {
        if let Ok(timestamp) = meeting_id.trim_start_matches("recording-").parse::<i64>() {
            if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
                return Some(dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string());
            }
        }
    }

    // Final fallback: current time
    Some(Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_meetings,
            add_meeting,
            get_meeting_transcript,
            get_meeting_audio,
            get_meeting_transcript_json,
            get_meeting_metadata,
            llm::get_meeting_summary,
            llm::generate_summary,
            llm::is_summarizing,
            llm::test_llm_connection,
            whisperx::check_python_installation,
            whisperx::check_whisperx_installation,
            whisperx::transcribe,
            whisperx::transcribe_with_chunking,
            whisperx::is_transcribing,
            whisperx::download_python,
            whisperx::download_whisperx,
            meeting::save_speaker_names,
            audio::check_ffmpeg_installation_command,
            audio::get_audio_duration_command,
            audio::analyze_audio_command,
            audio::split_audio_into_chunks_command,
            audio::convert_user_audio,
            get_llm_config,
            set_llm_config,
            rename_meeting
        ])
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
