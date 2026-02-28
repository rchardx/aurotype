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
    "siliconflow": SiliconFlowSTTProvider,
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
