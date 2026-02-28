use reqwest::Client;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[cfg(not(feature = "dev-sidecar"))]
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
#[cfg(not(feature = "dev-sidecar"))]
use tauri_plugin_shell::ShellExt;

/// Wrapper so both dev (std::process::Child) and release (CommandChild) can be stored.
#[cfg(not(feature = "dev-sidecar"))]
type ChildHandle = CommandChild;
#[cfg(feature = "dev-sidecar")]
type ChildHandle = std::process::Child;

#[derive(Clone)]
pub struct SidecarState {
    pub port: Arc<Mutex<Option<u16>>>,
    pub child: Arc<Mutex<Option<ChildHandle>>>,
    pub client: Client,
}

impl SidecarState {
    pub fn new() -> Self {
        Self {
            port: Arc::new(Mutex::new(None)),
            child: Arc::new(Mutex::new(None)),
            client: Client::new(),
        }
    }
}

pub fn spawn_sidecar(app: AppHandle) -> Result<SidecarState, Box<dyn std::error::Error>> {
    let state = SidecarState::new();
    start_sidecar_process(&app, &state)?;
    start_health_check_loop(app, state.clone());
    Ok(state)
}

// ---------------------------------------------------------------------------
// Release mode: use Tauri shell plugin sidecar API
// ---------------------------------------------------------------------------
#[cfg(not(feature = "dev-sidecar"))]
fn start_sidecar_process(
    app: &AppHandle,
    state: &SidecarState,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut rx, child) = app
        .shell()
        .sidecar("aurotype-engine")
        .map_err(|err| format!("failed to create sidecar command: {err}"))?
        .spawn()
        .map_err(|err| format!("failed to spawn sidecar: {err}"))?;

    // Use a oneshot channel to receive the port from the async stdout reader.
    // This bridges the sync setup() context with the async sidecar output.
    let (port_tx, port_rx) = tokio::sync::oneshot::channel::<u16>();

    let port_arc = state.port.clone();
    tokio::spawn(async move {
        let mut port_tx = Some(port_tx);

        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    let line = line.trim();

                    // Try to parse the handshake JSON: {"port": N}
                    if let Some(tx) = port_tx.take() {
                        if let Ok(parsed) = serde_json::from_str::<Value>(line) {
                            if let Some(port) = parsed.get("port").and_then(Value::as_u64) {
                                if let Ok(p) = u16::try_from(port) {
                                    let mut guard = port_arc.lock().unwrap();
                                    *guard = Some(p);
                                    let _ = tx.send(p);
                                    continue;
                                }
                            }
                        }
                        eprintln!(
                            "[aurotype] Unexpected sidecar stdout before handshake: {line}"
                        );
                        port_tx = Some(tx);
                    }
                }
                CommandEvent::Stderr(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    eprint!("[aurotype engine] {line}");
                }
                CommandEvent::Terminated(payload) => {
                    eprintln!(
                        "[aurotype] Sidecar terminated: code={:?} signal={:?}",
                        payload.code, payload.signal
                    );
                    // If we never got the port, drop the sender so the receiver errors
                    drop(port_tx.take());
                    break;
                }
                CommandEvent::Error(err) => {
                    eprintln!("[aurotype] Sidecar stream error: {err}");
                }
                _ => {}
            }
        }
    });

    // Block on the oneshot receiver with a timeout to get the port synchronously
    let port = tauri::async_runtime::block_on(async {
        tokio::time::timeout(Duration::from_secs(15), port_rx).await
    })
    .map_err(|_| "timed out waiting for sidecar handshake (15s)")?
    .map_err(|_| "sidecar exited before emitting port")?;

    eprintln!("[aurotype] Sidecar started on port {port}");

    {
        let mut child_guard = state.child.lock().unwrap();
        *child_guard = Some(child);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Dev mode: spawn Python engine via `uv run` directly (no sidecar binary needed)
// ---------------------------------------------------------------------------
#[cfg(feature = "dev-sidecar")]
fn start_sidecar_process(
    _app: &AppHandle,
    state: &SidecarState,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};

    let mut child = Command::new("uv")
        .args(["run", "python", "-m", "aurotype_engine"])
        .current_dir("../engine")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("sidecar stdout was not captured"))?;
    let mut lines = BufReader::new(stdout).lines();
    let line = lines
        .next()
        .ok_or_else(|| std::io::Error::other("sidecar did not emit handshake line"))??;

    let port_value: Value = serde_json::from_str(&line)?;
    let raw_port = port_value
        .get("port")
        .and_then(Value::as_u64)
        .ok_or_else(|| std::io::Error::other("missing sidecar port in handshake"))?;
    let port = u16::try_from(raw_port)
        .map_err(|_| std::io::Error::other("sidecar port out of range"))?;

    eprintln!("[aurotype] Sidecar (dev) started on port {port}");

    {
        let mut port_guard = state.port.lock().unwrap();
        *port_guard = Some(port);
    }
    {
        let mut child_guard = state.child.lock().unwrap();
        *child_guard = Some(child);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// HTTP communication (shared between dev & release)
// ---------------------------------------------------------------------------

pub async fn sidecar_post(state: &SidecarState, path: &str, body: Value) -> Result<String, String> {
    let port = get_sidecar_port(state)?;
    let url = format!("http://127.0.0.1:{port}{path}");
    let response = state
        .client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|err| format!("sidecar POST failed: {err}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|err| format!("failed to read sidecar POST body: {err}"))?;

    if !status.is_success() {
        return Err(format!("sidecar POST returned {status}: {text}"));
    }

    Ok(text)
}

pub async fn sidecar_get(state: &SidecarState, path: &str) -> Result<String, String> {
    let port = get_sidecar_port(state)?;
    let url = format!("http://127.0.0.1:{port}{path}");
    let response = state
        .client
        .get(url)
        .send()
        .await
        .map_err(|err| format!("sidecar GET failed: {err}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|err| format!("failed to read sidecar GET body: {err}"))?;

    if !status.is_success() {
        return Err(format!("sidecar GET returned {status}: {text}"));
    }

    Ok(text)
}

fn get_sidecar_port(state: &SidecarState) -> Result<u16, String> {
    state
        .port
        .lock()
        .unwrap()
        .as_ref()
        .copied()
        .ok_or_else(|| "sidecar port not initialized".to_string())
}

// ---------------------------------------------------------------------------
// Health check, respawn, shutdown
// ---------------------------------------------------------------------------

fn start_health_check_loop(app: AppHandle, state: SidecarState) {
    tokio::spawn(async move {
        let mut consecutive_failures = 0u8;

        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            match sidecar_get(&state, "/health").await {
                Ok(_) => {
                    consecutive_failures = 0;
                }
                Err(err) => {
                    consecutive_failures = consecutive_failures.saturating_add(1);

                    if consecutive_failures >= 3 {
                        eprintln!(
                            "[aurotype] Sidecar health check failed {consecutive_failures} times, respawning..."
                        );
                        let _ = app.emit(
                            "sidecar-restarting",
                            serde_json::json!({ "reason": err, "failures": consecutive_failures }),
                        );
                        if let Err(respawn_err) = respawn_sidecar(&app, &state) {
                            let _ = app.emit(
                                "sidecar-error",
                                serde_json::json!({ "error": respawn_err }),
                            );
                        }
                        consecutive_failures = 0;
                    }
                }
            }
        }
    });
}

pub fn respawn_sidecar(app: &AppHandle, state: &SidecarState) -> Result<(), String> {
    stop_current_sidecar(state);
    start_sidecar_process(app, state).map_err(|err| format!("failed to respawn sidecar: {err}"))
}

pub fn shutdown_sidecar(state: &SidecarState) {
    stop_current_sidecar(state);
    let mut port_guard = state.port.lock().unwrap();
    *port_guard = None;
}

#[cfg(not(feature = "dev-sidecar"))]
fn stop_current_sidecar(state: &SidecarState) {
    let maybe_child = {
        let mut child_guard = state.child.lock().unwrap();
        child_guard.take()
    };

    if let Some(child) = maybe_child {
        if let Err(err) = child.kill() {
            eprintln!("[aurotype] Failed to kill sidecar: {err}");
        }
    }
}

#[cfg(feature = "dev-sidecar")]
fn stop_current_sidecar(state: &SidecarState) {
    let maybe_child = {
        let mut child_guard = state.child.lock().unwrap();
        child_guard.take()
    };

    if let Some(mut child) = maybe_child {
        // Graceful shutdown: try SIGTERM first, then kill
        send_sigterm(&child);

        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            match child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) => std::thread::sleep(Duration::from_millis(100)),
                Err(_) => break,
            }
        }

        let _ = child.kill();
        let _ = child.wait();
    }
}

#[cfg(all(feature = "dev-sidecar", unix))]
fn send_sigterm(child: &std::process::Child) {
    let _ = std::process::Command::new("kill")
        .arg("-TERM")
        .arg(child.id().to_string())
        .status();
}

#[cfg(all(feature = "dev-sidecar", not(unix)))]
fn send_sigterm(child: &std::process::Child) {
    let _ = std::process::Command::new("taskkill")
        .args(["/PID", &child.id().to_string(), "/T"])
        .status();
}

#[tauri::command]
pub async fn get_health(sidecar: tauri::State<'_, SidecarState>) -> Result<String, String> {
    sidecar_get(&sidecar, "/health").await
}
