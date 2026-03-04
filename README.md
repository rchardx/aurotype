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
- 🗣️ **Speech-to-text** powered by Alibaba Cloud DashScope Paraformer (configurable)
- ✨ **LLM polishing** — cleans up filler words, fixes grammar, preserves mixed-language speech
- 📋 **Smart paste fallback** — copies text to clipboard when cursor isn't in an editable field
- 📜 **Recording history** — review and copy past transcriptions
- 🔌 **OpenAI-compatible LLM** — works with DeepSeek, OpenAI, vLLM, Ollama, LM Studio, etc.
- 🔒 **Privacy first** — all recordings, transcription history, and settings are stored locally on your machine. Nothing is uploaded except to your configured STT/LLM API endpoints.

## Installation

Aurotype currently supports Windows.

1. Download the latest `.exe` or `.msi` installer from the [GitHub Releases page](https://github.com/rchardx/aurotype/releases). Both options work; pick the one you prefer.
2. Run the installer and follow the prompts.
3. Once launched, the app runs in your system tray.

## Quick Start

Follow these steps for your first successful voice transcription:

1. **Launch Aurotype** — The app appears in the system tray (bottom-right). Click the tray icon to open Settings.
2. **Check Engine Status** — At the top of the Settings page, confirm that "Engine Status" shows **Connected** (green). If it shows disconnected, wait a few seconds for the engine to start.
3. **Configure STT (Speech-to-Text)** — In the "STT Provider" section:
   - Provider: Alibaba Cloud DashScope (default).
   - API Key: Paste your DashScope API key (get one at https://dashscope.console.aliyun.com/).
   - Model: `paraformer-realtime-v2` (default).
   - Click **Test Connection** — It should show "Success!".
4. **Configure LLM (Text Polishing)** — In the "LLM Provider" section:
   - Provider: DeepSeek (default) or OpenAI Compatible.
   - If DeepSeek: Paste your DeepSeek API key (get one at https://platform.deepseek.com/).
   - If OpenAI Compatible: Set your Base URL, API key, and model name (works with OpenAI, vLLM, Ollama, LM Studio, etc.).
   - Click **Test Connection** — It should show "Success!".
5. **Try it!** — Press `Ctrl+Alt+Space` (default hotkey), speak naturally, and press `Ctrl+Alt+Space` again to stop. A floating overlay shows the recording status and transcription progress. The polished text is automatically pasted at your cursor position. If no text field is focused, the text is copied to your clipboard.

### Hotkey

- Default: `Ctrl+Alt+Space` (toggle mode — press to start, press again to stop).
- Alternative mode: "Hold to Record" (hold key to record, release to stop).
- Change these settings in the Settings → Hotkey section.
- Available shortcuts: `Ctrl+Alt+Space`, `Ctrl+Shift+Space`, `Ctrl+Shift+A`, `Ctrl+Shift+R`, `Ctrl+Shift+V`, `Ctrl+Space`, `F9`, `F10`.

## Architecture

```
src/              → React 19 + TypeScript frontend (Vite 7)
src-tauri/src/    → Rust/Tauri 2 backend (hotkeys, tray, sidecar, text injection)
engine/           → Python 3.12 sidecar (FastAPI + uvicorn, STT/LLM providers)
```

Tauri spawns the Python engine as a sidecar process. They communicate over HTTP on localhost (dynamic port).

## Building from Source

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

## Configuration (Environment Variables)

Most users should configure Aurotype through the in-app Settings page (see Quick Start above). Environment variables are for advanced use or automation. Alternatively, use variables with the `AUROTYPE_` prefix:

| Variable | Default | Description |
|---|---|---|
| `AUROTYPE_STT_PROVIDER` | `aliyun_dashscope` | Speech-to-text provider |
| `AUROTYPE_LLM_PROVIDER` | `deepseek` | LLM provider for text polishing |
| `AUROTYPE_ALIYUN_DASHSCOPE_API_KEY` | — | Alibaba Cloud DashScope API key (for STT) |
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
