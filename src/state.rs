// src-tauri/src/state.rs
//! Shared application state injected into all Tauri commands.

use std::path::PathBuf;
use std::sync::Mutex;
use rusqlite::Connection;
use crate::agent::CancelToken;

pub struct AppState {
    /// The primary SQLite connection. Wrapped in Mutex for thread-safe access.
    pub db: Mutex<Connection>,

    /// Path to the SQLite file. Stored so that async memory operations can
    /// open their own connections via `tokio::task::spawn_blocking`.
    pub db_path: PathBuf,

    /// Cancel token for aborting the current in-flight agent run.
    pub cancel_token: Mutex<CancelToken>,
}

impl AppState {
    pub fn new(conn: Connection, db_path: PathBuf) -> Self {
        Self {
            db: Mutex::new(conn),
            db_path,
            cancel_token: Mutex::new(CancelToken::new()),
        }
    }
}
