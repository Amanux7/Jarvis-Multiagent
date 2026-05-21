// src-tauri/src/tools/shell_tools.rs
//! Shell tool — runs an allowlisted command and returns stdout/stderr.

use std::process::Command;
use crate::errors::{AppError, AppResult};

pub fn run_command(args: &serde_json::Value, allowlist: &[String]) -> AppResult<String> {
    let command = args
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Tool("Missing 'command' argument".to_string()))?;

    // Security: only allow commands in the user-defined allowlist
    let binary = std::path::Path::new(command)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(command);

    if !allowlist.iter().any(|a| a == binary || a == command) {
        return Err(AppError::Tool(format!(
            "Command '{}' is not in the shell allowlist. Add it in Settings to enable it.",
            binary
        )));
    }

    let cmd_args: Vec<String> = args
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let output = Command::new(command)
        .args(&cmd_args)
        .output()
        .map_err(|e| AppError::Tool(format!("Failed to execute '{}': {}", command, e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let combined = if output.status.success() {
        if stdout.is_empty() {
            "(command produced no output)".to_string()
        } else {
            stdout
        }
    } else {
        format!(
            "Command exited with code {}.\nSTDOUT: {}\nSTDERR: {}",
            output.status.code().unwrap_or(-1),
            stdout,
            stderr
        )
    };

    Ok(combined)
}
