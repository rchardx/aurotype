use reqwest::Client;
use serde_json::Value;
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Clone)]
pub struct SidecarState {
    pub port: Arc<Mutex<Option<u16>>>,
    pub child: Arc<Mutex<Option<Child>>>,
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

pub fn spawn_sidecar(app: AppHandle) -> Result<SidecarState, Box<dyn Error>> {
    let state = SidecarState::new();
    start_sidecar_process(&state)?;
    start_health_check_loop(app, state.clone());
    Ok(state)
}

fn start_sidecar_process(state: &SidecarState) -> Result<(), Box<dyn Error>> {
    let mut child = Command::new("uv")
        .args(["run", "python", "-m", "aurotype_engine"])
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
                        let _ = app.emit(
                            "sidecar-restarting",
                            serde_json::json!({ "reason": err, "failures": consecutive_failures }),
                        );
                        if let Err(respawn_err) = respawn_sidecar(&state) {
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

pub fn respawn_sidecar(state: &SidecarState) -> Result<(), String> {
    stop_current_sidecar(state);
    start_sidecar_process(state).map_err(|err| format!("failed to respawn sidecar: {err}"))
}

pub fn shutdown_sidecar(state: &SidecarState) {
    stop_current_sidecar(state);
    let mut port_guard = state.port.lock().unwrap();
    *port_guard = None;
}

fn stop_current_sidecar(state: &SidecarState) {
    let maybe_child = {
        let mut child_guard = state.child.lock().unwrap();
        child_guard.take()
    };

    if let Some(mut child) = maybe_child {
        terminate_child(&mut child);
    }
}

fn terminate_child(child: &mut Child) {
    send_sigterm(child);

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

#[cfg(unix)]
fn send_sigterm(child: &Child) {
    let _ = Command::new("kill")
        .arg("-TERM")
        .arg(child.id().to_string())
        .status();
}

#[cfg(not(unix))]
fn send_sigterm(child: &Child) {
    let _ = Command::new("taskkill")
        .args(["/PID", &child.id().to_string(), "/T"])
        .status();
}

#[tauri::command]
pub async fn get_health(sidecar: tauri::State<'_, SidecarState>) -> Result<String, String> {
    sidecar_get(&sidecar, "/health").await
}
