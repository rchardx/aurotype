use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::fs::{self, OpenOptions};
use tauri::Manager;

pub fn init(app: &tauri::AppHandle) {
    let log_dir = match app.path().app_data_dir() {
        Ok(dir) => dir.join("logs"),
        Err(_) => {
            // Can't log without a log directory, silently fail
            return;
        }
    };

    let _ = fs::create_dir_all(&log_dir);

    let now = chrono::Local::now();
    let filename = now.format("aurotype-%Y-%m-%d.log").to_string();

    let log_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join(filename))
    {
        Ok(file) => file,
        Err(_) => {
            // Can't open log file, silently fail (stderr logger will still work)
            return;
        }
    };

    // CombinedLogger::init can only be called once; if it fails, we've already
    // initialized in a previous call (e.g., during hot reload in dev mode)
    let _ = CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::Info, Config::default(), log_file),
    ]);
}
