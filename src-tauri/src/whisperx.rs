use tauri::{AppHandle, Manager};
use tokio::fs;
use tokio::process::Command;
use kalosm::sound::*;
use kalosm::sound::dasp::sample::ToSample;

#[tauri::command]
pub async fn check_python_installation(app: AppHandle) -> Result<(), String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let python_dir = app_dir.join("python");

    if fs::try_exists(&python_dir).await.map_err(|e| e.to_string())? {
        // Check version
        let python_exe = if cfg!(target_os = "windows") {
            python_dir.join("python.exe")
        } else {
            python_dir.join("bin/python")
        };
        if !fs::try_exists(&python_exe).await.map_err(|e| e.to_string())? {
            return Err("Python executable not found".to_string());
        }
        let output = Command::new(python_exe)
            .arg("--version")
            .output()
            .await
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Python version check failed: {}", stderr));
        }
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("Python version: {}", version);

        return Ok(());
    }

    //    Return error
    Err("Python installation check is not implemented yet.".to_string())
}

#[tauri::command]
pub async fn install_python(app: AppHandle) -> Result<(), String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    let python_dir = app_dir.join("python");

    fs::create_dir_all(&python_dir)
        .await
        .map_err(|e| e.to_string())?;

    return Ok(());
}

#[tauri::command(async)]
pub  async fn transcribe(app: AppHandle) -> Result<(), String> {
    let model = Whisper::builder()
        .build_with_loading_handler(|progress| match &progress {
            ModelLoadingProgress::Downloading {
                source,
                progress: file_loading_progress,
            } => {
                let progress = (progress.progress() * 100.0) as u32;
                let elapsed = file_loading_progress.start_time.elapsed().as_secs_f32();
                println!("Downloading file {source} {progress}% ({elapsed}s)");
            }
            ModelLoadingProgress::Loading { progress } => {
                let progress = (progress * 100.0) as u32;
                println!("Loading model {progress}%");
            }
        })
        .await.map_err(|e| e.to_string())?;

    // Stream audio from the microphone
    let mic = MicInput::default();
    let stream = mic.stream();

    // Transcribe the audio.
    let mut transcribed = stream.transcribe(model);

    // As the model transcribes the audio, print the text to the console.
    transcribed.to_std_out().await.map_err(|e| e.to_string())?;
    return Ok(());
}