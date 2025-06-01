use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::ipc::Response;
use tauri::{AppHandle, Manager};
use tokio::fs;

mod whisperx;
mod llm;
mod audio;
mod meeting;

#[derive(Default)]
struct AppState {
    currently_transcribing: Option<String>,
    currently_summarizing: Option<String>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[derive(Serialize, Deserialize)]
struct MeetingMetadata {
    id: String,
    name: Option<String>,
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
    let content = fs::read_to_string(metadata_path)
        .await
        .map_err(|e| e.to_string());

    if let Ok(content) = content {
        serde_json::from_str::<MeetingMetadata>(&content).map_err(|e| e.to_string())
    } else {
        Ok(MeetingMetadata {
            id: meeting_id.to_string(),
            name: None
        })
    }
    // serde_json::from_str(&content).map_err(|e| e.to_string())
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
    }
}



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            llm::generate_meeting_name,
            llm::generate_summary,
            llm::is_summarizing,
            whisperx::check_python_installation,
            whisperx::check_whisperx_installation,
            whisperx::transcribe,
            whisperx::is_transcribing,
            meeting::save_speaker_names
        ])
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
