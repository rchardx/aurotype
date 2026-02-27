from typing import override

from .llm_base import LLMProvider


class NoneLLMProvider(LLMProvider):
    def __init__(self, config: object | None = None):
        self._config: object | None = config

    @override
    async def polish(self, raw_text: str, language: str = "auto") -> str:
        return raw_text
