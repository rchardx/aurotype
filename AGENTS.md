# AGENTS.md — Aurotype

Voice-to-text desktop app: Tauri 2 (Rust) shell + React/TypeScript frontend + Python sidecar engine (FastAPI).

## Architecture

```
src/              → React 19 + TypeScript frontend (Vite 7)
src-tauri/src/    → Rust/Tauri 2 backend (hotkeys, tray, sidecar, text injection)
engine/           → Python 3.12 sidecar (FastAPI + uvicorn, STT/LLM providers)
tests/            → Python tests (pytest, unittest.mock)
```

Tauri spawns the Python engine as a sidecar. Communication via HTTP (localhost, dynamic port). Engine outputs `{"port": N}` on stdout at startup.

## Build & Run

```bash
make setup                    # bun install && cd engine && uv sync
make dev                      # bun run tauri dev (full app)
make engine-dev               # Python engine standalone
bun run build                 # tsc && vite build
cd src-tauri && cargo build   # Rust only
```

## Test Commands

```bash
# Python — all tests
cd engine && uv run pytest ../tests/ -v

# Python — single file
cd engine && uv run pytest ../tests/test_pipeline.py -v

# Python — single function
cd engine && uv run pytest ../tests/test_pipeline.py::test_happy_path -v

# TypeScript type-check
bunx tsc --noEmit

# Rust
cd src-tauri && cargo test
cd src-tauri && cargo clippy -- -D warnings
```

## Lint & Format

No linter/formatter configured. TypeScript strict mode with `noUnusedLocals` and `noUnusedParameters` in `tsconfig.json`. Match existing implicit standards.

## Code Style — TypeScript / React (src/)

- **Files**: PascalCase components (`FloatWindow.tsx`), camelCase entry points (`main.tsx`), paired CSS (`App.css`)
- **Exports**: `export default function ComponentName()`
- **State**: `useState`, `useEffect`, `useRef` — no external state library
- **Types**: `interface` for objects, `type` for unions. Generics with Tauri: `invoke<string>(...)`, `listen<Payload>(...)`
- **Imports**: React/external → Tauri APIs → local components → CSS (last)
- **Errors**: `try/catch` with `console.error`. No toast system — errors go to component state
- **Styling**: Plain CSS with `className`. No Tailwind, CSS modules, or CSS-in-JS

## Code Style — Python (engine/)

- **Files**: `snake_case.py`. Providers: `{layer}_{name}.py` (e.g., `stt_deepgram.py`, `llm_openai.py`)
- **Imports**: `__future__` → stdlib → third-party → relative local. Use relative imports within `aurotype_engine`
- **Types**: Python 3.12 syntax (`str | None`, `dict[str, str]`). `Protocol` for DI, `override` on implementations, `Final` for immutables, `cast` for narrowing
- **Classes**: `abc.ABC` + `@abstractmethod` for provider bases. Pydantic `BaseSettings` for config, `BaseModel` for schemas
- **Provider pattern**: base class → concrete implementations → registry dict → factory function
- **Errors**: Custom hierarchy (`AudioRecorderError(RuntimeError)` → `AudioDeviceError`). HTTP: `raise HTTPException(status_code=500, detail=str(exc)) from exc`. External APIs: `try/except httpx.HTTPError` re-raised as `RuntimeError`
- **Logging**: `print()` with `[aurotype]` prefix (no logging module)
- **Naming**: `snake_case` functions, `PascalCase` classes, `UPPER_SNAKE_CASE` constants, `_leading_underscore` for private

## Code Style — Rust (src-tauri/)

- **Modules**: One file per concern (`state.rs`, `hotkey.rs`, `sidecar.rs`, `injection.rs`, `tray.rs`), declared in `lib.rs`
- **Imports**: std → external crates → `crate::` local modules
- **Errors**: `Result<T, String>` for `#[tauri::command]` functions. `Result<(), Box<dyn std::error::Error>>` internally. `eprintln!("[aurotype] ...")` for logging
- **Patterns**: `Arc<Mutex<T>>` for shared state, `tokio::spawn` for async tasks, `#[cfg(target_os)]` for platform code
- **Naming**: `snake_case` functions, `PascalCase` structs/enums

## Test Conventions (Python)

- `unittest.mock` exclusively (AsyncMock, MagicMock, patch). No pytest fixtures or conftest.py
- Per-file helpers: `_build_config()`, `_mock_async_client()` etc.
- Imports via `sys.path.insert` or `import_module`/`getattr` for engine modules
- `SimpleNamespace` for mock config objects
- `asyncio.run()` to drive async tests
- Return type hints: `-> None` on all test functions

## Key Files

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
| App config (Pydantic) | `engine/aurotype_engine/config.py` |
| Provider bases | `engine/aurotype_engine/providers/{stt,llm}_base.py` |
| Provider registries | `engine/aurotype_engine/providers/{stt,llm}_registry.py` |
| React settings UI | `src/Settings.tsx` |
| React float overlay | `src/FloatWindow.tsx` |
| Vite config (multi-page) | `vite.config.ts` |
| PyInstaller spec | `engine/aurotype-engine.spec` |
| CI workflow | `.github/workflows/ci.yml` |
| Playwright GUI tests | `e2e/gui-debug.spec.ts` |

## Windows Dev Notes

- Frontend runs in Tauri webview, not a browser. Use Playwright for UI screenshots
- Vite must bind to `127.0.0.1` explicitly (IPv6 `::1` default breaks Playwright)
- Kill `aurotype-engine` and `tauri-app` processes before rebuilding (Windows file locking)
- PowerShell `$_`, `$()` get mangled in bash — use `.ps1` script files for non-trivial PS commands
- Local dev scripts (`*.ps1`) are gitignored and not tracked

## Interaction Rules

- When clarification is needed, prefer using the `question` tool to ask the user.
