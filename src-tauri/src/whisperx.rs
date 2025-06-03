use crate::audio::{analyze_audio, split_audio_into_chunks, AudioChunk};
use std::process::Stdio;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
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
    // check_whisperx_installation().await?;

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

    let resource_path = app_dir.join("whisper");

    println!("{:?}", resource_path);

    // Spawn whisperx process with piped stdout and inherited stderr
    let mut child = Command::new(format!("{}/python/bin/python", resource_path.display()))
        .arg("-m whisperx")
        .arg(&audio_path)
        .arg("--compute_type")
        .arg("int8")
        .arg("--diarize")
        .arg("--output_dir")
        .arg(&base_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn whisperx: {}", e))?;

    // Pipe and read stderr concurrently (uvx logs may come here)
    let stderr = child.stderr.take().expect("Failed to take stderr");
    let meeting_clone = meeting_id.to_string();
    let stderr_task = tokio::spawn(async move {
        let mut errs = BufReader::new(stderr).lines();
        while let Some(line) = errs.next_line().await.map_err(|e| format!("Error reading stderr: {}", e))? {
            println!("{}", line);
        }
        Ok::<(), String>(())
    });

    // Read stdout line by line, print and emit events
    if let Some(stdout) = child.stdout.take() {
        let mut lines = BufReader::new(stdout).lines();
        while let Some(line) = lines.next_line().await.map_err(|e| format!("Error reading stdout: {}", e))? {
            println!("{}", line);
        }
    }

    // Wait for stderr reader to finish and process exit
    stderr_task.await.map_err(|e| format!("stderr task join error: {}", e))??;
    let status = child.wait().await.map_err(|e| format!("Failed to wait on whisperx: {}", e))?;

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
pub async fn transcribe_with_chunking(app: AppHandle, meeting_id: &str, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
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
        println!("Audio is longer than 30 minutes, splitting into {} chunks", audio_info.chunk_count);
        split_audio_into_chunks(&audio_path, &base_dir, meeting_id).await.map_err(|e| {
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

    for (i, chunk) in chunks.iter().enumerate() {
        println!("Transcribing chunk {} of {}", i + 1, chunks.len());
        
        let chunk_path = std::path::Path::new(&chunk.file_path);
        let chunk_dir = chunk_path.parent().unwrap();
        
        // Run whisperx on this chunk
        let result = transcribe_single_chunk(chunk_path, chunk_dir).await;
        
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
async fn transcribe_single_chunk(audio_path: &std::path::Path, output_dir: &std::path::Path) -> Result<(), String> {
    println!("Transcribing: {}", audio_path.display());

    let output = Command::new("uvx")
        .arg("whisperx")
        .arg(audio_path)
        .arg("--compute_type")
        .arg("int8")
        .arg("--diarize")
        .arg("--output_dir")
        .arg(output_dir)
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