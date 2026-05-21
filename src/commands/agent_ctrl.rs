// src-tauri/src/commands/agent_ctrl.rs
//! Agent control commands — send_message (triggers the ReAct loop) and stop_agent.

use tauri::{State, Window};
use crate::state::AppState;
use crate::db::{self, models::MessageRole};
use crate::agent::{self, llm::OllamaProvider, llm::ChatMessage, llm::ChatRole, CancelToken};
use crate::errors::AppError;

/// Main entry point: persist user message, run ReAct loop, persist assistant response.
/// Streams tokens to frontend via jarvis:token events throughout.
#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    window: Window,
    conversation_id: String,
    content: String,
) -> Result<String, AppError> {
    // 1. Load settings
    let (settings, cancel) = {
        let db = state.db.lock().map_err(|_| AppError::Mutex)?;
        let s = db::load_settings(&db)?;
        let cancel = {
            let mut tok = state.cancel_token.lock().map_err(|_| AppError::Mutex)?;
            let new_token = CancelToken::new();
            *tok = new_token.clone();
            new_token
        };
        (s, cancel)
    };

    // 2. Persist user message
    let user_msg = {
        let db = state.db.lock().map_err(|_| AppError::Mutex)?;
        let msg = db::insert_message(&db, &conversation_id, &MessageRole::User, &content)?;
        db::touch_conversation(&db, &conversation_id)?;
        msg
    };

    // 3. Build history for the agent (exclude the current user message — it's passed separately)
    let history: Vec<ChatMessage> = {
        let db = state.db.lock().map_err(|_| AppError::Mutex)?;
        let all_msgs = db::get_messages(&db, &conversation_id)?;
        all_msgs
            .into_iter()
            .filter(|m| m.id != user_msg.id) // exclude the just-inserted message
            .map(|m| ChatMessage {
                role: match m.role {
                    MessageRole::User      => ChatRole::User,
                    MessageRole::Assistant => ChatRole::Assistant,
                    MessageRole::Tool      => ChatRole::Tool,
                    MessageRole::System    => ChatRole::System,
                },
                content: m.content,
            })
            .collect()
    };

    // 4. Build provider
    let provider = OllamaProvider::new(&settings.ollama_base_url, &settings.ollama_model);

    // 5. Run ReAct loop (this is the heavy async work — runs on Tokio runtime)
    let final_response = agent::run_react_loop(
        &provider,
        &settings.system_prompt,
        history,
        &content,
        &settings.shell_allowlist,
        settings.max_iterations as usize,
        &window,
        cancel,
    ).await?;

    // 6. Persist tool logs that were captured during the loop
    // (Tool logs are emitted as events; we also persist the assistant final message)
    {
        let db = state.db.lock().map_err(|_| AppError::Mutex)?;
        db::insert_message(&db, &conversation_id, &MessageRole::Assistant, &final_response)?;

        // Auto-set conversation title from first user message if still "New Chat"
        let convs = db::list_conversations(&db)?;
        if let Some(conv) = convs.iter().find(|c| c.id == conversation_id) {
            if conv.title == "New Chat" && !content.is_empty() {
                let title: String = content.chars().take(40).collect();
                let title = if content.len() > 40 { format!("{}…", title) } else { title };
                db::update_conversation_title(&db, &conversation_id, &title)?;
            }
        }
    }

    Ok(final_response)
}

/// Cancels any in-flight agent run for the current session.
#[tauri::command]
pub async fn stop_agent(
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let token = state.cancel_token.lock().map_err(|_| AppError::Mutex)?;
    token.cancel();
    Ok(())
}
