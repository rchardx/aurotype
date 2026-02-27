"""FastAPI server with endpoints for health check and placeholder text processing."""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

app = FastAPI()

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
async def transcribe():
    """Placeholder transcribe endpoint."""
    return {"text": ""}


@app.post("/polish")
async def polish():
    """Placeholder polish endpoint."""
    return {"text": ""}


@app.post("/process")
async def process():
    """Placeholder process endpoint combining transcribe and polish."""
    return {"raw_text": "", "polished_text": ""}
