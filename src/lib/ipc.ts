// src/lib/ipc.ts
// Typed wrappers around Tauri's invoke(). 
// NEVER call invoke() directly from components — always use these.

import { invoke } from "@tauri-apps/api/core";
import type {
  Conversation,
  Message,
  ToolLog,
  Settings,
} from "../types";

// ── Conversations ────────────────────────────────────────────────────────────

export const newConversation = (): Promise<Conversation> =>
  invoke("new_conversation");

export const listConversations = (): Promise<Conversation[]> =>
  invoke("list_conversations");

export const deleteConversation = (id: string): Promise<void> =>
  invoke("delete_conversation", { id });

export const renameConversation = (id: string, title: string): Promise<void> =>
  invoke("rename_conversation", { id, title });

// ── Messages ─────────────────────────────────────────────────────────────────

export const getMessages = (conversationId: string): Promise<Message[]> =>
  invoke("get_messages", { conversationId: conversationId });

export const getToolLogs = (conversationId: string): Promise<ToolLog[]> =>
  invoke("get_tool_logs", { conversationId: conversationId });

// ── Agent ─────────────────────────────────────────────────────────────────────

export const sendMessage = (
  conversationId: string,
  content: string
): Promise<string> =>
  invoke("send_message", { conversationId, content });

export const stopAgent = (): Promise<void> => invoke("stop_agent");

// ── Settings ──────────────────────────────────────────────────────────────────

export const getSettings = (): Promise<Settings> => invoke("get_settings");

export const saveSettings = (settings: Settings): Promise<void> =>
  invoke("save_settings", { settings });

// ── Foundational Processing ────────────────────────────────────────────────

export interface ProcessResponse {
  received:  string;
  processed: string;
  length:    number;
}

export const processUserInput = (input: string): Promise<ProcessResponse> =>
  invoke("process_user_input", { input });

// ── Memory Tree ───────────────────────────────────────────────────────────────

/** Mirrors src-tauri/src/memory.rs :: MemoryChunk */
export interface MemoryChunk {
  id:                   string;
  source:               string;
  raw_text:             string;
  compressed_markdown:  string;
  token_count:          number;
  created_at:           string;
}

/**
 * Compress raw_text, chunk it at ≈3 000 tokens, and store every chunk tagged
 * with `source`. Returns the list of persisted MemoryChunk records.
 */
export const storeMemory = (
  source: string,
  rawText: string,
): Promise<MemoryChunk[]> =>
  invoke("store_memory", { source, rawText });

/**
 * Fetch memory chunks. Pass a `source` string to filter by namespace,
 * or omit it to retrieve all chunks.
 */
export const getMemory = (source?: string): Promise<MemoryChunk[]> =>
  invoke("get_memory", { source: source ?? null });

/**
 * Delete all memory chunks for the given `source`.
 * Returns the number of rows deleted.
 */
export const removeMemory = (source: string): Promise<number> =>
  invoke("remove_memory", { source });

/**
 * Compress raw_text and return the clean Markdown preview without writing to
 * the database. Useful for settings UI previews.
 */
export const compressPreview = (rawText: string): Promise<string> =>
  invoke("compress_preview", { rawText });
