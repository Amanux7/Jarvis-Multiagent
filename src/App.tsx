// src/App.tsx
import { useEffect } from "react";
import { useAppStore } from "./store/useAppStore";
import * as ipc from "./lib/ipc";

import TitleBar from "./components/layout/TitleBar";
import Sidebar from "./components/layout/Sidebar";
import ChatView from "./components/chat/ChatView";
import AgentPanel from "./components/agent/AgentPanel";
import SettingsModal from "./components/settings/SettingsModal";

import styles from "./App.module.css";

export default function App() {
  const { setSettings } = useAppStore();

  // Load settings once on startup
  useEffect(() => {
    ipc.getSettings()
      .then(setSettings)
      .catch((e) => console.error("Failed to load settings:", e));
  }, []);

  return (
    <div className={styles.app} id="app-root">
      <TitleBar />

      <div className={styles.body}>
        <Sidebar />

        <main className={styles.main} id="main-content">
          <ChatView />
        </main>

        <AgentPanel />
      </div>

      <SettingsModal />
    </div>
  );
}
