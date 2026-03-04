# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false, reportUnknownArgumentType=false, reportAny=false

import io
import struct
import sys
import wave
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "engine"))

from aurotype_engine.server import app, _config_overrides, _is_silent

from fastapi.testclient import TestClient

client = TestClient(app)


def _make_wav(samples: list[int], sample_rate: int = 16000) -> bytes:
    """Create a WAV file bytes from a list of 16-bit signed samples."""
    buf = io.BytesIO()
    with wave.open(buf, "wb") as wav:
        wav.setnchannels(1)
        wav.setsampwidth(2)  # 16-bit
        wav.setframerate(sample_rate)
        raw = struct.pack(f"<{len(samples)}h", *samples)
        wav.writeframes(raw)
    return buf.getvalue()


def test_health_endpoint() -> None:
    """GET /health returns status ok and version."""
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "ok"
    assert data["version"] == "0.1.0"


def test_configure_endpoint() -> None:
    """POST /configure updates config overrides."""
    _config_overrides.clear()
    response = client.post(
        "/configure",
        json={"stt_provider": "aliyun_dashscope", "language": "en"},
    )
    assert response.status_code == 200
    assert response.json()["status"] == "configured"
    assert _config_overrides["stt_provider"] == "aliyun_dashscope"
    assert _config_overrides["language"] == "en"
    _config_overrides.clear()


def test_polish_endpoint() -> None:
    """POST /polish calls LLM provider and returns polished text."""
    mock_llm = MagicMock()
    mock_llm.polish = AsyncMock(return_value="Polished text.")

    with patch("aurotype_engine.server.get_llm_provider", return_value=mock_llm):
        response = client.post("/polish", json={"text": "uh raw text"})

    assert response.status_code == 200
    assert response.json()["text"] == "Polished text."
    mock_llm.polish.assert_called_once()


def test_transcribe_endpoint() -> None:
    """POST /transcribe calls STT provider and returns transcribed text."""
    mock_stt = MagicMock()
    mock_stt.transcribe = AsyncMock(return_value="hello world")

    with patch("aurotype_engine.server.get_stt_provider", return_value=mock_stt):
        response = client.post(
            "/transcribe",
            files={"audio": ("test.wav", b"fake-audio-bytes", "audio/wav")},
        )

    assert response.status_code == 200
    assert response.json()["text"] == "hello world"
    mock_stt.transcribe.assert_called_once()


def test_process_endpoint() -> None:
    """POST /process calls process_voice_input and returns result."""
    mock_result = {
        "raw_text": "raw",
        "polished_text": "polished",
        "audio_data": "base64data",
    }

    with patch(
        "aurotype_engine.server.process_voice_input",
        new_callable=AsyncMock,
        return_value=mock_result,
    ):
        response = client.post(
            "/process",
            files={"audio": ("test.wav", b"fake-audio", "audio/wav")},
        )

    assert response.status_code == 200
    data = response.json()
    assert data["raw_text"] == "raw"
    assert data["polished_text"] == "polished"
    assert data["audio_data"] == "base64data"


# --- _is_silent tests ---


def test_is_silent_returns_true_for_silent_audio() -> None:
    """Audio with very low amplitude is detected as silent."""
    # All zeros = complete silence
    silent_wav = _make_wav([0, 0, 0, 0, 0])
    assert _is_silent(silent_wav) is True


def test_is_silent_returns_true_for_very_quiet_audio() -> None:
    """Audio with RMS below threshold is detected as silent."""
    # Small values: RMS ~ 0.00003, well below default threshold 0.005
    quiet_wav = _make_wav([1, -1, 2, -2, 1])
    assert _is_silent(quiet_wav) is True


def test_is_silent_returns_false_for_loud_audio() -> None:
    """Audio with RMS above threshold is not silent."""
    # Large values: RMS ~ 0.5, well above threshold
    loud_wav = _make_wav([16000, -16000, 16000, -16000])
    assert _is_silent(loud_wav) is False


def test_is_silent_returns_true_for_empty_samples() -> None:
    """WAV with no samples is treated as silent."""
    empty_wav = _make_wav([])
    assert _is_silent(empty_wav) is True


def test_is_silent_respects_custom_threshold() -> None:
    """Custom threshold can make quiet audio non-silent."""
    quiet_wav = _make_wav([100, -100, 100, -100])
    # Default threshold 0.005: quiet_wav is silent
    assert _is_silent(quiet_wav) is True
    # Lower threshold: same audio is not silent
    assert _is_silent(quiet_wav, threshold=0.001) is False
