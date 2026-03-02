use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::Shortcut;
use tauri_plugin_store::StoreExt;

/// Application state machine for aurotype.
/// Tracks the current phase of the voice-to-text pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppState {
    Idle,
    Recording,
    Processing,
    Injecting,
    /// Injection failed — text available for manual copy.
    #[serde(rename = "copy_available")]
    CopyAvailable(String),
    Error(String),
}

impl AppState {
    /// Returns the state as a lowercase string identifier for frontend events.
    pub fn as_str(&self) -> &str {
        match self {
            AppState::Idle => "idle",
            AppState::Recording => "recording",
            AppState::Processing => "processing",
            AppState::Injecting => "injecting",
            AppState::CopyAvailable(_) => "copy_available",
            AppState::Error(_) => "error",
        }
    }

    /// Returns the optional message payload (error message or copy text).
    pub fn message(&self) -> Option<String> {
        match self {
            AppState::Error(msg) => Some(msg.clone()),
            AppState::CopyAvailable(text) => Some(text.clone()),
            _ => None,
        }
    }
}

/// A single transcription history record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionRecord {
    pub raw_text: String,
    pub polished_text: String,
    pub timestamp: String,
    #[serde(default)]
    pub audio_file: Option<String>,
}

/// Hotkey interaction mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HotkeyMode {
    /// Press to start recording, press again to stop.
    Toggle,
    /// Hold key to record, release to stop.
    HoldToRecord,
}

/// Managed state wrapper holding the app state and hotkey mode behind a mutex.
pub struct AppStateManager {
    pub state: Arc<Mutex<AppState>>,
    pub mode: Arc<Mutex<HotkeyMode>>,
    pub current_shortcut: Arc<Mutex<Option<Shortcut>>>,
    pub history: Arc<Mutex<Vec<TranscriptionRecord>>>,
    /// `true` after POST `/record/start` succeeds; `run_pipeline` waits on this.
    pub engine_recording: Arc<AtomicBool>,
}

