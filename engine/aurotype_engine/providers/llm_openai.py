from typing import Protocol, override

import openai

from .llm_base import LLMProvider, SYSTEM_PROMPT


class OpenAILLMProvider(LLMProvider):
    def __init__(self, config: "OpenAIConfig"):
        self._model: str = "gpt-4o-mini"
        self._client = openai.AsyncOpenAI(api_key=config.openai_api_key, timeout=10.0)

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


class OpenAIConfig(Protocol):
    openai_api_key: str | None
