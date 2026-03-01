import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LazyStore } from "@tauri-apps/plugin-store";
import "./Settings.css";

// Create a store instance
const store = new LazyStore("settings.json");

interface SettingsData {
  stt_provider: string;
  stt_api_key: string;
  stt_model: string;
  llm_provider: string;
  llm_api_key: string;
  llm_model: string;
  llm_base_url: string;
  hotkey: string;
  hotkey_mode: "hold" | "toggle";
  language: string;
  system_prompt: string;
}
interface TranscriptionRecord {
  raw_text: string;
  polished_text: string;
  timestamp: string;
  audio_file?: string;
}


const defaultSettings: SettingsData = {
  stt_provider: "dashscope",
  stt_api_key: "",
  stt_model: "paraformer-realtime-v2",
  llm_provider: "deepseek",
  llm_api_key: "",
  llm_model: "deepseek-chat",
  llm_base_url: "",
  hotkey: "Ctrl+Alt+Space",
  hotkey_mode: "hold",
  language: "auto",
  system_prompt: "",
};

const DEFAULT_SYSTEM_PROMPT = "You are a text polisher for voice transcription. Clean up the following speech transcription: remove filler words (um, uh, like, you know), fix grammar and punctuation, preserve meaning and tone. IMPORTANT: If the speaker mixes languages (e.g. Chinese with English words/phrases), keep the original language for each part as spoken. Do NOT translate English words into Chinese or vice versa. Preserve code terms, brand names, and technical jargon in their original language. Return ONLY the polished text, no explanations.";

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await invoke("copy_to_clipboard", { text });
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      console.error("Failed to copy:", e);
    }
  };

  return (
    <button 
      className={`history-copy-btn ${copied ? "copied" : ""}`}
      onClick={handleCopy}
    >
      {copied ? "Copied!" : "Copy"}
    </button>
  );
}

function PlayButton({ audioFile }: { audioFile: string }) {
  const [playing, setPlaying] = useState(false);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  const handlePlay = async () => {
    if (playing && audioRef.current) {
      audioRef.current.pause();
      audioRef.current.currentTime = 0;
      setPlaying(false);
      return;
    }
    try {
      const dataUrl = await invoke<string>("get_audio_data", { path: audioFile });
      const audio = new Audio(dataUrl);
      audioRef.current = audio;
      audio.onended = () => setPlaying(false);
      audio.onerror = () => setPlaying(false);
      setPlaying(true);
      await audio.play();
    } catch (e) {
      console.error("Failed to play audio:", e);
      setPlaying(false);
    }
  };

  return (
    <button
      className={`history-play-btn ${playing ? "playing" : ""}`}
      onClick={handlePlay}
      title={playing ? "Stop" : "Play recording"}
    >
      {playing ? "\u25A0" : "\u25B6"}
    </button>
  );
}

