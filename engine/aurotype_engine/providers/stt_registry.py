from collections.abc import Callable
from typing import Protocol

from .stt_base import STTProvider
from .stt_deepgram import DeepgramSTTProvider
from .stt_siliconflow import SiliconFlowSTTProvider
from .stt_dashscope import DashScopeSTTProvider


class STTConfig(Protocol):
    deepgram_api_key: str | None
    siliconflow_api_key: str | None
    dashscope_api_key: str | None


STT_PROVIDER_REGISTRY: dict[str, Callable[[STTConfig], STTProvider]] = {
    "deepgram": DeepgramSTTProvider,
    "siliconflow": SiliconFlowSTTProvider,
    "dashscope": DashScopeSTTProvider,
}


def get_stt_provider(name: str, config: STTConfig) -> STTProvider:
    provider_cls = STT_PROVIDER_REGISTRY.get(name)
    if provider_cls is None:
        raise ValueError(f"Unknown STT provider: {name}")
    return provider_cls(config)