impl AppStateManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AppState::Idle)),
            mode: Arc::new(Mutex::new(HotkeyMode::Toggle)),
            current_shortcut: Arc::new(Mutex::new(None)),
            history: Arc::new(Mutex::new(Vec::new())),
            engine_recording: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get a clone of the current state.
    pub fn get(&self) -> AppState {
        self.state.lock().unwrap().clone()
    }

    /// Add a transcription record to history and persist to disk.
    /// Deletes audio files for evicted records when exceeding max capacity.
    pub fn add_history(&self, record: TranscriptionRecord, app: &AppHandle) {
        {
            let mut history = self.history.lock().unwrap();
            history.push(record);
            // Keep max 50 records — delete audio files of evicted records
            while history.len() > 50 {
                let evicted = history.remove(0);
                Self::delete_audio_file(&evicted);
            }
        }
        self.persist_history(app);
    }

    /// Get all history records.
    pub fn get_history(&self) -> Vec<TranscriptionRecord> {
        self.history.lock().unwrap().clone()
    }

    /// Clear all history records, deleting associated audio files, and persist.
    pub fn clear_history(&self, app: &AppHandle) {
        {
            let mut history = self.history.lock().unwrap();
            for record in history.iter() {
                Self::delete_audio_file(record);
            }
            history.clear();
        }
        self.persist_history(app);
    }

    /// Load history from the on-disk store into memory.
    pub fn load_history_from_store(&self, app: &AppHandle) {
        match app.store("history.json") {
            Ok(store) => {
                if let Some(val) = store.get("records") {
                    match serde_json::from_value::<Vec<TranscriptionRecord>>(val.clone()) {
                        Ok(records) => {
                            let mut history = self.history.lock().unwrap();
                            *history = records;
                            eprintln!("[aurotype] Loaded {} history records from disk", history.len());
                        }
                        Err(e) => {
                            eprintln!("[aurotype] Failed to parse history: {e}");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[aurotype] Failed to open history store: {e}");
            }
        }
    }

    /// Persist current in-memory history to disk.
    fn persist_history(&self, app: &AppHandle) {
        let records = self.history.lock().unwrap().clone();
        match app.store("history.json") {
            Ok(store) => {
                let val = serde_json::to_value(&records).unwrap_or_default();
                store.set("records", val);
                if let Err(e) = store.save() {
                    eprintln!("[aurotype] Failed to save history to disk: {e}");
                }
            }
            Err(e) => {
                eprintln!("[aurotype] Failed to open history store for saving: {e}");
            }
        }
    }

    /// Delete the audio file associated with a transcription record, if any.
    fn delete_audio_file(record: &TranscriptionRecord) {
        if let Some(ref path) = record.audio_file {
            let p = std::path::Path::new(path);
            if p.exists() {
                if let Err(e) = std::fs::remove_file(p) {
                    eprintln!("[aurotype] Failed to delete audio file {path}: {e}");
                }
            }
        }
    }

    /// Transition to a new state and emit a `state-changed` event to the frontend.
    pub fn transition(&self, new_state: AppState, app: &AppHandle) {
        let state_str = new_state.as_str().to_string();
        let message = new_state.message();
        let should_show = !matches!(new_state, AppState::Idle);
        {
            let mut current = self.state.lock().unwrap();
            *current = new_state;
        }

        // Show/hide the float window from Rust to bypass JS background throttling.
        // The float window's JS may be suspended when hidden, so we drive visibility here.
        if let Some(float_win) = app.get_webview_window("float") {
            if should_show {
                // Position the float window at bottom-center of the primary monitor
                if let Ok(Some(monitor)) = app.primary_monitor() {
                    let monitor_size = monitor.size();
                    let scale = monitor.scale_factor();
                    let win_width = 320.0;
                    let win_height = 96.0;
                    let x = (monitor_size.width as f64 / scale - win_width) / 2.0;
                    let y = monitor_size.height as f64 / scale - win_height - 100.0;
                    let _ = float_win.set_size(tauri::Size::Logical(
                        tauri::LogicalSize::new(win_width, win_height),
                    ));
                    let _ = float_win.set_position(tauri::Position::Logical(
                        tauri::LogicalPosition::new(x, y),
                    ));
                }
                let _ = float_win.show();
                let _ = float_win.set_always_on_top(true);
            } else {
                let _ = float_win.hide();
            }
        }

        let payload = serde_json::json!({
            "state": state_str,
            "message": message,
        });
        let _ = app.emit("state-changed", payload);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(raw: &str, polished: &str, ts: &str) -> TranscriptionRecord {
        TranscriptionRecord {
            raw_text: raw.to_string(),
            polished_text: polished.to_string(),
            timestamp: ts.to_string(),
            audio_file: None,
        }
    }

    // --- AppState::as_str tests ---

    #[test]
    fn as_str_idle() {
        assert_eq!(AppState::Idle.as_str(), "idle");
    }

    #[test]
    fn as_str_recording() {
        assert_eq!(AppState::Recording.as_str(), "recording");
    }

    #[test]
    fn as_str_processing() {
        assert_eq!(AppState::Processing.as_str(), "processing");
    }

    #[test]
    fn as_str_injecting() {
        assert_eq!(AppState::Injecting.as_str(), "injecting");
    }

    #[test]
    fn as_str_copy_available() {
        assert_eq!(
            AppState::CopyAvailable("text".to_string()).as_str(),
            "copy_available"
        );
    }

    #[test]
    fn as_str_error() {
        assert_eq!(AppState::Error("oops".to_string()).as_str(), "error");
    }

    // --- AppState::message tests ---

    #[test]
    fn message_idle_returns_none() {
        assert_eq!(AppState::Idle.message(), None);
    }

    #[test]
    fn message_recording_returns_none() {
        assert_eq!(AppState::Recording.message(), None);
    }

    #[test]
    fn message_processing_returns_none() {
        assert_eq!(AppState::Processing.message(), None);
    }

    #[test]
    fn message_injecting_returns_none() {
        assert_eq!(AppState::Injecting.message(), None);
    }

    #[test]
    fn message_error_returns_some() {
        let msg = "something went wrong".to_string();
        assert_eq!(AppState::Error(msg.clone()).message(), Some(msg));
    }

    #[test]
    fn message_copy_available_returns_some() {
        let text = "copied text".to_string();
        assert_eq!(AppState::CopyAvailable(text.clone()).message(), Some(text));
    }

    // --- AppStateManager::new tests ---

    #[test]
    fn new_manager_initial_state_is_idle() {
        let manager = AppStateManager::new();
        assert_eq!(manager.get(), AppState::Idle);
    }

    #[test]
    fn new_manager_initial_mode_is_toggle() {
        let manager = AppStateManager::new();
        let mode = manager.mode.lock().unwrap().clone();
        assert_eq!(mode, HotkeyMode::Toggle);
    }

    #[test]
    fn new_manager_initial_shortcut_is_none() {
        let manager = AppStateManager::new();
        let shortcut = manager.current_shortcut.lock().unwrap().clone();
        assert!(shortcut.is_none());
    }

    #[test]
    fn new_manager_engine_recording_is_false() {
        let manager = AppStateManager::new();
        assert!(!manager.engine_recording.load(std::sync::atomic::Ordering::SeqCst));
    }

    // --- AppStateManager::get tests ---

    #[test]
    fn get_returns_cloned_state() {
        let manager = AppStateManager::new();
        {
            let mut state = manager.state.lock().unwrap();
            *state = AppState::Recording;
        }
        assert_eq!(manager.get(), AppState::Recording);
    }

    #[test]
    fn get_returns_error_state_with_message() {
        let manager = AppStateManager::new();
        {
            let mut state = manager.state.lock().unwrap();
            *state = AppState::Error("fail".to_string());
        }
        assert_eq!(manager.get(), AppState::Error("fail".to_string()));
    }

    // --- History (in-memory) tests ---

    #[test]
    fn get_history_initially_empty() {
        let manager = AppStateManager::new();
        assert!(manager.get_history().is_empty());
    }

    #[test]
    fn history_add_record_directly() {
        let manager = AppStateManager::new();
        let record = make_record("hello", "Hello.", "2025-01-01T00:00:00Z");
        {
            let mut history = manager.history.lock().unwrap();
            history.push(record);
        }
        let records = manager.get_history();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].raw_text, "hello");
        assert_eq!(records[0].polished_text, "Hello.");
        assert_eq!(records[0].timestamp, "2025-01-01T00:00:00Z");
        assert!(records[0].audio_file.is_none());
    }

    #[test]
    fn history_50_record_cap() {
        let manager = AppStateManager::new();
        {
            let mut history = manager.history.lock().unwrap();
            for i in 0..51 {
                history.push(make_record(
                    &format!("raw_{i}"),
                    &format!("polished_{i}"),
                    &format!("ts_{i}"),
                ));
            }
            // Simulate the same eviction logic as add_history
            while history.len() > 50 {
                history.remove(0);
            }
        }
        let records = manager.get_history();
        assert_eq!(records.len(), 50);
        // First record (raw_0) was evicted; oldest remaining is raw_1
        assert_eq!(records[0].raw_text, "raw_1");
        assert_eq!(records[49].raw_text, "raw_50");
    }

    #[test]
    fn history_clear_directly() {
        let manager = AppStateManager::new();
        {
            let mut history = manager.history.lock().unwrap();
            history.push(make_record("a", "A", "ts1"));
            history.push(make_record("b", "B", "ts2"));
        }
        assert_eq!(manager.get_history().len(), 2);
        {
            let mut history = manager.history.lock().unwrap();
            history.clear();
        }
        assert!(manager.get_history().is_empty());
    }

    #[test]
    fn history_multiple_records_order_preserved() {
        let manager = AppStateManager::new();
        {
            let mut history = manager.history.lock().unwrap();
            history.push(make_record("first", "First.", "ts1"));
            history.push(make_record("second", "Second.", "ts2"));
            history.push(make_record("third", "Third.", "ts3"));
        }
        let records = manager.get_history();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].raw_text, "first");
        assert_eq!(records[1].raw_text, "second");
        assert_eq!(records[2].raw_text, "third");
    }

    #[test]
    fn history_record_with_audio_file() {
        let manager = AppStateManager::new();
        {
            let mut history = manager.history.lock().unwrap();
            history.push(TranscriptionRecord {
                raw_text: "voice".to_string(),
                polished_text: "Voice.".to_string(),
                timestamp: "ts".to_string(),
                audio_file: Some("/tmp/audio.wav".to_string()),
            });
        }
        let records = manager.get_history();
        assert_eq!(records[0].audio_file, Some("/tmp/audio.wav".to_string()));
    }

    // --- HotkeyMode tests ---

    #[test]
    fn hotkey_mode_toggle_equality() {
        assert_eq!(HotkeyMode::Toggle, HotkeyMode::Toggle);
        assert_ne!(HotkeyMode::Toggle, HotkeyMode::HoldToRecord);
    }

    #[test]
    fn hotkey_mode_hold_to_record_equality() {
        assert_eq!(HotkeyMode::HoldToRecord, HotkeyMode::HoldToRecord);
        assert_ne!(HotkeyMode::HoldToRecord, HotkeyMode::Toggle);
    }

    // --- AppState equality tests ---

    #[test]
    fn app_state_equality() {
        assert_eq!(AppState::Idle, AppState::Idle);
        assert_ne!(AppState::Idle, AppState::Recording);
        assert_eq!(
            AppState::Error("x".to_string()),
            AppState::Error("x".to_string())
        );
        assert_ne!(
            AppState::Error("x".to_string()),
            AppState::Error("y".to_string())
        );
        assert_eq!(
            AppState::CopyAvailable("t".to_string()),
            AppState::CopyAvailable("t".to_string())
        );
        assert_ne!(
            AppState::CopyAvailable("a".to_string()),
            AppState::CopyAvailable("b".to_string())
        );
    }
}
