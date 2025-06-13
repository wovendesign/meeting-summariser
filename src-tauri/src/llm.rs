use crate::{get_meeting_transcript, AppState, MeetingMetadata};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_http::reqwest::Client;
use tokio::fs;
use tokio::sync::Mutex;

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct Attendee {
    id: usize,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct KeyFact {
    responisible_for_moderation: Option<String>,
    responisible_for_protocol: Option<String>,
    responisible_for_timekeeping: Option<String>,
    attendees: Option<Vec<Attendee>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct Topic {
    title: String,
    bullet_points: Vec<String>,
    sub_topics: Option<Vec<Topic>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ToDo {
    assignees: Option<Vec<String>>,
    task: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct FirstSummaryFormat {
    key_facts: KeyFact,
    topics: Vec<Topic>,
    todos: Option<Vec<ToDo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct Title {
    emoji: String,
    text: String,
}

impl Title {
    fn to_string(&self) -> String {
        format!("{} {}", self.emoji, self.text)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct FinalSummaryFormat {
    title: Title,
    key_facts: KeyFact,
    summary: String,
    topics: Vec<Topic>,
    todos: Vec<ToDo>,
}
impl MeetingToMarkdown for FinalSummaryFormat {
    fn to_markdown(&self) -> String {
        let mut markdown = format!("# {}\n\n", self.title.text);
        markdown.push_str(self.summary.as_str());
        markdown.push_str("\n\n");
        markdown.push_str("## Key Facts\n");
        if let Some(moderation) = &self.key_facts.responisible_for_moderation {
            markdown.push_str(&format!("- **Moderation:** {}\n", moderation));
        }
        if let Some(protocol) = &self.key_facts.responisible_for_protocol {
            markdown.push_str(&format!("- **Protocol:** {}\n", protocol));
        }
        if let Some(timekeeping) = &self.key_facts.responisible_for_timekeeping {
            markdown.push_str(&format!("- **Timekeeping:** {}\n", timekeeping));
        }
        if let Some(attendees) = &self.key_facts.attendees {
            markdown.push_str("- **Attendees:**\n");
            for attendee in attendees {
                markdown.push_str(&format!("  - {}\n", attendee.name));
            }
        }
        markdown.push_str("## Topics\n");
        for topic in &self.topics {
            markdown.push_str(&format!("### {} \n", topic.title));
            for bullet in &topic.bullet_points {
                markdown.push_str(&format!("- {}\n", bullet));
            }
        }
        markdown.push_str("## To-Dos\n");
        for todo in &self.todos {
            markdown.push_str(&format!("### {} \n", todo.task));
            if let Some(assignees) = &todo.assignees {
                markdown.push_str("  - **Assignees:** ");
                markdown.push_str(&assignees.join(", "));
                markdown.push('\n');
            }
        }
        markdown
    }
}

trait MeetingToMarkdown {
    fn to_markdown(&self) -> String;
}

fn get_chunk_summarization_prompt(language: &Language, key_facts: Option<&KeyFact>) -> String {
    match language {
        Language::English => "
You are a meeting summarization assistant. Summarize the provided meeting transcript chunk in a structured format:

- 📌 Introduction: Brief context about what was discussed
- 📝 Key Points: Main topics and decisions (use bullet points)
- ✅ Action Items: Tasks, assignments, or next steps mentioned (format: • [Person]: Task description)

Keep the summary concise but comprehensive. Maintain any speaker names or roles mentioned. if abbreviations are used, do not explain them.".to_string(),

        Language::German => {
            let key_facts_str: String = if let Some(key_facts) = key_facts {
                json!(key_facts).to_string()
            } else {
                "Noch keine vorhandenen Key Facts.".into()
            };

            format!("
Sie sind ein Assistent für Meeting-Zusammenfassungen. 
Fassen Sie den bereitgestellten Abschnitt eines Meeting-Transkripts möglichst vollständig zusammen:

Falls eine Person noch nicht in den vorherigen Key Facts erwähnt wurde, erwähnen Sie sie im Abschnitt Key Facts.

{}

Statt Namen zu erwähnen, nutze die ID der Attendees aus den Key Facts (z. B. `[1] fragt …`).
Bei den Keyfacts sollen folgende Punkte beachtet werden:
´attendees´ enthält eine Liste von Personen, die am Meeting teilgenommen haben.
´responisible_for_moderation´ enthält den Namen einer oder meherer Personen, die das Meeting moderiert hat.
´responisible_for_protocol´ enthält den Namen einer oder meherer Personen, die für das Protokoll zuständig sind.
Wie der Entscheidungsprozess der Protokollführung ablief und welche Gründe es für diese Entscheidung gab müssen nicht Erwähnt werden.
´responisible_for_timekeeping´ enthält den Namen einer oder meherer Personen, die für die Zeitmessung verantwortig sind.

Verkürzen Sie nichts zu stark. 
Fassen Sie möglichst alle relevanten Inhalte zusammen.
Der Stil darf sachlich, aber detailliert sein. 
Die `bullet_points` sollen als Stichpunkte geschrieben werden.
Verben und unnötige Füllwörter sollen vermieden werden.
Halten Sie Redebeiträge einzelner Personen getrennt, wenn möglich. 
Wenn abkürzungen genannt werden, erklären Sie diese nicht. 
Inhaltliche Wiederholungen können zusammengefasst werden. 
Nebensächlichkeiten wie technische Probleme oder persönliche Anekdoten müssen nicht beachtet werden.
Unter ´ToDo´ sollen die wichtigsten Aufgaben (´tasks´), die im Meeting besprochen wurden, mit Bezug auf die jeweilige Person(´ateendee´), in das Feld ´asignee´ aufgelistet werden.
Ergänze keine Kommentare oder Erklärungen, sondern gebe nur den finalen Output ohne Kommentare an.", key_facts_str)
        },
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

Als `summary` geben Sie eine kurze Zusammenfassung des Meetings an, die den Zweck des Meetings und die wichtigsten Ergebnisse zusammenfasst.
Es soll möglichst der gesamte Inhalt des Meetings zusammengefasst werden, ohne dass wichtige Details verloren gehen. 
In erster Linie sollst du die Stichpunkte gruppieren, ohne sie zu verändern oder zu kürzen.

Die `topics` enthalten die wichtigsten Themen des Meetings, die in den einzelnen Abschnitten behandelt wurden. Diese sollten in einer strukturierten Form mit Stichpunkten und gegebenenfalls Unterpunkten dargestellt werden. Kombinieren Sie überlappende Themen und bewahren Sie Details. Vermeiden Sie Wiederholungen und konzentrieren Sie sich auf relevante Punkte. Meetinginterne Inhalte wie technische Probleme oder persönliche Anekdoten müssen nicht beachtet werden.
Die `todos` enthalten die wichtigsten Aufgaben, die im Meeting besprochen wurden. Falls eine oder mehrere Personen für eine Aufgabe verantwortlich sind, listen Sie diese in der `assignees`-Liste auf. Die Aufgaben sollten klar und präzise formuliert sein. Aufgaben, die sich nur auf das Meetings beziehen, sollten nicht in den To-Dos auftauchen, sondern nur die Aufgaben, die für die Zukunft relevant sind. Bei unklarer Verantwortlichkeit oder fehlender Zuweisung, `assignees` schreibe sie mehrer Namen hin oder lassen Sie das Feld.",
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

fn combine_structured_first_summaries(summaries: Vec<FirstSummaryFormat>) -> FirstSummaryFormat {
    let mut combined = FirstSummaryFormat {
        key_facts: KeyFact {
            responisible_for_moderation: None,
            responisible_for_protocol: None,
            responisible_for_timekeeping: None,
            attendees: None,
        },
        topics: Vec::new(),
        todos: None,
    };

    for summary in summaries {
        // Combine key facts
        if let Some(moderation) = summary.key_facts.responisible_for_moderation {
            combined.key_facts.responisible_for_moderation = Some(moderation);
        }
        if let Some(protocol) = summary.key_facts.responisible_for_protocol {
            combined.key_facts.responisible_for_protocol = Some(protocol);
        }
        if let Some(timekeeping) = summary.key_facts.responisible_for_timekeeping {
            combined.key_facts.responisible_for_timekeeping = Some(timekeeping);
        }
        if let Some(attendees) = summary.key_facts.attendees {
            if combined.key_facts.attendees.is_none() {
                combined.key_facts.attendees = Some(attendees);
            } else {
                // Merge attendees, avoiding duplicates
                let existing_ids: Vec<usize> = combined
                    .key_facts
                    .attendees
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|a| a.id)
                    .collect();
                for attendee in attendees {
                    if !existing_ids.contains(&attendee.id) {
                        combined
                            .key_facts
                            .attendees
                            .as_mut()
                            .unwrap()
                            .push(attendee);
                    }
                }
            }
        }

        // Combine topics
        combined.topics.extend(summary.topics);

        // Combine todos
        if let Some(todos) = summary.todos {
            if combined.todos.is_none() {
                combined.todos = Some(todos);
            } else {
                combined.todos.as_mut().unwrap().extend(todos);
            }
        }
    }

    combined
}

async fn summarize_chunks(
    app: AppHandle,
    chunks: Vec<String>,
    language: &Language,
    meeting_id: &str,
) -> Result<FinalSummaryFormat, String> {
    use std::time::Instant;

    let mut chunk_summaries = Vec::new();
    let mut chunk_times = Vec::new();

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

    let mut key_facts = KeyFact {
        responisible_for_moderation: None,
        responisible_for_protocol: None,
        responisible_for_timekeeping: None,
        attendees: None,
    };

    // Total steps: chunk processing + final summary generation
    let total_steps = chunks.len() + 1;

    // Emit summarization start event
    app.emit("summarization-chunk-start", total_steps).unwrap();

    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_start_time = Instant::now();
        let current_step = i + 1;

        // Emit progress for current chunk
        app.emit("summarization-chunk-progress", i).unwrap();

        app.emit(
            "llm-progress",
            &format!(
                "Step {}/{}: Summarizing chunk {} of {}",
                current_step,
                total_steps,
                i + 1,
                chunks.len()
            ),
        )
        .unwrap();

        let chunk_system_prompt = get_chunk_summarization_prompt(language, Some(&key_facts));

        let chunk_summary_json = generate_text_with_llm(
            app.clone(),
            &chunk_system_prompt,
            chunk,
            Some(key_facts.clone()),
            Some(schema_for!(FirstSummaryFormat)),
        )
        .await?;

        let chunk_summary: FirstSummaryFormat = serde_json::from_str(&chunk_summary_json)
            .map_err(|e| format!("Failed to parse chunk summary JSON: {}", e))?;

        let chunk_duration = chunk_start_time.elapsed();
        chunk_times.push(chunk_duration);
        println!(
            "✅ Chunk {} completed in {:.2}s",
            i + 1,
            chunk_duration.as_secs_f64()
        );

        let chunk_summary: FirstSummaryFormat = serde_json::from_str(&chunk_summary_json)
            .map_err(|e| format!("Failed to parse chunk summary JSON: {}", e))?;

        // Update key facts if they are present in the chunk summary.
        if let Some(moderation) = &chunk_summary.key_facts.responisible_for_moderation {
            key_facts.responisible_for_moderation = Some(moderation.clone());
        }
        if let Some(protocol) = &chunk_summary.key_facts.responisible_for_protocol {
            key_facts.responisible_for_protocol = Some(protocol.clone());
        }
        if let Some(timekeeping) = &chunk_summary.key_facts.responisible_for_timekeeping {
            key_facts.responisible_for_timekeeping = Some(timekeeping.clone());
        }
        if let Some(attendees) = &chunk_summary.key_facts.attendees {
            if key_facts.attendees.is_none() {
                key_facts.attendees = Some(attendees.clone());
            } else {
                // Merge attendees, avoiding duplicates
                let mut existing_ids: Vec<usize> = key_facts
                    .attendees
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|a| a.id)
                    .collect();
                for attendee in attendees {
                    if !existing_ids.contains(&attendee.id) {
                        existing_ids.push(attendee.id);
                        key_facts.attendees.as_mut().unwrap().push(attendee.clone());
                    }
                }
            }
        }

        // Save individual chunk and its summary
        let chunk_file = chunks_dir.join(format!("chunk_{:03}.txt", i + 1));
        let summary_file = chunks_dir.join(format!("chunk_{:03}_summary.json", i + 1));

        if let Err(e) = fs::write(&chunk_file, chunk).await {
            println!("Warning: Failed to save chunk {}: {}", i + 1, e);
        }

        let chunk_summary_json = serde_json::to_string_pretty(&chunk_summary)
            .unwrap_or_else(|_| "Failed to serialize chunk summary".to_string());
        if let Err(e) = fs::write(&summary_file, &chunk_summary_json).await {
            println!("Warning: Failed to save chunk summary {}: {}", i + 1, e);
        }

        chunk_summaries.push(chunk_summary);
    }

    // Calculate and log chunk timing statistics
    if !chunk_times.is_empty() {
        let total_chunk_time: std::time::Duration = chunk_times.iter().sum();
        let average_chunk_time = total_chunk_time / chunk_times.len() as u32;
        let min_chunk_time = chunk_times.iter().min().unwrap();
        let max_chunk_time = chunk_times.iter().max().unwrap();

        println!("📊 Chunk timing statistics:");
        println!(
            "   Total chunk processing time: {:.2}s",
            total_chunk_time.as_secs_f64()
        );
        println!(
            "   Average chunk time: {:.2}s",
            average_chunk_time.as_secs_f64()
        );
        println!("   Fastest chunk: {:.2}s", min_chunk_time.as_secs_f64());
        println!("   Slowest chunk: {:.2}s", max_chunk_time.as_secs_f64());

        app.emit(
            "llm-progress",
            &format!(
                "📊 Chunk stats: Avg {:.1}s/chunk, Total {:.1}s for {} chunks",
                average_chunk_time.as_secs_f64(),
                total_chunk_time.as_secs_f64(),
                chunks.len()
            ),
        )
        .unwrap();
    }

    // Save all chunk summaries as one file
    let all_chunks_summary_file = chunks_dir.join("all_chunk_summaries.md");
    let all_summaries_content = chunk_summaries
        .iter()
        .enumerate()
        .map(|(i, summary)| {
            let summary_json = serde_json::to_string_pretty(summary)
                .unwrap_or_else(|_| "Failed to serialize summary".to_string());
            format!("# Chunk {} Summary\n\n{}", i + 1, summary_json)
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");

    if let Err(e) = fs::write(&all_chunks_summary_file, &all_summaries_content).await {
        println!("Warning: Failed to save all chunk summaries: {}", e);
    }

    let final_step = total_steps;
    let final_summary_start_time = Instant::now();

    // Emit final step progress
    app.emit("summarization-chunk-progress", chunks.len())
        .unwrap();

    app.emit(
        "llm-progress",
        &format!(
            "Step {}/{}: Combining chunk summaries into final summary...",
            final_step, total_steps
        ),
    )
    .unwrap();

    let final_system_prompt = get_final_summary_prompt(language);

    let final_string = generate_text_with_llm(
        app.clone(),
        final_system_prompt,
        json!(combine_structured_first_summaries(chunk_summaries))
            .to_string()
            .as_str(),
        None,
        Some(schema_for!(FinalSummaryFormat)),
    )
    .await?;

    let final_summary: FinalSummaryFormat = serde_json::from_str(&final_string)
        .map_err(|e| format!("Failed to parse final summary JSON: {}", e))?;

    let final_summary_duration = final_summary_start_time.elapsed();
    println!(
        "✅ Final summary generation completed in {:.2}s",
        final_summary_duration.as_secs_f64()
    );

    return Ok(final_summary);
}

async fn generate_text_with_llm(
    app: AppHandle,
    system_prompt: &str,
    user_prompt: &str,
    _key_facts: Option<KeyFact>,
    // What structure to send? FirstSummaryFormat or FinalSummaryFormat?
    structure: Option<schemars::Schema>,
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
    app.emit("llm-progress", "🔄 Trying external API...")
        .unwrap();
    let api_start = Instant::now();

    match try_external_api(system_prompt, user_prompt, structure).await {
        Ok(response) => {
            let api_duration = api_start.elapsed();
            let total_duration = start_time.elapsed();
            println!(
                "✅ API successful! API time: {:.2}s, Total time: {:.2}s",
                api_duration.as_secs_f64(),
                total_duration.as_secs_f64()
            );
            app.emit("llm-progress", "✅ External API successful")
                .unwrap();
            return Ok(response);
        }
        Err(e) => {
            let api_duration = api_start.elapsed();
            println!(
                "❌ API failed after {:.2}s: {}, falling back to Kalosm",
                api_duration.as_secs_f64(),
                e
            );
            app.emit(
                "llm-progress",
                "❌ External API failed, switching to local model...",
            )
            .unwrap();
            return Err(e);
        }
    }
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
    structure: Option<schemars::Schema>,
) -> Result<String, String> {
    println!("trying external ollama");

    let client = Client::new();

    // Merge system and user prompts into one string
    let full_prompt = format!("System: {}\nUser: {}", system_prompt, user_prompt);

    let mut json = json!({
        "model": "llama3.1",
        "prompt": full_prompt,
        "stream": false,
        "num_ctx": 8096,
    });

    if let Some(schema) = structure {
        json.as_object_mut().unwrap().insert(
            "format".to_string(),
            <serde_json::Value>::from(schema.clone()),
        );
    }

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&json)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<OllamaResponse>()
        .await
        .map_err(|e| e.to_string())?;

    return Ok(response.response);
}

#[tauri::command]
pub async fn generate_summary(app: AppHandle, meeting_id: &str) -> Result<String, String> {
    use std::time::Instant;

    let summary_start_time = Instant::now();
    println!("🚀 Starting full meeting summary generation...");

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
        app.emit(
            "llm-progress",
            "📄 Transcript is long, splitting into chunks for processing...",
        )
        .unwrap();

        // Split transcript into manageable chunks
        let chunks = split_text_into_chunks(&transcript, 10_000);
        println!("📦 Split transcript into {} chunks", chunks.len());

        // Summarize chunks and combine
        summarize_chunks(app.clone(), chunks, &Language::default(), meeting_id).await?
    } else {
        // Use direct summarization for shorter transcripts
        let system_prompt = get_direct_summarization_prompt(&Language::default());

        // TODO
        return Err("Direct summarization is not implemented yet.".to_string());
        // generate_text_with_llm(app.clone(), system_prompt, &transcript, None).await?
    };

    // Reset state before final operations
    // {
    //     let state = app.state::<Mutex<AppState>>();
    //     let mut state = state.lock().await;
    //     state.currently_summarizing = None;
    // }

    // Add it to meeting.json if it exists
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let summary_path = app_dir.join("uploads").join(meeting_id).join("summary.md");
    let summary_json_path = app_dir
        .join("uploads")
        .join(meeting_id)
        .join("summary.json");

    fs::write(summary_path, content.to_markdown())
        .await
        .map_err(|e| e.to_string())?;

    fs::write(summary_json_path, serde_json::to_string(&content).unwrap())
        .await
        .map_err(|e| e.to_string())?;

    save_meeting_name(&app, meeting_id, content.title.to_string())?;

    let total_duration = summary_start_time.elapsed();
    println!("🎉 Full meeting summary completed!");
    println!(
        "⏱️  Total summary generation time: {:.2}s",
        total_duration.as_secs_f64()
    );

    app.emit(
        "llm-progress",
        &format!(
            "✅ Summary completed in {:.1}s",
            total_duration.as_secs_f64()
        ),
    )
    .unwrap();

    Ok(content.to_markdown())
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
    let summary_path = base_dir.join("summary.json");

    // read summary file
    let summary_json = fs::read_to_string(summary_path)
        .await
        .map_err(|e| e.to_string())?;

    let summary: FinalSummaryFormat =
        serde_json::from_str(&summary_json).map_err(|e| e.to_string())?;

    let markdown = summary.to_markdown();

    Ok(markdown)
}

fn save_meeting_name(app: &AppHandle, meeting_id: &str, name: String) -> Result<(), String> {
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
        name: Some(name),
    };
    let json = serde_json::to_string(&metadata).map_err(|e| e.to_string())?;
    std::fs::write(metadata_path, json).map_err(|e| e.to_string())?;

    Ok(())
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

    let result = generate_text_with_llm(
        app.clone(),
        test_system_prompt,
        test_user_prompt,
        None,
        None,
    )
    .await;

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
