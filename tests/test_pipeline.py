# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false, reportUnknownArgumentType=false, reportAny=false

import asyncio
import base64
import sys
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "engine"))

from aurotype_engine.pipeline import process_voice_input


def _build_config() -> SimpleNamespace:
    return SimpleNamespace(
        stt_provider="aliyun_dashscope",
        llm_provider="none",
        language="auto",
        aliyun_dashscope_api_key="test-key",
        stt_model=None,
    )


def test_process_voice_input_happy_path() -> None:
    """Mock STT + LLM, verify returns dict with raw_text, polished_text, audio_data."""
    audio = b"fake-audio-bytes"
    config = _build_config()

    mock_stt = MagicMock()
    mock_stt.transcribe = AsyncMock(return_value="hello world")

    mock_llm = MagicMock()
    mock_llm.polish = AsyncMock(return_value="Hello, world.")

    with (
        patch("aurotype_engine.pipeline.get_stt_provider", return_value=mock_stt),
        patch("aurotype_engine.pipeline.get_llm_provider", return_value=mock_llm),
    ):
        result = asyncio.run(process_voice_input(audio, config))

    assert result["raw_text"] == "hello world"
    assert result["polished_text"] == "Hello, world."
    assert result["audio_data"] == base64.b64encode(audio).decode("ascii")
    mock_stt.transcribe.assert_awaited_once_with(audio, language="auto")
    mock_llm.polish.assert_awaited_once_with("hello world", language="auto")


def test_stt_retry_succeeds_on_third_attempt() -> None:
    """Mock STT to fail twice then succeed, verify it retries."""
    audio = b"audio"
    config = _build_config()

    mock_stt = MagicMock()
    mock_stt.transcribe = AsyncMock(
        side_effect=[RuntimeError("fail 1"), RuntimeError("fail 2"), "ok text"]
    )

    mock_llm = MagicMock()
    mock_llm.polish = AsyncMock(return_value="polished")

    with (
        patch("aurotype_engine.pipeline.get_stt_provider", return_value=mock_stt),
        patch("aurotype_engine.pipeline.get_llm_provider", return_value=mock_llm),
        patch(
            "aurotype_engine.pipeline.asyncio.sleep", new_callable=AsyncMock
        ) as mock_sleep,
    ):
        result = asyncio.run(process_voice_input(audio, config))

    assert result["raw_text"] == "ok text"
    assert mock_stt.transcribe.await_count == 3
    assert mock_sleep.await_count == 2


def test_stt_all_retries_fail_raises_runtime_error() -> None:
    """Mock STT to always fail, verify raises RuntimeError after 3 attempts."""
    audio = b"audio"
    config = _build_config()

    mock_stt = MagicMock()
    mock_stt.transcribe = AsyncMock(side_effect=RuntimeError("always fails"))

    with (
        patch("aurotype_engine.pipeline.get_stt_provider", return_value=mock_stt),
        patch("aurotype_engine.pipeline.asyncio.sleep", new_callable=AsyncMock),
    ):
        try:
            asyncio.run(process_voice_input(audio, config))
            assert False, "Expected RuntimeError"
        except RuntimeError as exc:
            assert "STT failed after 3 attempts" in str(exc)
            assert "always fails" in str(exc)

    assert mock_stt.transcribe.await_count == 3


def test_llm_failure_after_stt_success() -> None:
    """Verify LLM error propagates after STT succeeds."""
    audio = b"audio"
    config = _build_config()

    mock_stt = MagicMock()
    mock_stt.transcribe = AsyncMock(return_value="raw text")

    mock_llm = MagicMock()
    mock_llm.polish = AsyncMock(side_effect=RuntimeError("LLM broke"))

    with (
        patch("aurotype_engine.pipeline.get_stt_provider", return_value=mock_stt),
        patch("aurotype_engine.pipeline.get_llm_provider", return_value=mock_llm),
    ):
        try:
            asyncio.run(process_voice_input(audio, config))
            assert False, "Expected RuntimeError"
        except RuntimeError as exc:
            assert "LLM broke" in str(exc)


def test_base64_audio_encoding_in_result() -> None:
    """Verify the audio_data field contains correct base64 encoding."""
    audio = b"\x00\x01\x02\xff\xfe\xfd"
    config = _build_config()

    mock_stt = MagicMock()
    mock_stt.transcribe = AsyncMock(return_value="text")

    mock_llm = MagicMock()
    mock_llm.polish = AsyncMock(return_value="text")

    with (
        patch("aurotype_engine.pipeline.get_stt_provider", return_value=mock_stt),
        patch("aurotype_engine.pipeline.get_llm_provider", return_value=mock_llm),
    ):
        result = asyncio.run(process_voice_input(audio, config))

    decoded = base64.b64decode(result["audio_data"])
    assert decoded == audio


def test_empty_transcription_skips_llm() -> None:
    """When STT returns empty text (silence), LLM should not be called."""
    audio = b"silent-audio"
    config = _build_config()

    mock_stt = MagicMock()
    mock_stt.transcribe = AsyncMock(return_value="")

    mock_llm = MagicMock()
    mock_llm.polish = AsyncMock(return_value="should not be called")

    with (
        patch("aurotype_engine.pipeline.get_stt_provider", return_value=mock_stt),
        patch("aurotype_engine.pipeline.get_llm_provider", return_value=mock_llm),
    ):
        result = asyncio.run(process_voice_input(audio, config))

    assert result["raw_text"] == ""
    assert result["polished_text"] == ""
    assert result["audio_data"] == base64.b64encode(audio).decode("ascii")
    mock_llm.polish.assert_not_awaited()


def test_whitespace_transcription_skips_llm() -> None:
    """When STT returns only whitespace, treat as empty and skip LLM."""
    audio = b"silent-audio"
    config = _build_config()

    mock_stt = MagicMock()
    mock_stt.transcribe = AsyncMock(return_value="   \n  ")

    mock_llm = MagicMock()
    mock_llm.polish = AsyncMock(return_value="should not be called")

    with (
        patch("aurotype_engine.pipeline.get_stt_provider", return_value=mock_stt),
        patch("aurotype_engine.pipeline.get_llm_provider", return_value=mock_llm),
    ):
        result = asyncio.run(process_voice_input(audio, config))

    assert result["raw_text"] == ""
    assert result["polished_text"] == ""
    mock_llm.polish.assert_not_awaited()
