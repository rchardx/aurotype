# AGENTS.md — Aurotype

Voice-to-text desktop app: Tauri 2 (Rust) shell → React/TypeScript frontend → Python sidecar engine (FastAPI).
The Rust layer manages hotkeys, window lifecycle, and clipboard injection. The Python engine handles audio recording, STT, and LLM polishing.

## Architecture

```
src/              → React 19 + TypeScript frontend (Vite 7)
src-tauri/src/    → Rust/Tauri 2 backend (hotkeys, tray, sidecar management, text injection)
engine/           → Python 3.12 sidecar (FastAPI + uvicorn, STT/LLM providers)
tests/            → Python tests (pytest, unittest.mock)
```

The Tauri app spawns the Python engine as a sidecar process. Communication is via HTTP (localhost, dynamic port). The engine outputs `{"port": N}` on stdout at startup for handshake.

## Build & Run Commands

```bash
# Install all dependencies (frontend + Python)
make setup                    # bun install && cd engine && uv sync

# Run full app (Tauri + Vite + sidecar)
make dev                      # bun run tauri dev

# Run Python engine standalone
make engine-dev               # cd engine && uv run python -m aurotype_engine

# TypeScript type-check + build
bun run build                 # tsc && vite build

# Rust build (from src-tauri/)
cargo build                   # in src-tauri/
```

## Test Commands

```bash
# Run all Python tests
cd engine && uv run pytest ../tests/ -v

# Run a single test file
cd engine && uv run pytest ../tests/test_stt_providers.py -v

# Run a single test function
cd engine && uv run pytest ../tests/test_stt_providers.py::test_deepgram_transcribe_uses_mocked_httpx -v

# Rust tests
cd src-tauri && cargo test
```

Tests use `pytest` with `unittest.mock` (AsyncMock, MagicMock, patch). No conftest.py or fixtures file — helpers are defined per test file (e.g., `_build_config()`, `_mock_async_client()`). Tests import engine modules via `sys.path.insert` or dynamic `import_module`/`getattr`.

## Lint & Format

No linter or formatter is currently configured (no ESLint, Prettier, Ruff, or Clippy configs). TypeScript strict mode is enabled in `tsconfig.json` with `noUnusedLocals` and `noUnusedParameters`.

When adding linting, match these existing implicit standards rather than introducing new rules.

## Code Style — TypeScript / React (src/)

### File Naming
- **Components**: PascalCase — `FloatWindow.tsx`, `Settings.tsx`, `App.tsx`
- **Entry points**: camelCase — `main.tsx`, `float.tsx`
- **Styles**: Component-paired CSS — `App.css`, `Settings.css`, `float.css`

### Imports
Order: React/external libs → Tauri APIs → local components → CSS (last).
```tsx
import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import FloatWindow from './FloatWindow.tsx';
import './float.css';
```
Both single and double quotes appear — double quotes in some files (`main.tsx`, `Settings.tsx`), single quotes in others (`FloatWindow.tsx`, `float.tsx`). No enforced rule.

### Components
- Functional components only, exported as `export default function ComponentName()`.
- State via `useState`, side effects via `useEffect`, refs via `useRef`.
- No external state management library — local state plus Tauri `invoke`/`listen` for backend communication.

### Types
- `interface` for object shapes: `interface SettingsData { ... }`, `interface StateChangedPayload { ... }`
- `type` for unions: `type AppState = 'idle' | 'recording' | 'processing' | 'injecting' | 'error' | 'done';`
- Generic usage with Tauri APIs: `invoke<string>(...)`, `listen<StateChangedPayload>(...)`

### Error Handling
```tsx
try {
  const state = await invoke<string>('get_state');
} catch (e) {
  console.error('Failed to get initial state:', e);
}
```
Errors logged with `console.error` or `console.warn`. No toast/notification system — errors are reflected in component state (e.g., `setErrorMessage`).

### Styling
Plain CSS with `className`. No CSS modules, Tailwind, or CSS-in-JS. Dark theme in `Settings.css`, system preference media query in `App.css`.

## Code Style — Python (engine/)

