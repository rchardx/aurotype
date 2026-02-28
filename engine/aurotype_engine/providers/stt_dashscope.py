# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false
from __future__ import annotations

import tempfile
from http import HTTPStatus
from typing import Final, Protocol, override

from .stt_base import STTProvider


class DashScopeSTTProvider(STTProvider):
    class _Config(Protocol):
        dashscope_api_key: str | None

    def __init__(self, config: _Config):
        self._api_key: str = config.dashscope_api_key or ""
        self._model: Final[str] = "paraformer-realtime-v2"

    @override
    async def transcribe(self, audio_bytes: bytes, language: str = "auto") -> str:
        import asyncio

        # dashscope SDK's Recognition.call() is blocking, so run in a thread
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(
            None, self._transcribe_sync, audio_bytes, language
        )

    def _transcribe_sync(self, audio_bytes: bytes, language: str) -> str:
        try:
            import dashscope
            from dashscope.audio.asr import Recognition
        except ImportError as exc:
            raise RuntimeError(
                "dashscope package is required for DashScope STT provider. "
                + "Install it with: pip install dashscope"
            ) from exc

        dashscope.api_key = self._api_key

        language_hints: list[str] = []
        if language != "auto":
            language_hints = [language]

        # Write audio bytes to a temp file since the SDK expects a file path
        with tempfile.NamedTemporaryFile(suffix=".wav", delete=True) as tmp:
            _ = tmp.write(audio_bytes)
            tmp.flush()

            recognition = Recognition(
                model=self._model,
                format="wav",
                sample_rate=16000,
                language_hints=language_hints if language_hints else ["zh", "en"],
                callback=None,
            )
            result = recognition.call(tmp.name)

        if result.status_code != HTTPStatus.OK:
            raise RuntimeError(
                f"DashScope STT failed with status {result.status_code}: {result.message}"
            )

        sentences = result.get_sentence()
        if not sentences:
            return ""

        # Concatenate all sentence texts
        parts: list[str] = []
        for sentence in sentences:
            text = sentence.get("text", "")
            if isinstance(text, str) and text:
                parts.append(text)

        return "".join(parts)
