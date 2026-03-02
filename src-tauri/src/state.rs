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
    /// Hold key to record, release to stop.
    HoldToRecord,
    /// Press to start recording, press again to stop.
    Toggle,
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
            mode: Arc::new(Mutex::new(HotkeyMode::HoldToRecord)),
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
