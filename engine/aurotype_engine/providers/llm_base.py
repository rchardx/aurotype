import abc


SYSTEM_PROMPT = (
    "You are a text polisher. Clean up the following speech transcription: "
    "remove filler words (um, uh, like, you know), fix grammar and punctuation, "
    "preserve meaning and tone. Return ONLY the polished text."
)


class LLMProvider(abc.ABC):
    @abc.abstractmethod
    async def polish(self, raw_text: str, language: str = "auto") -> str:
        raise NotImplementedError
