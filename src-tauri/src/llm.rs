use std::io::Write;
use crate::{get_meeting_transcript, AppState, MeetingMetadata};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};
use tokio::fs;
use tokio::sync::Mutex;
use tauri_plugin_http::reqwest::Client;

#[derive(Debug, Clone)]
pub enum Language {
    English,
    German,
}

impl Default for Language {
    fn default() -> Self {
        Language::German
    }
}

fn get_chunk_summarization_prompt(language: &Language) -> &'static str {
    match language {
        Language::English => "
You are a meeting summarization assistant. Summarize the provided meeting transcript chunk in a structured format:

- 📌 Introduction: Brief context about what was discussed
- 📝 Key Points: Main topics and decisions (use bullet points)
- ✅ Action Items: Tasks, assignments, or next steps mentioned (format: • [Person]: Task description)

Keep the summary concise but comprehensive. Maintain any speaker names or roles mentioned. if abbreviations are used, do not explain them.",

        Language::German => "
Sie sind ein Assistent für Meeting-Zusammenfassungen. Fassen Sie den bereitgestellten Abschnitt eines Meeting-Transkripts möglichst vollständig zusammen:

- 📌 Einführung: Worum ging es zu Beginn? Aufgaben wie Moderation, Protokollführung oder Zeiterfassung sollen nur einmal zu Beginn des Protokolls stichpunktartig aufgeführt werden. Sie sind keine weiterführenden Aktionspunkte und dürfen daher nicht im Abschnitt zu den To-Dos oder nächsten Schritten erscheinen. 
- 📝 Wichtige Punkte: Alle besprochenen Themen und Argumente mit Verweis darauf, wer es gesagt hat (Format: • Beschreibung des Diskussionspunktes). Finde außerdem Zwischenüberschriften, um den Text besser zu gliedern.
- ✅ Aktionspunkte, To-Dos, nächste Schritte: Aufgaben, Zuweisungen oder nächste Schritte (Format: • [Name]: Aufgabenbeschreibung) Bei doppelter Vergabe von Aufgaben soll diese kenntlich gemacht werden und auf die Dopplung hingewiesen werden.

Verkürzen Sie nichts zu stark. Fassen Sie möglichst alle relevanten Inhalte zusammen. Der Stil darf sachlich, aber detailliert sein. Halten Sie Redebeiträge einzelner Personen getrennt, wenn möglich. Wenn abkürzungen genannt werden, erklären Sie diese nicht. Inhaltliche Wiederholungen können zusammengefasst werden. Nebensächlichkeiten wie technische Probleme oder persönliche Anekdoten müssen nicht beachtet werden.
Ergänze keine Kommentare oder Erklärungen, sondern gebe nur den finalen Output ohne Kommentare an.",
    }
}

fn get_final_summary_prompt(language: &Language) -> &'static str {
    match language {
        Language::English => "
Summarize the following transcript chunk. Focus on:

1. What was discussed?
2. What was decided?
3. What needs to happen next?

Preserve speaker names. Use bullet points. Do not use \"Introduction\"/\"Key Points\"/\"Action Items\" as section headers.",

        Language::German => "
Fassen Sie die folgenden Abschnittszusammenfassungen zu einer vollständigen und detaillierten Meeting-Zusammenfassung zusammen. Aufgaben wie Moderation, Protokollführung oder Zeiterfassung sollen zu Beginn des Protokolls stichpunktartig aufgeführt werden. Sie sind keine weiterführenden Aktionspunkte und dürfen daher nicht im Abschnitt zu den To-Dos oder nächsten Schritten erscheinen. 

Erstellen Sie eine gegliederte Zusammenfassung mit:
- 📌 Gesamtkontext
- 🧩 Zusammengeführte Hauptthemen (mit Bullet Points und Namen, wenn vorhanden)
- ✅ Aktionspunkte, To-Dos, nächste Schritte nach Personen gruppiert (Format: • [Name]: Aufgabenbeschreibung)
Vermeiden Sie Wiederholungen und konzentrieren Sie sich auf die wichtigsten Punkte. 
Behalten Sie den Charakter des Meetings (z. B. informell, aktivistisch) bei und vermeiden Sie oberflächliche Generalisierungen.
Ergänze keine Kommentare oder Erklärungen, sondern gebe nur den finalen Output ohne Kommentare an.",

    }
}

