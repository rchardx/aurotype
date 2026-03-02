# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false, reportUnknownParameterType=false, reportUnknownArgumentType=false

from typing import Annotated

from fastapi import FastAPI, File, HTTPException, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

from .audio import AudioDeviceError, AudioRecorder
from .config import Settings, get_settings
from .pipeline import process_voice_input
from .providers.llm_registry import get_llm_provider
from .providers.stt_registry import get_stt_provider

app = FastAPI()
recorder = AudioRecorder()
_config_overrides: dict[str, str | None] = {}


def get_effective_settings() -> Settings:
    base = get_settings()
    # Filter out None and empty strings — treat them as "use default"
    overrides = {k: v for k, v in _config_overrides.items() if v is not None and v != ""}
    return base.model_copy(update=overrides)


class PolishRequest(BaseModel):
    text: str


class ConfigureRequest(BaseModel):
    stt_provider: str | None = None
    stt_model: str | None = None
    llm_provider: str | None = None
    openai_api_key: str | None = None
    aliyun_dashscope_api_key: str | None = None
    deepseek_api_key: str | None = None
    llm_base_url: str | None = None
    llm_model: str | None = None
    language: str | None = None
    system_prompt: str | None = None


# Configure CORS for localhost and Tauri
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost",
        "http://127.0.0.1",
        "tauri://localhost",
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/health")
async def health():
    """Health check endpoint."""
    return {"status": "ok", "version": "0.1.0"}


@app.post("/transcribe")
async def transcribe(audio: Annotated[UploadFile, File(...)]):
    audio_bytes = await audio.read()
    cfg = get_effective_settings()
    stt = get_stt_provider(cfg.stt_provider, cfg)
    text = await stt.transcribe(audio_bytes, language=cfg.language)
    return {"text": text}


@app.post("/polish")
async def polish(payload: PolishRequest):
    cfg = get_effective_settings()
    llm = get_llm_provider(cfg.llm_provider, cfg)
    text = await llm.polish(payload.text, language=cfg.language)
    return {"text": text}


@app.post("/process")
async def process(audio: Annotated[UploadFile, File(...)]):
    audio_bytes = await audio.read()
    cfg = get_effective_settings()
    return await process_voice_input(audio_bytes, cfg)


@app.post("/configure")
async def configure(payload: ConfigureRequest):
    _config_overrides.update(payload.model_dump())
    return {"status": "configured"}


@app.post("/test-llm")
async def test_llm():
    """Test the current LLM provider with a short prompt."""
    cfg = get_effective_settings()
    try:
        llm = get_llm_provider(cfg.llm_provider, cfg)
        result = await llm.polish("Hello, this is a test.", language="en")
        return {"status": "ok", "result": result}
    except Exception as exc:
        raise HTTPException(status_code=500, detail=str(exc)) from exc


@app.post("/test-stt")
async def test_stt():
    """Test the current STT provider by instantiating it and verifying credentials."""
    cfg = get_effective_settings()
    try:
        stt = get_stt_provider(cfg.stt_provider, cfg)
        # Send a tiny silent WAV to verify the provider works
        import io
        import wave
        buf = io.BytesIO()
        with wave.open(buf, "wb") as wav:
            wav.setnchannels(1)
            wav.setsampwidth(2)
            wav.setframerate(16000)
            wav.writeframes(b"\x00\x00" * 1600)  # 0.1s silence
        result = await stt.transcribe(buf.getvalue(), language="en")
        return {"status": "ok", "result": result}
    except Exception as exc:
        raise HTTPException(status_code=500, detail=str(exc)) from exc
@app.post("/record/start")
async def start_recording():
    try:
        recorder.start_recording()
    except AudioDeviceError as exc:
        raise HTTPException(status_code=500, detail=str(exc)) from exc
    return {"status": "recording"}



def _audio_duration_ms(wav_bytes: bytes) -> float:
    """Return the duration of a WAV byte string in milliseconds."""
    import io
    import wave

    with wave.open(io.BytesIO(wav_bytes), "rb") as wav:
        return wav.getnframes() / wav.getframerate() * 1000


@app.post("/record/stop")
async def stop_recording():
    try:
        audio_bytes = recorder.stop_recording()
    except AudioDeviceError as exc:
        print(f"[aurotype] Audio device error in /record/stop: {exc}")
        raise HTTPException(status_code=500, detail=str(exc)) from exc

    # Discard recordings shorter than 100ms — likely accidental key taps
    duration_ms = _audio_duration_ms(audio_bytes)
    if duration_ms < 100:
        print(f"[aurotype] Recording too short ({duration_ms:.0f}ms), discarding")
        return {"too_short": True, "duration_ms": duration_ms}

    cfg = get_effective_settings()
    try:
        return await process_voice_input(audio_bytes, cfg)
    except Exception as exc:
        print(f"[aurotype] Pipeline error in /record/stop: {exc}")
        raise HTTPException(status_code=500, detail=str(exc)) from exc


@app.post("/record/cancel")
async def cancel_recording():
    try:
        _ = recorder.stop_recording()
    except AudioDeviceError:
        pass
    return {"status": "cancelled"}


@app.get("/volume")
@app.post("/volume")
async def volume():
    return {"volume": recorder.get_volume()}
