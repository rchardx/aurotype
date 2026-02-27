from typing import Final, Protocol, cast, override

import httpx

from .stt_base import STTProvider


class GroqSTTProvider(STTProvider):
    class _Config(Protocol):
        groq_api_key: str | None

    def __init__(self, config: _Config):
        self._api_key: str = config.groq_api_key or ""
        self._url: Final[str] = "https://api.groq.com/openai/v1/audio/transcriptions"
        self._model: Final[str] = "whisper-large-v3"

    @override
    async def transcribe(self, audio_bytes: bytes, language: str = "auto") -> str:
        data: dict[str, str] = {"model": self._model}
        if language != "auto":
            data["language"] = language

        files = {"file": ("audio.wav", audio_bytes, "audio/wav")}
        headers = {"Authorization": f"Bearer {self._api_key}"}

        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                response = await client.post(
                    self._url, data=data, files=files, headers=headers
                )
        except httpx.HTTPError as exc:
            raise RuntimeError(f"Groq STT request failed: {exc}") from exc

        if response.status_code != 200:
            raise RuntimeError(
                f"Groq STT transcription failed with status {response.status_code}: {response.text}"
            )

        payload = cast(dict[str, object], response.json())
        text = payload.get("text", "")
        if isinstance(text, str):
            return text
        return ""
