# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false

from __future__ import annotations

import asyncio
import base64

from .config import Settings
from .providers.llm_registry import get_llm_provider
from .providers.stt_registry import get_stt_provider

_MAX_STT_RETRIES = 3
_RETRY_BACKOFF_BASE = 2.0  # seconds: 2, 4, 8


async def process_voice_input(audio_bytes: bytes, config: Settings) -> dict[str, str]:
    stt = get_stt_provider(config.stt_provider, config)

    # Retry STT up to 3 times with exponential backoff
    last_error: Exception | None = None
    raw_text = ""
    for attempt in range(_MAX_STT_RETRIES):
        try:
            raw_text = await stt.transcribe(audio_bytes, language=config.language)
            last_error = None
            break
        except Exception as exc:
            last_error = exc
            if attempt < _MAX_STT_RETRIES - 1:
                wait = _RETRY_BACKOFF_BASE * (2**attempt)
                print(
                    f"[aurotype] STT attempt {attempt + 1} failed: {exc}, "
                    f"retrying in {wait:.0f}s..."
                )
                await asyncio.sleep(wait)
            else:
                print(
                    f"[aurotype] STT attempt {attempt + 1} failed: {exc}, no more retries"
                )

    if last_error is not None:
        raise RuntimeError(
            f"STT failed after {_MAX_STT_RETRIES} attempts: {last_error}"
        ) from last_error

    llm = get_llm_provider(config.llm_provider, config)
    polished_text = await llm.polish(raw_text, language=config.language)

    # Include base64-encoded audio for history persistence
    audio_data = base64.b64encode(audio_bytes).decode("ascii")

    return {
        "raw_text": raw_text,
        "polished_text": polished_text,
        "audio_data": audio_data,
    }
