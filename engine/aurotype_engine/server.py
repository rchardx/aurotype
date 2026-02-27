"""FastAPI server stub (expanded in Task 2)."""

from fastapi import FastAPI

app = FastAPI()


@app.get("/health")
async def health():
    return {"status": "ok", "version": "0.1.0"}
