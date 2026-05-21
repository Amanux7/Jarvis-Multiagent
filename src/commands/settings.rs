// src-tauri/src/commands/settings.rs
//! Settings get/save Tauri commands.

use tauri::State;
use crate::state::AppState;
use crate::db::{self, models::Settings};
use crate::errors::AppError;

#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<Settings, AppError> {
    let db = state.db.lock().map_err(|_| AppError::Mutex)?;
    db::load_settings(&db)
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<(), AppError> {
    let db = state.db.lock().map_err(|_| AppError::Mutex)?;
    db::save_settings(&db, &settings)
}
