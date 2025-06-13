use crate::llm::models::KeyFact;
use serde_json::json;

#[derive(Debug, Clone)]
#[allow(dead_code)]
#[derive(Default)]
pub enum Language {
    English,
    #[default]
    German,
}

pub struct PromptManager;

impl PromptManager {
    pub fn chunk_summarization(language: &Language, key_facts: Option<&KeyFact>) -> String {
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

    pub fn final_summary(language: &Language) -> &'static str {
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

    #[allow(dead_code)]
    pub fn direct_summarization(language: &Language) -> &'static str {
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

    pub fn test_connection(language: &Language) -> &'static str {
        match language {
            Language::English => "You are a helpful assistant. Respond concisely.",
            Language::German => "Sie sind ein hilfreicher Assistent. Antworten Sie prägnant.",
        }
    }

    pub fn test_user_message(language: &Language) -> &'static str {
        match language {
            Language::English => "Say 'Hello! LLM test successful.' and nothing else.",
            Language::German => "Sagen Sie 'Hallo! LLM-Test erfolgreich.' und nichts anderes.",
        }
    }
}
