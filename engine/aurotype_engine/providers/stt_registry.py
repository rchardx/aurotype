from collections.abc import Callable
from typing import Protocol

from .stt_base import STTProvider
from .stt_groq import GroqSTTProvider
from .stt_siliconflow import SiliconFlowSTTProvider


class STTConfig(Protocol):
    groq_api_key: str | None
    siliconflow_api_key: str | None


STT_PROVIDER_REGISTRY: dict[str, Callable[[STTConfig], STTProvider]] = {
    "groq": GroqSTTProvider,
    "siliconflow": SiliconFlowSTTProvider,
}


def get_stt_provider(name: str, config: STTConfig) -> STTProvider:
    provider_cls = STT_PROVIDER_REGISTRY.get(name)
    if provider_cls is None:
        raise ValueError(f"Unknown STT provider: {name}")
    return provider_cls(config)
