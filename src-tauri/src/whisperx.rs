use crate::audio::{analyze_audio, split_audio_into_chunks, AudioChunk};
use crate::AppState;
use std::process::Stdio;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_http::reqwest;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

/// Detects the current platform and returns the appropriate Python download URL
fn get_python_download_url() -> Result<String, String> {
    let base_url =
        "https://github.com/astral-sh/python-build-standalone/releases/download/20250529";

    // Detect OS
    let os = std::env::consts::OS;

    // Detect architecture
    let arch = std::env::consts::ARCH;

    let filename = match (os, arch) {
        // Linux x86_64
        ("linux", "x86_64") => {
            "cpython-3.12.10+20250529-x86_64-unknown-linux-gnu-install_only.tar.gz"
        }

        // Linux aarch64 (ARM64)
        ("linux", "aarch64") => {
            "cpython-3.12.10+20250529-aarch64-unknown-linux-gnu-install_only.tar.gz"
        }

        // macOS x86_64 (Intel)
        ("macos", "x86_64") => "cpython-3.12.10+20250529-x86_64-apple-darwin-install_only.tar.gz",

        // macOS aarch64 (Apple Silicon)
        ("macos", "aarch64") => "cpython-3.12.10+20250529-aarch64-apple-darwin-install_only.tar.gz",

        // Windows x86_64
        ("windows", "x86_64") => {
            "cpython-3.12.10+20250529-x86_64-pc-windows-msvc-install_only.tar.gz"
        }

        // Windows x86 (32-bit)
        ("windows", "x86") => "cpython-3.12.10+20250529-i686-pc-windows-msvc-install_only.tar.gz",

        // Additional Linux architectures
        ("linux", "arm") => {
            "cpython-3.12.10+20250529-armv7-unknown-linux-gnueabihf-install_only.tar.gz"
        }
        ("linux", "powerpc64") => {
            "cpython-3.12.10+20250529-ppc64le-unknown-linux-gnu-install_only.tar.gz"
        }
        ("linux", "riscv64") => {
            "cpython-3.12.10+20250529-riscv64-unknown-linux-gnu-install_only.tar.gz"
        }
        ("linux", "s390x") => {
            "cpython-3.12.10+20250529-s390x-unknown-linux-gnu-install_only.tar.gz"
        }

        // Unsupported combination
        _ => {
            return Err(format!(
                "Unsupported platform: {} on {}. Supported platforms are:\n\
            - Linux: x86_64, aarch64, arm, powerpc64, riscv64, s390x\n\
            - macOS: x86_64, aarch64\n\
            - Windows: x86_64, x86",
                arch, os
            ))
        }
    };

    Ok(format!("{}/{}", base_url, filename))
}

/// Gets the Python executable path based on the platform
fn get_python_executable_path(python_dir: &std::path::Path) -> String {
    if cfg!(windows) {
        format!("{}/python.exe", python_dir.display())
    } else {
        format!("{}/bin/python3", python_dir.display())
    }
}

#[tauri::command]
pub async fn check_python_installation(app: AppHandle) -> Result<(), String> {
    // Check if Python is installed
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let resource_path = app_dir.join("python");

    let python_exe = get_python_executable_path(&resource_path);

    let output = Command::new(&python_exe)
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to execute python3: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "python3 not found or returned error: {}",
            stderr.trim()
        ))
    }
}

