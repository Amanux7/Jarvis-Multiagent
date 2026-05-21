// src-tauri/src/db/mod.rs
use rusqlite::{Connection, params};
use std::path::Path;
use uuid::Uuid;
use chrono::Utc;

use crate::errors::AppResult;
use crate::db::models::{Conversation, Message, MessageRole, Settings, ToolLog};

pub mod models;

/// Initialise the database at the given path, running all migrations.
pub fn init_db(path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    // Enable WAL for better concurrent read performance
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        );
    ")?;

    let version: i64 = conn
        .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |r| r.get(0))
        .unwrap_or(0);

    if version < 1 {
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS conversations (
                id          TEXT PRIMARY KEY,
                title       TEXT NOT NULL DEFAULT 'New Chat',
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS messages (
                id               TEXT PRIMARY KEY,
                conversation_id  TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                role             TEXT NOT NULL,
                content          TEXT NOT NULL,
                created_at       TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tool_logs (
                id               TEXT PRIMARY KEY,
                conversation_id  TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                message_id       TEXT NOT NULL,
                tool_name        TEXT NOT NULL,
                input            TEXT NOT NULL,
                output           TEXT NOT NULL,
                status           TEXT NOT NULL DEFAULT 'success',
                created_at       TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            INSERT OR IGNORE INTO schema_version (version) VALUES (1);
        ")?;

        // Seed default settings
        let defaults = Settings::default();
        let pairs: Vec<(&str, String)> = vec![
            ("ollama_base_url",  defaults.ollama_base_url),
            ("ollama_model",     defaults.ollama_model),
            ("provider",         defaults.provider),
            ("max_iterations",   defaults.max_iterations.to_string()),
            ("shell_allowlist",  serde_json::to_string(&defaults.shell_allowlist).unwrap()),
            ("system_prompt",    defaults.system_prompt),
        ];
        for (k, v) in pairs {
            conn.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
                params![k, v],
            )?;
        }
    }

    Ok(())
}

// ── Conversation CRUD ─────────────────────────────────────────────────────────

pub fn create_conversation(conn: &Connection) -> AppResult<Conversation> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO conversations (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, "New Chat", now, now],
    )?;
    Ok(Conversation { id, title: "New Chat".to_string(), created_at: now.clone(), updated_at: now })
}

pub fn list_conversations(conn: &Connection) -> AppResult<Vec<Conversation>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Conversation {
            id:         r.get(0)?,
            title:      r.get(1)?,
            created_at: r.get(2)?,
            updated_at: r.get(3)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn update_conversation_title(conn: &Connection, id: &str, title: &str) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    )?;
    Ok(())
}

pub fn delete_conversation(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn touch_conversation(conn: &Connection, id: &str) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    Ok(())
}

// ── Message CRUD ──────────────────────────────────────────────────────────────

pub fn insert_message(conn: &Connection, conv_id: &str, role: &MessageRole, content: &str) -> AppResult<Message> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, conv_id, role.as_str(), content, now],
    )?;
    Ok(Message {
        id,
        conversation_id: conv_id.to_string(),
        role: role.clone(),
        content: content.to_string(),
        created_at: now,
    })
}

pub fn get_messages(conn: &Connection, conv_id: &str) -> AppResult<Vec<Message>> {
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, created_at FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC"
    )?;
    let rows = stmt.query_map(params![conv_id], |r| {
        let role_str: String = r.get(2)?;
        Ok(Message {
            id:              r.get(0)?,
            conversation_id: r.get(1)?,
            role:            MessageRole::from_str(&role_str),
            content:         r.get(3)?,
            created_at:      r.get(4)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn update_message_content(conn: &Connection, msg_id: &str, content: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE messages SET content = ?1 WHERE id = ?2",
        params![content, msg_id],
    )?;
    Ok(())
}

// ── Tool Log CRUD ─────────────────────────────────────────────────────────────

pub fn insert_tool_log(
    conn: &Connection,
    conv_id: &str,
    msg_id: &str,
    tool_name: &str,
    input: &str,
    output: &str,
    status: &str,
) -> AppResult<ToolLog> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO tool_logs (id, conversation_id, message_id, tool_name, input, output, status, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![id, conv_id, msg_id, tool_name, input, output, status, now],
    )?;
    Ok(ToolLog {
        id,
        conversation_id: conv_id.to_string(),
        message_id: msg_id.to_string(),
        tool_name: tool_name.to_string(),
        input: input.to_string(),
        output: output.to_string(),
        status: status.to_string(),
        created_at: now,
    })
}

pub fn get_tool_logs(conn: &Connection, conv_id: &str) -> AppResult<Vec<ToolLog>> {
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, message_id, tool_name, input, output, status, created_at FROM tool_logs WHERE conversation_id = ?1 ORDER BY created_at ASC"
    )?;
    let rows = stmt.query_map(params![conv_id], |r| {
        Ok(ToolLog {
            id:              r.get(0)?,
            conversation_id: r.get(1)?,
            message_id:      r.get(2)?,
            tool_name:       r.get(3)?,
            input:           r.get(4)?,
            output:          r.get(5)?,
            status:          r.get(6)?,
            created_at:      r.get(7)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

// ── Settings ──────────────────────────────────────────────────────────────────

pub fn load_settings(conn: &Connection) -> AppResult<Settings> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let pairs: std::collections::HashMap<String, String> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    let defaults = Settings::default();
    Ok(Settings {
        ollama_base_url: pairs.get("ollama_base_url").cloned().unwrap_or(defaults.ollama_base_url),
        ollama_model:    pairs.get("ollama_model").cloned().unwrap_or(defaults.ollama_model),
        provider:        pairs.get("provider").cloned().unwrap_or(defaults.provider),
        max_iterations:  pairs.get("max_iterations").and_then(|v| v.parse().ok()).unwrap_or(defaults.max_iterations),
        shell_allowlist: pairs.get("shell_allowlist")
            .and_then(|v| serde_json::from_str(v).ok())
            .unwrap_or(defaults.shell_allowlist),
        system_prompt:   pairs.get("system_prompt").cloned().unwrap_or(defaults.system_prompt),
    })
}

pub fn save_settings(conn: &Connection, s: &Settings) -> AppResult<()> {
    let pairs: Vec<(&str, String)> = vec![
        ("ollama_base_url",  s.ollama_base_url.clone()),
        ("ollama_model",     s.ollama_model.clone()),
        ("provider",         s.provider.clone()),
        ("max_iterations",   s.max_iterations.to_string()),
        ("shell_allowlist",  serde_json::to_string(&s.shell_allowlist)?),
        ("system_prompt",    s.system_prompt.clone()),
    ];
    for (k, v) in pairs {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![k, v],
        )?;
    }
    Ok(())
}
