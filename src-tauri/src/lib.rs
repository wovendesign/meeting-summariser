use tauri_plugin_fs::FsExt;
use tokio::fs;
use tauri::{AppHandle, Manager};
use tauri::ipc::Response;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_meetings(app: AppHandle) -> Result<Vec<String>, String> {
    // resolve <app>/uploads
    let app_dir = app.path().app_local_data_dir().expect("Failed to get app local data directory");
    let uploads = app_dir.join("uploads");

    // read directory
    let mut rd = fs::read_dir(uploads)
        .await
        .map_err(|e| e.to_string())?;

    let mut folders = Vec::new();
    while let Some(entry) = rd
        .next_entry()
        .await
        .map_err(|e| e.to_string())?
    {
        let ft = entry
            .file_type()
            .await
            .map_err(|e| e.to_string())?;
        if ft.is_dir() {
            folders.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    Ok(folders)
}

#[tauri::command]
async fn add_meeting(app: AppHandle, name: &str) -> Result<(), String> {
    // resolve <app>/uploads
    let app_dir = app.path().app_local_data_dir().expect("Failed to get app local data directory");
    let uploads = app_dir.join("uploads");

    // ensure uploads directory exists
    fs::create_dir_all(&uploads).await.map_err(|e| e.to_string())?;

    // create new meeting folder
    let meeting_dir = uploads.join(name);
    fs::create_dir(&meeting_dir).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_meeting_transcript(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    // resolve <app>/uploads/<name>/transcript.txt
    let app_dir = app.path().app_local_data_dir().expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let file_name = format!("{}.txt", meeting_id);
    let transcript_path = base_dir.join(file_name);

    // read transcript file
    fs::read_to_string(transcript_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_meeting_transcript_json(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    // resolve <app>/uploads/<name>/transcript.txt
    let app_dir = app.path().app_local_data_dir().expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let file_name = format!("{}.json", meeting_id);
    let transcript_path = base_dir.join(file_name);

    // read transcript file
    fs::read_to_string(transcript_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_meeting_summary(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    // resolve <app>/uploads/<name>/summary.md
    let app_dir = app.path().app_local_data_dir().expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let summary_path = base_dir.join("summary.md");

    // read summary file
    fs::read_to_string(summary_path)
        .await
        .map_err(|e| e.to_string())
}

struct MeetingData {
    id: String,
    name: String,
    transcript: String,
    summary: String,
    audio: String,
}


#[tauri::command]
async fn get_meeting_audio(app: AppHandle, meeting_id: &str) -> Result<Response, String> {
    let app_dir = app.path().app_local_data_dir().expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let file_name = format!("{}.webm", meeting_id);
    let audio_path = base_dir.join(file_name);

    let data = fs::read(audio_path);
    match data.await {
        Ok(audio_data) => {
            // Create a response with the audio data
            let response = Response::new(audio_data);
            return Ok(response);
        },
        Err(e) => return Err(e.to_string()),
    }
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let scope = app.fs_scope();
            // scope.allow_directory("/path/to/directory", false);
            // dbg!(scope.allowed());

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            get_meetings, 
            add_meeting,
            get_meeting_transcript,
            get_meeting_summary,
            get_meeting_audio,
            get_meeting_transcript_json
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
