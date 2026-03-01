# AGENTS.md — Aurotype

<!-- Tier 1: Quick Orientation — read every time -->

## Project Overview

Voice-to-text desktop app — speak naturally, get polished text injected where your cursor is. Tauri 2 (Rust) shell +
React/TypeScript frontend + Python sidecar engine (FastAPI).

**Tech stack**: Tauri 2, React 19, TypeScript, Vite 7 (frontend); Rust/tokio (backend); Python 3.12, FastAPI, uvicorn,
httpx, Pydantic (engine); DashScope/Deepgram (STT), DeepSeek/OpenAI-compatible (LLM).

**Purpose**: Desktop app for voice-to-text with push-to-talk hotkey, speech-to-text transcription, LLM polishing, and
smart text injection at cursor position. Modular provider pattern — STT and LLM providers are independently swappable.

**Communication**: Tauri spawns the Python engine as a sidecar. Communication via HTTP (localhost, dynamic port). Engine
outputs `{"port": N}` on stdout at startup.

## Quick Start

```bash
make setup                    # bun install && cd engine && uv sync
make dev                      # bun run tauri dev (full app)
make engine-dev               # Python engine standalone
bun run build                 # tsc && vite build
cd src-tauri && cargo build   # Rust only
# All checks (run before committing)
cd engine && uv run pytest ../tests/ -v && cd .. && bunx tsc --noEmit && cd src-tauri && cargo test && cargo clippy -- -D warnings
```

## Project Structure

```
src/              → React 19 + TypeScript frontend (Vite 7)
  Settings.tsx      Settings UI
  FloatWindow.tsx   Float overlay during recording
src-tauri/src/    → Rust/Tauri 2 backend
  lib.rs            App setup & Tauri commands
  state.rs          App state machine
  hotkey.rs         Hotkey handling
  sidecar.rs        Python sidecar lifecycle
  injection.rs      Text injection (clipboard)
  tray.rs           System tray
engine/           → Python 3.12 sidecar (FastAPI + uvicorn)
  aurotype_engine/
    server.py       FastAPI server
    pipeline.py     Voice pipeline (STT→LLM)
    audio.py        Audio recording
    config.py       App config (Pydantic BaseSettings)
    providers/
      stt_base.py     STT provider base
      llm_base.py     LLM provider base
      stt_registry.py STT provider registry
      llm_registry.py LLM provider registry
tests/            → Python tests (pytest, unittest.mock)
e2e/              → Playwright GUI tests
```

### Data Flow

- **Record**: Hotkey pressed → Rust state machine → start audio recording via engine
- **Transcribe**: Audio → STT provider (DashScope/Deepgram) → raw text
- **Polish**: Raw text → LLM provider (DeepSeek/OpenAI-compatible) → polished text
- **Inject**: Polished text → clipboard → simulate paste at cursor position

### Key Files

| Purpose                    | File                                                       |
| -------------------------- | ---------------------------------------------------------- |
| Tauri app setup & commands | `src-tauri/src/lib.rs`                                     |
| App state machine          | `src-tauri/src/state.rs`                                   |
| Hotkey handling            | `src-tauri/src/hotkey.rs`                                  |
| Python sidecar lifecycle   | `src-tauri/src/sidecar.rs`                                 |
| Text injection (clipboard) | `src-tauri/src/injection.rs`                               |
| FastAPI server             | `engine/aurotype_engine/server.py`                         |
| Voice pipeline (STT→LLM)  | `engine/aurotype_engine/pipeline.py`                       |
| Audio recording            | `engine/aurotype_engine/audio.py`                          |
| App config (Pydantic)      | `engine/aurotype_engine/config.py`                         |
| Provider bases             | `engine/aurotype_engine/providers/{stt,llm}_base.py`       |
| Provider registries        | `engine/aurotype_engine/providers/{stt,llm}_registry.py`   |
| React settings UI          | `src/Settings.tsx`                                         |
| React float overlay        | `src/FloatWindow.tsx`                                      |
| Vite config (multi-page)   | `vite.config.ts`                                           |
| PyInstaller spec           | `engine/aurotype-engine.spec`                              |
| CI workflow                | `.github/workflows/ci.yml`                                 |
| Playwright GUI tests       | `e2e/gui-debug.spec.ts`                                    |

<!-- Tier 2: Development Standards — reference when writing code -->

## Boundaries

### Always Do

