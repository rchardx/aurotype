# pyright: reportMissingImports=false, reportUnknownVariableType=false

from collections.abc import Callable
from typing import Protocol

from .llm_base import LLMProvider
from .llm_none import NoneLLMProvider
from .llm_openai import OpenAILLMProvider
from .llm_siliconflow import SiliconFlowLLMProvider


class LLMProviderConfig(Protocol):
    openai_api_key: str | None
    siliconflow_api_key: str | None


LLM_PROVIDER_REGISTRY: dict[str, Callable[[LLMProviderConfig], LLMProvider]] = {
    "openai": OpenAILLMProvider,
    "siliconflow": SiliconFlowLLMProvider,
    "none": NoneLLMProvider,
}


def get_llm_provider(name: str, config: LLMProviderConfig) -> LLMProvider:
    provider_cls = LLM_PROVIDER_REGISTRY.get(name)
    if provider_cls is None:
        raise ValueError(f"Unknown LLM provider: {name}")
    return provider_cls(config)
