// src-tauri/src/agent/llm.rs
//! Ollama LLM provider. Streams tokens back to the frontend via Tauri events.
//! The provider trait is defined here for future extension (OpenAI, Claude, Gemini).

use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Window};

use crate::errors::{AppError, AppResult};

// ── Shared Types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

/// Events emitted to the frontend during streaming
pub const EVENT_TOKEN: &str = "jarvis:token";
pub const EVENT_DONE: &str  = "jarvis:done";
pub const EVENT_ERROR: &str = "jarvis:error";
pub const EVENT_TOOL_START: &str = "jarvis:tool_start";
pub const EVENT_TOOL_DONE: &str  = "jarvis:tool_done";
pub const EVENT_THINKING: &str   = "jarvis:thinking";

// ── Provider Trait ────────────────────────────────────────────────────────────

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Stream the LLM response. Emits `jarvis:token` events per chunk.
    /// Returns the full assembled response string when done.
    async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        window: &Window,
    ) -> AppResult<String>;
}

// ── Ollama Provider ───────────────────────────────────────────────────────────

pub struct OllamaProvider {
    pub base_url: String,
    pub model: String,
    pub client: Client,
}

impl OllamaProvider {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    messages: &'a Vec<ChatMessage>,
    stream: bool,
}

#[derive(Deserialize, Debug)]
struct OllamaChunk {
    message: Option<OllamaChunkMessage>,
    done: Option<bool>,
    error: Option<String>,
}

#[derive(Deserialize, Debug)]
struct OllamaChunkMessage {
    content: Option<String>,
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        window: &Window,
    ) -> AppResult<String> {
        let url = format!("{}/api/chat", self.base_url);

        let body = OllamaRequest {
            model: &self.model,
            messages: &messages,
            stream: true,
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Agent(format!("Ollama unreachable at '{}': {}", url, e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Agent(format!(
                "Ollama returned HTTP {}: {}",
                status, body
            )));
        }

        let mut stream = response.bytes_stream();
        let mut full_response = String::new();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk_bytes = chunk_result
                .map_err(|e| AppError::Network(e))?;

            // Append incoming text to buffer
            buffer.push_str(&String::from_utf8_lossy(&chunk_bytes));

            // Extract and process all complete lines (separated by \n)
            while let Some(newline_idx) = buffer.find('\n') {
                let line = buffer[..newline_idx].trim().to_string();
                buffer = buffer[newline_idx + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                match serde_json::from_str::<OllamaChunk>(&line) {
                    Ok(parsed) => {
                        if let Some(err) = parsed.error {
                            let _ = window.emit(EVENT_ERROR, &err);
                            return Err(AppError::Agent(err));
                        }

                        if let Some(msg) = parsed.message {
                            if let Some(token) = msg.content {
                                if !token.is_empty() {
                                    full_response.push_str(&token);
                                    // Emit token to frontend
                                    let _ = window.emit(EVENT_TOKEN, &token);
                                }
                            }
                        }

                        if parsed.done.unwrap_or(false) {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("[LLM Stream] Failed to parse JSON line: {} (Error: {})", line, e);
                        continue;
                    }
                }
            }
        }

        Ok(full_response)
    }
}
