use tauri::{AppHandle, Emitter, Manager, State};
use tokio::process::Command;
use tokio::sync::Mutex;
use crate::AppState;

#[tauri::command]
pub async fn check_python_installation() -> Result<(), String> {
    let output = Command::new("python3")
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to execute python3: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("python3 not found or returned error: {}", stderr.trim()))
    }
}

#[tauri::command]
pub async fn check_whisperx_installation() -> Result<(), String> {
    check_python_installation().await?;

    let output = Command::new("whisperx")
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to execute whisperx: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("whisperx not found or returned error: {}", stderr.trim()))
    }
}


#[tauri::command]
pub async fn transcribe(app: AppHandle, meeting_id: &str, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    // Check if WhisperX is Available
    check_whisperx_installation().await?;

    // Check if another transcription is already running
    // Lock the mutex to get mutable access:
    let mut state = state.lock().await;
    
    if state.currently_transcribing.is_some() {
       return Err("Another Transcription is running".to_string());
    }
    
    // Modify the state:
    state.currently_transcribing = Some(meeting_id.to_string());
    
    app.emit(meeting_id, "transcription-started").unwrap();

    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let file_name = format!("{}.ogg", meeting_id);
    let audio_path = base_dir.join(file_name);

    let output = Command::new("uvx")
        .arg("whisperx")
        .arg(&audio_path)
        .arg("--compute_type")
        .arg("int8")
        .arg("--diarize")
        .arg("--output_dir")
        .arg(&base_dir)
        .output()
        .await;

    state.currently_transcribing = None;
    
    match output {
        Ok(output) => {
            if output.status.success() {
                return Ok(())
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("whisperx not found or returned error: {}", stderr.trim()))
        },
        Err(e) => {
            return Err(format!("Failed to execute whisperx: {}", e));
        }
    }
}

#[tauri::command]
pub async fn is_transcribing(app: AppHandle) -> Result<Option<String>, String> {
    let state = app.state::<Mutex<AppState>>();
    // Lock the mutex to get mutable access:
    let state = state.lock().await;

    Ok(state.currently_transcribing.clone())
}