use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

/// Application state machine for aurotype.
/// Tracks the current phase of the voice-to-text pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppState {
    Idle,
    Recording,
    Processing,
    Injecting,
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
            AppState::Error(_) => "error",
        }
    }

    /// Returns the optional error message, if in error state.
    pub fn message(&self) -> Option<String> {
        match self {
            AppState::Error(msg) => Some(msg.clone()),
            _ => None,
        }
    }
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
}

impl AppStateManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AppState::Idle)),
            mode: Arc::new(Mutex::new(HotkeyMode::Toggle)),
        }
    }

    /// Get a clone of the current state.
    pub fn get(&self) -> AppState {
        self.state.lock().unwrap().clone()
    }

    /// Transition to a new state and emit a `state-changed` event to the frontend.
    pub fn transition(&self, new_state: AppState, app: &AppHandle) {
        let state_str = new_state.as_str().to_string();
        let message = new_state.message();
        {
            let mut current = self.state.lock().unwrap();
            *current = new_state;
        }
        let payload = serde_json::json!({
            "state": state_str,
            "message": message,
        });
        let _ = app.emit("state-changed", payload);
    }
}
