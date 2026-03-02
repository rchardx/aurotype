from typing import Protocol, override

import openai

from .llm_base import LLMProvider, get_system_prompt


class DeepSeekLLMProvider(LLMProvider):
    def __init__(self, config: "DeepSeekConfig"):
        self._model: str = config.llm_model or "deepseek-chat"
        self._system_prompt = get_system_prompt(getattr(config, 'system_prompt', None))
        self._client: openai.AsyncOpenAI
        self._client = openai.AsyncOpenAI(
            api_key=config.deepseek_api_key,
            base_url="https://api.deepseek.com/v1",
            timeout=10.0,
        )

    @override
    async def polish(self, raw_text: str, language: str = "auto") -> str:
        response = await self._client.chat.completions.create(
            model=self._model,
            messages=[
                {"role": "system", "content": self._system_prompt},
                {"role": "user", "content": raw_text},
            ],
        )
        content = response.choices[0].message.content
        return content if isinstance(content, str) else ""


class DeepSeekConfig(Protocol):
    deepseek_api_key: str | None
    llm_model: str | None
