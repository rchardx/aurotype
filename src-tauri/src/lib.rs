mod hotkey;
mod injection;
mod permissions;
mod sidecar;
mod state;
mod tray;

use state::{AppState, AppStateManager, HotkeyMode, TranscriptionRecord};
use tauri::Manager;
use tauri_plugin_store::StoreExt;

pub async fn run_pipeline(app: tauri::AppHandle) {
    use std::sync::atomic::Ordering;
    use std::time::Duration;
    use tokio::time::timeout;

    let sidecar = app.state::<sidecar::SidecarState>();
    let state_mgr = app.state::<AppStateManager>();

    // Wait for POST /record/start to complete before sending /record/stop.
    // Without this gate the stop request can arrive before the engine has
    // actually begun recording, producing empty audio.
    let engine_recording = state_mgr.engine_recording.clone();
    let wait_start = tokio::time::Instant::now();
    while !engine_recording.load(Ordering::SeqCst) {
        if wait_start.elapsed() > Duration::from_secs(5) {
            eprintln!("[aurotype] Timed out waiting for engine to start recording");
            state_mgr.transition(AppState::Error("Recording failed to start".to_string()), &app);
            tokio::time::sleep(Duration::from_secs(3)).await;
            state_mgr.transition(AppState::Idle, &app);
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let result = timeout(
        Duration::from_secs(60),
        sidecar::sidecar_post(&sidecar, "/record/stop", serde_json::json!({})),
    )
    .await;

    match result {
        Err(_elapsed) => {
            eprintln!("[aurotype] Pipeline timeout: /record/stop exceeded 60s");
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
            let parsed = serde_json::from_str::<serde_json::Value>(&response_text).ok();
            let raw_text = parsed.as_ref()
                .and_then(|v| v["raw_text"].as_str().map(str::to_string))
                .unwrap_or_default();
            let polished = parsed.as_ref()
                .and_then(|v| v["polished_text"].as_str().map(str::to_string))
                .unwrap_or_default();

            if polished.is_empty() {
                state_mgr.transition(AppState::Error("No text transcribed".to_string()), &app);
                tokio::time::sleep(Duration::from_secs(3)).await;
                state_mgr.transition(AppState::Idle, &app);
                return;
            }
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

            // Save audio recording to disk if present
            let audio_data = parsed.as_ref()
                .and_then(|v| v["audio_data"].as_str().map(str::to_string));
            let audio_file = if let Some(b64) = audio_data {
                match save_audio_file(&app, &b64, &timestamp) {
                    Ok(path) => Some(path),
                    Err(e) => {
                        eprintln!("[aurotype] Failed to save audio: {e}");
                        None
                    }
                }
            } else {
                None
            };

            // Record to history
            state_mgr.add_history(TranscriptionRecord {
                raw_text: raw_text.clone(),
                polished_text: polished.clone(),
                timestamp,
                audio_file,
            }, &app);

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

            // Dispatch inject_text to the main thread — macOS enigo APIs
            // (TSMGetInputSourceProperty) abort if called from a tokio worker.
            let polished_for_inject = polished.clone();
            let (tx, rx) = tokio::sync::oneshot::channel();
            if let Err(e) = app.run_on_main_thread(move || {
                let result = injection::inject_text(&polished_for_inject);
                let _ = tx.send(result);
            }) {
                eprintln!("[aurotype] Failed to dispatch injection to main thread: {e}");
                state_mgr.transition(AppState::CopyAvailable(polished), &app);
                return;
            }
            let inject_result = match rx.await {
                Ok(r) => r,
                Err(_) => Err("injection channel closed".to_string()),
            };
            if let Err(e) = inject_result {
                eprintln!("[aurotype] Injection error (offering copy): {e}");
                state_mgr.transition(AppState::CopyAvailable(polished), &app);
                return;
            }

            // Brief pause so the "Complete" indicator is visible
            tokio::time::sleep(Duration::from_millis(800)).await;
            state_mgr.transition(AppState::Idle, &app);
        }
    }
}

fn save_audio_file(app: &tauri::AppHandle, b64_data: &str, timestamp: &str) -> Result<String, String> {
    use base64::Engine;
    use std::fs;
    use std::path::PathBuf;

    let app_data = app.path().app_data_dir().map_err(|e| format!("no app_data_dir: {e}"))?;
    let recordings_dir: PathBuf = app_data.join("recordings");
    fs::create_dir_all(&recordings_dir).map_err(|e| format!("mkdir recordings: {e}"))?;

    // Sanitize timestamp for filename: "2025-01-15 14:30:25" -> "2025-01-15_14-30-25"
    let safe_name = timestamp.replace(' ', "_").replace(':', "-");
    let filename = format!("{safe_name}.wav");
    let filepath = recordings_dir.join(&filename);

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64_data)
        .map_err(|e| format!("base64 decode: {e}"))?;

    fs::write(&filepath, &bytes).map_err(|e| format!("write wav: {e}"))?;

    let path_str = filepath.to_string_lossy().to_string();
    eprintln!("[aurotype] Saved audio recording: {path_str} ({} bytes)", bytes.len());
    Ok(path_str)
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
        .unwrap_or("aliyun_dashscope");
    let stt_api_key = config
        .get("stt_api_key")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let llm_provider = config
        .get("llm_provider")
        .and_then(|v| v.as_str())
        .unwrap_or("deepseek");
    let llm_api_key = config
        .get("llm_api_key")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let language = config
        .get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("auto");
    let llm_base_url = config
        .get("llm_base_url")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let llm_model = config
        .get("llm_model")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let stt_model = config
        .get("stt_model")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let system_prompt = config
        .get("system_prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut openai_api_key = String::new();
    let mut aliyun_dashscope_api_key = String::new();
    let mut deepseek_api_key = String::new();

    if stt_provider == "aliyun_dashscope" {
        aliyun_dashscope_api_key = stt_api_key.to_string();
    }

    match llm_provider {
        "openai" => openai_api_key = llm_api_key.to_string(),
        "deepseek" => deepseek_api_key = llm_api_key.to_string(),
        _ => {}
    }

    let body = serde_json::json!({
        "stt_provider": stt_provider,
        "stt_model": if stt_model.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(stt_model.to_string()) },
        "llm_provider": llm_provider,
        "openai_api_key": if openai_api_key.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(openai_api_key) },
        "aliyun_dashscope_api_key": if aliyun_dashscope_api_key.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(aliyun_dashscope_api_key) },
        "deepseek_api_key": if deepseek_api_key.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(deepseek_api_key) },
        "llm_base_url": if llm_base_url.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(llm_base_url.to_string()) },
        "llm_model": if llm_model.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(llm_model.to_string()) },
        "system_prompt": if system_prompt.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(system_prompt.to_string()) },
        "language": language,
    });

    let sidecar = app.state::<sidecar::SidecarState>();
    sidecar::sidecar_post(&sidecar, "/configure", body).await?;

    // Sync hotkey mode
    let hotkey_mode = config
        .get("hotkey_mode")
        .and_then(|v| v.as_str())
        .unwrap_or("hold");
    {
        let state_manager = app.state::<AppStateManager>();
        let mut mode = state_manager.mode.lock().unwrap();
        *mode = match hotkey_mode {
            "toggle" => HotkeyMode::Toggle,
            _ => HotkeyMode::HoldToRecord,
        };
    }

    // Sync hotkey shortcut if present (skip deprecated Alt+Space — Windows-reserved)
    if let Some(hotkey_str) = config.get("hotkey").and_then(|v| v.as_str()) {
        if !hotkey_str.is_empty() && hotkey_str != "Alt+Space" {
            let _ = hotkey::update_hotkey(app.clone(), hotkey_str.to_string());
        }
    }

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
        AppState::Recording | AppState::Processing | AppState::Error(_) | AppState::CopyAvailable(_) => {
            state.transition(AppState::Idle, &app);
            Ok(())
        }
        AppState::Idle => Ok(()),
        AppState::Injecting => Err("Cannot cancel during text injection".to_string()),
    }
}

