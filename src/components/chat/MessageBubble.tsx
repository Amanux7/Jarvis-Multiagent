// src/components/chat/MessageBubble.tsx
import styles from "./MessageBubble.module.css";
import type { Message } from "../../types";

interface Props {
  message: Message;
  isStreaming?: boolean;
  streamContent?: string;
}

// Minimal markdown: code blocks, inline code, bold, newlines
function renderContent(text: string): React.ReactNode[] {
  const parts: React.ReactNode[] = [];
  const codeBlockRegex = /```[\w]*\n?([\s\S]*?)```/g;
  let lastIndex = 0;
  let match;

  while ((match = codeBlockRegex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push(
        <span key={lastIndex}>{renderInline(text.slice(lastIndex, match.index))}</span>
      );
    }
    parts.push(
      <pre key={match.index} className={styles.codeBlock}>
        <code>{match[1].trim()}</code>
      </pre>
    );
    lastIndex = match.index + match[0].length;
  }

  if (lastIndex < text.length) {
    parts.push(<span key={lastIndex}>{renderInline(text.slice(lastIndex))}</span>);
  }

  return parts;
}

function renderInline(text: string): React.ReactNode[] {
  // Bold + inline code + newlines
  return text.split('\n').flatMap((line, i, arr) => {
    const result: React.ReactNode[] = [];
    const inlineRegex = /(`[^`]+`|\*\*[^*]+\*\*)/g;
    let last = 0;
    let m;
    while ((m = inlineRegex.exec(line)) !== null) {
      if (m.index > last) result.push(line.slice(last, m.index));
      if (m[0].startsWith('`')) {
        result.push(<code key={m.index} className={styles.inlineCode}>{m[0].slice(1,-1)}</code>);
      } else {
        result.push(<strong key={m.index}>{m[0].slice(2,-2)}</strong>);
      }
      last = m.index + m[0].length;
    }
    if (last < line.length) result.push(line.slice(last));
    if (i < arr.length - 1) result.push(<br key={`br-${i}`} />);
    return result;
  });
}

export default function MessageBubble({ message, isStreaming, streamContent }: Props) {
  const isUser = message.role === "user";
  const isTool = message.role === "tool";
  const content = isStreaming && streamContent !== undefined ? streamContent : message.content;

  return (
    <div
      className={`${styles.wrapper} ${isUser ? styles.user : ""} ${isTool ? styles.tool : ""} animate-fade-in`}
      id={`msg-${message.id}`}
    >
      {!isUser && (
        <div className={styles.avatar}>
          {isTool ? "⚙" : "✦"}
        </div>
      )}

      <div className={`${styles.bubble} ${isUser ? styles.userBubble : isTool ? styles.toolBubble : styles.assistantBubble}`}>
        {isTool && (
          <div className={styles.toolLabel}>Tool Output</div>
        )}
        <div className={`${styles.content} ${isStreaming ? "cursor-blink" : ""}`}>
          {renderContent(content)}
        </div>
        <div className={styles.timestamp}>
          {new Date(message.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
        </div>
      </div>

      {isUser && (
        <div className={styles.avatar}>U</div>
      )}
    </div>
  );
}
