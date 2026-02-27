import abc


class STTProvider(abc.ABC):
    @abc.abstractmethod
    async def transcribe(self, audio_bytes: bytes, language: str = "auto") -> str:
        raise NotImplementedError
