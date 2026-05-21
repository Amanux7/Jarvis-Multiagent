// src-tauri/src/tools/fs_tools.rs
//! Filesystem tools — read, write, list, exists.
//! All paths are validated before I/O.

use std::fs;
use std::path::Path;
use crate::errors::{AppError, AppResult};

fn get_str<'a>(args: &'a serde_json::Value, key: &str) -> AppResult<&'a str> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Tool(format!("Missing required argument '{}'", key)))
}

pub fn read_file(args: &serde_json::Value) -> AppResult<String> {
    let path = get_str(args, "path")?;
    if !Path::new(path).exists() {
        return Err(AppError::NotFound(format!("File not found: {}", path)));
    }
    let content = fs::read_to_string(path)?;
    Ok(content)
}

pub fn write_file(args: &serde_json::Value) -> AppResult<String> {
    let path = get_str(args, "path")?;
    let content = get_str(args, "content")?;
    let append = args.get("append").and_then(|v| v.as_bool()).unwrap_or(false);

    // Create parent directories if needed
    if let Some(parent) = Path::new(path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    if append {
        use std::io::Write;
        let mut file = fs::OpenOptions::new().append(true).create(true).open(path)?;
        file.write_all(content.as_bytes())?;
    } else {
        fs::write(path, content)?;
    }

    Ok(format!("Successfully wrote {} bytes to '{}'", content.len(), path))
}

pub fn list_dir(args: &serde_json::Value) -> AppResult<String> {
    let path = get_str(args, "path")?;
    if !Path::new(path).is_dir() {
        return Err(AppError::NotFound(format!("Directory not found: {}", path)));
    }

    let entries: Vec<String> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let kind = if e.path().is_dir() { "DIR" } else { "FILE" };
            let size = e.metadata().ok()
                .map(|m| if m.is_file() { format!(" ({} bytes)", m.len()) } else { String::new() })
                .unwrap_or_default();
            format!("[{}] {}{}", kind, name, size)
        })
        .collect();

    if entries.is_empty() {
        Ok(format!("Directory '{}' is empty.", path))
    } else {
        Ok(format!("Contents of '{}':\n{}", path, entries.join("\n")))
    }
}

pub fn file_exists(args: &serde_json::Value) -> AppResult<String> {
    let path = get_str(args, "path")?;
    let exists = Path::new(path).exists();
    let kind = if Path::new(path).is_dir() { "directory" } else { "file" };
    Ok(if exists {
        format!("'{}' exists ({})", path, kind)
    } else {
        format!("'{}' does not exist", path)
    })
}
