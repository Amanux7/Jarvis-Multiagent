// src-tauri/src/agent/mod.rs
//! ReAct agent loop.
//! Reason → Act (tool call) → Observe → repeat until "finish" or max iterations.

pub mod llm;

use std::sync::{Arc, Mutex};
use tauri::Window;

use crate::errors::{AppError, AppResult};
use crate::tools::{self, ToolCall};
use crate::agent::llm::{
    ChatMessage, ChatRole, LlmProvider,
    EVENT_DONE, EVENT_ERROR, EVENT_THINKING, EVENT_TOOL_DONE, EVENT_TOOL_START,
};
use tauri::Emitter;

/// Payload emitted on jarvis:tool_start
#[derive(serde::Serialize, Clone)]
pub struct ToolStartPayload {
    pub tool: String,
    pub args: serde_json::Value,
}

/// Payload emitted on jarvis:tool_done
#[derive(serde::Serialize, Clone)]
pub struct ToolDonePayload {
    pub tool: String,
    pub output: String,
    pub success: bool,
}

/// Payload emitted on jarvis:done
#[derive(serde::Serialize, Clone)]
pub struct DonePayload {
    pub final_response: String,
    pub iterations: usize,
}

/// Cancellation token — a shared flag the command handler can flip to abort the loop.
#[derive(Clone)]
pub struct CancelToken(Arc<Mutex<bool>>);

impl CancelToken {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(false)))
    }
    pub fn cancel(&self) {
        if let Ok(mut v) = self.0.lock() {
            *v = true;
        }
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.lock().map(|v| *v).unwrap_or(false)
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Runs the full ReAct agent loop.
///
/// - `provider`: the LLM provider (Ollama for now)
/// - `system_prompt`: injected as first system message
/// - `history`: prior conversation messages
/// - `user_message`: the new user input
/// - `shell_allowlist`: passed to the tool dispatcher
/// - `max_iterations`: hard cap to prevent infinite loops
/// - `window`: Tauri window handle for event emission
/// - `cancel`: cancellation token
///
/// Returns the final assistant response string.
pub async fn run_react_loop(
    provider: &dyn LlmProvider,
    system_prompt: &str,
    history: Vec<ChatMessage>,
    user_message: &str,
    shell_allowlist: &[String],
    max_iterations: usize,
    window: &Window,
    cancel: CancelToken,
) -> AppResult<String> {
    // Build the initial message list:
    // [system, ...history, tools_description (system), user]
    let tool_system = format!(
        "{}\n\n{}",
        system_prompt,
        tools::tool_descriptions()
    );

    let mut messages: Vec<ChatMessage> = Vec::new();
    messages.push(ChatMessage {
        role: ChatRole::System,
        content: tool_system,
    });
    messages.extend(history);
    messages.push(ChatMessage {
        role: ChatRole::User,
        content: user_message.to_string(),
    });

    let mut final_response = String::new();
    let mut iteration = 0;

    loop {
        if cancel.is_cancelled() {
            let msg = "Agent run cancelled by user.".to_string();
            let _ = window.emit(EVENT_ERROR, &msg);
            return Err(AppError::Agent(msg));
        }

        if iteration >= max_iterations {
            let msg = format!(
                "Reached maximum iteration limit ({}). Stopping.",
                max_iterations
            );
            let _ = window.emit(EVENT_ERROR, &msg);
            return Err(AppError::Agent(msg));
        }

        iteration += 1;
        let _ = window.emit(EVENT_THINKING, iteration);

        // Get LLM response (streams tokens to frontend)
        let raw_response = provider.chat_stream(messages.clone(), window).await?;

        if raw_response.trim().is_empty() {
            return Err(AppError::Agent("LLM returned an empty response.".to_string()));
        }

        // Check if response contains a tool call (a JSON line starting with {"tool":...)
        if let Some(tool_call) = extract_tool_call(&raw_response) {
            // Handle the special "finish" tool
            if tool_call.name == "finish" {
                let summary = tool_call
                    .args
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Task completed.")
                    .to_string();

                // If final_response is empty, use the summary
                if final_response.is_empty() {
                    final_response = summary.clone();
                }

                let _ = window.emit(EVENT_DONE, DonePayload {
                    final_response: final_response.clone(),
                    iterations: iteration,
                });
                return Ok(final_response);
            }

            // Emit tool start to UI
            let _ = window.emit(EVENT_TOOL_START, ToolStartPayload {
                tool: tool_call.name.clone(),
                args: tool_call.args.clone(),
            });

            // Add the assistant's reasoning + tool call to history
            messages.push(ChatMessage {
                role: ChatRole::Assistant,
                content: raw_response.clone(),
            });

            // Execute the tool
            let tool_result = tools::dispatch(&tool_call, shell_allowlist);

            let (output, success) = match &tool_result {
                Ok(r) => (r.output.clone(), r.success),
                Err(e) => (format!("Tool error: {}", e), false),
            };

            // Emit tool done to UI
            let _ = window.emit(EVENT_TOOL_DONE, ToolDonePayload {
                tool: tool_call.name.clone(),
                output: output.clone(),
                success,
            });

            // Feed observation back into message history
            messages.push(ChatMessage {
                role: ChatRole::Tool,
                content: format!(
                    "Tool '{}' result:\n{}",
                    tool_call.name, output
                ),
            });
        } else {
            // No tool call — this is the final answer
            final_response = raw_response.clone();

            // Add to history and signal done
            messages.push(ChatMessage {
                role: ChatRole::Assistant,
                content: raw_response,
            });

            let _ = window.emit(EVENT_DONE, DonePayload {
                final_response: final_response.clone(),
                iterations: iteration,
            });

            return Ok(final_response);
        }
    }
}

/// Attempts to parse a tool call JSON from within a response string.
/// Looks for a line matching: {"tool":"<name>","args":{...}}
fn extract_tool_call(text: &str) -> Option<ToolCall> {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let Some(name) = val.get("tool").and_then(|v| v.as_str()) {
                    let args = val.get("args").cloned().unwrap_or(serde_json::Value::Object(Default::default()));
                    return Some(ToolCall {
                        name: name.to_string(),
                        args,
                    });
                }
            }
        }
    }
    None
}
