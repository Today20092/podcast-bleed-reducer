"""Read-only lag-search comparison; floating-point diagnostic, not the Rust oracle."""

import json

import numpy as np

from probe_copy import read_section


def filtered(samples):
    size = 1 << (len(samples) + 4096).bit_length()
    z = np.exp(-2j * np.pi * np.arange(size // 2 + 1) / size)
    response = np.ones_like(z)
    for hz, high in [(150, True), (3000, False)]:
        angle = 2 * np.pi * hz / 48000
        c, alpha = np.cos(angle), np.sin(angle) / np.sqrt(2)
        b = (
            [(1 + c) / 2, -(1 + c), (1 + c) / 2]
            if high
            else [(1 - c) / 2, 1 - c, (1 - c) / 2]
        )
        response *= (b[0] + b[1] * z + b[2] * z**2) / (
            1 + alpha - 2 * c * z + (1 - alpha) * z**2
        )
    return np.fft.irfft(np.fft.rfft(samples, size) * response, size)[: len(samples) : 6]


def explained(a, b, start, end, lag, step):
    start, end = max(start, -lag, 0), min(end, len(b) - max(lag, 0))
    x, y = a[start:end:step], b[start + lag : end + lag : step]
    return float(np.dot(x, y) ** 2 / max(np.dot(x, x) * np.dot(y, y), 1e-30))


def main():
    start = 3226 * 0.256  # Global block alignment; discard the filter startup.
    tracks = [
        filtered(
            read_section(
                f"C:/Users/User/Downloads/chopshow podcast 155_A0{i}.wav", start, 832
            )
        )
        for i in [1, 2, 3]
    ]
    output = []
    for target in [1, 2]:
        scores = [[], [], []]
        lags = [[], []]
        for block in range(0, len(tracks[0]) - 2048, 2048):
            a, b = tracks[0][block : block + 2048], tracks[target][block : block + 2048]
            lag = max(
                range(-96, 97), key=lambda lag: explained(a, b, 0, len(a), lag, 16)
            )
            for frame in range(8):
                offset = block + frame * 256
                time = start + offset / 8000
                if not 828 <= time < 831:
                    continue
                left, right = max(0, (frame - 1) * 256), min(2048, (frame + 2) * 256)
                scores[0].append(
                    max(
                        explained(a, b, left, right, candidate, 1)
                        for candidate in range(lag - 1, lag + 2)
                    )
                )
                full_lag = max(
                    range(-96, 97),
                    key=lambda candidate: explained(a, b, left, right, candidate, 1),
                )
                scores[1].append(explained(a, b, left, right, full_lag, 1))
                scores[2].append(
                    max(
                        explained(
                            tracks[0],
                            tracks[target],
                            offset - 256,
                            offset + 512,
                            candidate,
                            1,
                        )
                        for candidate in range(-96, 97)
                    )
                )
                lags[0].append(lag)
                lags[1].append(full_lag)
        output.append(
            {
                "pair": ["A01", f"A0{target + 1}"],
                "frames": len(scores[0]),
                "method_order": [
                    "current_block_lag",
                    "per_frame_full_search",
                    "full_search_across_blocks",
                ],
                "explained_power_p10_p50_p90": [
                    np.percentile(values, [10, 50, 90]).tolist() for values in scores
                ],
                "frames_at_least_025": [
                    sum(value >= 0.25 for value in values) for values in scores
                ],
                "lag_difference_over_1_sample": sum(
                    abs(a - b) > 1 for a, b in zip(*lags, strict=True)
                ),
            }
        )
    print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
