from typing import Protocol, override

import openai

from .llm_base import LLMProvider, SYSTEM_PROMPT


class OpenAILLMProvider(LLMProvider):
    """OpenAI-compatible LLM provider.

    Accepts base_url, api_key, and model so it works with any
    OpenAI-compatible endpoint (OpenAI, vLLM, Ollama, LM Studio, etc.).
    """

    def __init__(self, config: "OpenAICompatibleConfig"):
        self._model: str = config.llm_model or "gpt-4o-mini"
        kwargs: dict[str, object] = {
            "api_key": config.openai_api_key or "sk-placeholder",
            "timeout": 10.0,
        }
        if config.llm_base_url:
            kwargs["base_url"] = config.llm_base_url
        self._client: openai.AsyncOpenAI = openai.AsyncOpenAI(**kwargs)  # type: ignore[arg-type]

    @override
    async def polish(self, raw_text: str, language: str = "auto") -> str:
        response = await self._client.chat.completions.create(
            model=self._model,
            messages=[
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": raw_text},
            ],
        )
        content = response.choices[0].message.content
        return content if isinstance(content, str) else ""


class OpenAICompatibleConfig(Protocol):
    openai_api_key: str | None
    llm_base_url: str | None
    llm_model: str | None