### File Naming
- `snake_case.py` for all files: `stt_deepgram.py`, `llm_registry.py`, `pipeline.py`
- Providers follow `{layer}_{name}.py` pattern: `stt_base.py`, `llm_openai.py`

### Imports
Order: `__future__` → stdlib → third-party → relative local.
```python
from __future__ import annotations
import io
import threading
import wave
from typing import ClassVar, Protocol, cast

import numpy as np
from numpy.typing import NDArray

from .config import Settings
from .providers.stt_registry import get_stt_provider
```
Relative imports within the `aurotype_engine` package (e.g., `from .stt_base import STTProvider`).

### Type Hints
- Extensive throughout. Uses Python 3.12 syntax: `str | None`, `dict[str, str]`.
- `typing.Protocol` for dependency injection (config interfaces defined per-provider).
- `typing.override` decorator on abstract method implementations.
- `typing.Final` for immutable instance attributes.
- `typing.cast` when type narrowing is needed.
- Some files suppress pyright warnings at file level:
  ```python
  # pyright: reportMissingImports=false, reportUnknownVariableType=false
  ```

### Classes
- Abstract base classes via `abc.ABC` + `@abc.abstractmethod` for providers.
- Pydantic `BaseSettings` for config, `BaseModel` for request/response schemas.
- Provider pattern: base class → concrete implementations → registry dict + factory function.

```python
# Base
class STTProvider(abc.ABC):
    @abc.abstractmethod
    async def transcribe(self, audio_bytes: bytes, language: str = "auto") -> str: ...

# Registry
STT_PROVIDER_REGISTRY: dict[str, Callable[[STTConfig], STTProvider]] = {
    "deepgram": DeepgramSTTProvider,
    "dashscope": DashScopeSTTProvider,
}

def get_stt_provider(name: str, config: STTConfig) -> STTProvider:
    provider_cls = STT_PROVIDER_REGISTRY.get(name)
    if provider_cls is None:
        raise ValueError(f"Unknown STT provider: {name}")
    return provider_cls(config)
```

### Error Handling
- Custom exception hierarchy: `AudioRecorderError(RuntimeError)` → `AudioDeviceError`.
- HTTP errors: `raise HTTPException(status_code=500, detail=str(exc)) from exc`.
- External API calls wrapped in `try/except httpx.HTTPError`, re-raised as `RuntimeError`.
- Logging via `print()` with `[aurotype]` prefix (no logging module).

### Naming
- Functions/variables: `snake_case` — `get_settings`, `process_voice_input`
- Classes: `PascalCase` — `DeepgramSTTProvider`, `AudioRecorder`
- Constants: `UPPER_SNAKE_CASE` — `SYSTEM_PROMPT`, `STT_PROVIDER_REGISTRY`
- Private/helper: leading underscore — `_config_overrides`, `_build_config()`

## Code Style — Rust (src-tauri/)

### Modules
One file per concern: `state.rs`, `hotkey.rs`, `sidecar.rs`, `injection.rs`, `tray.rs`.
Declared in `lib.rs` via `mod hotkey;` etc.

### Imports
Order: std → external crates → local crate modules.
```rust
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use crate::state::{AppState, AppStateManager};
```

### Error Handling
- `Result<T, String>` for Tauri commands (Tauri requires string errors).
- `Result<(), Box<dyn std::error::Error>>` for internal functions.
- `eprintln!("[aurotype] ...")` for error logging.
- State transitions to `AppState::Error(msg)` with 3-second display before returning to `Idle`.

### Naming
- Functions: `snake_case` — `run_pipeline`, `spawn_sidecar`
- Structs/Enums: `PascalCase` — `AppState`, `SidecarState`
- Tauri commands: `#[tauri::command]` on `snake_case` functions

### Patterns
- `Arc<Mutex<T>>` for shared mutable state.
- `tokio::spawn` for async background tasks.
- Platform-specific code via `#[cfg(target_os = "...")]` modules.

## Key Files Reference

