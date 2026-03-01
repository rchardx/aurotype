from .stt_base import STTProvider
from .stt_dashscope import DashScopeSTTProvider
from .stt_registry import get_stt_provider

__all__ = [
    "STTProvider",
    "DashScopeSTTProvider",
    "get_stt_provider",
]
