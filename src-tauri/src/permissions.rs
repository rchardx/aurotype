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
            eprintln!("[aurotype] AVCaptureDevice class not found");
            return;
        };

        // AVMediaTypeAudio
        let audio_type = NSString::from_str("soun");

        // Check current authorization status:
        // 0 = notDetermined, 1 = restricted, 2 = denied, 3 = authorized
        let status: isize = msg_send![cls, authorizationStatusForMediaType: &*audio_type];

        match status {
            0 => {
                eprintln!("[aurotype] Microphone permission not determined, requesting…");
                let handler = StackBlock::new(|granted: Bool| {
                    if granted.as_bool() {
                        eprintln!("[aurotype] Microphone permission granted");
                    } else {
                        eprintln!("[aurotype] Microphone permission denied by user");
                    }
                });
                let _: () = msg_send![
                    cls,
                    requestAccessForMediaType: &*audio_type,
                    completionHandler: &*handler
                ];
            }
            2 => {
                eprintln!(
                    "[aurotype] Microphone permission denied. \
                     Grant access in System Settings → Privacy & Security → Microphone."
                );
            }
            3 => {
                eprintln!("[aurotype] Microphone permission already granted");
            }
            other => {
                eprintln!("[aurotype] Microphone authorization status: {other}");
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_microphone_permission() {
    // No-op on non-macOS platforms.
}
