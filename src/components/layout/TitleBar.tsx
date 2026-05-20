// src/components/layout/TitleBar.tsx
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppStore } from "../../store/useAppStore";
import styles from "./TitleBar.module.css";

export default function TitleBar() {
  const { agentStatus, toggleSidebar, toggleAgentPanel, setSettingsOpen } = useAppStore();

  const win = getCurrentWindow();

  const handleMinimize = () => win.minimize();
  const handleMaximize = () => win.toggleMaximize();
  const handleClose    = () => win.close();

  const statusLabel =
    agentStatus === "running" ? "Agent running…" :
    agentStatus === "error"   ? "Agent error"    :
                                "Ready";

  return (
    <header className={styles.titlebar} id="titlebar">
      {/* Drag region covers the whole bar */}
      <div className={styles.dragRegion} data-tauri-drag-region />

      {/* Left — Logo + sidebar toggle */}
      <div className={styles.left}>
        <button
          id="btn-toggle-sidebar"
          className={styles.winBtn}
          onClick={toggleSidebar}
          title="Toggle Sidebar"
        >
          ☰
        </button>
        <div className={styles.logo}>
          <div className={styles.logoIcon}>J</div>
          <span className={styles.logoText}>JARvis</span>
        </div>
      </div>

      {/* Center — Agent status */}
      <div className={styles.center}>
        <div className={styles.status}>
          <div className={`${styles.statusDot} ${styles[agentStatus]}`} />
          <span>{statusLabel}</span>
        </div>
      </div>

      {/* Right — Panel toggles + window controls */}
      <div className={styles.right}>
        <button
          id="btn-toggle-agent-panel"
          className={styles.winBtn}
          onClick={toggleAgentPanel}
          title="Toggle Tool Panel"
        >
          ⚡
        </button>
        <button
          id="btn-open-settings"
          className={styles.winBtn}
          onClick={() => setSettingsOpen(true)}
          title="Settings"
        >
          ⚙
        </button>
        <button id="btn-minimize" className={styles.winBtn} onClick={handleMinimize} title="Minimize">─</button>
        <button id="btn-maximize" className={styles.winBtn} onClick={handleMaximize} title="Maximize">▭</button>
        <button id="btn-close" className={`${styles.winBtn} ${styles.close}`} onClick={handleClose} title="Close">✕</button>
      </div>
    </header>
  );
}
