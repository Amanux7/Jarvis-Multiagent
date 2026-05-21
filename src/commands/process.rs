// src-tauri/src/commands/process.rs
//! Foundational command: process_user_input
//! Accepts a raw string, logs it, and returns a structured dummy response.
//! This serves as the bridge / sanity-check layer before the full ReAct loop
//! and as a lightweight fallback for testing the IPC pipeline.

use tauri::State;
use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct ProcessResponse {
    pub received:  String,
    pub processed: String,
    pub length:    usize,
}

/// Validates, logs and echoes back a user input string.
/// Returns a `ProcessResponse` with metadata about the input.
#[tauri::command]
pub async fn process_user_input(
    _state: State<'_, AppState>,
    input: String,
) -> Result<ProcessResponse, String> {
    if input.trim().is_empty() {
        return Err("Input cannot be empty.".to_string());
    }

    let trimmed = input.trim().to_string();
    let length  = trimmed.len();

    // Log to the Tauri backend console (visible in `tauri dev`)
    println!("[JARvis] process_user_input — received {} chars: {:?}", length, &trimmed);

    let processed = format!(
        "[JARvis Echo] You sent {} character(s): \"{}\" — ready for full agent dispatch.",
        length, trimmed
    );

    Ok(ProcessResponse {
        received:  trimmed,
        processed,
        length,
    })
}
