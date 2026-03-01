<p align="center">
  <img src="assets/logo-with-text.png" alt="Aurotype" width="280" />
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
powershell -ExecutionPolicy Bypass -File build.ps1
```

## Requirements

- [Bun](https://bun.sh/) (or Node.js)
- [Rust](https://rustup.rs/) (stable)
- [Python 3.12+](https://www.python.org/) with [uv](https://docs.astral.sh/uv/)

## License

MIT
