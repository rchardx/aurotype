/// macOS microphone permission request using AVCaptureDevice API.
///
/// Requests access at app startup so the first recording doesn't produce
/// empty audio while the system permission dialog is shown.
#[cfg(target_os = "macos")]
pub fn request_microphone_permission() {
    use block2::StackBlock;
    use objc2::runtime::{AnyClass, Bool};
    use objc2::msg_send;
    use objc2_foundation::NSString;

    unsafe {
        let Some(cls) = AnyClass::get(c"AVCaptureDevice") else {
            log::error!("AVCaptureDevice class not found");
            return;
        };

        // AVMediaTypeAudio
        let audio_type = NSString::from_str("soun");

        // Check current authorization status:
        // 0 = notDetermined, 1 = restricted, 2 = denied, 3 = authorized
        let status: isize = msg_send![cls, authorizationStatusForMediaType: &*audio_type];

        match status {
            0 => {
                log::info!("Microphone permission not determined, requesting…");
                let handler = StackBlock::new(|granted: Bool| {
                    if granted.as_bool() {
                        log::info!("Microphone permission granted");
                    } else {
                        log::warn!("Microphone permission denied by user");
                    }
                });
                let _: () = msg_send![
                    cls,
                    requestAccessForMediaType: &*audio_type,
                    completionHandler: &*handler
                ];
            }
            2 => {
                log::warn!(
                    "Microphone permission denied. \
                     Grant access in System Settings → Privacy & Security → Microphone."
                );
            }
            3 => {
                log::info!("Microphone permission already granted");
            }
            other => {
                log::info!("Microphone authorization status: {other}");
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_microphone_permission() {
    // No-op on non-macOS platforms.
}

/// Check whether Accessibility permission is currently granted.
///
/// Returns `true` if the app is trusted for Accessibility (AXIsProcessTrusted).
#[cfg(target_os = "macos")]
pub fn is_accessibility_granted() -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }

    unsafe { AXIsProcessTrusted() }
}

#[cfg(not(target_os = "macos"))]
pub fn is_accessibility_granted() -> bool {
    true
}

/// Prompt the user to grant Accessibility permission via System Settings.
///
/// Required for enigo CGEvent key simulation (Cmd+V paste injection).
/// Shows the system dialog prompting the user to grant access.
#[cfg(target_os = "macos")]
pub fn prompt_accessibility_permission() {
    use std::ffi::c_void;
    use std::ptr;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
        static kAXTrustedCheckOptionPrompt: *const c_void;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFDictionaryCreate(
            allocator: *const c_void,
            keys: *const *const c_void,
            values: *const *const c_void,
            num_values: isize,
            key_callbacks: *const c_void,
            value_callbacks: *const c_void,
        ) -> *const c_void;
        fn CFRelease(cf: *const c_void);
        static kCFBooleanTrue: *const c_void;
        static kCFTypeDictionaryKeyCallBacks: c_void;
        static kCFTypeDictionaryValueCallBacks: c_void;
    }

    unsafe {
        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];
        let options = CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks as *const _,
            &kCFTypeDictionaryValueCallBacks as *const _,
        );

        let _trusted = AXIsProcessTrustedWithOptions(options);
        if !options.is_null() {
            CFRelease(options);
        }

        log::warn!(
            "Accessibility permission not granted. \
             Grant access in System Settings \u{2192} Privacy & Security \u{2192} Accessibility."
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn prompt_accessibility_permission() {
    // No-op on non-macOS platforms.
}