| Purpose | File |
|---|---|
| Tauri app setup & commands | `src-tauri/src/lib.rs` |
| App state machine | `src-tauri/src/state.rs` |
| Hotkey handling | `src-tauri/src/hotkey.rs` |
| Python sidecar lifecycle | `src-tauri/src/sidecar.rs` |
| Text injection (clipboard) | `src-tauri/src/injection.rs` |
| FastAPI server | `engine/aurotype_engine/server.py` |
| Voice pipeline (STT→LLM) | `engine/aurotype_engine/pipeline.py` |
| Audio recording | `engine/aurotype_engine/audio.py` |
| Provider base classes | `engine/aurotype_engine/providers/{stt,llm}_base.py` |
| Provider registries | `engine/aurotype_engine/providers/{stt,llm}_registry.py` |
| React settings UI | `src/Settings.tsx` |
| React float overlay | `src/FloatWindow.tsx` |
| Vite config (multi-page) | `vite.config.ts` |
| PyInstaller spec (onefile) | `engine/aurotype-engine.spec` |
| Full build script | `build.ps1` |
| Playwright config | `playwright.config.ts` |
| Playwright GUI tests | `e2e/gui-debug.spec.ts` |
| Debug launch script | `debug-run.ps1` |
| Vite dev server launcher | `start-vite.ps1` |

## Dev & Debug Workflow (Windows)

This project is a Tauri desktop app — the frontend runs inside a native webview, NOT a regular browser. You cannot simply open `localhost:1420` in Chrome to see the real app. However, for **UI development and visual inspection**, you can use Vite dev server + Playwright to screenshot and verify the React frontend in isolation.

### Three Debug Layers

| Layer | What to test | Method |
|---|---|---|
| Frontend UI/CSS | Layout, styling, form elements | Vite dev server + Playwright screenshots |
| Python sidecar | Engine startup, API endpoints | `make engine-dev` or run sidecar exe directly |
| Full integrated app | Hotkeys, tray, sidecar lifecycle | `debug-run.ps1` (launch release exe, capture stderr) |

### Layer 1: Frontend UI Inspection via Playwright

Use this when you need to **see** the UI — verify layout changes, check CSS, inspect form elements.

**Why Playwright instead of a browser?** Tauri webview ≠ browser. Vite serves the same React code, so Playwright screenshots are faithful for UI verification. Tauri `invoke()` calls will fail (no Rust backend), but the DOM/CSS renders identically.

#### Step 1: Start Vite dev server

```powershell
powershell -ExecutionPolicy Bypass -File start-vite.ps1
```

This starts Vite on `http://127.0.0.1:1420` in the background. Key detail: **must bind to `127.0.0.1`** explicitly — Vite defaults to IPv6 `::1` on Windows, which Playwright cannot reach.

The script sets `TAURI_DEV_HOST=127.0.0.1` and passes `--host 127.0.0.1` to Vite. It polls until the server responds.

#### Step 2: Run Playwright to capture screenshots

```bash
bunx playwright test e2e/gui-debug.spec.ts
```

Screenshots are saved to `e2e/screenshots/`. The spec captures:
- `settings-full.png` — Full settings page
- `section-*.png` — Each `.section` element individually
- `float-window.png` — The floating overlay window

#### Step 3: Inspect screenshots

Use the `look_at` tool on `e2e/screenshots/*.png` to visually verify the UI.

#### Step 4: Clean up

```powershell
powershell -Command "Get-Process -Name 'bun' -ErrorAction SilentlyContinue | Stop-Process -Force"
```

#### Gotchas

- Playwright must be installed first: `bun add -D @playwright/test && bunx playwright install chromium`
- `playwright.config.ts` has `webServer: undefined` — you manage the dev server manually
- Tauri API calls (`invoke`, `listen`) will throw in Playwright since there's no Rust backend. This is expected. The UI still renders correctly for visual inspection.
- If Vite appears to start but Playwright can't connect, check `vite-dev.log` and `vite-dev-err.log` in project root.

### Layer 2: Python Sidecar Debugging

For engine-only issues (API errors, provider crashes, import failures):

```bash
# Run engine directly from source (no PyInstaller)
cd engine && uv run python -m aurotype_engine

# Or run the PyInstaller-built binary directly
engine\dist\aurotype-engine.exe
```

