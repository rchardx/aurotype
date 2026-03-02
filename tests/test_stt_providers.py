import asyncio
import sys
from importlib import import_module
from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock, patch

STTProvider = getattr(
    import_module("aurotype_engine.providers.stt_base"), "STTProvider"
)
AliyunDashScopeSTTProvider = getattr(
    import_module("aurotype_engine.providers.stt_aliyun_dashscope"), "AliyunDashScopeSTTProvider"
)
get_stt_provider = getattr(
    import_module("aurotype_engine.providers.stt_registry"), "get_stt_provider"
)


def _build_config() -> SimpleNamespace:
    return SimpleNamespace(aliyun_dashscope_api_key="ds-key", stt_model=None)


def _mock_async_client(response: MagicMock) -> tuple[MagicMock, MagicMock]:
    mock_client = MagicMock()
    mock_client.post = AsyncMock(return_value=response)

    mock_cm = MagicMock()
    mock_cm.__aenter__ = AsyncMock(return_value=mock_client)
    mock_cm.__aexit__ = AsyncMock(return_value=False)
    return mock_client, mock_cm


def test_registry_returns_aliyun_dashscope_provider() -> None:
    provider = get_stt_provider("aliyun_dashscope", _build_config())
    assert isinstance(provider, AliyunDashScopeSTTProvider)
    assert isinstance(provider, STTProvider)


def test_registry_raises_for_unknown_provider() -> None:
    try:
        get_stt_provider("unknown", _build_config())
        assert False, "Expected ValueError"
    except ValueError as exc:
        assert str(exc) == "Unknown STT provider: unknown"


def test_aliyun_dashscope_default_model() -> None:
    provider = AliyunDashScopeSTTProvider(_build_config())
    assert provider._model == "paraformer-realtime-v2"


def test_aliyun_dashscope_custom_model() -> None:
    config = SimpleNamespace(aliyun_dashscope_api_key="ds-key", stt_model="paraformer-v2")
    provider = AliyunDashScopeSTTProvider(config)
    assert provider._model == "paraformer-v2"


def _mock_dashscope_modules(mock_recognition_cls: MagicMock) -> dict[str, MagicMock]:
    """Build fake sys.modules entries so local imports inside _transcribe_sync resolve to mocks."""
    mock_dashscope = MagicMock()
    mock_audio = MagicMock()
    mock_asr = MagicMock()
    mock_asr.Recognition = mock_recognition_cls
    mock_audio.asr = mock_asr
    mock_dashscope.audio = mock_audio
    mock_dashscope.audio.asr = mock_asr
    return {
        "dashscope": mock_dashscope,
        "dashscope.audio": mock_audio,
        "dashscope.audio.asr": mock_asr,
    }


def test_aliyun_dashscope_transcribe_uses_mocked_sdk() -> None:
    mock_result = MagicMock()
    mock_result.status_code = 200
    mock_result.get_sentence.return_value = [
        {"text": "你好"},
        {"text": "世界"},
    ]

    mock_recognition = MagicMock()
    mock_recognition.call.return_value = mock_result
    mock_recognition_cls = MagicMock(return_value=mock_recognition)

    modules = _mock_dashscope_modules(mock_recognition_cls)
    with patch.dict(sys.modules, modules):
        provider = AliyunDashScopeSTTProvider(_build_config())
        text = asyncio.run(provider.transcribe(b"wav-bytes", language="zh"))

    assert text == "你好世界"
    mock_recognition_cls.assert_called_once_with(
        model="paraformer-realtime-v2",
        format="wav",
        sample_rate=16000,
        language_hints=["zh"],
        callback=None,
    )
    mock_recognition.call.assert_called_once()


def test_aliyun_dashscope_transcribe_error_raises_runtime_error() -> None:
    mock_result = MagicMock()
    mock_result.status_code = 401
    mock_result.message = "Invalid API key"

    mock_recognition = MagicMock()
    mock_recognition.call.return_value = mock_result
    mock_recognition_cls = MagicMock(return_value=mock_recognition)

    modules = _mock_dashscope_modules(mock_recognition_cls)
    with patch.dict(sys.modules, modules):
        provider = AliyunDashScopeSTTProvider(_build_config())
        try:
            asyncio.run(provider.transcribe(b"wav-bytes"))
            assert False, "Expected RuntimeError"
        except RuntimeError as exc:
            assert "401" in str(exc)
            assert "Invalid API key" in str(exc)


def test_aliyun_dashscope_transcribe_empty_result_returns_empty_string() -> None:
    mock_result = MagicMock()
    mock_result.status_code = 200
    mock_result.get_sentence.return_value = []

    mock_recognition = MagicMock()
    mock_recognition.call.return_value = mock_result
    mock_recognition_cls = MagicMock(return_value=mock_recognition)

    modules = _mock_dashscope_modules(mock_recognition_cls)
    with patch.dict(sys.modules, modules):
        provider = AliyunDashScopeSTTProvider(_build_config())
        text = asyncio.run(provider.transcribe(b"wav-bytes"))

    assert text == ""