fn get_direct_summarization_prompt(language: &Language) -> &'static str {
    match language {
        Language::English => "
You are a meeting summarization assistant. You will only generate the meeting summary, and not mention anything earlier in the chat, nor any confirmation that you understood.

You receive summaries of transcript *chunks* from a single meeting. Combine them into one structured summary with the following sections:

- 📌 **Overall Context**: Briefly describe the meeting's overarching goal or theme.
- 🧩 **Merged Key Topics**: Merge overlapping topics and preserve detail. Deduplicate similar points.
  - Use bullet points.
  - Keep speaker names/roles if mentioned.
  - Preserve tone (e.g., activist, formal, casual).
- ✅ **Action Items**:
  - Group by person if possible.
  - Use this format: • [Name]: Task description

Do NOT repeat the headers from the input chunks. Focus on *integration*, *concision*, and *completeness*. Avoid generic filler phrases like \"the speaker discusses\".",

        Language::German => "
Sie sind ein Assistent für Meeting-Zusammenfassungen. Sie werden nur die Meeting-Zusammenfassung erstellen und nichts Früheres im Chat erwähnen oder bestätigen, dass Sie verstanden haben.

Sie erhalten Zusammenfassungen von Transkript-*Abschnitten* aus einem einzigen Meeting. Kombinieren Sie sie zu einer strukturierten Zusammenfassung mit folgenden Abschnitten:

- 📌 **Gesamtkontext**: Beschreiben Sie kurz das übergeordnete Ziel oder Thema des Meetings.
- 🧩 **Zusammengeführte Hauptthemen**: Führen Sie überlappende Themen zusammen und bewahren Sie Details. Entfernen Sie ähnliche Punkte.
  - Verwenden Sie Aufzählungspunkte.
  - Behalten Sie Sprechernamen/Rollen bei, falls erwähnt.
  - Bewahren Sie den Ton (z.B. aktivistisch, formell, locker).
- ✅ **Aktionspunkte**:
  - Gruppieren Sie nach Person, wenn möglich.
  - Verwenden Sie dieses Format: • [Name]: Aufgabenbeschreibung

Wiederholen Sie NICHT die Überschriften aus den Eingabe-Abschnitten.",
    }
}

fn get_meeting_name_prompt(language: &Language) -> &'static str {
    match language {
        Language::English => "You are a meeting summarization assistant. generate a concise and relevant meeting name based on the provided summary. In front of the meeting name, add a fitting emoji. The name is supposed to be short (max 6 words), concise and relevant to the meeting summary.",

        Language::German => "Sie sind ein Assistent für Meeting-Zusammenfassungen. Erstellen Sie einen prägnanten und relevanten Meeting-Namen basierend auf der bereitgestellten Zusammenfassung. Vor dem Meeting-Namen fügen Sie ein passendes Emoji hinzu. Der Name soll kurz (max. 6 Wörter), prägnant und relevant zur Meeting-Zusammenfassung sein. Generiere nur den namen, keine sonstige bestätigung des prompts.",
    }
}

fn get_test_prompt(language: &Language) -> &'static str {
    match language {
        Language::English => "You are a helpful assistant. Respond concisely.",
        Language::German => "Sie sind ein hilfreicher Assistent. Antworten Sie prägnant.",
    }
}

fn get_test_user_prompt(language: &Language) -> &'static str {
    match language {
        Language::English => "Say 'Hello! LLM test successful.' and nothing else.",
        Language::German => "Sagen Sie 'Hallo! LLM-Test erfolgreich.' und nichts anderes.",
    }
}

fn split_text_into_chunks(text: &str, max_chars: usize) -> Vec<String> {
    if text.chars().count() <= max_chars {
        return vec![text.to_string()];
    }
    
    let mut chunks = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = text.chars().collect();
    
    while current_pos < chars.len() {
        let end_pos = std::cmp::min(current_pos + max_chars, chars.len());
        
        // Try to find a good breaking point (sentence end, paragraph break, or whitespace)
        let mut break_pos = end_pos;
        if end_pos < chars.len() {
            // Look for sentence end within the chunk
            let chunk_text: String = chars[current_pos..end_pos].iter().collect();
            
            // Look for sentence end
            if let Some(sentence_end) = chunk_text
                .rfind(". ")
                .or_else(|| chunk_text.rfind(".\n"))
                .or_else(|| chunk_text.rfind("? "))
                .or_else(|| chunk_text.rfind("! "))
            {
                // Convert byte position back to char position
                let prefix: String = chunk_text.chars().take(sentence_end + 1).collect();
                break_pos = current_pos + prefix.chars().count();
            }
            // If no sentence end found, look for paragraph break
            else if let Some(para_break) = chunk_text.rfind("\n\n") {
                let prefix: String = chunk_text.chars().take(para_break + 2).collect();
                break_pos = current_pos + prefix.chars().count();
            }
            // Finally, look for any whitespace
            else if let Some(space) = chunk_text.rfind(' ') {
                let prefix: String = chunk_text.chars().take(space + 1).collect();
                break_pos = current_pos + prefix.chars().count();
            }
        }
        
        let chunk: String = chars[current_pos..break_pos].iter().collect();
        chunks.push(chunk.trim().to_string());
        current_pos = break_pos;
    }
    
    chunks
}

