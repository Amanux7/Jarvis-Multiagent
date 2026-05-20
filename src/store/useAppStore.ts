// src/store/useAppStore.ts
import { create } from "zustand";
import type {
  Conversation,
  Message,
  ToolLog,
  Settings,
  AgentStatus,
  LiveToolCall,
} from "../types";

interface AppStore {
  // ── Conversations ──────────────────────────────────────────────────────────
  conversations: Conversation[];
  activeConvId: string | null;
  setConversations: (c: Conversation[]) => void;
  setActiveConvId: (id: string | null) => void;
  addConversation: (c: Conversation) => void;
  removeConversation: (id: string) => void;
  updateConversationTitle: (id: string, title: string) => void;

  // ── Messages ───────────────────────────────────────────────────────────────
  messages: Record<string, Message[]>;
  setMessages: (convId: string, msgs: Message[]) => void;
  appendMessage: (msg: Message) => void;

  // ── Streaming state ────────────────────────────────────────────────────────
  streamBuffer: string;       // tokens accumulating in real-time
  appendToken: (t: string) => void;
  clearStreamBuffer: () => void;

  // ── Tool activity ──────────────────────────────────────────────────────────
  liveToolCalls: LiveToolCall[];
  addLiveToolCall: (call: LiveToolCall) => void;
  updateLiveToolCall: (id: string, updates: Partial<LiveToolCall>) => void;
  clearLiveToolCalls: () => void;
  toolLogs: Record<string, ToolLog[]>;
  setToolLogs: (convId: string, logs: ToolLog[]) => void;

  // ── Agent status ───────────────────────────────────────────────────────────
  agentStatus: AgentStatus;
  setAgentStatus: (s: AgentStatus) => void;
  currentIteration: number;
  setCurrentIteration: (n: number) => void;

  // ── Settings ───────────────────────────────────────────────────────────────
  settings: Settings | null;
  setSettings: (s: Settings) => void;

  // ── UI ─────────────────────────────────────────────────────────────────────
  sidebarOpen: boolean;
  toggleSidebar: () => void;
  agentPanelOpen: boolean;
  toggleAgentPanel: () => void;
  settingsOpen: boolean;
  setSettingsOpen: (v: boolean) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  // ── Conversations ──────────────────────────────────────────────────────────
  conversations: [],
  activeConvId: null,
  setConversations: (c) => set({ conversations: c }),
  setActiveConvId: (id) => set({ activeConvId: id }),
  addConversation: (c) =>
    set((s) => ({ conversations: [c, ...s.conversations] })),
  removeConversation: (id) =>
    set((s) => ({
      conversations: s.conversations.filter((c) => c.id !== id),
      activeConvId: s.activeConvId === id ? null : s.activeConvId,
    })),
  updateConversationTitle: (id, title) =>
    set((s) => ({
      conversations: s.conversations.map((c) =>
        c.id === id ? { ...c, title } : c
      ),
    })),

  // ── Messages ───────────────────────────────────────────────────────────────
  messages: {},
  setMessages: (convId, msgs) =>
    set((s) => ({ messages: { ...s.messages, [convId]: msgs } })),
  appendMessage: (msg) =>
    set((s) => {
      const existing = s.messages[msg.conversation_id] ?? [];
      return {
        messages: {
          ...s.messages,
          [msg.conversation_id]: [...existing, msg],
        },
      };
    }),

  // ── Streaming ──────────────────────────────────────────────────────────────
  streamBuffer: "",
  appendToken: (t) => set((s) => ({ streamBuffer: s.streamBuffer + t })),
  clearStreamBuffer: () => set({ streamBuffer: "" }),

  // ── Tool activity ──────────────────────────────────────────────────────────
  liveToolCalls: [],
  addLiveToolCall: (call) =>
    set((s) => ({ liveToolCalls: [...s.liveToolCalls, call] })),
  updateLiveToolCall: (id, updates) =>
    set((s) => ({
      liveToolCalls: s.liveToolCalls.map((c) =>
        c.id === id ? { ...c, ...updates } : c
      ),
    })),
  clearLiveToolCalls: () => set({ liveToolCalls: [] }),
  toolLogs: {},
  setToolLogs: (convId, logs) =>
    set((s) => ({ toolLogs: { ...s.toolLogs, [convId]: logs } })),

  // ── Agent status ───────────────────────────────────────────────────────────
  agentStatus: "idle",
  setAgentStatus: (s) => set({ agentStatus: s }),
  currentIteration: 0,
  setCurrentIteration: (n) => set({ currentIteration: n }),

  // ── Settings ───────────────────────────────────────────────────────────────
  settings: null,
  setSettings: (s) => set({ settings: s }),

  // ── UI ─────────────────────────────────────────────────────────────────────
  sidebarOpen: true,
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  agentPanelOpen: true,
  toggleAgentPanel: () => set((s) => ({ agentPanelOpen: !s.agentPanelOpen })),
  settingsOpen: false,
  setSettingsOpen: (v) => set({ settingsOpen: v }),
}));
