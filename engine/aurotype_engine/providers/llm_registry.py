from collections.abc import Callable
from typing import Any

from .llm_base import LLMProvider
from .llm_none import NoneLLMProvider
from .llm_openai import OpenAILLMProvider
from .llm_siliconflow import SiliconFlowLLMProvider

LLM_PROVIDER_REGISTRY: dict[str, Callable[[Any], LLMProvider]] = {
    "openai": OpenAILLMProvider,
    "siliconflow": SiliconFlowLLMProvider,
    "none": NoneLLMProvider,
}


def get_llm_provider(name: str, config: Any) -> LLMProvider:
    provider_cls = LLM_PROVIDER_REGISTRY.get(name)
    if provider_cls is None:
        raise ValueError(f"Unknown LLM provider: {name}")
    return provider_cls(config)
