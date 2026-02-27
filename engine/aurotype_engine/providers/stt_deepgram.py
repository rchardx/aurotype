from typing import Final, Protocol, cast, override

import httpx

from .stt_base import STTProvider


class DeepgramSTTProvider(STTProvider):
    class _Config(Protocol):
        deepgram_api_key: str | None

    def __init__(self, config: _Config):
        self._api_key: str = config.deepgram_api_key or ""
        self._base_url: Final[str] = "https://api.deepgram.com/v1/listen"
        self._model: Final[str] = "nova-2"

    @override
    async def transcribe(self, audio_bytes: bytes, language: str = "auto") -> str:
        params: dict[str, str] = {"model": self._model}
        if language != "auto":
            params["language"] = language

        headers = {
            "Authorization": f"Token {self._api_key}",
            "Content-Type": "audio/wav",
        }

        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.post(
                    self._base_url, params=params, content=audio_bytes, headers=headers
                )
        except httpx.HTTPError as exc:
            raise RuntimeError(f"Deepgram STT request failed: {exc}") from exc

        if response.status_code != 200:
            raise RuntimeError(
                f"Deepgram STT transcription failed with status {response.status_code}: {response.text}"
            )

        payload = cast(dict[str, object], response.json())
        # Deepgram returns: { results: { channels: [{ alternatives: [{ transcript: "..." }] }] } }
        try:
            results = payload.get("results", {})
            channels = (
                results.get("channels", [{}]) if isinstance(results, dict) else [{}]
            )  # type: ignore[union-attr]
            alternatives = channels[0].get("alternatives", [{}]) if channels else [{}]  # type: ignore[union-attr]
            transcript = alternatives[0].get("transcript", "") if alternatives else ""  # type: ignore[union-attr]
            return str(transcript)
        except (IndexError, AttributeError, TypeError):
            return ""
