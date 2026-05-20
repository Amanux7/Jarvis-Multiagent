// src/components/layout/Sidebar.tsx
import { useAppStore } from "../../store/useAppStore";
import { useChat } from "../../hooks/useChat";
import styles from "./Sidebar.module.css";

export default function Sidebar() {
  const { conversations, activeConvId, sidebarOpen, setActiveConvId, settings } = useAppStore();
  const { createConversation, deleteConversation } = useChat();

  return (
    <aside className={`${styles.sidebar} ${sidebarOpen ? "" : styles.collapsed}`} id="sidebar">
      <div className={styles.header}>
        <button
          id="btn-new-conversation"
          className={styles.newBtn}
          onClick={createConversation}
        >
          <span className={styles.icon}>✦</span>
          New Conversation
        </button>
      </div>

      <div className={styles.list} id="conversation-list">
        {conversations.length > 0 && (
          <div className={styles.sectionLabel}>Recents</div>
        )}
        {conversations.length === 0 ? (
          <div className={styles.empty}>
            No conversations yet.<br />Start a new one above.
          </div>
        ) : (
          conversations.map((conv) => (
            <div
              key={conv.id}
              id={`conv-${conv.id}`}
              className={`${styles.item} ${activeConvId === conv.id ? styles.active : ""}`}
              onClick={() => setActiveConvId(conv.id)}
            >
              <span className={styles.itemIcon}>💬</span>
              <span className={styles.itemText} title={conv.title}>
                {conv.title}
              </span>
              <button
                className={styles.itemDelete}
                id={`btn-delete-conv-${conv.id}`}
                onClick={(e) => {
                  e.stopPropagation();
                  deleteConversation(conv.id);
                }}
                title="Delete conversation"
              >
                ✕
              </button>
            </div>
          ))
        )}
      </div>

      <div className={styles.footer}>
        <div className={styles.modelBadge}>
          <div className={styles.modelDot} />
          <span className={styles.modelText}>
            {settings ? `Ollama · ${settings.ollama_model}` : "Offline"}
          </span>
        </div>
      </div>
    </aside>
  );
}
