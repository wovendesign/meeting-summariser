use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Attendee {
    pub id: usize,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KeyFact {
    pub responisible_for_moderation: Option<String>,
    pub responisible_for_protocol: Option<String>,
    pub responisible_for_timekeeping: Option<String>,
    pub attendees: Option<Vec<Attendee>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Topic {
    pub title: String,
    pub bullet_points: Vec<String>,
    pub sub_topics: Option<Vec<Topic>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToDo {
    pub assignees: Option<Vec<String>>,
    pub task: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FirstSummaryFormat {
    pub key_facts: KeyFact,
    pub topics: Vec<Topic>,
    pub todos: Option<Vec<ToDo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Title {
    pub emoji: String,
    pub text: String,
}

impl Title {
    pub fn to_string(&self) -> String {
        format!("{} {}", self.emoji, self.text)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FinalSummaryFormat {
    pub title: Title,
    pub key_facts: KeyFact,
    pub summary: String,
    pub topics: Vec<Topic>,
    pub todos: Vec<ToDo>,
}

pub trait MeetingToMarkdown {
    fn to_markdown(&self) -> String;
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

#[derive(Serialize, Deserialize)]
pub struct OllamaResponse {
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
