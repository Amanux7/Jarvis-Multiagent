// src-tauri/src/memory.rs
//! Memory Tree + Token Compression layer.
//!
//! Responsibilities:
//!   - `compress_text`  — strips HTML, removes non-ASCII, outputs clean Markdown
//!   - `estimate_tokens`— rough word-based token estimator (1 word ≈ 1.3 tokens)
//!   - `chunk_text`     — splits text into chunks ≤ max_tokens
//!   - `insert_memory`  — async: compress → chunk → persist each chunk to SQLite
//!   - `query_memory`   — async: fetch chunks, optionally filtered by source
//!   - `delete_memory`  — async: remove all chunks for a given source
//!
//! All database I/O runs inside `tokio::task::spawn_blocking` because rusqlite
//! is a synchronous library. Each call opens its own connection; SQLite's WAL
//! mode (enabled by the main `init_db` call) handles concurrent readers safely.

use std::path::PathBuf;

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tokio::task;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

// ── Data Model ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryChunk {
    pub id: String,
    /// Logical namespace / origin label (e.g. "web", "file:/path", conv-id …)
    pub source: String,
    /// Original raw text (kept for auditability / re-compression)
    pub raw_text: String,
    /// Compressed, de-HTML'd, ASCII-safe Markdown representation
    pub compressed_markdown: String,
    /// Rough token count for the compressed chunk
    pub token_count: i64,
    pub created_at: String,
}

// ── Text Processing ───────────────────────────────────────────────────────────

/// Strip HTML tags, decode common entities, remove non-ASCII characters, and
/// normalise whitespace into clean, compact Markdown paragraphs.
///
/// This is a zero-dependency, regex-free implementation. It handles nested
/// angle brackets gracefully and replaces removed tags with a space so that
/// adjacent words from different elements are not concatenated.
pub fn compress_text(input: &str) -> String {
    // ── Phase 1: strip HTML tags via a char-state machine ────────────────────
    let mut stripped = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                // Insert a space so adjacent words don't merge
                stripped.push(' ');
            }
            _ if !in_tag => stripped.push(ch),
            _ => { /* skip tag body */ }
        }
    }

    // ── Phase 2: decode common HTML entities ─────────────────────────────────
    let decoded = stripped
        .replace("&amp;",   "&")
        .replace("&lt;",    "<")
        .replace("&gt;",    ">")
        .replace("&quot;",  "\"")
        .replace("&apos;",  "'")
        .replace("&#39;",   "'")
        .replace("&nbsp;",  " ")
        .replace("&hellip;","...")
        .replace("&mdash;", "--")
        .replace("&ndash;", "-")
        .replace("&laquo;", "<<")
        .replace("&raquo;", ">>");

    // ── Phase 3: keep only printable ASCII + whitespace ──────────────────────
    let ascii_only: String = decoded
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .collect();

    // ── Phase 4: normalise whitespace, build paragraphs ──────────────────────
    // Collapse each run of whitespace-only lines into a paragraph break,
    // trim every line, and emit the result as a compact multi-line string.
    let mut paragraphs: Vec<String> = Vec::new();
    let mut current_para: Vec<&str> = Vec::new();

    for raw_line in ascii_only.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            if !current_para.is_empty() {
                paragraphs.push(current_para.join(" "));
                current_para.clear();
            }
        } else {
            current_para.push(line);
        }
    }
    if !current_para.is_empty() {
        paragraphs.push(current_para.join(" "));
    }

    paragraphs.join("\n\n")
}

/// Rough token estimator: 1 word ≈ 1.3 tokens (consistent with common GPT-3/4
/// empirical observations for English prose).
pub fn estimate_tokens(text: &str) -> usize {
    let word_count = text.split_whitespace().count();
    (word_count as f64 * 1.3).ceil() as usize
}

