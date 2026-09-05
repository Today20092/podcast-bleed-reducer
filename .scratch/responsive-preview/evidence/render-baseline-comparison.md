# Render baseline comparison

Compared accepted commit `c268882` with responsive integration `a3f3c57` on September 5, 2026. Each release-gate binary was built from its stated checkout before running. Inputs were the three original `00005603_A01/A02/A03.wav` excerpts and `reference-labels.csv`; no protections or parameter overrides.

Both runs passed atomic render and independently verified format, BEXT, sample count, finite output and peak integrity. Both reported:

- Preservation track-samples: 2,565,744; changed: 100,693.
- Overlap track-samples: 131,376; changed: 0.
- Bleed input RMS: 0.004544658; output RMS: 0.003753987.
- Exact-unity automated acceptance gate: **fail**, identically in both builds.

All three rendered WAV files were byte-identical (SHA-256):

| Output | SHA-256 |
|---|---|
| 01-00005603_A01.wav | 6069A76179D07F1CCA2C3ED53836B741B48A459D32C3651590E81BB71D312792 |
| 02-00005603_A02.wav | E18485685CE100E81590EC47C6E56F578FE53C7DFF5AF1A4A66C9B533A0CFE00 |
| 03-00005603_A03.wav | 49847EBA048D8CDF8A62295028CC7A22BDAC1548E4E14EB9405FF592A6DDC7D9 |

This establishes that the gate failure is pre-existing, not introduced by gain construction or prepared-analysis changes. The detector-quality evidence quantifies the small one-pole recovery tails behind these strict non-unity counts. The gate has not been relaxed, and no new human listening sign-off is claimed. Final integration must repeat this comparison after playback/source-reading changes.

Reproduce with `cargo build --release --locked --bin release_gate -j 1`, then `release_gate <reference-labels.csv> <new-output-directory> <A01.wav> <A02.wav> <A03.wav>` in each checkout. A nonzero exit is expected for the recorded exact-unity failure. Compare output WAV hashes; do not mistake process exit for file-integrity failure.