- Read relevant files before modifying code.
- Run all checks before committing (see [Quick Start](#quick-start)).
- Follow existing code patterns in the same module.
- Add tests for new functionality.

### Ask First

- Adding new dependencies to `package.json` or `pyproject.toml` or `Cargo.toml`.
- Modifying provider base classes (`stt_base.py`, `llm_base.py`) — breaking change to all implementations.
- Changing `Settings` / config structure in `engine/aurotype_engine/config.py`.
- Deleting or renaming public APIs.

### Never Do

- `as any`, `@ts-ignore`, `@ts-expect-error` in TypeScript.
- Bare `except:` or `except Exception:` without re-raise/log.
- Suppress Rust warnings with `#[allow(...)]` without justification.
- Hardcode secrets, API keys, or endpoints.
- Add `Co-authored-by` trailers or attribution footers to git commits.

## Code Conventions

### TypeScript / React (`src/`)

| Rule         | Standard                                                                                       |
| ------------ | ---------------------------------------------------------------------------------------------- |
| Files        | PascalCase components (`FloatWindow.tsx`), camelCase entry points (`main.tsx`), paired CSS      |
| Exports      | `export default function ComponentName()`                                                      |
| State        | `useState`, `useEffect`, `useRef` — no external state library                                  |
| Types        | `interface` for objects, `type` for unions. Generics with Tauri: `invoke<string>(...)`         |
| Imports      | React/external → Tauri APIs → local components → CSS (last)                                    |
| Errors       | `try/catch` with `console.error`. No toast system — errors go to component state               |
| Styling      | Plain CSS with `className`. No Tailwind, CSS modules, or CSS-in-JS                             |
| Lint         | No linter configured. TypeScript strict mode with `noUnusedLocals` and `noUnusedParameters`    |

### Python (`engine/`)

| Rule             | Standard                                                                                       |
| ---------------- | ---------------------------------------------------------------------------------------------- |
| Files            | `snake_case.py`. Providers: `{layer}_{name}.py` (e.g., `stt_deepgram.py`, `llm_openai.py`)    |
| Imports          | `__future__` → stdlib → third-party → relative local. Relative imports within `aurotype_engine` |
| Types            | Python 3.12 syntax (`str \| None`, `dict[str, str]`). `Protocol` for DI, `override` on impls  |
| Classes          | `abc.ABC` + `@abstractmethod` for provider bases. Pydantic `BaseSettings` for config           |
| Provider pattern | base class → concrete implementations → registry dict → factory function                       |
| Errors           | Custom hierarchy (`AudioRecorderError(RuntimeError)` → `AudioDeviceError`). HTTP: `raise HTTPException(...)` from exc. External APIs: `try/except httpx.HTTPError` re-raised as `RuntimeError` |
| Logging          | `print()` with `[aurotype]` prefix (no logging module)                                         |

| Naming    | Pattern            | Example                              |
| --------- | ------------------ | ------------------------------------ |
| Functions | `snake_case`       | `get_quote`, `transcribe`            |
| Classes   | `PascalCase`       | `DashScopeSTT`, `AudioRecorder`      |
| Constants | `UPPER_SNAKE_CASE` | `DEFAULT_TIMEOUT`                    |
| Private   | `_` prefix         | `self._settings`                     |

### Rust (`src-tauri/`)

| Rule    | Standard                                                                                               |
| ------- | ------------------------------------------------------------------------------------------------------ |
| Modules | One file per concern (`state.rs`, `hotkey.rs`, `sidecar.rs`, `injection.rs`, `tray.rs`), in `lib.rs`  |
| Imports | std → external crates → `crate::` local modules                                                       |
| Errors  | `Result<T, String>` for `#[tauri::command]`. `Result<(), Box<dyn std::error::Error>>` internally       |
| Logging | `eprintln!("[aurotype] ...")`                                                                          |
| Patterns | `Arc<Mutex<T>>` for shared state, `tokio::spawn` for async tasks, `#[cfg(target_os)]` for platform   |
| Naming  | `snake_case` functions, `PascalCase` structs/enums                                                     |

## Design Patterns

- **Provider pattern**: STT and LLM are independently swappable via base class → concrete impl → registry → factory.
- **Sidecar architecture**: Tauri (Rust) spawns Python engine as a child process, communicates via HTTP on localhost.
- **State machine**: Rust `state.rs` manages app states (Idle → Recording → Processing → Injecting).
- **Clipboard injection**: Text injection via clipboard + simulated paste, with fallback behavior.

## Testing

### Python Tests

- `unittest.mock` exclusively (AsyncMock, MagicMock, patch). No pytest fixtures or conftest.py.
- Per-file helpers: `_build_config()`, `_mock_async_client()` etc.
- Imports via `sys.path.insert` or `import_module`/`getattr` for engine modules.
- `SimpleNamespace` for mock config objects.
- `asyncio.run()` to drive async tests.
- Return type hints: `-> None` on all test functions.

```bash
# All tests
cd engine && uv run pytest ../tests/ -v

# Single file
cd engine && uv run pytest ../tests/test_pipeline.py -v

# Single function
cd engine && uv run pytest ../tests/test_pipeline.py::test_happy_path -v
```

### TypeScript

```bash
bunx tsc --noEmit
```

### Rust

```bash
cd src-tauri && cargo test
cd src-tauri && cargo clippy -- -D warnings
```

<!-- Tier 3: Workflows — reference when committing / releasing -->

## Git Workflow

### Conventional Commits

Format: `<type>(<scope>): <subject>` + body (1-3 sentences, what/why).

| Type       | When to Use                     |
| ---------- | ------------------------------- |
| `feat`     | New feature or capability       |
| `fix`      | Bug fix                         |
| `docs`     | Documentation only              |
| `refactor` | Code change without feature/fix |
| `test`     | Adding or fixing tests          |
| `chore`    | Build, deps, config changes     |
| `perf`     | Performance improvement         |

| Scope      | File Path                                      |
| ---------- | ---------------------------------------------- |
| `frontend` | `src/`                                         |
| `tauri`    | `src-tauri/`                                   |
| `engine`   | `engine/`                                      |
| `stt`      | `engine/aurotype_engine/providers/stt_*.py`    |
| `llm`      | `engine/aurotype_engine/providers/llm_*.py`    |
| `pipeline` | `engine/aurotype_engine/pipeline.py`           |
| `config`   | `engine/aurotype_engine/config.py`             |
| `deps`     | `package.json`, `Cargo.toml`, `pyproject.toml` |
| *omit*     | Multiple areas or project-wide                 |

**Rules**: Subject in imperative mood, ~50-72 chars, no period. Body mandatory for non-trivial commits. Write as a human
engineer — **NEVER** include AI-internal concepts (phase numbers, todo IDs, agent names, workflow metadata).

### Commit Policy

Commit and push after completing each logical change with all checks passing. Each commit should represent ONE logical
change. Split unrelated concerns into separate commits — never bundle multiple unrelated changes into a single large
commit. Never commit broken code.

## CI & Tooling

**CI/CD**: GitHub Actions (push/PR to `main`): Python tests, TypeScript type check, Rust check + clippy. See
`.github/workflows/ci.yml`.

**Lint**: No linter/formatter configured. TypeScript strict mode enforced via `tsconfig.json` (`noUnusedLocals`,
`noUnusedParameters`). Match existing implicit standards.

<!-- Tier 4: Extended Reference — consult when needed -->

## Extending the Project

| Task                    | Reference                                              |
| ----------------------- | ------------------------------------------------------ |
| Add STT provider        | `engine/aurotype_engine/providers/stt_*.py`            |
| Add LLM provider        | `engine/aurotype_engine/providers/llm_*.py`            |
| Add Tauri command       | `src-tauri/src/lib.rs`                                 |
| Add React page/component | `src/`                                                |
| Change engine config    | `engine/aurotype_engine/config.py`                     |
| Add Python tests        | `tests/`                                               |
| Add Rust module         | `src-tauri/src/` + declare in `lib.rs`                 |
| Add E2E test            | `e2e/`                                                 |

## Gotchas

- Frontend runs in Tauri webview, not a browser. Use Playwright for UI screenshots.
- Vite must bind to `127.0.0.1` explicitly (IPv6 `::1` default breaks Playwright).
- Kill `aurotype-engine` and `tauri-app` processes before rebuilding (Windows file locking).
- PowerShell `$_`, `$()` get mangled in bash — use `.ps1` script files for non-trivial PS commands.
- Local dev scripts (`*.ps1`) are gitignored and not tracked.
- Engine outputs `{"port": N}` on stdout at startup — Tauri reads this to discover the sidecar port.

## Agent Integration

- Use `question` tool for discussions (multiple-choice over open-ended).
- NEVER add `Co-authored-by` trailers or attribution footers to git commits.