/// Split `text` into chunks where each chunk's estimated token count does not
/// exceed `max_tokens`. Splitting is done at line boundaries first; if a single
/// line is itself too long, it is further split at word boundaries.
pub fn chunk_text(text: &str, max_tokens: usize) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        // Would adding this line exceed the budget?
        let candidate = if current.is_empty() {
            line.to_string()
        } else {
            format!("{}\n{}", current, line)
        };

        if estimate_tokens(&candidate) > max_tokens {
            // Flush the current accumulator
            if !current.is_empty() {
                chunks.push(current.trim().to_string());
                current.clear();
            }

            // If the line itself is too long, split at word boundaries
            if estimate_tokens(line) > max_tokens {
                let mut word_buf = String::new();
                for word in line.split_whitespace() {
                    let proposed = if word_buf.is_empty() {
                        word.to_string()
                    } else {
                        format!("{} {}", word_buf, word)
                    };

                    if estimate_tokens(&proposed) > max_tokens {
                        if !word_buf.is_empty() {
                            chunks.push(word_buf.trim().to_string());
                        }
                        word_buf = word.to_string();
                    } else {
                        word_buf = proposed;
                    }
                }
                // What remains starts the next accumulator
                current = word_buf;
            } else {
                current = line.to_string();
            }
        } else {
            current = candidate;
        }
    }

    // Flush the final accumulator
    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    // Guard: never return an empty Vec for non-empty input
    if chunks.is_empty() && !text.trim().is_empty() {
        chunks.push(text.trim().to_string());
    }

    chunks
}

// ── SQLite helpers ────────────────────────────────────────────────────────────

/// Create the `memory_chunks` table and index if they don't already exist.
/// Safe to call every time — uses `IF NOT EXISTS`.
fn init_memory_table(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS memory_chunks (
            id                  TEXT PRIMARY KEY,
            source              TEXT NOT NULL,
            raw_text            TEXT NOT NULL,
            compressed_markdown TEXT NOT NULL,
            token_count         INTEGER NOT NULL DEFAULT 0,
            created_at          TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_memory_source
            ON memory_chunks (source);
        CREATE INDEX IF NOT EXISTS idx_memory_created
            ON memory_chunks (created_at DESC);
        ",
    )?;
    Ok(())
}

/// Map a `rusqlite::Row` to a `MemoryChunk`.
fn row_to_chunk(r: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryChunk> {
    Ok(MemoryChunk {
        id:                   r.get(0)?,
        source:               r.get(1)?,
        raw_text:             r.get(2)?,
        compressed_markdown:  r.get(3)?,
        token_count:          r.get(4)?,
        created_at:           r.get(5)?,
    })
}

// ── Public async API ──────────────────────────────────────────────────────────

/// Compress `raw_text`, chunk it at ≈3 000 tokens, and persist every chunk to
/// SQLite. Returns all inserted `MemoryChunk` records.
///
/// Uses `tokio::task::spawn_blocking` so the calling async context is never
/// stalled by SQLite I/O.
pub async fn insert_memory(
    db_path: PathBuf,
    source: String,
    raw_text: String,
) -> AppResult<Vec<MemoryChunk>> {
    task::spawn_blocking(move || {
        let conn = Connection::open(&db_path)?;
        // Piggy-back on WAL mode already set by init_db; harmless to repeat.
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        init_memory_table(&conn)?;

        let compressed = compress_text(&raw_text);
        let chunks     = chunk_text(&compressed, 3_000);

        let mut saved: Vec<MemoryChunk> = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            let tokens = estimate_tokens(&chunk) as i64;
            let id     = Uuid::new_v4().to_string();
            let now    = Utc::now().to_rfc3339();

            conn.execute(
                "INSERT INTO memory_chunks
                     (id, source, raw_text, compressed_markdown, token_count, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, source, raw_text, chunk, tokens, now],
            )?;

            saved.push(MemoryChunk {
                id,
                source:              source.clone(),
                raw_text:            raw_text.clone(),
                compressed_markdown: chunk,
                token_count:         tokens,
                created_at:          now,
            });
        }

        Ok(saved)
    })
    .await
    .map_err(|e| AppError::Agent(format!("Memory insert task panicked: {e}")))?
}

