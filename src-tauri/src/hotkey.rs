use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};

use crate::{injection, sidecar};
use crate::state::{AppState, AppStateManager, HotkeyMode};

/// Default hotkey: Ctrl+Alt+Space
fn default_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space)
}

/// Escape key shortcut for cancelling recording.
fn escape_shortcut() -> Shortcut {
    Shortcut::new(None, Code::Escape)
}

/// Register all global hotkeys on the app.
/// Called during `setup` in `lib.rs`.
pub fn register_hotkeys(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let esc_shortcut = escape_shortcut();
    let main_shortcut = default_shortcut();

    // CRITICAL: Windows requires BOTH with_handler AND on_shortcut for hotkeys to fire.
    // Using only one of them does not work (confirmed via simulated keypress testing).

    // Step 1: Build plugin with a global handler that routes escape keys.
    // The global handler catches escape; main hotkey is handled by on_shortcut below.
    let esc_for_handler = esc_shortcut;
    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if *shortcut == esc_for_handler && event.state() == ShortcutState::Pressed {
                    let state_manager = app.state::<AppStateManager>();
                    handle_escape(&state_manager, app);
                }
                // Main hotkey is handled by on_shortcut below — intentional no-op here.
            })
            .build(),
    )?;

    // Step 2: Register escape via register() (handled by with_handler above).
    app.global_shortcut().register(esc_shortcut)?;

    // Step 3: Register main hotkey via on_shortcut() — this is the handler that actually fires.
    app.global_shortcut().on_shortcut(main_shortcut, move |app, _shortcut, event| {
        let state_manager = app.state::<AppStateManager>();
        handle_main_hotkey(&state_manager, app, event.state());
    })?;

    {
        let state_manager = app.state::<AppStateManager>();
        *state_manager.current_shortcut.lock().unwrap() = Some(main_shortcut);
    }

    eprintln!("[aurotype] Hotkeys registered: main={main_shortcut:?}, escape=Escape");
    Ok(())
}

/// Handle the main hotkey (Ctrl+Alt+Space).
/// Behavior depends on the configured HotkeyMode.
fn handle_main_hotkey(
    manager: &AppStateManager,
    app: &AppHandle,
    shortcut_state: ShortcutState,
) {
    let mode = manager.mode.lock().unwrap().clone();
    let current = manager.get();

    match mode {
        HotkeyMode::Toggle => {
            // Only act on press events for toggle mode
            if shortcut_state == ShortcutState::Pressed {
                match current {
                    AppState::Idle => {
                        start_recording(manager, app);
                    }
                    AppState::Recording => {
                        stop_recording(manager, app);
                    }
                    // Ignore hotkey in other states
                    _ => {}
                }
            }
        }
        HotkeyMode::HoldToRecord => {
            match shortcut_state {
                ShortcutState::Pressed => {
                    if current == AppState::Idle {
                        start_recording(manager, app);
                    }
                }
                ShortcutState::Released => {
                    if current == AppState::Recording {
                        stop_recording(manager, app);
                    }
                }
            }
        }
    }
}

fn start_recording(manager: &AppStateManager, app: &AppHandle) {
    let _ = injection::capture_foreground_window();
    manager.engine_recording.store(false, std::sync::atomic::Ordering::SeqCst);
    manager.transition(AppState::Recording, app);

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let sidecar_state = app_clone.state::<sidecar::SidecarState>();
        match sidecar::sidecar_post(&sidecar_state, "/record/start", serde_json::json!({})).await {
            Ok(_) => {
                let state_mgr = app_clone.state::<AppStateManager>();
                state_mgr.engine_recording.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            Err(err) => {
                let state_mgr = app_clone.state::<AppStateManager>();
                eprintln!("[aurotype] Failed to start recording: {err}");
                let err_lower = err.to_lowercase();
                let error_msg = if err_lower.contains("audiodeviceerror")
                    || err_lower.contains("no default input device")
                    || err_lower.contains("audio")
                {
                    "No microphone found".to_string()
                } else {
                    format!("Failed to start recording: {err}")
                };
                state_mgr.transition(AppState::Error(error_msg), &app_clone);
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                state_mgr.transition(AppState::Idle, &app_clone);
            }
        }
    });
}

