use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

/// State captured at the moment recording starts.
struct CapturedState {
    window: WindowRef,
    has_text_focus: bool,
}

static CAPTURED_STATE: Mutex<Option<CapturedState>> = Mutex::new(None);

#[cfg(target_os = "windows")]
type WindowRef = isize;

#[cfg(not(target_os = "windows"))]
type WindowRef = u64;

#[cfg(target_os = "windows")]
mod platform {
    use super::WindowRef;
    use windows::core::Interface;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, IUIAutomationElement,
        IUIAutomationLegacyIAccessiblePattern, IUIAutomationValuePattern,
        UIA_DocumentControlTypeId, UIA_EditControlTypeId,
        UIA_LegacyIAccessiblePatternId, UIA_ValuePatternId,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow};

    /// LegacyIAccessible role constants (from oleacc.h)
    const ROLE_SYSTEM_TEXT: u32 = 42;
    const ROLE_SYSTEM_COMBOBOX: u32 = 46;

    /// Terminal window class names that always support text input.
    const TERMINAL_CLASSES: &[&str] = &["TermControl", "ConsoleWindowClass"];

    pub fn current_window() -> Option<WindowRef> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.is_invalid() {
            None
        } else {
            Some(hwnd.0 as WindowRef)
        }
    }

    pub fn focus_window(window: WindowRef) -> Result<(), String> {
        if window == 0 {
            return Ok(());
        }

        let hwnd = HWND(window as *mut _);
        let ok = unsafe { SetForegroundWindow(hwnd) };
        if !ok.as_bool() {
            return Err(
                "failed to focus previously captured window (it may be closed)".to_string(),
            );
        }
        Ok(())
    }

    /// Check whether the currently focused UI element supports text input.
    ///
    /// Uses Windows UI Automation to inspect the focused element via four
    /// strategies: Edit control type, terminal class names, writable Document,
    /// and LegacyIAccessible role.
    pub fn has_text_focus() -> bool {
        // COM must be initialised on the calling thread.
        let com_ok = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        // RPC_E_CHANGED_MODE means COM was already initialised — that's fine.
        let need_uninit = com_ok.is_ok();
        if !com_ok.is_ok() && com_ok != windows::Win32::Foundation::RPC_E_CHANGED_MODE {
            return false;
        }

        let result = check_focused_element();

        if need_uninit {
            unsafe {
                CoUninitialize();
            }
        }

        result
    }

    /// Extract properties from a UIA element for strategy evaluation.
    struct ElementInfo {
        control_type: i32,
        class_name: String,
        name: String,
        legacy_role: u32,
    }

    impl ElementInfo {
        unsafe fn from_element(element: &IUIAutomationElement) -> Self {
            Self {
                control_type: element.CurrentControlType().map(|c| c.0).unwrap_or(0),
                class_name: element
                    .CurrentClassName()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                name: element
                    .CurrentName()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                legacy_role: element
                    .GetCurrentPattern(UIA_LegacyIAccessiblePatternId)
                    .and_then(|p| p.cast::<IUIAutomationLegacyIAccessiblePattern>())
                    .and_then(|lp| lp.CurrentRole())
                    .unwrap_or(0),
            }
        }
    }

    fn check_focused_element() -> bool {
        let (_automation, element) = match get_automation_and_focused_element() {
            Some(pair) => pair,
            None => return false,
        };

        let info = unsafe { ElementInfo::from_element(&element) };

        // Strategy 1: Edit control — always editable
        if is_edit_control(info.control_type) {
            log_hit("S1:Edit", &info);
            return true;
        }

        // Strategy 2: Terminal windows — always pasteable
        if is_terminal_window(&info.class_name) {
            log_hit("S2:Terminal", &info);
            return true;
        }

        // Strategy 3: Document control — only if writable ValuePattern
        if is_document_control(info.control_type) {
            match has_writable_value_pattern(&element) {
                Some(true) => {
                    log_hit("S3:DocValue", &info);
                    return true;
                }
                Some(false) => {
                    log_miss("Doc", &info);
                    return false;
                }
                None => {}
            }
        }

        // Strategy 4: LegacyIAccessible role (Chromium inputs, older Win32)
        if has_legacy_text_role(info.legacy_role) {
            log_hit("S4:Legacy", &info);
            return true;
        }

        log_miss("", &info);
        false
    }

    fn get_automation_and_focused_element() -> Option<(IUIAutomation, IUIAutomationElement)> {
        unsafe {
            let automation: IUIAutomation =
                match CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL) {
                    Ok(a) => a,
                    Err(_) => {
                        log::warn!("Focus detection: CoCreateInstance failed");
                        return None;
                    }
                };

            let element: IUIAutomationElement = match automation.GetFocusedElement() {
                Ok(e) => e,
                Err(e) => {
                    log::warn!("Focus detection: GetFocusedElement failed: {e}");
                    return None;
                }
            };

            Some((automation, element))
        }
    }

    fn is_edit_control(control_type: i32) -> bool {
        control_type == UIA_EditControlTypeId.0
    }

    fn is_terminal_window(class_name: &str) -> bool {
        TERMINAL_CLASSES.contains(&class_name)
    }

    fn is_document_control(control_type: i32) -> bool {
        control_type == UIA_DocumentControlTypeId.0
    }

    fn has_writable_value_pattern(element: &IUIAutomationElement) -> Option<bool> {
        unsafe {
            let is_readonly = element
                .GetCurrentPattern(UIA_ValuePatternId)
                .and_then(|p| p.cast::<IUIAutomationValuePattern>())
                .and_then(|vp| vp.CurrentIsReadOnly())
                .ok()?;
            Some(!is_readonly.as_bool())
        }
    }

    fn has_legacy_text_role(legacy_role: u32) -> bool {
        legacy_role == ROLE_SYSTEM_TEXT || legacy_role == ROLE_SYSTEM_COMBOBOX
    }

    fn log_hit(strategy: &str, info: &ElementInfo) {
        log::info!(
            "HIT {}: ct={} class={:?} name={:?} role={}",
            strategy, info.control_type, info.class_name, info.name, info.legacy_role
        );
    }

    fn log_miss(reason: &str, info: &ElementInfo) {
        let prefix = if reason.is_empty() { "" } else { ":" };
        log::info!(
            "MISS{}{} ct={} class={:?} name={:?} role={}",
            prefix, reason, info.control_type, info.class_name, info.name, info.legacy_role
        );
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

    /// macOS text-focus detection not yet implemented — assume true.
    // TODO: Implement using NSAccessibility or Carbon Accessibility API.
    // Should check if the focused element is an NSTextField, NSTextView, or
    // other text-input control. See AXUIElement API.
    pub fn has_text_focus() -> bool {
        true
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

    /// Linux text-focus detection not yet implemented — assume true.
    // TODO: Implement using AT-SPI (Assistive Technology Service Provider Interface).
    // Should check if the focused accessible object has a text-editable role.
    // See libatspi and atspi crate.
    pub fn has_text_focus() -> bool {
        true
    }
}

pub fn capture_foreground_window() -> Result<(), String> {
    let window = platform::current_window();
    let text_focus = platform::has_text_focus();

    log::info!(
        "Captured window: {:?}, has_text_focus: {}",
        window, text_focus
    );

    let mut slot = CAPTURED_STATE
        .lock()
        .map_err(|_| "failed to acquire foreground-window lock".to_string())?;

    *slot = window.map(|w| CapturedState {
        window: w,
        has_text_focus: text_focus,
    });

    Ok(())
}

fn refocus_captured_window() -> Result<(), String> {
    let state = CAPTURED_STATE
        .lock()
        .map_err(|_| "failed to acquire foreground-window lock".to_string())?;

    if let Some(ref captured) = *state {
        platform::focus_window(captured.window)?;
    }

    Ok(())
}

pub fn inject_text(text: &str) -> Result<(), String> {
    // Check whether text focus was detected when recording started.
    {
        let state = CAPTURED_STATE
            .lock()
            .map_err(|_| "failed to acquire foreground-window lock".to_string())?;
        match &*state {
            Some(captured) if !captured.has_text_focus => {
                return Err(
                    "No text input field was focused when recording started".to_string()
                );
            }
            _ => {}
        }
    }

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let saved_clipboard = clipboard.get_text().ok();

    if !crate::permissions::is_accessibility_granted() {
        return Err(
            "Accessibility permission required. \
             Grant access in System Settings \u{2192} Privacy & Security \u{2192} Accessibility."
                .to_string(),
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
