mod hotkey;
mod injection;
mod sidecar;
mod state;
mod tray;

use state::{AppState, AppStateManager};
use tauri::Manager;

#[tauri::command]
fn get_state(state: tauri::State<AppStateManager>) -> String {
    state.get().as_str().to_string()
}

#[tauri::command]
async fn start_recording(
    state: tauri::State<'_, AppStateManager>,
    app: tauri::AppHandle,
    sidecar: tauri::State<'_, sidecar::SidecarState>,
) -> Result<(), String> {
    let current = state.get();
    if current != AppState::Idle {
        return Err(format!(
            "Cannot start recording from state: {}",
            current.as_str()
        ));
    }

    sidecar::sidecar_post(&sidecar, "/record/start", serde_json::json!({})).await?;
    state.transition(AppState::Recording, &app);
    Ok(())
}

#[tauri::command]
async fn stop_recording(
    state: tauri::State<'_, AppStateManager>,
    app: tauri::AppHandle,
    sidecar: tauri::State<'_, sidecar::SidecarState>,
) -> Result<String, String> {
    let current = state.get();
    if current != AppState::Recording {
        return Err(format!(
            "Cannot stop recording from state: {}",
            current.as_str()
        ));
    }

    let response = sidecar::sidecar_post(&sidecar, "/record/stop", serde_json::json!({})).await?;
    state.transition(AppState::Processing, &app);
    Ok(response)
}

#[tauri::command]
fn cancel(state: tauri::State<AppStateManager>, app: tauri::AppHandle) -> Result<(), String> {
    let current = state.get();
    match current {
        AppState::Recording | AppState::Processing | AppState::Error(_) => {
            state.transition(AppState::Idle, &app);
            Ok(())
        }
        AppState::Idle => Ok(()),
        AppState::Injecting => Err("Cannot cancel during text injection".to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppStateManager::new())
        .invoke_handler(tauri::generate_handler![
            get_state,
            start_recording,
            stop_recording,
            cancel,
            sidecar::get_health,
            injection::inject_text_at_cursor,
        ])
        .setup(|app| {
            let sidecar_state = sidecar::spawn_sidecar(app.handle().clone())
                .map_err(|err| std::io::Error::other(format!("failed to spawn sidecar: {err}")))?;
            app.manage(sidecar_state);

            #[cfg(desktop)]
            {
                let handle = app.handle();
                hotkey::register_hotkeys(handle)?;
                tray::create_tray(handle)?;
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|app, event| {
        if matches!(event, tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit) {
            if let Some(sidecar_state) = app.try_state::<sidecar::SidecarState>() {
                sidecar::shutdown_sidecar(&sidecar_state);
            }
        }
    });
}
