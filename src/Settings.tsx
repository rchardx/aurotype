import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LazyStore } from "@tauri-apps/plugin-store";
import "./Settings.css";

// Create a store instance
const store = new LazyStore("settings.json");

interface SettingsData {
  stt_provider: string;
  stt_api_key: string;
  llm_provider: string;
  llm_api_key: string;
  llm_model: string;
  hotkey_mode: "hold" | "toggle";
  language: string;
}

const defaultSettings: SettingsData = {
  stt_provider: "groq",
  stt_api_key: "",
  llm_provider: "openai",
  llm_api_key: "",
  llm_model: "gpt-4o-mini",
  hotkey_mode: "hold",
  language: "auto",
};

export default function SettingsPage() {
  const [settings, setSettings] = useState<SettingsData>(defaultSettings);
  const [loading, setLoading] = useState(true);
  const [healthStatus, setHealthStatus] = useState<boolean | null>(null);
  const [testStatus, setTestStatus] = useState<string>("");

  useEffect(() => {
    loadSettings();
    checkHealth();
  }, []);

  const loadSettings = async () => {
    try {
      // Load individual keys or a single object. Using a single object "config" for simplicity
      const saved = await store.get<SettingsData>("config");
      if (saved) {
        setSettings({ ...defaultSettings, ...saved });
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
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  };

  const handleChange = (field: keyof SettingsData, value: string) => {
    saveSettings({ ...settings, [field]: value });
  };

  const checkHealth = async () => {
    try {
      const status = await invoke<boolean>("get_health");
      setHealthStatus(status);
    } catch (e) {
      console.error("Failed to check health:", e);
      setHealthStatus(false);
    }
  };

  const testConnection = async () => {
    setTestStatus("Testing...");
    try {
      // Stub implementation as requested
      await new Promise(resolve => setTimeout(resolve, 500));
      setTestStatus("Success!");
      setTimeout(() => setTestStatus(""), 2000);
    } catch (e) {
      setTestStatus("Failed");
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

  if (loading) return <div className="settings-container">Loading...</div>;

  return (
    <div className="settings-container">
      <h1>Settings</h1>

      {/* STT Provider Section */}
      <div className="section">
        <div className="section-header">STT Provider</div>
        <div className="form-group">
          <label>Provider</label>
          <select
            value={settings.stt_provider}
            onChange={(e) => handleChange("stt_provider", e.target.value)}
          >
            <option value="groq">Groq (Default)</option>
            <option value="siliconflow">SiliconFlow</option>
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
        <button onClick={testConnection} className="secondary">
          {testStatus || "Test Connection"}
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
            <option value="openai">OpenAI (Default)</option>
            <option value="siliconflow">SiliconFlow</option>
            <option value="none">None</option>
          </select>
        </div>

        {settings.llm_provider !== "none" && (
          <>
            <div className="form-group">
              <label>API Key</label>
              <input
                type="password"
                placeholder="Enter API Key"
                value={settings.llm_api_key}
                onChange={(e) => handleChange("llm_api_key", e.target.value)}
              />
            </div>
            <div className="form-group">
              <label>Model</label>
              <input
                type="text"
                value={settings.llm_model}
                onChange={(e) => handleChange("llm_model", e.target.value)}
              />
            </div>
          </>
        )}
      </div>

      {/* Hotkey Section */}
      <div className="section">
        <div className="section-header">Hotkey</div>
        <div className="form-group">
          <label>Current Hotkey</label>
          <div className="hotkey-display">Ctrl+Shift+Space</div>
          <p className="hint">To change hotkey, please restart the app.</p>
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

      {/* Advanced Section */}
      <div className="section">
        <div className="section-header">
          Advanced
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
        <button onClick={restartEngine} className="secondary">
          Restart Engine
        </button>
      </div>
    </div>
  );
}