The engine prints `{"port": N}` on stdout at startup. If it crashes, the traceback goes to stderr. Common failure: `ModuleNotFoundError` in PyInstaller builds due to missing `hiddenimports` in `aurotype-engine.spec`.

### Layer 3: Full App Debug (Release Exe)

For integrated testing (sidecar spawning, hotkeys, tray icon):

```powershell
powershell -ExecutionPolicy Bypass -File debug-run.ps1
```

This script:
1. Launches `src-tauri\target\release\tauri-app.exe`
2. Redirects stdout/stderr to temp log files
3. Waits 8 seconds
4. Reports whether the process is still alive or exited (with exit code)
5. Dumps both logs

**Interpreting results:**
- "Process still running" + "Sidecar started on port XXXX" = SUCCESS
- "Process EXITED" + Python traceback in stderr = sidecar crash (fix the Python error)
- "Process EXITED" + Rust panic in stderr = Tauri setup failure

**Important:** After running `debug-run.ps1`, always kill leftover processes before rebuilding:
```powershell
powershell -Command "Get-Process -Name 'aurotype-engine','tauri-app' -ErrorAction SilentlyContinue | Stop-Process -Force"
```
Otherwise `cargo build` / `tauri build` will fail with `PermissionDenied` on locked exe files.

### Build Pipeline

#### Quick iteration (no Python changes)

If only frontend (src/) or Rust (src-tauri/) changed:

```bash
bun run tauri build        # ~40s — recompiles Rust + bundles frontend
```

The sidecar in `src-tauri\binaries\` is reused as-is.

#### Full rebuild (Python changed)

```powershell
powershell -ExecutionPolicy Bypass -File build.ps1
```

This runs: PyInstaller (onefile) → copy exe to `src-tauri\binaries\` → `bun run tauri build`.

Output: `src-tauri\target\release\bundle\nsis\Aurotype_0.1.0_x64-setup.exe`

#### Direct exe testing (skip installer)

After any `tauri build`, you can run the exe directly without installing:

```
src-tauri\target\release\tauri-app.exe
```

Tauri resolves the sidecar from the same directory (`src-tauri\target\release\aurotype-engine.exe`). The build copies it there automatically.

**Shortcut for sidecar-only fix:** If you only changed Python code and want to test without a full `tauri build`:

```powershell
# Rebuild sidecar
cd engine && uv run pyinstaller aurotype-engine.spec --noconfirm && cd ..
# Copy to release dir (skip tauri build)
Copy-Item engine\dist\aurotype-engine.exe src-tauri\target\release\aurotype-engine.exe -Force
# Test
powershell -ExecutionPolicy Bypass -File debug-run.ps1
```

### PyInstaller Notes

- **Mode**: Onefile (`a.binaries` and `a.datas` passed directly to `EXE`, no `COLLECT` block).
- **Spec file**: `engine/aurotype-engine.spec`. When adding new Python dependencies, add them to `hiddenimports`.
- **Common failure**: `ModuleNotFoundError` at runtime — means a module is missing from `hiddenimports` or `datas` in the spec file.
- **Output path**: `engine/dist/aurotype-engine.exe` (single file, ~35MB).

### Windows-Specific Caveats

1. **PowerShell variable escaping**: Bash tool runs commands through `/usr/bin/bash`. PowerShell `$_`, `$()`, `{}` get mangled. For anything non-trivial, write a `.ps1` script file and invoke it with `powershell -ExecutionPolicy Bypass -File script.ps1`.
2. **IPv6 binding**: Vite binds to `::1` (IPv6) by default on Windows. Playwright and `Invoke-WebRequest` use IPv4 `127.0.0.1`. Always pass `--host 127.0.0.1` to Vite.
3. **File locking**: Windows locks running executables. Always kill `aurotype-engine` and `tauri-app` processes before rebuilding.
4. **Process management**: Use `Start-Process -PassThru` + `Stop-Process` pattern. No `tmux` on Windows — use PowerShell background processes or script files.

## 交互规则

- 需要提问时，尽量使用 question 工具来反问。
