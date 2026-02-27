#!/usr/bin/env python
"""Generate test WAV audio files for STT pipeline testing."""

import wave
import struct
import numpy as np
from pathlib import Path


def generate_wav(filename, duration_s, sample_rate=16000, freq=440):
    """Generate a WAV file with a sine tone.

    Args:
        filename: Path to output WAV file
        duration_s: Duration in seconds
        sample_rate: Sample rate in Hz (default 16000)
        freq: Frequency of sine tone in Hz (default 440)
    """
    # Generate sine wave samples
    num_samples = sample_rate * duration_s
    samples = np.sin(2 * np.pi * freq * np.arange(num_samples) / sample_rate)

    # Convert to 16-bit PCM
    samples_int16 = (samples * 32767).astype(np.int16)

    # Write WAV file
    with wave.open(str(filename), "w") as wf:
        wf.setnchannels(1)  # mono
        wf.setsampwidth(2)  # 16-bit = 2 bytes
        wf.setframerate(sample_rate)
        wf.writeframes(samples_int16.tobytes())

    print(f"Generated {filename}: {duration_s}s at {sample_rate}Hz, {freq}Hz tone")


if __name__ == "__main__":
    tests_dir = Path(__file__).parent

    # Generate 5-second test audio
    generate_wav(tests_dir / "test_audio.wav", duration_s=5)

    # Generate 1-second test audio
    generate_wav(tests_dir / "test_audio_short.wav", duration_s=1)

    print("Test audio files generated successfully!")
