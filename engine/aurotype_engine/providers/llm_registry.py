# pyright: reportMissingImports=false, reportUnknownVariableType=false

from collections.abc import Callable
from typing import Protocol

from .llm_base import LLMProvider
from .llm_none import NoneLLMProvider
from .llm_openai import OpenAILLMProvider

from .llm_deepseek import DeepSeekLLMProvider


class LLMProviderConfig(Protocol):
    openai_api_key: str | None
    deepseek_api_key: str | None
    llm_base_url: str | None
    llm_model: str | None


LLM_PROVIDER_REGISTRY: dict[str, Callable[[LLMProviderConfig], LLMProvider]] = {
    "openai": OpenAILLMProvider,
    "deepseek": DeepSeekLLMProvider,
    "none": NoneLLMProvider,
}


def get_llm_provider(name: str, config: LLMProviderConfig) -> LLMProvider:
    if not name:
        name = "deepseek"
    provider_cls = LLM_PROVIDER_REGISTRY.get(name)
    if provider_cls is None:
        available = ", ".join(LLM_PROVIDER_REGISTRY.keys())
        raise ValueError(f"Unknown LLM provider: '{name}'. Available: {available}")
    return provider_cls(config)
