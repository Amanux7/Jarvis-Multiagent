// src-tauri/src/lib.rs
//! Application entry point — wires all modules, state, and Tauri commands together.

pub mod errors;
pub mod state;
pub mod db;
pub mod tools;
pub mod agent;
pub mod commands;
pub mod memory;

use tauri::Manager;
use state::AppState;
use db::init_db;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Resolve the app data directory for the database file
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data directory");

            let db_path = app_data_dir.join("jarvis.db");
            let conn = init_db(&db_path)
                .expect("Failed to initialise database");

            // Pass both the connection AND the path so async memory commands
            // can open their own connections via spawn_blocking.
            app.manage(AppState::new(conn, db_path));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Chat / conversation commands
            commands::chat::new_conversation,
            commands::chat::list_conversations,
            commands::chat::delete_conversation,
            commands::chat::rename_conversation,
            commands::chat::get_messages,
            commands::chat::get_tool_logs,
            // Agent commands
            commands::agent_ctrl::send_message,
            commands::agent_ctrl::stop_agent,
            // Settings commands
            commands::settings::get_settings,
            commands::settings::save_settings,
            // Foundational input processing
            commands::process::process_user_input,
            // Memory Tree commands
            commands::memory::store_memory,
            commands::memory::get_memory,
            commands::memory::remove_memory,
            commands::memory::compress_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
