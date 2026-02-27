use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};

use crate::state::{AppState, AppStateManager, HotkeyMode};

/// Default hotkey: CmdOrCtrl+Shift+Space
fn default_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Space)
}

/// Escape key shortcut for cancelling recording.
fn escape_shortcut() -> Shortcut {
    Shortcut::new(None, Code::Escape)
}

/// Register all global hotkeys on the app.
/// Called during `setup` in `lib.rs`.
pub fn register_hotkeys(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let main_shortcut = default_shortcut();
    let esc_shortcut = escape_shortcut();

    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                let state_manager = app.state::<AppStateManager>();

                if *shortcut == main_shortcut {
                    handle_main_hotkey(&state_manager, app, event.state());
                } else if *shortcut == esc_shortcut {
                    // Escape only acts on press
                    if event.state() == ShortcutState::Pressed {
                        handle_escape(&state_manager, app);
                    }
                }
            })
            .build(),
    )?;

    app.global_shortcut().register(default_shortcut())?;
    app.global_shortcut().register(escape_shortcut())?;

    Ok(())
}

/// Handle the main hotkey (CmdOrCtrl+Shift+Space).
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
                        manager.transition(AppState::Recording, app);
                    }
                    AppState::Recording => {
                        manager.transition(AppState::Processing, app);
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
                        manager.transition(AppState::Recording, app);
                    }
                }
                ShortcutState::Released => {
                    if current == AppState::Recording {
                        manager.transition(AppState::Processing, app);
                    }
                }
            }
        }
    }
}

/// Handle Escape key: cancel recording and return to idle.
fn handle_escape(manager: &AppStateManager, app: &AppHandle) {
    let current = manager.get();
    if current == AppState::Recording {
        manager.transition(AppState::Idle, app);
    }
}
