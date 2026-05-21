// src-tauri/src/db/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    Tool,
    System,
}

impl MessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool",
            MessageRole::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "assistant" => MessageRole::Assistant,
            "tool" => MessageRole::Tool,
            "system" => MessageRole::System,
            _ => MessageRole::User,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: MessageRole,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolLog {
    pub id: String,
    pub conversation_id: String,
    pub message_id: String,
    pub tool_name: String,
    pub input: String,
    pub output: String,
    pub status: String, // "success" | "error"
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub provider: String, // "ollama" only for now
    pub max_iterations: i64,
    pub shell_allowlist: Vec<String>,
    pub system_prompt: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ollama_base_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3".to_string(),
            provider: "ollama".to_string(),
            max_iterations: 10,
            shell_allowlist: vec![
                "ls".to_string(),
                "dir".to_string(),
                "cat".to_string(),
                "echo".to_string(),
                "pwd".to_string(),
                "git".to_string(),
                "cargo".to_string(),
                "node".to_string(),
                "python".to_string(),
            ],
            system_prompt: "You are JARvis, a highly capable AI agent with access to filesystem and shell tools. Use tools when needed to complete tasks accurately. Always reason step by step.".to_string(),
        }
    }
}
