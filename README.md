<p align="center">
  <img src="assets/logo-with-text.png" alt="Aurotype" width="280" />
</p>

<p align="center">
  <a href="https://github.com/rchardx/aurotype/actions/workflows/ci.yml"><img src="https://github.com/rchardx/aurotype/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
</p>

<p align="center">
  Voice-to-text desktop app — speak naturally, get polished text injected where your cursor is.
</p>

---

**Aurotype** is a desktop application built with Tauri 2 (Rust) + React/TypeScript + Python sidecar. Press a hotkey, speak, and the transcribed & polished text is automatically inserted at your cursor position.

## Features

- 🎤 **Push-to-talk recording** via global hotkey (default: `Ctrl+Alt+Space`)
- 🗣️ **Speech-to-text** powered by DashScope Paraformer (configurable)
- ✨ **LLM polishing** — cleans up filler words, fixes grammar, preserves mixed-language speech
- 📋 **Smart paste fallback** — copies text to clipboard when cursor isn't in an editable field
- 📜 **Recording history** — review and copy past transcriptions
- 🔌 **OpenAI-compatible LLM** — works with DeepSeek, OpenAI, vLLM, Ollama, LM Studio, etc.

## Architecture

```
src/              → React 19 + TypeScript frontend (Vite 7)
src-tauri/src/    → Rust/Tauri 2 backend (hotkeys, tray, sidecar management, text injection)
engine/           → Python 3.12 sidecar (FastAPI + uvicorn, STT/LLM providers)
```

## Quick Start

```bash
# Install dependencies
make setup          # bun install && cd engine && uv sync

# Run in development mode
make dev            # bun run tauri dev

# Build release installer
bun run tauri build
```

## Development

### Prerequisites

- [Bun](https://bun.sh/) (or Node.js)
- [Rust](https://rustup.rs/) (stable)
- [Python 3.12+](https://www.python.org/) with [uv](https://docs.astral.sh/uv/)

### Setup

```bash
make setup          # Install frontend + Python dependencies
```

### Running

```bash
make dev            # Run full app (Tauri + Vite + Python sidecar)
make engine-dev     # Run Python engine standalone
```

## Testing

### Python Engine Tests

```bash
cd engine && uv run pytest ../tests/ -v          # Run all tests
cd engine && uv run pytest ../tests/test_pipeline.py -v  # Single file
```

### TypeScript Type Check

```bash
bun run build       # Full build (tsc + vite)
```

### Rust

```bash
cd src-tauri && cargo test      # Run Rust tests
cd src-tauri && cargo clippy    # Lint
```

## CI

GitHub Actions runs on every push to `main` and on pull requests:

- **Python Tests** — pytest across all engine unit tests
- **TypeScript Check** — `tsc --noEmit` type verification
- **Rust Check** — `cargo check` + `cargo clippy -D warnings`


## License

MIT
