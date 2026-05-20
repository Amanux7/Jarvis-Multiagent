// src/components/agent/AgentPanel.tsx
import { useAppStore } from "../../store/useAppStore";
import styles from "./AgentPanel.module.css";
import type { LiveToolCall } from "../../types";

function ToolEntry({ call }: { call: LiveToolCall }) {
  const statusIcon =
    call.status === "running" ? "⟳" :
    call.status === "done"    ? "✓" :
                                "✗";

  const argsStr = JSON.stringify(call.args, null, 2);

  return (
    <div className={`${styles.toolEntry} ${styles[call.status]}`} id={`tool-${call.id}`}>
      <div className={styles.toolHeader}>
        <span className={`${styles.toolStatus} ${call.status === "running" ? "animate-spin" : ""}`}>
          {statusIcon}
        </span>
        <span className={styles.toolName}>{call.tool}</span>
        <span className={`${styles.toolBadge} ${call.success === false ? styles.errorBadge : ""}`}>
          {call.status}
        </span>
      </div>

      <details className={styles.details}>
        <summary className={styles.summary}>Args</summary>
        <pre className={styles.pre}>{argsStr}</pre>
      </details>

      {call.output && (
        <details className={styles.details} open={call.status !== "running"}>
          <summary className={styles.summary}>Output</summary>
          <pre className={styles.pre}>{call.output}</pre>
        </details>
      )}
    </div>
  );
}

export default function AgentPanel() {
  const { agentPanelOpen, liveToolCalls, agentStatus, currentIteration } = useAppStore();

  return (
    <aside
      className={`${styles.panel} ${agentPanelOpen ? "" : styles.collapsed}`}
      id="agent-panel"
    >
      <div className={styles.header}>
        <span className={styles.title}>⚡ Tool Activity</span>
        {agentStatus === "running" && (
          <span className={styles.iter}>
            iter {currentIteration}
          </span>
        )}
      </div>

      <div className={styles.content}>
        {liveToolCalls.length === 0 ? (
          <div className={styles.empty}>
            {agentStatus === "running"
              ? "Agent reasoning…"
              : "No tool calls yet.\nTools will appear here when the agent runs."}
          </div>
        ) : (
          <div className={styles.list}>
            {liveToolCalls.map((call) => (
              <ToolEntry key={call.id} call={call} />
            ))}
          </div>
        )}
      </div>

      <div className={styles.footer}>
        <div className={styles.legend}>
          <span className={styles.legendItem}>
            <span className={styles.dot} style={{ background: "var(--accent-secondary)" }} />
            done
          </span>
          <span className={styles.legendItem}>
            <span className={styles.dot} style={{ background: "var(--accent-primary)" }} />
            running
          </span>
          <span className={styles.legendItem}>
            <span className={styles.dot} style={{ background: "var(--accent-danger)" }} />
            error
          </span>
        </div>
      </div>
    </aside>
  );
}
