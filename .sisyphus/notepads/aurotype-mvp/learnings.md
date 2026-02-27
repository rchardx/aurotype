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
