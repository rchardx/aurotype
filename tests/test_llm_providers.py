# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false, reportUnknownArgumentType=false, reportAny=false

import asyncio
import sys
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "engine"))

from aurotype_engine.providers.llm_none import NoneLLMProvider
from aurotype_engine.providers.llm_openai import OpenAILLMProvider
from aurotype_engine.providers.llm_deepseek import DeepSeekLLMProvider
from aurotype_engine.providers.llm_registry import get_llm_provider


def _build_config() -> SimpleNamespace:
    return SimpleNamespace(
        openai_api_key="openai-key",
        deepseek_api_key="deepseek-key",
        llm_base_url=None,
        llm_model=None,
    )


def _mock_chat_response(content: str) -> MagicMock:
    mock_message = SimpleNamespace(content=content)
    mock_choice = SimpleNamespace(message=mock_message)
    return SimpleNamespace(choices=[mock_choice])


# ── Registry tests ────────────────────────────────────────────────────


def test_registry_returns_openai_provider() -> None:
    config = _build_config()
    with patch(
        "aurotype_engine.providers.llm_openai.openai.AsyncOpenAI",
        return_value=MagicMock(),
    ):
        provider = get_llm_provider("openai", config)
        assert isinstance(provider, OpenAILLMProvider)


def test_registry_returns_deepseek_provider() -> None:
    config = _build_config()
    with patch(
        "aurotype_engine.providers.llm_deepseek.openai.AsyncOpenAI",
        return_value=MagicMock(),
    ):
        provider = get_llm_provider("deepseek", config)
        assert isinstance(provider, DeepSeekLLMProvider)


def test_registry_returns_none_provider() -> None:
    config = _build_config()
    provider = get_llm_provider("none", config)
    assert isinstance(provider, NoneLLMProvider)


def test_registry_raises_for_unknown_provider() -> None:
    config = _build_config()
    try:
        get_llm_provider("unknown", config)
    except ValueError as exc:
        assert "Unknown LLM provider" in str(exc)
    else:
        raise AssertionError("Expected ValueError for unknown provider")


# ── None provider ─────────────────────────────────────────────────────


def test_none_provider_returns_raw_text_unchanged() -> None:
    provider = NoneLLMProvider()
    raw_text = "um hello this is like a test"
    result = asyncio.run(provider.polish(raw_text))
    assert result == raw_text


# ── OpenAI-compatible provider ────────────────────────────────────────


def test_openai_provider_polish_calls_chat_completions() -> None:
    config = _build_config()
    mock_client = MagicMock()
    mock_client.chat.completions.create = AsyncMock(
        return_value=_mock_chat_response("Polished output")
    )

    with patch(
        "aurotype_engine.providers.llm_openai.openai.AsyncOpenAI",
        return_value=mock_client,
    ):
        provider = OpenAILLMProvider(config)
        result = asyncio.run(provider.polish("uh this is raw"))

    assert result == "Polished output"
    mock_client.chat.completions.create.assert_awaited_once()


def test_openai_provider_uses_default_model() -> None:
    config = _build_config()
    with patch(
        "aurotype_engine.providers.llm_openai.openai.AsyncOpenAI",
        return_value=MagicMock(),
    ):
        provider = OpenAILLMProvider(config)
        assert provider._model == "gpt-4o-mini"


def test_openai_provider_uses_custom_model() -> None:
    config = SimpleNamespace(
        openai_api_key="key",
        llm_base_url=None,
        llm_model="gpt-4o",
    )
    with patch(
        "aurotype_engine.providers.llm_openai.openai.AsyncOpenAI",
        return_value=MagicMock(),
    ):
        provider = OpenAILLMProvider(config)
        assert provider._model == "gpt-4o"


def test_openai_provider_passes_base_url() -> None:
    config = SimpleNamespace(
        openai_api_key="key",
        llm_base_url="https://my-endpoint.example.com/v1",
        llm_model=None,
    )
    with patch(
        "aurotype_engine.providers.llm_openai.openai.AsyncOpenAI",
        return_value=MagicMock(),
    ) as mock_cls:
        OpenAILLMProvider(config)
        _, kwargs = mock_cls.call_args
        assert kwargs["base_url"] == "https://my-endpoint.example.com/v1"


def test_openai_provider_omits_base_url_when_none() -> None:
    config = _build_config()
    with patch(
        "aurotype_engine.providers.llm_openai.openai.AsyncOpenAI",
        return_value=MagicMock(),
    ) as mock_cls:
        OpenAILLMProvider(config)
        _, kwargs = mock_cls.call_args
        assert "base_url" not in kwargs


# ── DeepSeek provider ─────────────────────────────────────────────────


def test_deepseek_provider_polish_calls_chat_completions() -> None:
    config = _build_config()
    mock_client = MagicMock()
    mock_client.chat.completions.create = AsyncMock(
        return_value=_mock_chat_response("Clean text")
    )

    with patch(
        "aurotype_engine.providers.llm_deepseek.openai.AsyncOpenAI",
        return_value=mock_client,
    ):
        provider = DeepSeekLLMProvider(config)
        result = asyncio.run(provider.polish("you know this is raw"))

    assert result == "Clean text"
    mock_client.chat.completions.create.assert_awaited_once()


def test_deepseek_provider_uses_default_model() -> None:
    config = _build_config()
    with patch(
        "aurotype_engine.providers.llm_deepseek.openai.AsyncOpenAI",
        return_value=MagicMock(),
    ):
        provider = DeepSeekLLMProvider(config)
        assert provider._model == "deepseek-chat"


def test_deepseek_provider_uses_hardcoded_base_url() -> None:
    config = _build_config()
    with patch(
        "aurotype_engine.providers.llm_deepseek.openai.AsyncOpenAI",
        return_value=MagicMock(),
    ) as mock_cls:
        DeepSeekLLMProvider(config)
        _, kwargs = mock_cls.call_args
        assert kwargs["base_url"] == "https://api.deepseek.com/v1"


def test_deepseek_provider_uses_custom_model() -> None:
    config = SimpleNamespace(
        deepseek_api_key="key",
        llm_model="deepseek-reasoner",
    )
    with patch(
        "aurotype_engine.providers.llm_deepseek.openai.AsyncOpenAI",
        return_value=MagicMock(),
    ):
        provider = DeepSeekLLMProvider(config)
        assert provider._model == "deepseek-reasoner"
