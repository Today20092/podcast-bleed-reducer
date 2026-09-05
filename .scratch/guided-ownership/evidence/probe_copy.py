"""Read-only short-window copy measurements; no audio files are generated."""

import json
import struct
from pathlib import Path

import numpy as np


def read_section(path, start, end):
    with Path(path).open("rb") as source:
        assert source.read(4) == b"RIFF"
        source.read(4)
        assert source.read(4) == b"WAVE"
        fmt = None
        while chunk := source.read(8):
            kind, size = struct.unpack("<4sI", chunk)
            if kind == b"fmt ":
                fmt = source.read(size)
                if size % 2:
                    source.read(1)
            elif kind == b"data":
                assert fmt is not None
                tag, channels, rate, _, stride, bits = struct.unpack(
                    "<HHIIHH", fmt[:16]
                )
                if tag == 65534:
                    tag = struct.unpack("<H", fmt[24:26])[0]
                assert channels == 1 and rate == 48000 and bits == 32
                first, last = round(start * rate), round(end * rate)
                assert 0 <= first < last <= size // stride
                source.seek(first * stride, 1)
                samples = np.frombuffer(
                    source.read((last - first) * stride),
                    dtype="<f4" if tag == 3 else "<i4",
                ).astype(np.float64)
                if tag == 1:
                    samples /= 2147483648.0
                else:
                    assert tag == 3
                return samples
            else:
                source.seek(size + size % 2, 1)
    raise ValueError("missing WAV data")


def bandpass(samples):
    positions = np.arange(127) - 63
    high = 2 * 3400 / 48000 * np.sinc(2 * 3400 / 48000 * positions)
    low = 2 * 300 / 48000 * np.sinc(2 * 300 / 48000 * positions)
    taps = (high - low) * np.blackman(127)
    return np.convolve(samples, taps, "same")[::6]


def probe(paths, start, end):
    tracks = [bandpass(read_section(path, start, end)) for path in paths]
    summaries = []
    for left, right in [(0, 1), (0, 2), (1, 2)]:
        correlations, lags, differences = [], [], []
        for offset in range(0, len(tracks[left]) - 768, 256):
            a, b = [
                track[offset : offset + 768] for track in [tracks[left], tracks[right]]
            ]
            a, b = a - np.mean(a), b - np.mean(b)
            ea, eb = np.dot(a, a), np.dot(b, b)
            if min(ea, eb) < 1e-12:
                continue
            correlation = np.fft.irfft(
                np.conj(np.fft.rfft(a, 2048)) * np.fft.rfft(b, 2048), 2048
            )
            candidates = np.concatenate((correlation[-96:], correlation[:97]))
            index = np.argmax(np.abs(candidates))
            correlations.append(float(abs(candidates[index]) / np.sqrt(ea * eb)))
            lags.append(float((index - 96) / 8))
            differences.append(float(10 * np.log10(ea / eb)))
        summaries.append(
            {
                "pair": [Path(paths[left]).name, Path(paths[right]).name],
                "frames": len(correlations),
                "absolute_correlation_p10_p50_p90": np.percentile(
                    correlations, [10, 50, 90]
                ).tolist(),
                "left_minus_right_db_p10_p50_p90": np.percentile(
                    differences, [10, 50, 90]
                ).tolist(),
                "lag_ms_p10_p50_p90": np.percentile(lags, [10, 50, 90]).tolist(),
            }
        )
    return {
        "start_seconds": start,
        "end_seconds": end,
        "speech_band_rms": [float(np.sqrt(np.mean(track * track))) for track in tracks],
        "pairs": summaries,
    }


if __name__ == "__main__":
    sources = [
        f"C:/Users/User/Downloads/chopshow podcast 155_A0{track}.wav"
        for track in [3, 2, 1]
    ]
    print(
        json.dumps(
            {
                "method": "300-3400 Hz, 8kHz sidechain, 96ms windows/32ms hop, +/-12ms normalized cross-correlation; exploratory not classifier",
                "tracks": sources,
                "intervals": [probe(sources, 828, 831), probe(sources, 819, 847)],
            },
            indent=2,
        )
    )
