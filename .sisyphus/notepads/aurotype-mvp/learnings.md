- Task 1 scaffolded with .
- WSL non-sudo environment required local shims for  and  to complete Bun install and Tauri cargo check.
- Python engine uses hatchling build backend and explicit  dependency.
- Task 1 scaffolded with bun create tauri-app . --template react-ts --manager bun --yes --force --tauri-version 2.
- WSL non-sudo environment required local shims for unzip and pkg-config to complete Bun install and Tauri cargo check.
- Python engine uses hatchling build backend and explicit pydantic-settings dependency.
- Task 2 FastAPI sidecar implemented with: socket.bind(('', 0)) for port discovery, parent PID monitoring in daemon thread, CORS for Tauri+localhost, pydantic-settings with AUROTYPE_ env prefix, placeholder endpoints (/transcribe, /polish, /process), uvicorn with log_level="warning" to preserve stdout JSON port line.

## Task 3: Test WAV Audio File Generation (2026-02-28)

### Key Learnings
1. **WAV Generation with Python stdlib**: Use Python's built-in `wave` module for WAV file creation—no external audio libraries needed
2. **16-bit PCM Format**: For STT pipelines, 16-bit PCM is standard (sample width = 2 bytes). Use `w.setsampwidth(2)`
3. **Mono Audio**: STT typically uses mono; set with `w.setnchannels(1)`
4. **Sample Rate**: 16kHz is standard for speech recognition (matches common STT requirements)
5. **Sine Wave Generation**: Use `np.sin(2 * np.pi * freq * sample_indices / sample_rate)` for clean tone generation
6. **Int16 Conversion**: Scale float samples [-1, 1] to int16 range: `(samples * 32767).astype(np.int16)`

### Technical Details
- **5s file**: 160KB (5 × 16000 × 2 bytes per sample)
- **1s file**: 32KB (1 × 16000 × 2 bytes per sample)
- **Tone**: 440 Hz (A note) chosen for clarity and standard reference
- **Generation method**: Via uv-managed Python environment in engine directory to access numpy

### Process Notes
1. Created `tests/` directory at project root
2. Created `tests/generate_test_audio.py` with reusable function signature
3. Ran via: `cd engine && uv run python ../tests/generate_test_audio.py`
4. Validated format with Python wave module introspection
5. Files well under size limits (both <200KB vs. typical 1MB limits)

### Files Created
- `tests/generate_test_audio.py` (46 lines)
- `tests/test_audio.wav` (157KB, 5s)
- `tests/test_audio_short.wav` (32KB, 1s)
- `.sisyphus/evidence/task-3-format.txt` (validation report)

### Commit
Message: "test: add WAV test audio files for STT pipeline testing"
Commit: 9f08c6a (3 files changed, 45 insertions)

Ready for: STT pipeline integration testing in subsequent tasks.

## Task 4: Audio Capture Module (2026-02-28)

- In WSL2, `sounddevice` may install but fail import at runtime if PortAudio shared library is unavailable; keep module importable by deferring hard failure to `start_recording()` and surfacing a descriptive `AudioDeviceError`.
- `sounddevice` callback should copy `indata` immediately and flatten mono frames before storing to avoid thread-timing/data-mutation issues.
- For byte output, `numpy.concatenate()` + `wave` over `io.BytesIO` produces STT-ready WAV (mono, 16kHz, 16-bit PCM) without touching disk.
- `stop_recording()` should always return a valid WAV header even when no audio chunks exist (44-byte empty PCM WAV).
- RMS volume reporting is stable when computed from the latest chunk only: `sqrt(mean(chunk.astype(float32)**2)) / 32768` and clipped to `[0.0, 1.0]`.

## Task 5: STT Provider Abstraction (2026-02-28)

- OpenAI-compatible STT APIs from Groq and SiliconFlow both accept the same multipart schema: `file`, `model`, and optional `language`.
- Keeping provider response handling permissive (`text` missing or non-string returns `""`) avoids unnecessary runtime failures for empty transcripts.
- A simple constructor registry (`dict[str, ProviderClass]`) is enough for provider selection and keeps Settings-based wiring testable.
- Provider tests can stay offline by patching `httpx.AsyncClient` context manager and using `AsyncMock` for `post`.

