# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false, reportUnknownArgumentType=false, reportAny=false

import asyncio
import sys
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "engine"))

from aurotype_engine.providers.llm_none import NoneLLMProvider
from aurotype_engine.providers.llm_openai import OpenAILLMProvider
from aurotype_engine.providers.llm_registry import get_llm_provider
from aurotype_engine.providers.llm_siliconflow import SiliconFlowLLMProvider


def test_registry_returns_correct_provider_class() -> None:
    config = SimpleNamespace(openai_api_key="openai-key", siliconflow_api_key="sf-key")

    with (
        patch("aurotype_engine.providers.llm_openai.openai.AsyncOpenAI") as openai_mock,
        patch(
            "aurotype_engine.providers.llm_siliconflow.openai.AsyncOpenAI"
        ) as siliconflow_mock,
    ):
        openai_mock.return_value = MagicMock()
        siliconflow_mock.return_value = MagicMock()

        assert isinstance(get_llm_provider("openai", config), OpenAILLMProvider)
        assert isinstance(
            get_llm_provider("siliconflow", config), SiliconFlowLLMProvider
        )
        assert isinstance(get_llm_provider("none", config), NoneLLMProvider)


def test_registry_raises_for_unknown_provider() -> None:
    config = SimpleNamespace(openai_api_key="openai-key", siliconflow_api_key="sf-key")

    with (
        patch("aurotype_engine.providers.llm_openai.openai.AsyncOpenAI") as openai_mock,
        patch(
            "aurotype_engine.providers.llm_siliconflow.openai.AsyncOpenAI"
        ) as siliconflow_mock,
    ):
        openai_mock.return_value = MagicMock()
        siliconflow_mock.return_value = MagicMock()

        try:
            get_llm_provider("unknown", config)
        except ValueError as exc:
            assert "Unknown LLM provider" in str(exc)
        else:
            raise AssertionError("Expected ValueError for unknown provider")


def test_none_provider_polish_returns_raw_text_unchanged() -> None:
    provider = NoneLLMProvider()
    raw_text = "um hello this is like a test"

    result = asyncio.run(provider.polish(raw_text))

    assert result == raw_text


def test_openai_provider_polish_uses_openai_client() -> None:
    config = SimpleNamespace(openai_api_key="openai-key")
    mock_client = MagicMock()
    mock_message = SimpleNamespace(content="Polished output")
    mock_choice = SimpleNamespace(message=mock_message)
    mock_response = SimpleNamespace(choices=[mock_choice])
    mock_client.chat.completions.create = AsyncMock(return_value=mock_response)

    with patch(
        "aurotype_engine.providers.llm_openai.openai.AsyncOpenAI",
        return_value=mock_client,
    ):
        provider = OpenAILLMProvider(config)
        result = asyncio.run(provider.polish("uh this is raw"))

    assert result == "Polished output"
    mock_client.chat.completions.create.assert_awaited_once()


def test_siliconflow_provider_polish_uses_openai_compatible_client() -> None:
    config = SimpleNamespace(siliconflow_api_key="sf-key")
    mock_client = MagicMock()
    mock_message = SimpleNamespace(content="Polished output")
    mock_choice = SimpleNamespace(message=mock_message)
    mock_response = SimpleNamespace(choices=[mock_choice])
    mock_client.chat.completions.create = AsyncMock(return_value=mock_response)

    with patch(
        "aurotype_engine.providers.llm_siliconflow.openai.AsyncOpenAI",
        return_value=mock_client,
    ):
        provider = SiliconFlowLLMProvider(config)
        result = asyncio.run(provider.polish("you know this is raw"))

    assert result == "Polished output"
    mock_client.chat.completions.create.assert_awaited_once()