#[tauri::command]
pub async fn download_python(app: AppHandle) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let resource_path = app_dir;

    // Ensure the resource directory exists
    fs::create_dir_all(&resource_path)
        .await
        .map_err(|e| format!("Failed to create resource directory: {}", e))?;

    app.emit(
        "python-download-progress",
        "Detecting platform and selecting Python version...",
    )
    .unwrap();

    // Get the appropriate download URL for this platform
    let download_url = get_python_download_url()?;

    app.emit(
        "python-download-progress",
        &format!("Downloading Python from: {}", download_url),
    )
    .unwrap();

    let res = reqwest::get(&download_url).await;

    if res.is_err() {
        return Err(format!("Failed to download Python: {}", res.unwrap_err()));
    }
    let response = res.unwrap();
    if !response.status().is_success() {
        return Err(format!("Failed to download Python: {}", response.status()));
    }

    app.emit("python-download-progress", "Downloading Python tarball...")
        .unwrap();

    let tarball = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read Python tarball bytes: {}", e))?;

    // Determine file extension based on URL
    let file_extension = if download_url.ends_with(".tar.gz") {
        "python.tar.gz"
    } else {
        "python.tar.zst"
    };

    let tarball_path = resource_path.join(file_extension);
    fs::write(&tarball_path, &tarball)
        .await
        .map_err(|e| format!("Failed to write Python tarball: {}", e))?;

    app.emit("python-download-progress", "Extracting Python...")
        .unwrap();

    // Extract the tarball - use appropriate command based on file type
    let extract_result = if download_url.ends_with(".tar.gz") {
        Command::new("tar")
            .arg("-xzf")
            .arg(&tarball_path)
            .arg("-C")
            .arg(&resource_path)
            .output()
            .await
    } else {
        // For .tar.zst files, use tar with zstd support
        Command::new("tar")
            .arg("--use-compress-program=zstd")
            .arg("-xf")
            .arg(&tarball_path)
            .arg("-C")
            .arg(&resource_path)
            .output()
            .await
    };

    let output = extract_result.map_err(|e| format!("Failed to extract Python tarball: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Failed to extract Python tarball: {}",
            stderr.trim()
        ));
    }

    app.emit("python-download-progress", "Cleaning up...")
        .unwrap();

    // Clean up the tarball
    fs::remove_file(&tarball_path)
        .await
        .map_err(|e| format!("Failed to remove Python tarball: {}", e))?;

    // Set permissions (Unix-like systems only)
    if cfg!(unix) {
        let python_dir = resource_path.join("python");

        app.emit("python-download-progress", "Setting permissions...")
            .unwrap();

        let output = Command::new("chmod")
            .arg("-R")
            .arg("755")
            .arg(&python_dir)
            .output()
            .await
            .map_err(|e| format!("Failed to set permissions on Python directory: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Failed to set permissions on Python directory: {}",
                stderr.trim()
            ));
        }
    }

    app.emit(
        "python-download-progress",
        "Python installation completed successfully!",
    )
    .unwrap();

    Ok(())
}

#[tauri::command]
pub async fn check_whisperx_installation(app: AppHandle) -> Result<(), String> {
    check_python_installation(app.clone()).await?;

    // Get python resource path
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let resource_path = app_dir.join("python");
    let lib_path = resource_path
        .join("lib")
        .join("python3.12")
        .join("site-packages");
    let python_exe = get_python_executable_path(&resource_path);
    let output = Command::new(&python_exe)
        .env("PYTHONPATH", &lib_path)
        .arg("-m")
        .arg("whisperx")
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to execute whisperx: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "whisperx not found or returned error: {}",
            stderr.trim()
        ))
    }
}

#[tauri::command]
pub async fn download_whisperx(app: AppHandle) -> Result<(), String> {
    // Check if Python is installed
    check_python_installation(app.clone()).await?;

    // Emit start event
    app.emit(
        "whisperx-download-progress",
        "Starting WhisperX download...",
    )
    .unwrap();

    // Get python resource path
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let resource_path = app_dir.join("python");

    // Ensure the lib directory exists for packages
    let lib_path = resource_path
        .join("lib")
        .join("python3.12")
        .join("site-packages");
    fs::create_dir_all(&lib_path)
        .await
        .map_err(|e| format!("Failed to create lib directory: {}", e))?;

    app.emit(
        "whisperx-download-progress",
        "Installing WhisperX and dependencies...",
    )
    .unwrap(); // Spawn pip install process with piped output for progress tracking
    let python_exe = get_python_executable_path(&resource_path);
    let mut child = Command::new(&python_exe)
        .arg("-m")
        .arg("pip")
        .arg("install")
        .arg("--target")
        .arg(&lib_path)
        .arg("--verbose")
        .arg("whisperx")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn pip install: {}", e))?;

    // Read and emit progress from both stdout and stderr
    let stdout = child.stdout.take().expect("Failed to take stdout");
    let stderr = child.stderr.take().expect("Failed to take stderr");

    let app_clone = app.clone();
    let stdout_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Some(line) = lines.next_line().await.unwrap_or(None) {
            if line.contains("Downloading")
                || line.contains("Installing")
                || line.contains("Successfully")
            {
                app_clone.emit("whisperx-download-progress", &line).unwrap();
            }
        }
    });

    let app_clone2 = app.clone();
    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Some(line) = lines.next_line().await.unwrap_or(None) {
            if line.contains("Downloading")
                || line.contains("Installing")
                || line.contains("Successfully")
            {
                app_clone2
                    .emit("whisperx-download-progress", &line)
                    .unwrap();
            }
        }
    });

    // Wait for all tasks to complete
    let _ = tokio::try_join!(stdout_task, stderr_task);

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait on pip install: {}", e))?;

    if !status.success() {
        app.emit("whisperx-download-progress", "Installation failed")
            .unwrap();
        return Err("Failed to install whisperx".to_string());
    }

    app.emit(
        "whisperx-download-progress",
        "WhisperX installation completed successfully!",
    )
    .unwrap();
    Ok(())
}

