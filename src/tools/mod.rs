// src-tauri/src/tools/mod.rs
//! Tool registry — maps tool names to their Rust implementations.
//! The agent loop calls `dispatch` with the tool name and JSON args.

pub mod fs_tools;
pub mod shell_tools;

use serde::{Deserialize, Serialize};
use crate::errors::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub output: String,
    pub success: bool,
}

/// Dispatches a tool call to its implementation.
/// shell_allowlist is passed from AppSettings.
pub fn dispatch(call: &ToolCall, shell_allowlist: &[String]) -> AppResult<ToolResult> {
    let result = match call.name.as_str() {
        "read_file" => fs_tools::read_file(&call.args),
        "write_file" => fs_tools::write_file(&call.args),
        "list_dir" => fs_tools::list_dir(&call.args),
        "file_exists" => fs_tools::file_exists(&call.args),
        "run_command" => shell_tools::run_command(&call.args, shell_allowlist),
        unknown => Err(AppError::Tool(format!("Unknown tool: '{}'", unknown))),
    };

    match result {
        Ok(output) => Ok(ToolResult {
            tool_name: call.name.clone(),
            output,
            success: true,
        }),
        Err(e) => Ok(ToolResult {
            tool_name: call.name.clone(),
            output: format!("ERROR: {}", e),
            success: false,
        }),
    }
}

/// Returns the tool schema description sent to the LLM in the system prompt.
pub fn tool_descriptions() -> &'static str {
    r#"You have access to the following tools. To use a tool, output ONLY valid JSON on a single line in this exact format:
{"tool":"<name>","args":{...}}

Available tools:

read_file: Read the text content of a file.
  args: {"path": "<absolute or relative file path>"}

write_file: Write or overwrite a file with the given content.
  args: {"path": "<file path>", "content": "<text content>", "append": false}

list_dir: List all entries in a directory.
  args: {"path": "<directory path>"}

file_exists: Check whether a file or directory exists.
  args: {"path": "<path>"}

run_command: Run a shell command from the allowlist and return its output.
  args: {"command": "<binary name>", "args": ["<arg1>", "<arg2>"]}

finish: Signal that you have completed the task. Use this as your final action.
  args: {"summary": "<brief description of what was accomplished>"}

Rules:
- Think step by step before deciding which tool to use.
- After receiving a tool result, incorporate it into your reasoning before proceeding.
- Only call one tool per response.
- When the task is done, call {"tool":"finish","args":{"summary":"..."}}.
"#
}
