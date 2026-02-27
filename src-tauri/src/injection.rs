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

    pub fn current_window() -> Option<WindowRef> {
        None
    }

    pub fn focus_window(_window: WindowRef) -> Result<(), String> {
        Ok(())
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

    clipboard
        .set_text(text.to_owned())
        .map_err(|e| e.to_string())?;

    refocus_captured_window()?;

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