fn stop_recording(manager: &AppStateManager, app: &AppHandle) {
    manager.transition(AppState::Processing, app);
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        crate::run_pipeline(app_clone).await;
    });
}

/// Handle Escape key: cancel recording and return to idle.
fn handle_escape(manager: &AppStateManager, app: &AppHandle) {
    let current = manager.get();
    match current {
        AppState::Recording => {
            manager.transition(AppState::Idle, app);
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                let sidecar_state = app_clone.state::<sidecar::SidecarState>();
                if let Err(err) =
                    sidecar::sidecar_post(&sidecar_state, "/record/cancel", serde_json::json!({}))
                        .await
                {
                    eprintln!("[aurotype] Failed to cancel recording: {err}");
                }
            });
        }
        AppState::Processing => {
            manager.transition(AppState::Idle, app);
        }
        _ => {}
    }
}

/// Parse a shortcut string like "Ctrl+Shift+Space" into a Shortcut.
fn parse_shortcut(s: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = s.split('+').map(str::trim).collect();
    if parts.is_empty() {
        return Err("Empty shortcut string".to_string());
    }

    let mut modifiers = Modifiers::empty();
    let key_str = parts.last().unwrap();

    for &part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "shift" => modifiers |= Modifiers::SHIFT,
            "alt" => modifiers |= Modifiers::ALT,
            "super" | "cmd" | "meta" | "cmdorctrl" => modifiers |= Modifiers::SUPER,
            other => return Err(format!("Unknown modifier: {other}")),
        }
    }

    let code = match key_str.to_lowercase().as_str() {
        "space" => Code::Space,
        "f1" => Code::F1,
        "f2" => Code::F2,
        "f3" => Code::F3,
        "f4" => Code::F4,
        "f5" => Code::F5,
        "f6" => Code::F6,
        "f7" => Code::F7,
        "f8" => Code::F8,
        "f9" => Code::F9,
        "f10" => Code::F10,
        "f11" => Code::F11,
        "f12" => Code::F12,
        "a" => Code::KeyA,
        "b" => Code::KeyB,
        "c" => Code::KeyC,
        "d" => Code::KeyD,
        "e" => Code::KeyE,
        "f" => Code::KeyF,
        "g" => Code::KeyG,
        "h" => Code::KeyH,
        "i" => Code::KeyI,
        "j" => Code::KeyJ,
        "k" => Code::KeyK,
        "l" => Code::KeyL,
        "m" => Code::KeyM,
        "n" => Code::KeyN,
        "o" => Code::KeyO,
        "p" => Code::KeyP,
        "q" => Code::KeyQ,
        "r" => Code::KeyR,
        "s" => Code::KeyS,
        "t" => Code::KeyT,
        "u" => Code::KeyU,
        "v" => Code::KeyV,
        "w" => Code::KeyW,
        "x" => Code::KeyX,
        "y" => Code::KeyY,
        "z" => Code::KeyZ,
        other => return Err(format!("Unknown key: {other}")),
    };

    let mods = if modifiers.is_empty() { None } else { Some(modifiers) };
    Ok(Shortcut::new(mods, code))
}

