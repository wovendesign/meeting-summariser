use tauri::{AppHandle, Manager};
use tokio::process::Command;

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


#[tauri::command(async)]
pub async fn transcribe(app: AppHandle, meeting_id: &str) -> Result<(), String> {
    check_whisperx_installation().await?;

    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let file_name = format!("{}.ogg", meeting_id);
    let audio_path = base_dir.join(file_name);

    // whisperx '${filepath}' --compute_type int8 --diarize --output_dir '${outDir}'

    let output = Command::new("uvx")
        .arg("whisperx")
        .arg(&audio_path)
        .arg("--compute_type")
        .arg("int8")
        .arg("--diarize")
        .arg("--output_dir")
        .arg(&base_dir)
        .output()
        .await
        .map_err(|e| format!("Failed to execute whisperx: {}", e))?;

    if output.status.success() {
        return Ok(())
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("whisperx not found or returned error: {}", stderr.trim()))
}