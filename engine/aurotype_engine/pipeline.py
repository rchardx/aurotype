# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false

from .config import Settings
from .providers.llm_registry import get_llm_provider
from .providers.stt_registry import get_stt_provider


async def process_voice_input(audio_bytes: bytes, config: Settings) -> dict[str, str]:
    stt = get_stt_provider(config.stt_provider, config)
    raw_text = await stt.transcribe(audio_bytes, language=config.language)

    llm = get_llm_provider(config.llm_provider, config)
    polished_text = await llm.polish(raw_text, language=config.language)

    return {"raw_text": raw_text, "polished_text": polished_text}
