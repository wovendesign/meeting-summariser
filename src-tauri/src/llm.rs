use std::io::Write;
use crate::{get_meeting_transcript, AppState, LlmConfig, MeetingMetadata};
use kalosm::language::*;
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion;
use openai_api_rs::v1::chat_completion::ChatCompletionRequest;
use tauri::{AppHandle, Emitter, Manager};
use tokio::fs;
use tokio::sync::Mutex;

async fn generate_text_with_llm(
    app: AppHandle,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    println!("Generating text with llm");
    // let state = app.state::<Mutex<AppState>>();
    // let config = {
    //     let state = state.lock().await;
    //     state.llm_config.clone()
    // };

    // Try external API first if enabled
    if true {
        app.emit("llm-progress", "Trying external API...").unwrap();
        match try_external_api(system_prompt, user_prompt).await {
            Ok(response) => {
                app.emit("llm-progress", "External API successful").unwrap();
                return Ok(response);
            }
            Err(e) => {
                println!("External API failed: {}, falling back to Kalosm", e);
                app.emit("llm-progress", "External API failed, switching to local model...").unwrap();
            }
        }
    } else {
        app.emit("llm-progress", "Using local Kalosm model...").unwrap();
    }

    // Fallback to Kalosm
    try_kalosm(app.clone(), system_prompt, user_prompt).await
}

