// src/components/chat/ChatView.tsx
import { useEffect, useRef } from "react";
import { useAppStore } from "../../store/useAppStore";
import { useChat } from "../../hooks/useChat";
import MessageBubble from "./MessageBubble";
import InputBar from "./InputBar";
import styles from "./ChatView.module.css";

export default function ChatView() {
  const { activeConvId, streamBuffer, agentStatus } = useAppStore();
  const { currentMessages, sendMessage, stopAgent } = useChat();
  const bottomRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom on new messages or streaming
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [currentMessages.length, streamBuffer]);

  if (!activeConvId) {
    return (
      <div className={styles.empty} id="chat-empty-state">
        <div className={styles.emptyGlow} />
        <div className={styles.emptyIcon}>✦</div>
        <h1 className={styles.emptyTitle}>JARvis</h1>
        <p className={styles.emptySubtitle}>
          Your local AI agent — powered by Ollama.<br />
          Start a new conversation to begin.
        </p>
        <div className={styles.emptyFeatures}>
          <div className={styles.feature}><span>📁</span> Read &amp; write files</div>
          <div className={styles.feature}><span>⚡</span> Run shell commands</div>
          <div className={styles.feature}><span>🔄</span> ReAct reasoning loop</div>
          <div className={styles.feature}><span>🔒</span> Fully offline</div>
        </div>
      </div>
    );
  }

  const isStreaming = agentStatus === "running" && streamBuffer.length > 0;

  return (
    <div className={styles.container} id="chat-view">
      <div className={styles.messages} id="message-list">
        {currentMessages.map((msg) => (
          <MessageBubble key={msg.id} message={msg} />
        ))}

        {/* Streaming assistant message */}
        {isStreaming && streamBuffer && (
          <MessageBubble
            key="streaming"
            message={{
              id: "streaming",
              conversation_id: activeConvId,
              role: "assistant",
              content: streamBuffer,
              created_at: new Date().toISOString(),
            }}
            isStreaming
            streamContent={streamBuffer}
          />
        )}

        {/* Thinking indicator (no tokens yet) */}
        {agentStatus === "running" && !streamBuffer && (
          <div className={styles.thinking} id="thinking-indicator">
            <div className={styles.thinkingAvatar}>✦</div>
            <div className={styles.thinkingDots}>
              <span /><span /><span />
            </div>
          </div>
        )}

        <div ref={bottomRef} />
      </div>

      <InputBar onSend={sendMessage} onStop={stopAgent} />
    </div>
  );
}
