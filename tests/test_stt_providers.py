import asyncio
from importlib import import_module
from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock, patch

STTProvider = getattr(
    import_module("aurotype_engine.providers.stt_base"), "STTProvider"
)
DeepgramSTTProvider = getattr(
    import_module("aurotype_engine.providers.stt_deepgram"), "DeepgramSTTProvider"
)
SiliconFlowSTTProvider = getattr(
    import_module("aurotype_engine.providers.stt_siliconflow"), "SiliconFlowSTTProvider"
)
get_stt_provider = getattr(
    import_module("aurotype_engine.providers.stt_registry"), "get_stt_provider"
)


def _build_config() -> SimpleNamespace:
    return SimpleNamespace(deepgram_api_key="dg-key", siliconflow_api_key="sf-key")


def _mock_async_client(response: MagicMock) -> tuple[MagicMock, MagicMock]:
    mock_client = MagicMock()
    mock_client.post = AsyncMock(return_value=response)

    mock_cm = MagicMock()
    mock_cm.__aenter__ = AsyncMock(return_value=mock_client)
    mock_cm.__aexit__ = AsyncMock(return_value=False)
    return mock_client, mock_cm


def test_registry_returns_deepgram_provider() -> None:
    provider = get_stt_provider("deepgram", _build_config())
    assert isinstance(provider, DeepgramSTTProvider)
    assert isinstance(provider, STTProvider)


def test_registry_returns_siliconflow_provider() -> None:
    provider = get_stt_provider("siliconflow", _build_config())
    assert isinstance(provider, SiliconFlowSTTProvider)
    assert isinstance(provider, STTProvider)


def test_registry_raises_for_unknown_provider() -> None:
    with patch(
        "aurotype_engine.providers.stt_registry.STT_PROVIDER_REGISTRY",
        {"deepgram": DeepgramSTTProvider},
    ):
        try:
            get_stt_provider("unknown", _build_config())
            assert False, "Expected ValueError"
        except ValueError as exc:
            assert str(exc) == "Unknown STT provider: unknown"


def test_deepgram_transcribe_uses_mocked_httpx() -> None:
    response = MagicMock()
    response.status_code = 200
    response.json.return_value = {
        "results": {"channels": [{"alternatives": [{"transcript": "hello world"}]}]}
    }

    mock_client, mock_cm = _mock_async_client(response)

    with patch(
        "aurotype_engine.providers.stt_deepgram.httpx.AsyncClient", return_value=mock_cm
    ) as client_cls:
        provider = DeepgramSTTProvider(_build_config())
        text = asyncio.run(provider.transcribe(b"wav-bytes", language="en"))

    assert text == "hello world"
    client_cls.assert_called_once_with(timeout=30.0)
    mock_client.post.assert_awaited_once()


def test_siliconflow_transcribe_uses_mocked_httpx() -> None:
    response = MagicMock()
    response.status_code = 200
    response.json.return_value = {"text": ""}

    mock_client, mock_cm = _mock_async_client(response)

    with patch(
        "aurotype_engine.providers.stt_siliconflow.httpx.AsyncClient",
        return_value=mock_cm,
    ) as client_cls:
        provider = SiliconFlowSTTProvider(_build_config())
        text = asyncio.run(provider.transcribe(b"wav-bytes"))

    assert text == ""
    client_cls.assert_called_once_with(timeout=10.0)
    mock_client.post.assert_awaited_once()