async fn try_external_api(
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    let mut client = OpenAIClient::builder()
        .with_endpoint("http://localhost:11434/v1")
        .build()
        .map_err(|e| e.to_string())?;

    println!("trying external ollama");

    let req = ChatCompletionRequest::new(
        "llama3".to_string(),
        vec![
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::system,
                content: chat_completion::Content::Text(system_prompt.to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: chat_completion::Content::Text(user_prompt.to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ],
    );

    let result = client
        .chat_completion(req)
        .await
        .map_err(|e| e.to_string())?;

    result.choices[0]
        .message
        .content
        .clone()
        .ok_or_else(|| "No content returned from external API".to_string())
}

async fn try_kalosm(
    app: AppHandle,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    use kalosm::language::*;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::timeout;

    println!("Starting kalosm...");

    app.emit("llm-progress", "Initializing Kalosm model...")
        .unwrap();

    println!("Downloading Kalosm model...\n");

    // Clone app handle for use in the closure
    let app_clone = app.clone();

    // Try to load the model with progress tracking
    let model = Llama::builder()
        .with_source(LlamaSource::llama_3_2_1b_chat())
        .build_with_loading_handler(|progress| match progress {
            ModelLoadingProgress::Downloading { source, progress } => {
                // progress.progress is already a fraction between 0 and 1
                let percentage = progress.progress / progress.size;
                let elapsed = progress.start_time.elapsed().as_secs_f32();
                let message = format!("Downloading model: {}%", percentage);
                print!("\rDownloading the model ({}%) MBs Downloaded: {}", percentage, progress.progress / 1000000);
                std::io::stdout().flush().expect("TODO: panic message");
                // println!("Downloading file {source} {percentage}% ({elapsed:.1}s)");
            }
            ModelLoadingProgress::Loading { progress } => {
                // progress is already a fraction between 0 and 1
                let progress_percent = (progress * 100.0).clamp(0.0, 100.0) as u32;
                let message = format!("Loading model: {}%", progress_percent);
                println!("Loading model {progress_percent}%");
            }
        })
        .await
        .map_err(|e| format!("Failed to load Kalosm model: {}", e))?;

    // Signal completion of model loading
    app.emit("llm-download-progress", 100).unwrap();
    app.emit("llm-loading-progress", 100).unwrap();
    app.emit(
        "llm-progress",
        "Model loaded successfully! Preparing chat session...",
    )
    .unwrap();
    println!("Preparing chat session with Kalosm model...");

    // Generate response with timeout
    let response_result = timeout(Duration::from_secs(120), async {
        app.emit("llm-progress", "Generating response...").unwrap();

        let chat = model.chat();

        let response = chat
            .with_system_prompt(system_prompt)
            .add_message(user_prompt)
            .await
            .map_err(|e| e.to_string())?;

        app.emit("llm-progress", "Response generated successfully!")
            .unwrap();
        Ok::<String, String>(response)
    })
    .await;

    match response_result {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Response generation timed out after 2 minutes.".to_string()),
    }
}

#[tauri::command]
pub async fn generate_summary(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    // Check if another transcription is already running
    let state = app.state::<Mutex<AppState>>();
    // Lock the mutex to get mutable access:
    let mut state = state.lock().await;

    if state.currently_summarizing.is_some() {
        return Err("Another Summarization is running".to_string());
    }

    // Modify the state:
    state.currently_summarizing = Some(meeting_id.to_string());

    println!();
    println!("Summarization started!");
    let transcript = get_meeting_transcript(app.clone(), meeting_id).await?;

    if transcript.is_empty() {
        return Err("No Transcript to Summarise".to_string());
    }

    app.emit("summarization-started", &meeting_id).unwrap();

    let system_prompt = "
You are a helpful assistant who combines multiple structured meeting summaries into a single cohesive summary. Preserve the original structure:

- 📌 Introduction
- 📝 Detailed Summary (merge and deduplicate bullet points)
- ✅ Action Items (merge all to-do lists, grouped by person)";

    let content = generate_text_with_llm(app.clone(), system_prompt, &transcript).await?;

    state.currently_summarizing = None;

    // Add it to meeting.json if it exists
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let summary_path = app_dir.join("uploads").join(meeting_id).join("summary.md");

    fs::write(summary_path, content.clone())
        .await
        .map_err(|e| e.to_string())?;

    generate_meeting_name(app.clone(), meeting_id).await?;
    Ok(content)
}

#[tauri::command]
pub async fn is_summarizing(app: AppHandle) -> Result<Option<String>, String> {
    let state = app.state::<Mutex<AppState>>();
    // Lock the mutex to get mutable access:
    let state = state.lock().await;

    Ok(state.currently_summarizing.clone())
}

#[tauri::command]
pub async fn get_meeting_summary(app: AppHandle, meeting_id: &str) -> Result<String, String> {
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

#[tauri::command]
pub async fn generate_meeting_name(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    println!("Generating meeting name for {}", meeting_id);
    // notify frontend that generation has started
    app.emit("meeting-name-generation-started", meeting_id)
        .unwrap();

    // Get Meeting Summary
    let meeting_summary = get_meeting_summary(app.clone(), meeting_id).await;

    match meeting_summary {
        Ok(summary) => {
            let system_prompt = "You are a meeting summarization assistant. generate a concise and relevant meeting name based on the provided summary. In front of the meeting name, add a fitting emoji. The name is supposed to be short (max 6 words), concise and relevant to the meeting summary.";

            let name_str = generate_text_with_llm(app.clone(), system_prompt, &summary).await?;

            // Add it to meeting.json if it exists
            let app_dir = app
                .path()
                .app_local_data_dir()
                .expect("Failed to get app local data directory");
            let metadata_path = app_dir
                .join("uploads")
                .join(meeting_id)
                .join("meeting.json");

            let metadata = MeetingMetadata {
                id: meeting_id.to_string(),
                name: Some(name_str.clone()),
            };
            let json = serde_json::to_string(&metadata).map_err(|e| e.to_string())?;
            fs::write(metadata_path, json)
                .await
                .map_err(|e| e.to_string())?;

            Ok(name_str)
        }
        Err(e) => Err(format!("Failed to get meeting summary: {}", e)),
    }
}

#[tauri::command]
pub async fn test_llm_connection(app: AppHandle) -> Result<String, String> {
    let test_system_prompt = "You are a helpful assistant. Respond concisely.";
    let test_user_prompt = "Say 'Hello! LLM test successful.' and nothing else.";

    app.emit("llm-progress", "Starting LLM connection test...")
        .unwrap();
    // Reset progress indicators
    app.emit("llm-download-progress", 0).unwrap();
    app.emit("llm-loading-progress", 0).unwrap();

    let result = generate_text_with_llm(app.clone(), test_system_prompt, test_user_prompt).await;

    match result {
        Ok(response) => {
            app.emit("llm-progress", "LLM test completed successfully!")
                .unwrap();
            Ok(format!("Test successful! Response: {}", response.trim()))
        }
        Err(e) => {
            app.emit("llm-progress", &format!("LLM test failed: {}", e))
                .unwrap();
            Err(format!("Test failed: {}", e))
        }
    }
}
