# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false, reportUnknownParameterType=false, reportUnknownArgumentType=false

from typing import Annotated

from fastapi import FastAPI, File, HTTPException, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

from .audio import AudioDeviceError, AudioRecorder
from .config import get_settings
from .pipeline import process_voice_input
from .providers.llm_registry import get_llm_provider
from .providers.stt_registry import get_stt_provider

app = FastAPI()
recorder = AudioRecorder()
settings = get_settings()


class PolishRequest(BaseModel):
    text: str


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
    stt = get_stt_provider(settings.stt_provider, settings)
    text = await stt.transcribe(audio_bytes, language=settings.language)
    return {"text": text}


@app.post("/polish")
async def polish(payload: PolishRequest):
    llm = get_llm_provider(settings.llm_provider, settings)
    text = await llm.polish(payload.text, language=settings.language)
    return {"text": text}


@app.post("/process")
async def process(audio: Annotated[UploadFile, File(...)]):
    audio_bytes = await audio.read()
    return await process_voice_input(audio_bytes, settings)


@app.post("/record/start")
async def start_recording():
    try:
        recorder.start_recording()
    except AudioDeviceError as exc:
        raise HTTPException(status_code=500, detail=str(exc)) from exc
    return {"status": "recording"}


@app.post("/record/stop")
async def stop_recording():
    recorder.stop_recording()
    return {"status": "stopped"}


@app.get("/volume")
@app.post("/volume")
async def volume():
    return {"volume": recorder.get_volume()}
