use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

static CAPTURED_WINDOW: Mutex<Option<WindowRef>> = Mutex::new(None);

#[cfg(target_os = "windows")]
type WindowRef = isize;

#[cfg(not(target_os = "windows"))]
type WindowRef = u64;

#[cfg(target_os = "windows")]
mod platform {
    use super::WindowRef;

    #[link(name = "user32")]
    extern "system" {
        fn GetForegroundWindow() -> *mut core::ffi::c_void;
        fn SetForegroundWindow(hwnd: *mut core::ffi::c_void) -> i32;
    }

    pub fn current_window() -> Option<WindowRef> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.is_null() {
            None
        } else {
            Some(hwnd as WindowRef)
        }
    }

    pub fn focus_window(window: WindowRef) -> Result<(), String> {
        if window == 0 {
            return Ok(());
        }

        let focused = unsafe { SetForegroundWindow(window as *mut core::ffi::c_void) };
        if focused == 0 {
            return Err("failed to focus previously captured window (it may be closed)".to_string());
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::WindowRef;
    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject};

    /// Capture the PID of the frontmost application via NSWorkspace.
    pub fn current_window() -> Option<WindowRef> {
        unsafe {
            let cls = AnyClass::get(c"NSWorkspace")?;
            let workspace: *mut AnyObject = msg_send![cls, sharedWorkspace];
            if workspace.is_null() {
                return None;
            }
            let app: *mut AnyObject = msg_send![&*workspace, frontmostApplication];
            if app.is_null() {
                return None;
            }
            let pid: i32 = msg_send![&*app, processIdentifier];
            if pid <= 0 {
                return None;
            }
            Some(pid as WindowRef)
        }
    }

    /// Activate the application with the given PID so it regains focus.
    pub fn focus_window(window: WindowRef) -> Result<(), String> {
        unsafe {
            let cls = AnyClass::get(c"NSRunningApplication")
                .ok_or("NSRunningApplication class not found")?;
            let app: *mut AnyObject =
                msg_send![cls, runningApplicationWithProcessIdentifier: window as i32];
            if app.is_null() {
                return Err("Could not find application for stored PID".to_string());
            }
            // NSApplicationActivateIgnoringOtherApps = 1 << 1
            let ok: bool = msg_send![&*app, activateWithOptions: 2u64];
            if !ok {
                return Err("Failed to activate application".to_string());
            }
            Ok(())
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::WindowRef;

    pub fn current_window() -> Option<WindowRef> {
        None
    }

    pub fn focus_window(_window: WindowRef) -> Result<(), String> {
        Ok(())
    }
}

pub fn capture_foreground_window() -> Result<(), String> {
    let window = platform::current_window();
    let mut slot = CAPTURED_WINDOW
        .lock()
        .map_err(|_| "failed to acquire foreground-window lock".to_string())?;
    *slot = window;
    Ok(())
}

fn refocus_captured_window() -> Result<(), String> {
    let window = *CAPTURED_WINDOW
        .lock()
        .map_err(|_| "failed to acquire foreground-window lock".to_string())?;

    if let Some(window) = window {
        platform::focus_window(window)?;
    }

    Ok(())
}


pub fn inject_text(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let saved_clipboard = clipboard.get_text().ok();

    if !crate::permissions::is_accessibility_granted() {
        return Err(
            "Accessibility permission required. \
             Grant access in System Settings \u{2192} Privacy & Security \u{2192} Accessibility.".to_string()
        );
    }

    clipboard
        .set_text(text.to_owned())
        .map_err(|e| e.to_string())?;

    refocus_captured_window()?;
    // Give macOS time to complete the app switch before simulating paste.
    thread::sleep(Duration::from_millis(50));

    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    {
        enigo
            .key(Key::Meta, Direction::Press)
            .map_err(|e| e.to_string())?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| e.to_string())?;
        enigo
            .key(Key::Meta, Direction::Release)
            .map_err(|e| e.to_string())?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| e.to_string())?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| e.to_string())?;
        enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| e.to_string())?;
    }

    thread::sleep(Duration::from_millis(100));

    if let Some(saved) = saved_clipboard {
        clipboard.set_text(saved).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn inject_text_at_cursor(text: String) -> Result<(), String> {
    inject_text(&text).map_err(|e| e.to_string())
}
