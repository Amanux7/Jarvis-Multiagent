// src/lib/events.ts
// Typed wrappers around Tauri's listen() for backend-emitted events.
// Returns unlisten functions that components should call on unmount.

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ToolStartPayload, ToolDonePayload, DonePayload } from "../types";

export const onToken = (cb: (token: string) => void): Promise<UnlistenFn> =>
  listen<string>("jarvis:token", (e) => cb(e.payload));

export const onDone = (cb: (payload: DonePayload) => void): Promise<UnlistenFn> =>
  listen<DonePayload>("jarvis:done", (e) => cb(e.payload));

export const onError = (cb: (msg: string) => void): Promise<UnlistenFn> =>
  listen<string>("jarvis:error", (e) => cb(e.payload));

export const onThinking = (cb: (iteration: number) => void): Promise<UnlistenFn> =>
  listen<number>("jarvis:thinking", (e) => cb(e.payload));

export const onToolStart = (cb: (payload: ToolStartPayload) => void): Promise<UnlistenFn> =>
  listen<ToolStartPayload>("jarvis:tool_start", (e) => cb(e.payload));

export const onToolDone = (cb: (payload: ToolDonePayload) => void): Promise<UnlistenFn> =>
  listen<ToolDonePayload>("jarvis:tool_done", (e) => cb(e.payload));
