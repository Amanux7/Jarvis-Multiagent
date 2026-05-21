// src-tauri/src/commands/memory.rs
//! Tauri commands that expose the Memory Tree to the frontend.
//!
//! Commands:
//!   - `store_memory`   — compress + chunk + save raw text for a given source
//!   - `get_memory`     — retrieve chunks, optionally filtered by source
//!   - `remove_memory`  — delete all chunks for a given source
//!   - `compress_preview` — pure transform: compress text and return the result
//!                          (no DB write, useful for previewing what gets stored)

use tauri::State;

use crate::errors::AppError;
use crate::memory::{self, MemoryChunk};
use crate::state::AppState;

/// Compress `raw_text`, chunk it at ≈3 000 tokens, and store every chunk in
/// the `memory_chunks` table tagged with `source`.
///
/// Returns the list of persisted chunks so the frontend can display token
/// counts and confirm how many chunks were created.
#[tauri::command]
pub async fn store_memory(
    state: State<'_, AppState>,
    source: String,
    raw_text: String,
) -> Result<Vec<MemoryChunk>, AppError> {
    let db_path = state.db_path.clone();
    memory::insert_memory(db_path, source, raw_text).await
}

/// Retrieve memory chunks. Pass `source` to filter by namespace, or omit it
/// (pass `null` / `undefined` from JS) to retrieve all chunks.
#[tauri::command]
pub async fn get_memory(
    state: State<'_, AppState>,
    source: Option<String>,
) -> Result<Vec<MemoryChunk>, AppError> {
    let db_path = state.db_path.clone();
    memory::query_memory(db_path, source).await
}

/// Delete all memory chunks whose `source` matches. Returns the number of rows
/// that were removed.
#[tauri::command]
pub async fn remove_memory(
    state: State<'_, AppState>,
    source: String,
) -> Result<usize, AppError> {
    let db_path = state.db_path.clone();
    memory::delete_memory(db_path, source).await
}

/// Compress `raw_text` and return the clean Markdown without writing to the DB.
/// Useful for previewing compression quality from the settings UI.
#[tauri::command]
pub fn compress_preview(raw_text: String) -> String {
    memory::compress_text(&raw_text)
}
