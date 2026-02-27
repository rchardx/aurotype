use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

/// Build and attach the system tray icon with a context menu.
/// Called during `setup` in `lib.rs`.
pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let settings_item = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[&settings_item, &quit_item])
        .build()?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Aurotype")
        .on_menu_event(move |app: &AppHandle, event: tauri::menu::MenuEvent| match event.id().as_ref() {
            "settings" => {
                // Show/focus the main window for settings
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