#[tauri::command]
async fn test_llm(sidecar: tauri::State<'_, sidecar::SidecarState>) -> Result<String, String> {
    sidecar::sidecar_post(&sidecar, "/test-llm", serde_json::json!({})).await
}

#[tauri::command]
async fn test_stt(sidecar: tauri::State<'_, sidecar::SidecarState>) -> Result<String, String> {
    sidecar::sidecar_post(&sidecar, "/test-stt", serde_json::json!({})).await
}

#[tauri::command]
async fn get_volume(sidecar: tauri::State<'_, sidecar::SidecarState>) -> Result<f64, String> {
    let text = sidecar::sidecar_get(&sidecar, "/volume").await?;
    let parsed: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Failed to parse volume response: {e}"))?;
    Ok(parsed["volume"].as_f64().unwrap_or(0.0))
}

#[tauri::command]
fn get_history(state: tauri::State<AppStateManager>) -> Vec<TranscriptionRecord> {
    state.get_history()
}

#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn clear_history(state: tauri::State<AppStateManager>, app: tauri::AppHandle) {
    state.clear_history(&app);
}

#[tauri::command]
fn get_audio_data(path: String) -> Result<String, String> {
    use base64::Engine;
    let bytes = std::fs::read(&path).map_err(|e| format!("read audio: {e}"))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:audio/wav;base64,{b64}"))
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
            test_llm,
            test_stt,
            get_volume,
            get_history,
            copy_to_clipboard,
            sidecar::get_health,
            injection::inject_text_at_cursor,
            hotkey::update_hotkey,
            clear_history,
            get_audio_data,
        ])
        .setup(|app| {
            // Request permissions at startup so system dialogs appear immediately,
            // not during the first recording. Each function checks whether permission
            // is already granted before prompting.
            permissions::request_microphone_permission();
            permissions::prompt_accessibility_permission();

            let sidecar_state = sidecar::spawn_sidecar(app.handle().clone())
                .map_err(|err| std::io::Error::other(format!("failed to spawn sidecar: {err}")))?;
            app.manage(sidecar_state);

            let sync_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                let _ = sync_settings_internal(&sync_handle).await;
            });

            // Load persisted transcription history from disk
            let state_mgr = app.state::<AppStateManager>();
            state_mgr.load_history_from_store(app.handle());

            #[cfg(desktop)]
            {
                let handle = app.handle();
                hotkey::register_hotkeys(handle)?;
                tray::create_tray(handle)?;
            }

            // Explicitly set window icon for taskbar on Windows
            if let Some(icon) = app.default_window_icon().cloned() {
                if let Some(main_win) = app.get_webview_window("main") {
                    let _ = main_win.set_icon(icon);
                }
            }

            // Force WebView2 background to fully transparent on Windows
            if let Some(float_win) = app.get_webview_window("float") {
                let _ = float_win.set_background_color(Some(tauri::window::Color(0, 0, 0, 0)));
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Intercept close on main window: hide to tray instead of exiting
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
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
