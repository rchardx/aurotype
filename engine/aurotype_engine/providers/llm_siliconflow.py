from typing import Protocol, override

import openai

from .llm_base import LLMProvider, SYSTEM_PROMPT


class SiliconFlowLLMProvider(LLMProvider):
    def __init__(self, config: "SiliconFlowConfig"):
        self._model: str = "deepseek-ai/DeepSeek-V3"
        self._client: openai.AsyncOpenAI
        self._client = openai.AsyncOpenAI(
            api_key=config.siliconflow_api_key,
            base_url="https://api.siliconflow.cn/v1",
            timeout=10.0,
        )

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


class SiliconFlowConfig(Protocol):
    siliconflow_api_key: str | None