/// Update the main hotkey at runtime. Called from the frontend.
#[tauri::command]
pub fn update_hotkey(app: AppHandle, shortcut: String) -> Result<(), String> {
    let new_shortcut = parse_shortcut(&shortcut)?;
    let state_manager = app.state::<AppStateManager>();
    // Skip if the new shortcut is the same as current (avoids breaking the handler)
    {
        let current = state_manager.current_shortcut.lock().unwrap();
        if let Some(ref current_shortcut) = *current {
            if *current_shortcut == new_shortcut {
                eprintln!("[aurotype] Hotkey unchanged: {shortcut}");
                return Ok(());
            }
        }
    }

    // Unregister old shortcut
    let old = state_manager.current_shortcut.lock().unwrap().take();
    if let Some(old_shortcut) = old {
        if let Err(e) = app.global_shortcut().unregister(old_shortcut) {
            eprintln!("[aurotype] Failed to unregister old hotkey: {e}");
        }
    }

    // Register new shortcut via on_shortcut (same pattern as register_hotkeys).
    // CRITICAL: must use on_shortcut, not register, for the handler to fire on Windows.
    if let Err(e) = app.global_shortcut().on_shortcut(new_shortcut, move |app, _shortcut, event| {
        let state_manager = app.state::<AppStateManager>();
        handle_main_hotkey(&state_manager, app, event.state());
    }) {
        // Try to re-register the default if the new one fails
        let fallback = default_shortcut();
        let _ = app.global_shortcut().on_shortcut(fallback, move |app, _shortcut, event| {
            let state_manager = app.state::<AppStateManager>();
            handle_main_hotkey(&state_manager, app, event.state());
        });
        *state_manager.current_shortcut.lock().unwrap() = Some(fallback);
        return Err(format!("Failed to register hotkey '{shortcut}': {e}"));
    }

    *state_manager.current_shortcut.lock().unwrap() = Some(new_shortcut);
    eprintln!("[aurotype] Hotkey changed to: {shortcut}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri_plugin_global_shortcut::{Code, Modifiers};

    #[test]
    fn parse_ctrl_alt_space() {
        let shortcut = parse_shortcut("Ctrl+Alt+Space").unwrap();
        assert_eq!(shortcut.mods, Modifiers::CONTROL | Modifiers::ALT);
        assert_eq!(shortcut.key, Code::Space);
    }

    #[test]
    fn parse_ctrl_shift_space() {
        let shortcut = parse_shortcut("Ctrl+Shift+Space").unwrap();
        assert_eq!(shortcut.mods, Modifiers::CONTROL | Modifiers::SHIFT);
        assert_eq!(shortcut.key, Code::Space);
    }

    #[test]
    fn parse_cmdorctrl_shift_a() {
        let shortcut = parse_shortcut("CmdOrCtrl+Shift+A").unwrap();
        assert_eq!(shortcut.mods, Modifiers::SUPER | Modifiers::SHIFT);
        assert_eq!(shortcut.key, Code::KeyA);
    }

    #[test]
    fn parse_cmdorctrl_shift_r() {
        let shortcut = parse_shortcut("CmdOrCtrl+Shift+R").unwrap();
        assert_eq!(shortcut.mods, Modifiers::SUPER | Modifiers::SHIFT);
        assert_eq!(shortcut.key, Code::KeyR);
    }

    #[test]
    fn parse_cmdorctrl_shift_v() {
        let shortcut = parse_shortcut("CmdOrCtrl+Shift+V").unwrap();
        assert_eq!(shortcut.mods, Modifiers::SUPER | Modifiers::SHIFT);
        assert_eq!(shortcut.key, Code::KeyV);
    }

    #[test]
    fn parse_cmdorctrl_space() {
        let shortcut = parse_shortcut("CmdOrCtrl+Space").unwrap();
        assert_eq!(shortcut.mods, Modifiers::SUPER);
        assert_eq!(shortcut.key, Code::Space);
    }

    #[test]
    fn parse_f9() {
        let shortcut = parse_shortcut("F9").unwrap();
        assert_eq!(shortcut.mods, Modifiers::empty());
        assert_eq!(shortcut.key, Code::F9);
    }

    #[test]
    fn parse_f10() {
        let shortcut = parse_shortcut("F10").unwrap();
        assert_eq!(shortcut.mods, Modifiers::empty());
        assert_eq!(shortcut.key, Code::F10);
    }

    #[test]
    fn parse_empty_string_is_err() {
        let result = parse_shortcut("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_unknown_modifier_is_err() {
        let result = parse_shortcut("UnknownMod+Space");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown modifier"));
    }

    #[test]
    fn parse_unknown_key_is_err() {
        let result = parse_shortcut("Ctrl+UnknownKey");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown key"));
    }
}