#[tauri::command]
pub async fn transcribe(
    app: AppHandle,
    meeting_id: &str,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    // Check if WhisperX is Available
    check_whisperx_installation(app.clone()).await?;

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

    println!("Uploading to {}", audio_path.display());

    let resource_path = app_dir.join("python");
    let lib_path = resource_path
        .join("lib")
        .join("python3.12")
        .join("site-packages");
    println!("{:?}", resource_path); // Spawn whisperx process with piped stdout and inherited stderr
    let python_exe = get_python_executable_path(&resource_path);
    let mut child = Command::new(&python_exe)
        .env("PYTHONPATH", &lib_path)
        .arg("-m")
        .arg("whisperx")
        .arg(&audio_path)
        .arg("--device")
        .arg("cpu")
        .arg("--compute_type")
        .arg("int8")
        .arg("--diarize")
        .arg("--output_dir")
        .arg(&base_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn whisperx: {}", e))?; // Pipe and read stderr concurrently (uvx logs may come here)
    let stderr = child.stderr.take().expect("Failed to take stderr");
    let stderr_task = tokio::spawn(async move {
        let mut errs = BufReader::new(stderr).lines();
        while let Some(line) = errs
            .next_line()
            .await
            .map_err(|e| format!("Error reading stderr: {}", e))?
        {
            println!("{}", line);
        }
        Ok::<(), String>(())
    });

    // Read stdout line by line, print and emit events
    if let Some(stdout) = child.stdout.take() {
        let mut lines = BufReader::new(stdout).lines();
        while let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| format!("Error reading stdout: {}", e))?
        {
            println!("{}", line);
        }
    }

    // Wait for stderr reader to finish and process exit
    stderr_task
        .await
        .map_err(|e| format!("stderr task join error: {}", e))??;
    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait on whisperx: {}", e))?;

    // Clear transcription state
    state.currently_transcribing = None;

    if status.success() {
        Ok(())
    } else {
        Err(format!("whisperx exited with status: {}", status))
    }
}

#[tauri::command]
pub async fn is_transcribing(app: AppHandle) -> Result<Option<String>, String> {
    let state = app.state::<Mutex<AppState>>();
    // Lock the mutex to get mutable access:
    let state = state.lock().await;

    Ok(state.currently_transcribing.clone())
}

