mod hotkey;
mod injection;
mod sidecar;
mod state;
mod tray;

use state::{AppState, AppStateManager};
use tauri::Manager;
use tauri_plugin_store::StoreExt;

pub async fn run_pipeline(app: tauri::AppHandle) {
    use std::time::Duration;
    use tokio::time::timeout;

    let sidecar = app.state::<sidecar::SidecarState>();
    let state_mgr = app.state::<AppStateManager>();

    let result = timeout(
        Duration::from_secs(10),
        sidecar::sidecar_post(&sidecar, "/record/stop", serde_json::json!({})),
    )
    .await;

    match result {
        Err(_elapsed) => {
            eprintln!("[aurotype] Pipeline timeout: /record/stop exceeded 10s");
            state_mgr.transition(AppState::Error("Request timed out".to_string()), &app);
            tokio::time::sleep(Duration::from_secs(3)).await;
            state_mgr.transition(AppState::Idle, &app);
        }
        Ok(Err(e)) => {
            eprintln!("[aurotype] Pipeline error: {e}");
            state_mgr.transition(AppState::Error(format!("Pipeline failed: {e}")), &app);
            tokio::time::sleep(Duration::from_secs(3)).await;
            state_mgr.transition(AppState::Idle, &app);
        }
        Ok(Ok(response_text)) => {
            let polished = serde_json::from_str::<serde_json::Value>(&response_text)
                .ok()
                .and_then(|v| v["polished_text"].as_str().map(str::to_string))
                .unwrap_or_default();

            if polished.is_empty() {
                state_mgr.transition(AppState::Error("No text transcribed".to_string()), &app);
                tokio::time::sleep(Duration::from_secs(3)).await;
                state_mgr.transition(AppState::Idle, &app);
                return;
            }

            let current_state = state_mgr.get();
            if current_state == AppState::Idle {
                eprintln!("[aurotype] Pipeline result ignored: request was cancelled");
                return;
            }

            state_mgr.transition(AppState::Injecting, &app);
            let current_state = state_mgr.get();
            if current_state == AppState::Idle {
                eprintln!("[aurotype] Injection skipped: request was cancelled");
                return;
            }

            if let Err(e) = injection::inject_text(&polished) {
                eprintln!("[aurotype] Injection error: {e}");
                state_mgr.transition(AppState::Error(format!("Injection failed: {e}")), &app);
                tokio::time::sleep(Duration::from_secs(3)).await;
                state_mgr.transition(AppState::Idle, &app);
                return;
            }

            state_mgr.transition(AppState::Idle, &app);
        }
    }
}

async fn sync_settings_internal(app: &tauri::AppHandle) -> Result<(), String> {
    let store = app
        .store("settings.json")
        .map_err(|e| format!("store error: {e}"))?;

    let Some(config) = store.get("config") else {
        return Ok(());
    };

    let stt_provider = config
        .get("stt_provider")
        .and_then(|v| v.as_str())
        .unwrap_or("deepgram");
    let stt_api_key = config
        .get("stt_api_key")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let llm_provider = config
        .get("llm_provider")
        .and_then(|v| v.as_str())
        .unwrap_or("openai");
    let llm_api_key = config
        .get("llm_api_key")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let language = config
        .get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("auto");

    let mut deepgram_api_key = String::new();
    let mut openai_api_key = String::new();
    let mut siliconflow_api_key = String::new();
    let mut dashscope_api_key = String::new();

    match stt_provider {
        "deepgram" => deepgram_api_key = stt_api_key.to_string(),
        "siliconflow" => siliconflow_api_key = stt_api_key.to_string(),
        "dashscope" => dashscope_api_key = stt_api_key.to_string(),
        _ => {}
    }

    match llm_provider {
        "openai" => openai_api_key = llm_api_key.to_string(),
        "siliconflow" => siliconflow_api_key = llm_api_key.to_string(),
        _ => {}
    }

    let body = serde_json::json!({
        "stt_provider": stt_provider,
        "deepgram_api_key": if deepgram_api_key.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(deepgram_api_key) },
        "llm_provider": llm_provider,
        "openai_api_key": if openai_api_key.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(openai_api_key) },
        "siliconflow_api_key": if siliconflow_api_key.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(siliconflow_api_key) },
        "dashscope_api_key": if dashscope_api_key.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(dashscope_api_key) },
        "language": language,
    });

    let sidecar = app.state::<sidecar::SidecarState>();
    sidecar::sidecar_post(&sidecar, "/configure", body).await?;

    Ok(())
}

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

    let _ = injection::capture_foreground_window();

    sidecar::sidecar_post(&sidecar, "/record/start", serde_json::json!({})).await?;
    state.transition(AppState::Recording, &app);
    Ok(())
}

#[tauri::command]
async fn stop_recording(
    state: tauri::State<'_, AppStateManager>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let current = state.get();
    if current != AppState::Recording {
        return Err(format!(
            "Cannot stop recording from state: {}",
            current.as_str()
        ));
    }

    state.transition(AppState::Processing, &app);
    run_pipeline(app).await;
    Ok("processed".to_string())
}

#[tauri::command]
async fn sync_settings(app: tauri::AppHandle) -> Result<(), String> {
    sync_settings_internal(&app).await
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
            sync_settings,
            cancel,
            sidecar::get_health,
            injection::inject_text_at_cursor,
        ])
        .setup(|app| {
            let sidecar_state = sidecar::spawn_sidecar(app.handle().clone())
                .map_err(|err| std::io::Error::other(format!("failed to spawn sidecar: {err}")))?;
            app.manage(sidecar_state);

            let sync_handle = app.handle().clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                let _ = sync_settings_internal(&sync_handle).await;
            });

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
