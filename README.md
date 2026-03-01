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

**Aurotype** is a desktop app built with Tauri 2 (Rust) + React/TypeScript + a Python sidecar. Press a hotkey, speak, and the transcribed & polished text is automatically inserted at your cursor position.

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
src-tauri/src/    → Rust/Tauri 2 backend (hotkeys, tray, sidecar, text injection)
engine/           → Python 3.12 sidecar (FastAPI + uvicorn, STT/LLM providers)
```

Tauri spawns the Python engine as a sidecar process. They communicate over HTTP on localhost (dynamic port).

## Getting Started

### Prerequisites

- [Bun](https://bun.sh/) (or Node.js)
- [Rust](https://rustup.rs/) (stable)
- [Python 3.12+](https://www.python.org/) with [uv](https://docs.astral.sh/uv/)

### Setup & Run

```bash
make setup          # Install frontend + Python dependencies
make dev            # Run full app (Tauri + Vite + Python sidecar)
```

To run the Python engine standalone: `make engine-dev`

To build a release installer: `bun run tauri build`

## Configuration

Most settings are configurable from the in-app Settings page. Alternatively, use environment variables with the `AUROTYPE_` prefix:

| Variable | Default | Description |
|---|---|---|
| `AUROTYPE_STT_PROVIDER` | `dashscope` | Speech-to-text provider |
| `AUROTYPE_LLM_PROVIDER` | `deepseek` | LLM provider for text polishing |
| `AUROTYPE_DASHSCOPE_API_KEY` | — | DashScope API key (for STT) |
| `AUROTYPE_DEEPSEEK_API_KEY` | — | DeepSeek API key |
| `AUROTYPE_OPENAI_API_KEY` | — | OpenAI-compatible API key |
| `AUROTYPE_LLM_BASE_URL` | — | Custom LLM endpoint URL |
| `AUROTYPE_LLM_MODEL` | — | Override LLM model name |
| `AUROTYPE_SYSTEM_PROMPT` | — | Custom system prompt for polishing |
| `AUROTYPE_LANGUAGE` | `auto` | Target language |

## Development

### Testing

```bash
# Python
cd engine && uv run pytest ../tests/ -v

# TypeScript
bunx tsc --noEmit

# Rust
cd src-tauri && cargo test
cd src-tauri && cargo clippy -- -D warnings
```

### CI

GitHub Actions runs on every push to `main` and on pull requests: Python tests, TypeScript type check, Rust check + clippy.

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).
