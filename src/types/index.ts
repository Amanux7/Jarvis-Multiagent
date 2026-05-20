// src/types/index.ts
// Shared TypeScript types mirroring the Rust backend models.
// These must stay in sync with src-tauri/src/db/models.rs

export type MessageRole = "user" | "assistant" | "tool" | "system";

export interface Conversation {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
}

export interface Message {
  id: string;
  conversation_id: string;
  role: MessageRole;
  content: string;
  created_at: string;
}

export interface ToolLog {
  id: string;
  conversation_id: string;
  message_id: string;
  tool_name: string;
  input: string;
  output: string;
  status: "success" | "error";
  created_at: string;
}

export interface Settings {
  ollama_base_url: string;
  ollama_model: string;
  provider: string;
  max_iterations: number;
  shell_allowlist: string[];
  system_prompt: string;
}

// ── Event Payloads (emitted from Rust) ─────────────────────────────────────

export interface ToolStartPayload {
  tool: string;
  args: Record<string, unknown>;
}

export interface ToolDonePayload {
  tool: string;
  output: string;
  success: boolean;
}

export interface DonePayload {
  final_response: string;
  iterations: number;
}

// ── Live tool activity (UI state only) ─────────────────────────────────────

export interface LiveToolCall {
  id: string; // ephemeral client-side id
  tool: string;
  args: Record<string, unknown>;
  output?: string;
  success?: boolean;
  status: "running" | "done" | "error";
}

export type AgentStatus = "idle" | "running" | "error";
