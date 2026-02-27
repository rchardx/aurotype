from __future__ import annotations

import io
import threading
import wave
from typing import ClassVar, Protocol, cast

import numpy as np
from numpy.typing import NDArray

try:
    import sounddevice as sd  # pyright: ignore[reportMissingTypeStubs]
except OSError as exc:
    sd = None
    sounddevice_import_error: OSError | None = exc
else:
    sounddevice_import_error = None


class _InputStreamLike(Protocol):
    def start(self) -> None: ...

    def stop(self) -> None: ...

    def close(self) -> None: ...


class AudioRecorderError(RuntimeError):
    pass


class AudioDeviceError(AudioRecorderError):
    pass


class AudioRecorder:
    _SAMPLE_RATE: ClassVar[int] = 16000
    _CHANNELS: ClassVar[int] = 1
    _DTYPE: ClassVar[str] = "int16"
    _SAMPLE_WIDTH_BYTES: ClassVar[int] = 2

    def __init__(self) -> None:
        self._stream: _InputStreamLike | None = None
        self._chunks: list[NDArray[np.int16]] = []
        self._latest_chunk: NDArray[np.int16] | None = None
        self._lock: threading.Lock = threading.Lock()
        self._is_recording: bool = False

    @property
    def is_recording(self) -> bool:
        with self._lock:
            return self._is_recording

    def start_recording(self) -> None:
        if sd is None:
            raise AudioDeviceError(
                f"Unable to initialize sounddevice/PortAudio: {sounddevice_import_error}"
            )

        with self._lock:
            if self._is_recording:
                return
            self._chunks = []
            self._latest_chunk = None

        try:
            stream = cast(
                _InputStreamLike,
                sd.InputStream(
                    samplerate=self._SAMPLE_RATE,
                    channels=self._CHANNELS,
                    dtype=self._DTYPE,
                    callback=self._audio_callback,
                ),
            )
            stream.start()
        except sd.PortAudioError as exc:
            raise AudioDeviceError(self._describe_portaudio_error(exc)) from exc

        with self._lock:
            self._stream = stream
            self._is_recording = True

    def stop_recording(self) -> bytes:
        stream_to_close: _InputStreamLike | None = None
        with self._lock:
            stream_to_close = self._stream
            self._stream = None
            self._is_recording = False

        if stream_to_close is not None:
            try:
                stream_to_close.stop()
                stream_to_close.close()
            except Exception as exc:
                raise AudioDeviceError(
                    f"Failed to stop audio capture stream: {exc}"
                ) from exc

        with self._lock:
            chunks = [chunk.copy() for chunk in self._chunks]
            self._chunks = []

        if chunks:
            audio = np.concatenate(chunks)
        else:
            audio = np.array([], dtype=np.int16)

        return self._to_wav_bytes(audio)

    def get_volume(self) -> float:
        with self._lock:
            latest = None if self._latest_chunk is None else self._latest_chunk.copy()

        if latest is None or latest.size == 0:
            return 0.0

        chunk_f32: NDArray[np.float32] = latest.astype(np.float32)
        rms = float(cast(float, np.sqrt(np.mean(chunk_f32 * chunk_f32)) / 32768.0))
        if rms < 0.0:
            return 0.0
        if rms > 1.0:
            return 1.0
        return rms

    def _audio_callback(
        self,
        indata: NDArray[np.int16],
        frames: int,
        time_info: dict[str, float],
        status: object,
    ) -> None:
        del frames, time_info, status
        chunk = cast(
            NDArray[np.int16], np.asarray(indata, dtype=np.int16).reshape(-1).copy()
        )
        with self._lock:
            self._chunks.append(chunk)
            self._latest_chunk = chunk

    @classmethod
    def _to_wav_bytes(cls, audio: NDArray[np.int16]) -> bytes:
        buffer = io.BytesIO()
        with wave.open(buffer, "wb") as wav:
            wav.setnchannels(cls._CHANNELS)
            wav.setsampwidth(cls._SAMPLE_WIDTH_BYTES)
            wav.setframerate(cls._SAMPLE_RATE)
            wav.writeframes(audio.astype(np.int16, copy=False).tobytes())
        return buffer.getvalue()

    @staticmethod
    def _describe_portaudio_error(exc: Exception) -> str:
        base = str(exc)
        lowered = base.lower()

        if "permission" in lowered or "denied" in lowered:
            reason = "Microphone permission denied"
        elif "busy" in lowered or "in use" in lowered:
            reason = "Microphone device is busy"
        elif "no default input device" in lowered or "no device" in lowered:
            reason = "No microphone input device found"
        else:
            reason = "Unable to start microphone capture"

        return f"{reason}: {base}"