## Task 8: Text Injection Module (2026-02-28)

- Implemented `src-tauri/src/injection.rs` with `capture_foreground_window()`, `inject_text(text: &str)`, and Tauri command `inject_text_at_cursor(text: String)`.
- Injection flow is clipboard-first: save current text clipboard as `Option<String>`, set new text, attempt re-focus of captured window, simulate paste shortcut, sleep 100ms, and restore clipboard only if prior text existed.
- Linux/WSL2 focus behavior is cfg-gated no-op for local compilation; Windows path uses direct `user32` FFI (`GetForegroundWindow`/`SetForegroundWindow`) without adding extra crates.
- `lib.rs` invoke handler now includes `injection::inject_text_at_cursor` so frontend can call it directly.

## Task 6: LLM Provider Abstraction (2026-02-28)

- Reusing the OpenAI SDK for both OpenAI and SiliconFlow keeps implementation symmetric; only `api_key`, `base_url`, and model differ.
- Keeping a shared `SYSTEM_PROMPT` in the LLM base module guarantees prompt consistency across providers and prevents drift.
- A constructor registry (`dict[str, ProviderClass]`) plus a passthrough `none` provider gives deterministic behavior for local/offline mode while preserving the same provider interface.


## Task 7: Global Hotkey, System Tray, and State Machine (2026-02-28)

### Key Learnings
- Tauri v2 `tray` module requires `tray-icon` feature on the `tauri` crate in Cargo.toml: `tauri = { version = "2", features = ["tray-icon"] }`
- `tauri::Manager` trait must be in scope to call `.state::<T>()` on `AppHandle` inside closures (e.g., global shortcut handler)
- `TrayIconBuilder::on_menu_event` closure needs explicit type annotations for `app: &AppHandle` and `event: tauri::menu::MenuEvent` when type inference fails
- Global shortcut plugin must be registered via `app.plugin()` (not on the builder directly) and then shortcuts registered via `app.global_shortcut().register()` in `setup`
- `ShortcutState::Pressed` and `ShortcutState::Released` enable hold-to-record mode without needing separate press/release tracking
- Capabilities/permissions for Tauri v2 plugins live in `src-tauri/capabilities/default.json` (not in `tauri.conf.json` `security` block)
- `Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Space)` maps to CmdOrCtrl+Shift+Space cross-platform
- Escape key shortcut: `Shortcut::new(None, Code::Escape)` — no modifiers needed

### Architecture Decisions
- State machine uses `Arc<Mutex<AppState>>` in `AppStateManager` struct managed via Tauri `.manage()`
- Two hotkey modes: `Toggle` (default, press toggles) and `HoldToRecord` (hold to record, release to process)
- Frontend events emitted via `app_handle.emit("state-changed", json)` with `{"state": string, "message": Option<String>}`
- Tray menu: "Settings" (shows/focuses main window) and "Quit" (exits app)

### Files Created/Modified
- `src-tauri/src/state.rs` — AppState enum, HotkeyMode enum, AppStateManager struct
- `src-tauri/src/hotkey.rs` — Global hotkey registration with toggle/hold-to-record modes + Escape cancellation
- `src-tauri/src/tray.rs` — System tray with Settings/Quit menu items
- `src-tauri/src/lib.rs` — Wired up state, hotkey, tray, + 4 Tauri commands (get_state, start_recording, stop_recording, cancel)
- `src-tauri/Cargo.toml` — Added tauri-plugin-global-shortcut and tauri-plugin-shell deps, tray-icon feature
- `src-tauri/capabilities/default.json` — Added global-shortcut and shell permissions

## Task 9: Engine Pipeline Integration (2026-02-28)

- FastAPI file upload endpoints require `python-multipart`; without it, app startup fails at route registration time for `UploadFile` + `File(...)`.
- Keeping `AudioRecorder` and `Settings` as module-level singletons in `server.py` makes recording state and provider config consistent across requests.
- Pipeline orchestration is simplest as one async function (`process_voice_input`) that serially awaits STT then LLM and returns a two-field dict.
- WSL/CI environments without PortAudio should map `/record/start` to HTTP 500 with the underlying `AudioDeviceError` message, while `/record/stop` remains safe/idempotent.