async fn summarize_chunks(
    app: AppHandle,
    chunks: Vec<String>,
    language: &Language,
    meeting_id: &str,
) -> Result<String, String> {
    let mut chunk_summaries = Vec::new();
    
    let chunk_system_prompt = get_chunk_summarization_prompt(language);
    
    // Get the app directory for saving chunk summaries
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let meeting_dir = app_dir.join("uploads").join(meeting_id);
    let chunks_dir = meeting_dir.join("chunks");
    
    // Create chunks directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&chunks_dir).await {
        println!("Warning: Failed to create chunks directory: {}", e);
    }
    
    for (i, chunk) in chunks.iter().enumerate() {
        app.emit("llm-progress", &format!("Summarizing chunk {} of {}", i + 1, chunks.len()))
            .unwrap();
        
        let chunk_summary = generate_text_with_llm(
            app.clone(),
            chunk_system_prompt,
            chunk,
        ).await?;
        
        // Save individual chunk and its summary
        let chunk_file = chunks_dir.join(format!("chunk_{:03}.txt", i + 1));
        let summary_file = chunks_dir.join(format!("chunk_{:03}_summary.md", i + 1));
        
        if let Err(e) = fs::write(&chunk_file, chunk).await {
            println!("Warning: Failed to save chunk {}: {}", i + 1, e);
        }
        
        if let Err(e) = fs::write(&summary_file, &chunk_summary).await {
            println!("Warning: Failed to save chunk summary {}: {}", i + 1, e);
        }
        
        chunk_summaries.push(chunk_summary);
    }
    
    // Save all chunk summaries as one file
    let all_chunks_summary_file = chunks_dir.join("all_chunk_summaries.md");
    let all_summaries_content = chunk_summaries
        .iter()
        .enumerate()
        .map(|(i, summary)| format!("# Chunk {} Summary\n\n{}", i + 1, summary))
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");
    
    if let Err(e) = fs::write(&all_chunks_summary_file, &all_summaries_content).await {
        println!("Warning: Failed to save all chunk summaries: {}", e);
    }
    
    // Combine all chunk summaries
    let combined_summaries = chunk_summaries.join("\n\n---\n\n");
    
    app.emit("llm-progress", "Combining chunk summaries into final summary...")
        .unwrap();
    
    let final_system_prompt = get_final_summary_prompt(language);
    
    generate_text_with_llm(
        app,
        final_system_prompt,
        &combined_summaries,
    ).await
}

async fn generate_text_with_llm(
    app: AppHandle,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    use std::time::Instant;
    
    let start_time = Instant::now();
    println!("🚀 Starting LLM text generation...");
    
    // let state = app.state::<Mutex<AppState>>();
    // let config = {
    //     let state = state.lock().await;
    //     state.llm_config.clone()
    // };

    // Try external API first if enabled
    app.emit("llm-progress", "Trying external API...").unwrap();
    let api_start = Instant::now();
    
    match try_external_api(system_prompt, user_prompt).await {
        Ok(response) => {
            let api_duration = api_start.elapsed();
            let total_duration = start_time.elapsed();
            println!("✅ External API successful! API time: {:.2}s, Total time: {:.2}s", 
                api_duration.as_secs_f64(), total_duration.as_secs_f64());
            app.emit("llm-progress", "External API successful").unwrap();
            return Ok(response);
        }
        Err(e) => {
            let api_duration = api_start.elapsed();
            println!("❌ External API failed after {:.2}s: {}, falling back to Kalosm", 
                api_duration.as_secs_f64(), e);
            app.emit("llm-progress", "External API failed, switching to local model...").unwrap();
            return Err(e);
        }
    }

    // Fallback to Kalosm
    // try_kalosm(app.clone(), system_prompt, user_prompt).await
}

