# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false, reportUnknownArgumentType=false, reportAny=false

import sys
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "engine"))

from aurotype_engine.server import app, _config_overrides

from fastapi.testclient import TestClient

client = TestClient(app)


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
        json={"stt_provider": "deepgram", "language": "en"},
    )
    assert response.status_code == 200
    assert response.json()["status"] == "configured"
    assert _config_overrides["stt_provider"] == "deepgram"
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
