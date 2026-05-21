// src-tauri/src/commands/chat.rs
//! Tauri commands for conversation and message management.

use tauri::State;
use crate::state::AppState;
use crate::db::{self, models::{Conversation, Message, ToolLog}};
use crate::errors::AppError;

#[tauri::command]
pub async fn new_conversation(
    state: State<'_, AppState>,
) -> Result<Conversation, AppError> {
    let db = state.db.lock().map_err(|_| AppError::Mutex)?;
    db::create_conversation(&db)
}

#[tauri::command]
pub async fn list_conversations(
    state: State<'_, AppState>,
) -> Result<Vec<Conversation>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::Mutex)?;
    db::list_conversations(&db)
}

#[tauri::command]
pub async fn delete_conversation(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::Mutex)?;
    db::delete_conversation(&db, &id)
}

#[tauri::command]
pub async fn rename_conversation(
    state: State<'_, AppState>,
    id: String,
    title: String,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::Mutex)?;
    db::update_conversation_title(&db, &id, &title)
}

#[tauri::command]
pub async fn get_messages(
    state: State<'_, AppState>,
    conversation_id: String,
) -> Result<Vec<Message>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::Mutex)?;
    db::get_messages(&db, &conversation_id)
}

#[tauri::command]
pub async fn get_tool_logs(
    state: State<'_, AppState>,
    conversation_id: String,
) -> Result<Vec<ToolLog>, AppError> {
    let db = state.db.lock().map_err(|_| AppError::Mutex)?;
    db::get_tool_logs(&db, &conversation_id)
}
