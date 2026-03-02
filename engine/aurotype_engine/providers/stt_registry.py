from collections.abc import Callable
from typing import Protocol

from .stt_base import STTProvider
from .stt_aliyun_dashscope import AliyunDashScopeSTTProvider


class STTConfig(Protocol):
    aliyun_dashscope_api_key: str | None
    stt_model: str | None


STT_PROVIDER_REGISTRY: dict[str, Callable[[STTConfig], STTProvider]] = {
    "aliyun_dashscope": AliyunDashScopeSTTProvider,
}


def get_stt_provider(name: str, config: STTConfig) -> STTProvider:
    provider_cls = STT_PROVIDER_REGISTRY.get(name)
    if provider_cls is None:
        raise ValueError(f"Unknown STT provider: {name}")
    return provider_cls(config)