/// Fetch memory chunks, optionally filtered by `source`.
/// Results are ordered newest-first.
pub async fn query_memory(
    db_path: PathBuf,
    source_filter: Option<String>,
) -> AppResult<Vec<MemoryChunk>> {
    task::spawn_blocking(move || {
        let conn = Connection::open(&db_path)?;
        init_memory_table(&conn)?;

        let chunks: Vec<MemoryChunk> = match source_filter {
            Some(src) => {
                let mut stmt = conn.prepare(
                    "SELECT id, source, raw_text, compressed_markdown, token_count, created_at
                     FROM   memory_chunks
                     WHERE  source = ?1
                     ORDER  BY created_at DESC",
                )?;
                let result: Vec<MemoryChunk> = stmt
                    .query_map(params![src], row_to_chunk)?
                    .filter_map(|r| r.ok())
                    .collect();
                result
            }
            None => {
                let mut stmt = conn.prepare(
                    "SELECT id, source, raw_text, compressed_markdown, token_count, created_at
                     FROM   memory_chunks
                     ORDER  BY created_at DESC",
                )?;
                let result: Vec<MemoryChunk> = stmt
                    .query_map([], row_to_chunk)?
                    .filter_map(|r| r.ok())
                    .collect();
                result
            }
        };

        Ok(chunks)
    })
    .await
    .map_err(|e| AppError::Agent(format!("Memory query task panicked: {e}")))?
}

/// Delete all memory chunks whose `source` equals `source`.
/// Returns the number of rows deleted.
pub async fn delete_memory(
    db_path: PathBuf,
    source: String,
) -> AppResult<usize> {
    task::spawn_blocking(move || {
        let conn = Connection::open(&db_path)?;
        init_memory_table(&conn)?;

        let deleted = conn.execute(
            "DELETE FROM memory_chunks WHERE source = ?1",
            params![source],
        )?;

        Ok(deleted)
    })
    .await
    .map_err(|e| AppError::Agent(format!("Memory delete task panicked: {e}")))?
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_html_tags() {
        let html = "<h1>Hello <b>World</b></h1><p>Line two.</p>";
        let result = compress_text(html);
        assert!(!result.contains('<'), "Should have no angle brackets");
        assert!(result.contains("Hello"), "Should retain text content");
        assert!(result.contains("World"));
        assert!(result.contains("Line two."));
    }

    #[test]
    fn decodes_entities() {
        let html = "AT&amp;T &lt;rocks&gt; &nbsp; &quot;yes&quot;";
        let result = compress_text(html);
        assert!(result.contains("AT&T"));
        assert!(result.contains("\"yes\""));
    }

    #[test]
    fn strips_non_ascii() {
        let text = "café résumé naïve";
        let result = compress_text(text);
        assert!(result.is_ascii(), "Output should be pure ASCII");
    }

    #[test]
    fn token_estimate_non_zero() {
        assert_eq!(estimate_tokens("hello world"), 3); // ceil(2 * 1.3) = 3
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn chunk_text_respects_limit() {
        // Build a text that is definitely > 3000 tokens
        let long_text = "word ".repeat(5_000);
        let chunks = chunk_text(&long_text, 3_000);
        assert!(chunks.len() >= 2, "Long text should be chunked");
        for chunk in &chunks {
            let tokens = estimate_tokens(chunk);
            assert!(
                tokens <= 3_100, // allow small overshoot at word boundaries
                "Chunk token count {} exceeded limit",
                tokens
            );
        }
    }

    #[test]
    fn empty_input_returns_empty_vec() {
        let chunks = chunk_text("", 3_000);
        assert!(chunks.is_empty());
    }
}
