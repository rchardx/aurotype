# pyright: reportMissingImports=false, reportUnknownVariableType=false, reportUnknownMemberType=false, reportUnknownArgumentType=false, reportAny=false

import io
import struct
import sys
import wave
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "engine"))

from aurotype_engine.audio import AudioRecorder


def test_to_wav_bytes_produces_valid_wav() -> None:
    """_to_wav_bytes() produces valid WAV with correct headers."""
    samples = np.array([0, 100, -100, 32767, -32768], dtype=np.int16)
    wav_bytes = AudioRecorder._to_wav_bytes(samples)

    # Must start with RIFF header
    assert wav_bytes[:4] == b"RIFF"
    # Must contain WAVE format marker
    assert wav_bytes[8:12] == b"WAVE"

    # Parse with wave module to verify headers
    buf = io.BytesIO(wav_bytes)
    with wave.open(buf, "rb") as wav:
        assert wav.getnchannels() == 1
        assert wav.getsampwidth() == 2
        assert wav.getframerate() == 16000
        assert wav.getnframes() == 5

        # Verify sample data matches
        raw_frames = wav.readframes(5)
        decoded = struct.unpack("<5h", raw_frames)
        assert list(decoded) == [0, 100, -100, 32767, -32768]


def test_to_wav_bytes_empty_audio() -> None:
    """_to_wav_bytes() with empty array produces valid zero-frame WAV."""
    samples = np.array([], dtype=np.int16)
    wav_bytes = AudioRecorder._to_wav_bytes(samples)

    buf = io.BytesIO(wav_bytes)
    with wave.open(buf, "rb") as wav:
        assert wav.getnchannels() == 1
        assert wav.getsampwidth() == 2
        assert wav.getframerate() == 16000
        assert wav.getnframes() == 0


def test_get_volume_returns_zero_when_not_recording() -> None:
    """get_volume() returns 0.0 when not recording (no chunks)."""
    recorder = AudioRecorder()
    assert recorder.get_volume() == 0.0


def test_is_recording_defaults_to_false() -> None:
    """is_recording property defaults to False."""
    recorder = AudioRecorder()
    assert recorder.is_recording is False


def test_init_state_no_stream_empty_chunks() -> None:
    """AudioRecorder init: no stream, empty chunks."""
    recorder = AudioRecorder()
    assert recorder._stream is None
    assert recorder._chunks == []
    assert recorder._latest_chunk is None
    assert recorder._is_recording is False


def test_describe_portaudio_error_permission_denied() -> None:
    """_describe_portaudio_error returns permission message for permission errors."""
    exc = Exception("Permission denied by the OS")
    msg = AudioRecorder._describe_portaudio_error(exc)
    assert "Microphone permission denied" in msg
    assert "Permission denied by the OS" in msg


def test_describe_portaudio_error_device_busy() -> None:
    """_describe_portaudio_error returns busy message for busy errors."""
    exc = Exception("Device is busy or in use")
    msg = AudioRecorder._describe_portaudio_error(exc)
    assert "Microphone device is busy" in msg


def test_describe_portaudio_error_no_device() -> None:
    """_describe_portaudio_error returns no-device message."""
    exc = Exception("No default input device available")
    msg = AudioRecorder._describe_portaudio_error(exc)
    assert "No microphone input device found" in msg


def test_describe_portaudio_error_unknown() -> None:
    """_describe_portaudio_error returns generic message for unknown errors."""
    exc = Exception("Something weird happened")
    msg = AudioRecorder._describe_portaudio_error(exc)
    assert "Unable to start microphone capture" in msg
    assert "Something weird happened" in msg


def test_get_volume_with_latest_chunk() -> None:
    """get_volume() computes RMS when a chunk is available."""
    recorder = AudioRecorder()
    # Manually set a chunk to simulate recording
    recorder._latest_chunk = np.array([16384, 16384, 16384, 16384], dtype=np.int16)
    vol = recorder.get_volume()
    assert 0.0 < vol < 1.0