#[derive(Serialize, Deserialize)]
struct OllamaResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
    pub done_reason: String,
    pub context: Vec<i64>,
    pub total_duration: i64,
    pub load_duration: i64,
    pub prompt_eval_count: i64,
    pub prompt_eval_duration: i64,
    pub eval_count: i64,
    pub eval_duration: i64,
}

async fn try_external_api(
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    println!("trying external ollama");

    let client = Client::new();

    // Merge system and user prompts into one string
    let full_prompt = format!("System: {}\nUser: {}", system_prompt, user_prompt);

    let response = client.post("http://localhost:11434/api/generate")
        .json(&json!({
            "model": "llama3.1",
            "prompt": full_prompt,
            "stream": false,
            "num_ctx": 8096
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<OllamaResponse>()
        .await.map_err(|e| e.to_string())?;
    
    return Ok(response.response);
}

// async fn try_kalosm(
//     app: AppHandle,
//     system_prompt: &str,
//     user_prompt: &str,
// ) -> Result<String, String> {
//     use kalosm::language::*;
//     use std::sync::Arc;
//     use std::time::Duration;
//     use tokio::time::timeout;

//     println!("Starting kalosm...");

//     app.emit("llm-progress", "Initializing Kalosm model...")
//         .unwrap();

//     println!("Downloading Kalosm model...\n");

//     // Clone app handle for use in the closure
//     let app_clone = app.clone();

//     // Try to load the model with progress tracking
//     let model = Llama::builder()
//         .with_source(LlamaSource::llama_3_2_1b_chat())
//         .build_with_loading_handler(|progress| match progress {
//             ModelLoadingProgress::Downloading { source, progress } => {
//                 // progress.progress is already a fraction between 0 and 1
//                 let percentage = progress.progress / progress.size;
//                 let elapsed = progress.start_time.elapsed().as_secs_f32();
//                 let message = format!("Downloading model: {}%", percentage);
//                 print!("\rDownloading the model ({}%) MBs Downloaded: {}", percentage, progress.progress / 1000000);
//                 std::io::stdout().flush().expect("TODO: panic message");
//                 // println!("Downloading file {source} {percentage}% ({elapsed:.1}s)");
//             }
//             ModelLoadingProgress::Loading { progress } => {
//                 // progress is already a fraction between 0 and 1
//                 let progress_percent = (progress * 100.0).clamp(0.0, 100.0) as u32;
//                 let message = format!("Loading model: {}%", progress_percent);
//                 println!("Loading model {progress_percent}%");
//             }
//         })
//         .await
//         .map_err(|e| format!("Failed to load Kalosm model: {}", e))?;

//     // Signal completion of model loading
//     app.emit("llm-download-progress", 100).unwrap();
//     app.emit("llm-loading-progress", 100).unwrap();
//     app.emit(
//         "llm-progress",
//         "Model loaded successfully! Preparing chat session...",
//     )
//     .unwrap();
//     println!("Preparing chat session with Kalosm model...");

//     // Generate response with timeout
//     let response_result = timeout(Duration::from_secs(120), async {
//         app.emit("llm-progress", "Generating response...").unwrap();

//         let chat = model.chat();

//         let response = chat
//             .with_system_prompt(system_prompt)
//             .add_message(user_prompt)
//             .await
//             .map_err(|e| e.to_string())?;

//         app.emit("llm-progress", "Response generated successfully!")
//             .unwrap();
//         Ok::<String, String>(response)
//     })
//     .await;

//     match response_result {
//         Ok(Ok(response)) => Ok(response),
//         Ok(Err(e)) => Err(e),
//         Err(_) => Err("Response generation timed out after 2 minutes.".to_string()),
//     }
// }

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

    // Check if transcript is longer than 6_000 characters
    let content = if transcript.len() > 10_000 {
        app.emit("llm-progress", "Transcript is long, splitting into chunks for processing...")
            .unwrap();
        
        // Split transcript into manageable chunks
        let chunks = split_text_into_chunks(&transcript, 10_000);
        println!("Split transcript into {} chunks", chunks.len());
        
        // Summarize chunks and combine
        summarize_chunks(app.clone(), chunks, &Language::default(), meeting_id).await?
    } else {
        // Use direct summarization for shorter transcripts
        let system_prompt = get_direct_summarization_prompt(&Language::default());

        generate_text_with_llm(app.clone(), system_prompt, &transcript).await?
    };

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
            let system_prompt = get_meeting_name_prompt(&Language::default());

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
    let test_system_prompt = get_test_prompt(&Language::default());
    let test_user_prompt = get_test_user_prompt(&Language::default());

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
