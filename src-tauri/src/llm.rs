use crate::{get_meeting_summary, get_meeting_transcript, MeetingMetadata};
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion;
use openai_api_rs::v1::chat_completion::ChatCompletionRequest;
use tauri::{AppHandle, Emitter, Manager};
use tokio::fs;

#[tauri::command]
pub async fn generate_summary(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    let transcript = get_meeting_transcript(app.clone(), meeting_id).await?;

    app.emit("summarization-started", &meeting_id).unwrap();

    let mut client = OpenAIClient::builder()
        .with_endpoint("http://localhost:11434/v1")
        .build()
        .map_err(|e| e.to_string())?;

    let req = ChatCompletionRequest::new(
        "llama3".to_string(),
        vec![
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::system,
                content: chat_completion::Content::Text(String::from("
You are a helpful assistant who combines multiple structured meeting summaries into a single cohesive summary. Preserve the original structure:

- 📌 Introduction
- 📝 Detailed Summary (merge and deduplicate bullet points)
- ✅ Action Items (merge all to-do lists, grouped by person)"
                )),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: chat_completion::Content::Text(transcript),
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
    if let Some(summary) = content {
        // Add it to meeting.json if it exists
        let app_dir = app
            .path()
            .app_local_data_dir()
            .expect("Failed to get app local data directory");
        let summary_path = app_dir.join("uploads").join(meeting_id).join("summary.md");

        fs::write(summary_path, summary.clone())
            .await
            .map_err(|e| e.to_string())?;

        generate_meeting_name(app.clone(), meeting_id).await?;
        return Ok(summary.to_string());
    } else {
        return Err("No content returned from Ollama".to_string());
    }
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
                            id: meeting_id.to_string(),
                            name: Some(name_str.to_string()),
                        };
                        let json = serde_json::to_string(&metadata).map_err(|e| e.to_string())?;
                        fs::write(metadata_path, json)
                            .await
                            .map_err(|e| e.to_string())?;
                        return Ok(name_str.to_string());
                    } else {
                        // If meeting.json does not exist, create it
                        let metadata = MeetingMetadata {
                            id: meeting_id.to_string(),
                            name: Some(name_str.to_string()),
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
