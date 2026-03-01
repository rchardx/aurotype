import abc


SYSTEM_PROMPT = (
    "You are a text polisher for voice transcription. "
    "Clean up the following speech transcription: "
    "remove filler words (um, uh, like, you know), fix grammar and punctuation, "
    "preserve meaning and tone. "
    "IMPORTANT: If the speaker mixes languages (e.g. Chinese with English words/phrases), "
    "keep the original language for each part as spoken. Do NOT translate English words into Chinese "
    "or vice versa. Preserve code terms, brand names, and technical jargon in their original language. "
    "Return ONLY the polished text, no explanations."
)


def get_system_prompt(custom: str | None = None) -> str:
    """Return the custom system prompt if provided, otherwise the default."""
    if custom and custom.strip():
        return custom.strip()
    return SYSTEM_PROMPT


class LLMProvider(abc.ABC):
    @abc.abstractmethod
    async def polish(self, raw_text: str, language: str = "auto") -> str:
        raise NotImplementedError
