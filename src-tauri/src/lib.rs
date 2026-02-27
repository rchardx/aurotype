mod hotkey;
mod injection;
mod state;
mod tray;

use state::{AppState, AppStateManager};

// ── Tauri Commands ──────────────────────────────────────────────────

#[tauri::command]
fn get_state(state: tauri::State<AppStateManager>) -> String {
    state.get().as_str().to_string()
}

#[tauri::command]
fn start_recording(
    state: tauri::State<AppStateManager>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let current = state.get();
    if current != AppState::Idle {
        return Err(format!(
            "Cannot start recording from state: {}",
            current.as_str()
        ));
    }
    state.transition(AppState::Recording, &app);
    Ok(())
}

#[tauri::command]
fn stop_recording(
    state: tauri::State<AppStateManager>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let current = state.get();
    if current != AppState::Recording {
        return Err(format!(
            "Cannot stop recording from state: {}",
            current.as_str()
        ));
    }
    state.transition(AppState::Processing, &app);
    Ok(())
}

#[tauri::command]
fn cancel(state: tauri::State<AppStateManager>, app: tauri::AppHandle) -> Result<(), String> {
    let current = state.get();
    match current {
        AppState::Recording | AppState::Processing | AppState::Error(_) => {
            state.transition(AppState::Idle, &app);
            Ok(())
        }
        AppState::Idle => Ok(()), // already idle, no-op
        AppState::Injecting => Err("Cannot cancel during text injection".to_string()),
    }
}

// ── App Entry Point ─────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppStateManager::new())
        .invoke_handler(tauri::generate_handler![
            get_state,
            start_recording,
            stop_recording,
            cancel,
            injection::inject_text_at_cursor,
        ])
        .setup(|app| {
            #[cfg(desktop)]
            {
                let handle = app.handle();
                hotkey::register_hotkeys(handle)?;
                tray::create_tray(handle)?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
