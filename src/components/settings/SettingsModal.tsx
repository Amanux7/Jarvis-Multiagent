// src/components/settings/SettingsModal.tsx
import { useState, useEffect } from "react";
import { useAppStore } from "../../store/useAppStore";
import * as ipc from "../../lib/ipc";
import type { Settings } from "../../types";
import styles from "./SettingsModal.module.css";

export default function SettingsModal() {
  const { settingsOpen, setSettingsOpen, settings, setSettings } = useAppStore();
  const [form, setForm] = useState<Settings | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [newAllowCmd, setNewAllowCmd] = useState("");

  useEffect(() => {
    if (settingsOpen && settings) setForm({ ...settings });
  }, [settingsOpen, settings]);

  if (!settingsOpen || !form) return null;

  const handleSave = async () => {
    if (!form) return;
    setSaving(true);
    try {
      await ipc.saveSettings(form);
      setSettings(form);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("Failed to save settings:", e);
    } finally {
      setSaving(false);
    }
  };

  const addAllowCmd = () => {
    const cmd = newAllowCmd.trim();
    if (!cmd || form.shell_allowlist.includes(cmd)) return;
    setForm({ ...form, shell_allowlist: [...form.shell_allowlist, cmd] });
    setNewAllowCmd("");
  };

  const removeAllowCmd = (cmd: string) => {
    setForm({ ...form, shell_allowlist: form.shell_allowlist.filter((c) => c !== cmd) });
  };

  return (
    <div className={styles.overlay} id="settings-overlay" onClick={() => setSettingsOpen(false)}>
      <div className={styles.modal} id="settings-modal" onClick={(e) => e.stopPropagation()}>
        <div className={styles.header}>
          <h2 className={styles.title}>Settings</h2>
          <button
            id="btn-close-settings"
            className={styles.close}
            onClick={() => setSettingsOpen(false)}
          >✕</button>
        </div>

        <div className={styles.body}>
          {/* Ollama Config */}
          <section className={styles.section}>
            <h3 className={styles.sectionTitle}>Ollama Configuration</h3>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="ollama-url">Base URL</label>
              <input
                id="ollama-url"
                className={styles.input}
                type="text"
                value={form.ollama_base_url}
                onChange={(e) => setForm({ ...form, ollama_base_url: e.target.value })}
                placeholder="http://localhost:11434"
              />
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="ollama-model">Model</label>
              <input
                id="ollama-model"
                className={styles.input}
                type="text"
                value={form.ollama_model}
                onChange={(e) => setForm({ ...form, ollama_model: e.target.value })}
                placeholder="llama3"
              />
              <span className={styles.hint}>Run `ollama list` to see available models</span>
            </div>
          </section>

          {/* Agent Config */}
          <section className={styles.section}>
            <h3 className={styles.sectionTitle}>Agent Behaviour</h3>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="max-iter">Max ReAct Iterations</label>
              <input
                id="max-iter"
                className={styles.input}
                type="number"
                min={1}
                max={50}
                value={form.max_iterations}
                onChange={(e) => setForm({ ...form, max_iterations: parseInt(e.target.value) || 10 })}
              />
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="system-prompt">System Prompt</label>
              <textarea
                id="system-prompt"
                className={`${styles.input} ${styles.textarea}`}
                value={form.system_prompt}
                onChange={(e) => setForm({ ...form, system_prompt: e.target.value })}
                rows={4}
              />
            </div>
          </section>

          {/* Shell Allowlist */}
          <section className={styles.section}>
            <h3 className={styles.sectionTitle}>Shell Command Allowlist</h3>
            <p className={styles.sectionDesc}>
              Only commands in this list can be run by the agent via the <code>run_command</code> tool.
            </p>

            <div className={styles.allowlist}>
              {form.shell_allowlist.map((cmd) => (
                <div key={cmd} className={styles.allowlistItem}>
                  <span className={styles.allowlistCmd}>{cmd}</span>
                  <button
                    className={styles.allowlistRemove}
                    onClick={() => removeAllowCmd(cmd)}
                    title={`Remove ${cmd}`}
                  >✕</button>
                </div>
              ))}
            </div>

            <div className={styles.addRow}>
              <input
                id="add-allowlist-cmd"
                className={styles.input}
                type="text"
                value={newAllowCmd}
                onChange={(e) => setNewAllowCmd(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && addAllowCmd()}
                placeholder="e.g. npm"
              />
              <button
                id="btn-add-allowlist"
                className={styles.addBtn}
                onClick={addAllowCmd}
              >Add</button>
            </div>
          </section>
        </div>

        <div className={styles.footer}>
          <button
            id="btn-cancel-settings"
            className={styles.cancelBtn}
            onClick={() => setSettingsOpen(false)}
          >Cancel</button>
          <button
            id="btn-save-settings"
            className={`${styles.saveBtn} ${saved ? styles.savedBtn : ""}`}
            onClick={handleSave}
            disabled={saving}
          >
            {saving ? "Saving…" : saved ? "✓ Saved" : "Save Settings"}
          </button>
        </div>
      </div>
    </div>
  );
}
