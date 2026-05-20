// src/components/chat/InputBar.tsx
import { useState, useRef, useCallback, KeyboardEvent } from "react";
import { useAppStore } from "../../store/useAppStore";
import styles from "./InputBar.module.css";

interface Props {
  onSend: (content: string) => void;
  onStop: () => void;
}

export default function InputBar({ onSend, onStop }: Props) {
  const [value, setValue] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const { agentStatus, activeConvId } = useAppStore();
  const isRunning = agentStatus === "running";
  const disabled = !activeConvId;

  const handleSend = useCallback(() => {
    const text = value.trim();
    if (!text || isRunning || disabled) return;
    onSend(text);
    setValue("");
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }
  }, [value, isRunning, disabled, onSend]);

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleInput = () => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, 200)}px`;
  };

  return (
    <div className={`${styles.container} ${disabled ? styles.disabled : ""}`} id="input-bar">
      <div className={styles.inner}>
        <textarea
          ref={textareaRef}
          id="chat-input"
          className={styles.textarea}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={handleKeyDown}
          onInput={handleInput}
          placeholder={
            disabled
              ? "Create or select a conversation…"
              : isRunning
              ? "Agent is thinking…"
              : "Ask JARvis anything… (Enter to send, Shift+Enter for new line)"
          }
          disabled={disabled || isRunning}
          rows={1}
        />

        <div className={styles.actions}>
          {isRunning ? (
            <button
              id="btn-stop-agent"
              className={`${styles.btn} ${styles.stop}`}
              onClick={onStop}
              title="Stop agent"
            >
              <span className={styles.stopIcon}>■</span>
              Stop
            </button>
          ) : (
            <button
              id="btn-send"
              className={`${styles.btn} ${styles.send}`}
              onClick={handleSend}
              disabled={!value.trim() || disabled}
              title="Send message (Enter)"
            >
              <span>▲</span>
            </button>
          )}
        </div>
      </div>

      <div className={styles.hint}>
        {isRunning ? (
          <span className={styles.runningHint}>
            <span className="animate-pulse">●</span> ReAct agent loop running…
          </span>
        ) : (
          <span>Enter ↵ to send · Shift+Enter for new line · Powered by Ollama</span>
        )}
      </div>
    </div>
  );
}
