use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use serde::{Deserialize, Serialize};
use tauri::ipc::Response;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_fs::FsExt;
use tokio::fs;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_meetings(app: AppHandle) -> Result<Vec<String>, String> {
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
    Ok(folders)
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
    // resolve <app>/uploads/<name>/transcript.txt
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
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
async fn get_meeting_summary(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    // resolve <app>/uploads/<name>/summary.md
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let summary_path = base_dir.join("summary.md");

    // read summary file
    fs::read_to_string(summary_path)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Serialize, Deserialize)]
struct MeetingMetadata {
    name: String,
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
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
async fn generate_meeting_name(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    println!("Generating meeting name for {}", meeting_id);
    // notify frontend that generation has started
    app.emit("meeting-name-generation-started", meeting_id)
        .unwrap();

    // Get Meeting Summary
    let meeting_summary = get_meeting_summary(app.clone(), meeting_id).await;

    match meeting_summary {
        Ok(summary) => {
            // Send Request to Ollama
            let mut client = OpenAIClient::builder()
                .with_endpoint("http://localhost:11434/v1")
                .build()
                .map_err(|e| e.to_string())?;

            let req = ChatCompletionRequest::new(
                "llama3".to_string(),
                vec![
                    chat_completion::ChatCompletionMessage {
                    role: chat_completion::MessageRole::system,
                    content: chat_completion::Content::Text(String::from("You are a meeting summarization assistant. generate a concise and relevant meeting name based on the provided summary. In front of the meeting name, add a fitting emoji. The name is supposed to be short (max 6 words), concise and relevant to the meeting summary.")),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    },
                    chat_completion::ChatCompletionMessage {
                    role: chat_completion::MessageRole::user,
                    content: chat_completion::Content::Text(summary),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                }
                ],
            );

            let result = client
                .chat_completion(req)
                .await
                .map_err(|e| e.to_string())?;
            println!("Content: {:?}", result.choices[0].message.content);
            let content = result.choices[0].message.content.clone();
            match content {
                Some(name_str) => {
                    // Add it to meeting.json if it exists
                    let app_dir = app
                        .path()
                        .app_local_data_dir()
                        .expect("Failed to get app local data directory");
                    let metadata_path = app_dir
                        .join("uploads")
                        .join(meeting_id)
                        .join("meeting.json");

                    if !metadata_path.exists() {
                        // Create file metadata.json
                        let metadata = MeetingMetadata {
                            name: name_str.to_string(),
                        };
                        let json = serde_json::to_string(&metadata).map_err(|e| e.to_string())?;
                        fs::write(metadata_path, json)
                            .await
                            .map_err(|e| e.to_string())?;
                        return Ok(name_str.to_string());
                    } else {
                        // If meeting.json does not exist, create it
                        let metadata = MeetingMetadata {
                            name: name_str.to_string(),
                        };
                        let json = serde_json::to_string(&metadata).map_err(|e| e.to_string())?;
                        fs::write(metadata_path, json)
                            .await
                            .map_err(|e| e.to_string())?;
                        return Ok(name_str.to_string());
                    }
                }
                None => {
                    return Err("No content returned from Ollama".to_string());
                }
            }
        }
        Err(e) => Err(format!("Failed to get meeting summary: {}", e)),
    }
}

#[tauri::command]
async fn get_meeting_audio(app: AppHandle, meeting_id: &str) -> Result<Response, String> {
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let file_name = format!("{}.webm", meeting_id);
    let audio_path = base_dir.join(file_name);

    let data = fs::read(audio_path);
    match data.await {
        Ok(audio_data) => {
            // Create a response with the audio data
            let response = Response::new(audio_data);
            return Ok(response);
        }
        Err(e) => return Err(e.to_string()),
    }
}

#[tauri::command]
async fn start_generate_summary(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    // This function is a placeholder for generating a summary
    // In a real application, you would implement the logic to generate a summary here
    let summary = format!("Summary for meeting {}", meeting_id);
    Ok(summary)
}

#[tauri::command]
fn generate_summary(app: AppHandle, meeting_id: &str) {
    app.emit("summarization-started", &meeting_id).unwrap();
    for progress in [1, 15, 50, 80, 100] {
        app.emit("summarization-progress", progress).unwrap();
    }
    app.emit("summarization-finished", &meeting_id).unwrap();
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
            get_meeting_transcript_json,
            get_meeting_metadata,
            generate_meeting_name,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
