// src/hooks/useChat.ts
// Orchestrates loading conversations/messages and sending messages.

import { useCallback, useEffect } from "react";
import { useAppStore } from "../store/useAppStore";
import * as ipc from "../lib/ipc";
import * as events from "../lib/events";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { v4 as uuidv4 } from "uuid";

export function useChat() {
  const {
    activeConvId,
    setConversations,
    addConversation,
    setActiveConvId,
    removeConversation,
    messages,
    setMessages,
    appendMessage,
    streamBuffer,
    appendToken,
    clearStreamBuffer,
    agentStatus,
    setAgentStatus,
    setCurrentIteration,
    addLiveToolCall,
    clearLiveToolCalls,
    liveToolCalls,
  } = useAppStore();

  // ── Load all conversations on mount ────────────────────────────────────────
  useEffect(() => {
    ipc.listConversations().then(setConversations).catch(console.error);
  }, []);

  // ── Load messages when active conversation changes ─────────────────────────
  useEffect(() => {
    if (!activeConvId) return;
    ipc.getMessages(activeConvId).then((msgs) => setMessages(activeConvId, msgs)).catch(console.error);
  }, [activeConvId]);

  // ── Register Tauri event listeners ────────────────────────────────────────
  useEffect(() => {
    const unlisteners: Promise<UnlistenFn>[] = [];

    unlisteners.push(
      events.onToken((token) => {
        appendToken(token);
      })
    );

    unlisteners.push(
      events.onThinking((iter) => {
        setCurrentIteration(iter);
      })
    );

    unlisteners.push(
      events.onToolStart((payload) => {
        const callId = uuidv4();
        addLiveToolCall({
          id: callId,
          tool: payload.tool,
          args: payload.args,
          status: "running",
        });
      })
    );

    unlisteners.push(
      events.onToolDone((payload) => {
        // Find the last running tool call for this tool name
        useAppStore.setState((s) => {
          const idx = [...s.liveToolCalls]
            .reverse()
            .findIndex((c) => c.tool === payload.tool && c.status === "running");
          if (idx === -1) return s;
          const realIdx = s.liveToolCalls.length - 1 - idx;
          const updated = [...s.liveToolCalls];
          updated[realIdx] = {
            ...updated[realIdx],
            output: payload.output,
            success: payload.success,
            status: payload.success ? "done" : "error",
          };
          return { liveToolCalls: updated };
        });
      })
    );

    unlisteners.push(
      events.onDone((_payload) => {
        clearStreamBuffer();
        clearLiveToolCalls();
        setAgentStatus("idle");
        setCurrentIteration(0);
        // Reload messages to get persisted assistant response
        if (activeConvId) {
          ipc.getMessages(activeConvId).then((msgs) => setMessages(activeConvId, msgs)).catch(console.error);
          ipc.listConversations().then(setConversations).catch(console.error);
        }
      })
    );

    unlisteners.push(
      events.onError((msg) => {
        console.error("Agent error:", msg);
        clearStreamBuffer();
        setAgentStatus("error");
        setCurrentIteration(0);
      })
    );

    return () => {
      unlisteners.forEach((p) => p.then((fn) => fn()));
    };
  }, [activeConvId]);

  // ── Create new conversation ────────────────────────────────────────────────
  const createConversation = useCallback(async () => {
    const conv = await ipc.newConversation();
    addConversation(conv);
    setActiveConvId(conv.id);
  }, []);

  // ── Delete conversation ────────────────────────────────────────────────────
  const deleteConversation = useCallback(async (id: string) => {
    await ipc.deleteConversation(id);
    removeConversation(id);
  }, []);

  // ── Send message ──────────────────────────────────────────────────────────
  const sendMessage = useCallback(
    async (content: string) => {
      if (!activeConvId || agentStatus === "running") return;

      setAgentStatus("running");
      clearStreamBuffer();
      clearLiveToolCalls();

      // Optimistically append user message to UI
      const tempUserMsg = {
        id: uuidv4(),
        conversation_id: activeConvId,
        role: "user" as const,
        content,
        created_at: new Date().toISOString(),
      };
      appendMessage(tempUserMsg);

      try {
        await ipc.sendMessage(activeConvId, content);
      } catch (e) {
        console.error("sendMessage failed:", e);
        setAgentStatus("error");
      }
    },
    [activeConvId, agentStatus]
  );

  // ── Stop agent ─────────────────────────────────────────────────────────────
  const stopAgent = useCallback(async () => {
    await ipc.stopAgent();
  }, []);

  const currentMessages = activeConvId ? (messages[activeConvId] ?? []) : [];

  return {
    currentMessages,
    streamBuffer,
    agentStatus,
    liveToolCalls,
    createConversation,
    deleteConversation,
    sendMessage,
    stopAgent,
  };
}