export default function SettingsPage() {
  const [settings, setSettings] = useState<SettingsData>(defaultSettings);
  const [loading, setLoading] = useState(true);
  const [healthStatus, setHealthStatus] = useState<boolean | null>(null);
  const [testLlmStatus, setTestLlmStatus] = useState<string>("");
  const [testSttStatus, setTestSttStatus] = useState<string>("");
  const [history, setHistory] = useState<TranscriptionRecord[]>([]);
  const [promptDraft, setPromptDraft] = useState("");
  const [promptSaved, setPromptSaved] = useState(false);
  useEffect(() => {
    loadSettings();
    loadHistory();
    checkHealth();
    // Poll health every 5 seconds
    const healthInterval = setInterval(() => {
      checkHealth();
      loadHistory();
    }, 5000);
    return () => clearInterval(healthInterval);
  }, []);

  const loadSettings = async () => {
    try {
      // Load individual keys or a single object. Using a single object "config" for simplicity
      const saved = await store.get<SettingsData>("config");
      if (saved) {
        // Migrate deprecated Alt+Space hotkey (Windows-reserved)
        const migrated = { ...defaultSettings, ...saved };
        if (migrated.hotkey === "Alt+Space") {
          migrated.hotkey = "Ctrl+Alt+Space";
        }
        setSettings(migrated);
        setPromptDraft(migrated.system_prompt || DEFAULT_SYSTEM_PROMPT);
        // Persist migrated settings if hotkey changed
        if (saved.hotkey === "Alt+Space") {
          await store.set("config", migrated);
          await store.save();
        }
      } else {
        setPromptDraft(DEFAULT_SYSTEM_PROMPT);
      }
    } catch (e) {
      console.error("Failed to load settings:", e);
    } finally {
      setLoading(false);
    }
  };

  const saveSettings = async (newSettings: SettingsData) => {
    setSettings(newSettings);
    try {
      await store.set("config", newSettings);
      await store.save();
      try {
        await invoke("sync_settings");
      } catch (e) {
        console.error("Failed to sync settings to sidecar:", e);
      }
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  };

  const handleChange = (field: keyof SettingsData, value: string) => {
    saveSettings({ ...settings, [field]: value });
  };

  const handleHotkeyChange = async (newHotkey: string) => {
    try {
      await invoke("update_hotkey", { shortcut: newHotkey });
      saveSettings({ ...settings, hotkey: newHotkey });
    } catch (e) {
      console.error("Failed to update hotkey:", e);
    }
  };

  const checkHealth = async () => {
    try {
      const raw = await invoke<string>("get_health");
      const parsed = JSON.parse(raw);
      setHealthStatus(parsed.status === "ok");
    } catch (e) {
      console.error("Failed to check health:", e);
      setHealthStatus(false);
    }
  };

  const testLlmConnection = async () => {
    setTestLlmStatus("Testing...");
    try {
      await invoke<string>("test_llm");
      setTestLlmStatus("Success!");
      setTimeout(() => setTestLlmStatus(""), 2000);
    } catch (e) {
      setTestLlmStatus("Failed: " + (e instanceof Error ? e.message : String(e)));
      setTimeout(() => setTestLlmStatus(""), 3000);
    }
  };

  const testSttConnection = async () => {
    setTestSttStatus("Testing...");
    try {
      await invoke<string>("test_stt");
      setTestSttStatus("Success!");
      setTimeout(() => setTestSttStatus(""), 2000);
    } catch (e) {
      setTestSttStatus("Failed: " + (e instanceof Error ? e.message : String(e)));
      setTimeout(() => setTestSttStatus(""), 3000);
    }
  };

  const restartEngine = async () => {
    try {
      await invoke("cancel");
      alert("Engine restart signal sent.");
    } catch (e) {
      console.error("Failed to restart engine:", e);
    }
  };
  const loadHistory = async () => {
    try {
      const records = await invoke<TranscriptionRecord[]>("get_history");
      setHistory(records.reverse());
    } catch (e) {
      console.error("Failed to load history:", e);
    }
  };



  if (loading) return <div className="settings-container">Loading...</div>;

  return (
    <div className="settings-container">
      <h1><img src="/logo.png" alt="Aurotype" className="brand-logo" />Aurotype</h1>

      {/* Engine Status Section (first) */}
      <div className="section">
        <div className="section-header">
          Engine Status
          <div style={{ display: 'flex', alignItems: 'center', fontSize: '14px' }}>
            <span
              className={`status-indicator ${
                healthStatus ? "status-connected" : "status-disconnected"
              }`}
            />
            {healthStatus ? "Connected" : "Disconnected"}
          </div>
        </div>
        <div className="form-group">
          <label>Sidecar Status</label>
          <p className="hint">
            Check if the background engine is running correctly.
          </p>
        </div>
        <button onClick={restartEngine} className="secondary" disabled={!healthStatus}>
          Restart Engine
        </button>
      </div>

      {/* History Section (second) */}
      <div className="section">
        <div className="section-header">
          <span>
            History
            <span style={{ fontSize: '11px', color: '#888', fontWeight: 400, marginLeft: '8px' }}>
              (keeps last 50 records)
            </span>
          </span>
          <span style={{ display: 'flex', gap: '4px' }}>
            <button 
              className="secondary" 
              style={{ fontSize: '12px', padding: '4px 8px' }}
              onClick={loadHistory}
            >
              Refresh
            </button>
            {history.length > 0 && (
              <button 
                className="secondary" 
                style={{ fontSize: '12px', padding: '4px 8px', color: '#e74c3c' }}
                onClick={async () => {
                  try {
                    await invoke('clear_history');
                    setHistory([]);
                  } catch (e) {
                    console.error('Failed to clear history:', e);
                  }
                }}
              >
                Clear
              </button>
            )}
          </span>
        </div>
        
        {history.length === 0 ? (
          <div className="history-empty">No recordings yet</div>
        ) : (
          <div className="history-list">
            {history.map((record, index) => (
              <div key={index} className="history-item">
                <div className="history-header">
                  <span className="history-timestamp">{record.timestamp}</span>
                  <span className="history-actions">
                    {record.audio_file && <PlayButton audioFile={record.audio_file} />}
                    <CopyButton text={record.polished_text} />
                  </span>
                </div>
                <div className="history-polished">{record.polished_text}</div>
                <div className="history-raw">{record.raw_text}</div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* STT Provider Section */}
      <div className="section">
        <div className="section-header">STT Provider</div>
        <div className="form-group">
          <label>Provider</label>
          <select
            value={settings.stt_provider}
            onChange={(e) => handleChange("stt_provider", e.target.value)}
          >
            <option value="dashscope">DashScope / Paraformer (Default)</option>
          </select>
        </div>
        <div className="form-group">
          <label>API Key</label>
          <input
            type="password"
            placeholder="Enter API Key"
            value={settings.stt_api_key}
            onChange={(e) => handleChange("stt_api_key", e.target.value)}
          />
        </div>
        <div className="form-group">
          <label>Model</label>
          <input
            type="text"
            placeholder="paraformer-realtime-v2"
            value={settings.stt_model}
            onChange={(e) => handleChange("stt_model", e.target.value)}
          />
        </div>
        <button onClick={testSttConnection} className="secondary" disabled={!healthStatus}>
          {testSttStatus || "Test Connection"}
        </button>
      </div>

      {/* LLM Provider Section */}
      <div className="section">
        <div className="section-header">LLM Provider</div>
        <div className="form-group">
          <label>Provider</label>
          <select
            value={settings.llm_provider}
            onChange={(e) => handleChange("llm_provider", e.target.value)}
          >
            <option value="deepseek">DeepSeek (Default)</option>
            <option value="openai">OpenAI Compatible</option>
          </select>
        </div>

            <div className="form-group">
              <label>API Key</label>
              <input
                type="password"
                placeholder="Enter API Key"
                value={settings.llm_api_key}
                onChange={(e) => handleChange("llm_api_key", e.target.value)}
              />
            </div>
            {settings.llm_provider === "openai" && (
              <div className="form-group">
                <label>Base URL</label>
                <input
                  type="text"
                  placeholder="https://api.openai.com/v1"
                  value={settings.llm_base_url}
                  onChange={(e) => handleChange("llm_base_url", e.target.value)}
                />
              </div>
            )}
            <div className="form-group">
              <label>Model</label>
              <input
                type="text"
                value={settings.llm_model}
                onChange={(e) => handleChange("llm_model", e.target.value)}
              />
            </div>
            <button onClick={testLlmConnection} className="secondary" disabled={!healthStatus}>
              {testLlmStatus || "Test Connection"}
            </button>
            <div className="form-group" style={{ marginTop: '12px' }}>
              <label>System Prompt</label>
              <p className="hint">
                Customize how the LLM polishes your transcription. Edit the prompt below and click Save.
              </p>
              <textarea
                rows={5}
                value={promptDraft}
                onChange={(e) => {
                  setPromptDraft(e.target.value);
                  setPromptSaved(false);
                }}
                style={{ width: '100%', resize: 'vertical', fontFamily: 'inherit', fontSize: '13px' }}
              />
              <div style={{ display: 'flex', gap: '8px', marginTop: '8px', alignItems: 'center' }}>
                <button
                  className="secondary"
                  onClick={() => {
                    const valueToSave = promptDraft.trim() === DEFAULT_SYSTEM_PROMPT.trim() ? "" : promptDraft;
                    handleChange("system_prompt", valueToSave);
                    setPromptSaved(true);
                    setTimeout(() => setPromptSaved(false), 2000);
                  }}
                >
                  {promptSaved ? "Saved!" : "Save"}
                </button>
                <button
                  className="secondary"
                  style={{ color: '#888' }}
                  onClick={() => {
                    setPromptDraft(DEFAULT_SYSTEM_PROMPT);
                    setPromptSaved(false);
                  }}
                >
                  Reset to Default
                </button>
              </div>
            </div>
      </div>

      {/* Hotkey Section */}
      <div className="section">
        <div className="section-header">Hotkey</div>
        <div className="form-group">
          <label>Shortcut</label>
          <select
            value={settings.hotkey}
            onChange={(e) => handleHotkeyChange(e.target.value)}
          >
            <option value="Ctrl+Alt+Space">Ctrl+Alt+Space (Default)</option>
            <option value="CmdOrCtrl+Shift+Space">Ctrl+Shift+Space</option>
            <option value="CmdOrCtrl+Shift+A">Ctrl+Shift+A</option>
            <option value="CmdOrCtrl+Shift+R">Ctrl+Shift+R</option>
            <option value="CmdOrCtrl+Shift+V">Ctrl+Shift+V</option>
            <option value="CmdOrCtrl+Space">Ctrl+Space</option>
            <option value="F9">F9</option>
            <option value="F10">F10</option>
          </select>
        </div>
        <div className="form-group">
          <label>Mode</label>
          <div className="radio-group">
            <label className="radio-option">
              <input
                type="radio"
                name="hotkey_mode"
                value="hold"
                checked={settings.hotkey_mode === "hold"}
                onChange={() => handleChange("hotkey_mode", "hold")}
              />
              Hold to Record
            </label>
            <label className="radio-option">
              <input
                type="radio"
                name="hotkey_mode"
                value="toggle"
                checked={settings.hotkey_mode === "toggle"}
                onChange={() => handleChange("hotkey_mode", "toggle")}
              />
              Toggle
            </label>
          </div>
        </div>
      </div>

      {/* Language Section */}
      <div className="section">
        <div className="section-header">Language</div>
        <div className="form-group">
          <label>Interface Language</label>
          <select
            value={settings.language}
            onChange={(e) => handleChange("language", e.target.value)}
          >
            <option value="auto">Auto</option>
            <option value="en">English</option>
            <option value="zh">Chinese</option>
            <option value="ja">Japanese</option>
            <option value="ko">Korean</option>
            <option value="es">Spanish</option>
            <option value="fr">French</option>
            <option value="de">German</option>
          </select>
        </div>
      </div>
    </div>
  );
}