/// Enhanced transcribe function that handles audio chunking automatically
#[tauri::command]
pub async fn transcribe_with_chunking(
    app: AppHandle,
    meeting_id: &str,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    // Check if another transcription is already running
    let mut state_lock = state.lock().await;

    if state_lock.currently_transcribing.is_some() {
        return Err("Another Transcription is running".to_string());
    }

    // Modify the state:
    state_lock.currently_transcribing = Some(meeting_id.to_string());
    drop(state_lock); // Release the lock early

    app.emit(meeting_id, "transcription-started").unwrap();

    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);
    let file_name = format!("{}.ogg", meeting_id);
    let audio_path = base_dir.join(file_name);

    println!("Analyzing audio file: {}", audio_path.display());

    // Analyze the audio to determine if chunking is needed
    let audio_info = analyze_audio(&audio_path).await.map_err(|e| {
        // Clear state on error
        let mut state_lock = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(state.lock())
        });
        state_lock.currently_transcribing = None;
        e
    })?;

    println!("Audio duration: {:.2} seconds", audio_info.duration_seconds);
    println!("Needs splitting: {}", audio_info.needs_splitting);
    println!("Chunk count: {}", audio_info.chunk_count);

    let chunks = if audio_info.needs_splitting {
        println!(
            "Audio is longer than 30 minutes, splitting into {} chunks",
            audio_info.chunk_count
        );
        split_audio_into_chunks(&audio_path, &base_dir, meeting_id, app.clone())
            .await
            .map_err(|e| {
                // Clear state on error
                let mut state_lock = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(state.lock())
                });
                state_lock.currently_transcribing = None;
                e
            })?
    } else {
        println!("Audio is under 30 minutes, processing as single file");
        vec![AudioChunk {
            chunk_index: 0,
            start_time: 0.0,
            end_time: audio_info.duration_seconds,
            file_path: audio_path.to_string_lossy().to_string(),
        }]
    };

    // Transcribe each chunk
    let mut all_transcripts = Vec::new();
    let mut all_json_parts = Vec::new();

    app.emit("whisperx-start", chunks.len()).unwrap();

    for (i, chunk) in chunks.iter().enumerate() {
        println!("Transcribing chunk {} of {}", i + 1, chunks.len());
        app.emit("whisperx-progress", i).unwrap();

        let chunk_path = std::path::Path::new(&chunk.file_path);
        let chunk_dir = chunk_path.parent().unwrap(); // Run whisperx on this chunk
        let result = transcribe_single_chunk(&app, chunk_path, chunk_dir).await;

        match result {
            Ok(_) => {
                // Read the generated transcript files for this chunk
                let chunk_stem = chunk_path.file_stem().unwrap().to_string_lossy();
                let txt_path = chunk_dir.join(format!("{}.txt", chunk_stem));
                let json_path = chunk_dir.join(format!("{}.json", chunk_stem));

                if let Ok(txt_content) = fs::read_to_string(&txt_path).await {
                    all_transcripts.push(txt_content);
                }

                if let Ok(json_content) = fs::read_to_string(&json_path).await {
                    all_json_parts.push(json_content);
                }
            }
            Err(e) => {
                println!("Warning: Failed to transcribe chunk {}: {}", i + 1, e);
                // Continue with other chunks rather than failing completely
            }
        }
    }

    // Combine all transcripts into final files
    let combined_transcript = all_transcripts.join("\n\n");
    let final_txt_path = base_dir.join(format!("{}.txt", meeting_id));

    if let Err(e) = fs::write(&final_txt_path, combined_transcript).await {
        println!("Warning: Failed to write combined transcript: {}", e);
    }

    // For JSON, we'll combine them into an array or concatenate based on format
    if !all_json_parts.is_empty() {
        let combined_json = if all_json_parts.len() == 1 {
            all_json_parts[0].clone()
        } else {
            // Combine multiple JSON chunks - this is a simple concatenation
            // In a real scenario, you might want to parse and properly merge JSON
            all_json_parts.join("\n")
        };

        let final_json_path = base_dir.join(format!("{}.json", meeting_id));
        if let Err(e) = fs::write(&final_json_path, combined_json).await {
            println!("Warning: Failed to write combined JSON transcript: {}", e);
        }
    }

    // Clear transcription state
    let mut state_lock = state.lock().await;
    state_lock.currently_transcribing = None;
    drop(state_lock);

    app.emit(meeting_id, "transcription-finished").unwrap();

    println!("Transcription completed for meeting {}", meeting_id);
    Ok(())
}

/// Helper function to transcribe a single audio chunk
async fn transcribe_single_chunk(
    app: &AppHandle,
    audio_path: &std::path::Path,
    output_dir: &std::path::Path,
) -> Result<(), String> {
    println!("Transcribing: {}", audio_path.display());

    // Get python resource path
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let resource_path = app_dir.join("python");
    let _lib_path = resource_path
        .join("lib")
        .join("python3.12")
        .join("site-packages");
    let _python_exe = get_python_executable_path(&resource_path);
    // let output = Command::new(&python_exe)
    //     .env("PYTHONPATH", &lib_path)
    //     .arg("-m")
    //     .arg("whisperx")
    //     .arg(audio_path)
    //     // .arg("--device")
    //     // .arg("cpu")
    //     .arg("--compute_type")
    //     .arg("int8")
    //     .arg("--diarize")
    //     .arg("--output_dir")
    //     .arg(output_dir)
    //     .arg("--hf_token")
    //     .arg("HFTOKEN")
    //     .output()
    //     .await
    //     .map_err(|e| format!("Failed to execute whisperx: {}", e))?;

    // uv run --with mlx_whisper mlx_whisper --model mlx-community/whisper-turbo --output-dir mlx --output-format all recording-1749583019.ogg
    let output = Command::new("uv")
        .arg("run")
        .arg("--with")
        .arg("mlx_whisper")
        .arg("mlx_whisper")
        .arg("--model")
        .arg("mlx-community/whisper-turbo")
        .arg("--output-dir")
        .arg(output_dir)
        .arg("--condition-on-previous-text")
        .arg("False")
        .arg(audio_path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute whisperx: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("whisperx failed: {}", stderr.trim()))
    }
}
