# Detector preprocessing comparison

September 5, 2026. Retain legacy preprocessing in the app and render. The tested filter failed promotion on the supplied human labels. The experimental path is available only in the comparison runner; no persisted app option was added. No source WAV was written or copied. No listening was performed.

## Reproduce

From this checkout, run PowerShell:

```powershell
cargo +1.95.0 run --bin detector_quality --target-dir target/detector-quality -j 1 -- .scratch/responsive-preview/evidence/reference-labels.csv C:/Users/User/Downloads/00005603_A01.wav C:/Users/User/Downloads/00005603_A02.wav C:/Users/User/Downloads/00005603_A03.wav > comparison.json
cargo +1.95.0 test --all-targets --target-dir target/detector-quality -j 1
cargo +1.95.0 clippy --all-targets --target-dir target/detector-quality -j 1 -- -D warnings
cargo +1.95.0 fmt --all -- --check
```

The first command reads original files, runs two preparations, reuses their facts across two thresholds, and writes JSON only. Full per-label gains, attenuation, microphone RMS, settings, and metadata are in `06-detector-quality.json`. The runner deliberately accepts the reference CSV's five-column format, not arbitrary CSV dialects. Unknown tracks, labels, invalid times, and out-of-range spans fail instead of disappearing from the score.

## Inputs and measurement

Track order is A01, A02, A03. Each excerpt contains 14,550,536 mono samples at 48,000 Hz, or 303.136167 seconds. Each has 32-bit format tag 65534 and BEXT time reference 8,973,764. These measured metadata control the evaluation; no full-session offset is added to the excerpt-relative labels. Validation returned no warnings.

SHA256 of the original read-only files:

| File | SHA256 |
|---|---|
| 00005603_A01.wav | E8921536F2C4E5F413B44459D427E9B20BFBC664D841A4D86A4071EA6B76139D |
| 00005603_A02.wav | B41E463CAD465E9D77F558B204FDAF86161F6E91DA8ADC1A91428DD5844D2AD9 |
| 00005603_A03.wav | 21344A74943A0B9F5B4C43E97E1120F46980E07A42AB7CBDBD2A7877B6F93A13 |
| reference-labels.csv | 8B835F449AFBBF301252FEB450001739653837D0DBFC157C13908156DE149EF1 |

There are 36 labels: 20 preservation, 13 bleed, and 3 room-noise spans. Preservation covers 2,565,744 track-samples; simultaneous labeled speech covers 131,376 unique timeline samples, or 2.737 seconds. Room noise is reported per span but excluded from speech and bleed totals. A01's bleed and direct-speech labels overlap by 8 ms near 17.36 seconds; labels are retained as supplied, so that short interval contributes to both categories. Unlabeled spans make no contribution.

Changed preservation means the shared gain envelope is not exactly unity, including tiny one-pole recovery tails and zero-valued input samples. This is stricter than checking quantized rendered samples and is not an audibility metric. Bleed attenuation is 10 log10 of summed output energy divided by summed input energy across bleed labels. More negative is more reduction. It uses the original samples and shared gain envelopes without generating rendered audio. Overlap is counted once even with three labeled speakers; any participating track below unity marks that timeline sample changed.

## Results and default decision

All runs use 6 dB bleed discrimination, -18 dB attenuation, 512 ms speech padding, 50 ms Open, and 500 ms Close. Threshold 0.50 is the app default; 0.20 is the previously recorded session setting.

| Preprocessing | Threshold | Changed preservation samples | Changed overlap samples | Bleed attenuation dB | Regions |
|---|---:|---:|---:|---:|---:|
| Legacy stride 3 | 0.20 | 57,841 | 0 | -0.660257 | 163 |
| Filtered | 0.20 | 79,427 | 0 | -0.439773 | 176 |
| Legacy stride 3 | 0.50 | 100,693 | 0 | -1.660171 | 168 |
| Filtered | 0.50 | 100,693 | 0 | -1.132610 | 172 |

At threshold 0.20 filtering adds 21,586 changed preservation track-samples and loses 0.220484 dB of bleed reduction. At 0.50 it ties the changed-sample count and loses 0.527560 dB of bleed reduction. Both preserve all labeled overlap. This gives no basis for default promotion.

The legacy exact-unity preservation failure is pre-existing. Its worst labeled minimum gain is 0.997191 on A01 at 228.511–229.135 seconds. Other affected default-threshold labels have minimum gains of at least 0.999714. The filter's extra changed samples at 0.20 are mostly A01's 17.360–20.450 span, whose minimum gain is 0.999753. These are small recovery-tail differences, not evidence of audible word loss. The strict preservation gate remains failed for both paths; this ticket does not weaken it or claim release acceptance. The parent's independent rendered release gate may count differently because of float multiplication and PCM quantization.

## Cross-microphone evidence

The runner reports raw RMS from all three original microphones over every label. Five of twenty labeled direct-speech spans have a different microphone with the largest RMS:

| Labeled direct speech | Seconds | Direct RMS | Larger microphone RMS |
|---|---|---:|---|
| A03 | 0.000–1.402 | 0.033548 | A01 0.038143 |
| A01 | 17.360–20.450 | 0.032966 | A03 0.035579 |
| A03 | 105.961–110.012 | 0.040082 | A02 0.057350 |
| A03 | 111.337–114.757 | 0.037727 | A01 0.039776 |
| A01 | 228.511–229.135 | 0.015492 | A02 0.017925 |

None of the thirteen bleed-labeled spans has its labeled microphone ranked largest by whole-span RMS. That suggests comparative energy deserves further study, but does not validate a frame-level selector. Whole-span RMS hides changes of speaker and differs with gain, distance, and overlap. A single-owner rule necessarily discards at least one labeled speaker over the 2.737 seconds of known overlap. Retain categorical multi-owner and uncertain decisions. No winner-takes-all or cross-microphone ranking change was installed.

## Implementation and checks

The filter is a 127-tap unity-DC Blackman-windowed sinc with a 7 kHz cutoff, evaluated only at every third source sample. Symmetry halves the multiplications. It reads across model frames, compensates its 63-source-sample group delay by centering on the original timestamp, and zero-extends only at file edges. Model frames remain 512 samples at 16 kHz with the existing 64-sample context and final-frame zero padding. Energy/noise-floor analysis still uses original 48 kHz audio. No resampled PCM file or full resampled buffer is created.

The preparation identity includes model, filter length/window/cutoff, alignment, edge behavior, energy framing, and a revision. The legacy identity is unchanged. A cache regression switches legacy to filter and back, verifies re-preparation and ownership invalidation, verifies warm reuse, and restores the legacy preview exactly.

Signal fixtures test 100 Hz, 1 kHz, and 6 kHz gain within 1%; 8, 9, 12, and 20 kHz residual RMS below 0.001 for unit-amplitude tones; unit DC; centered impulse at a model-frame boundary; odd sample count; empty and one-sample input. This is a sampled frequency check, not a claim about every stopband frequency. Tone generation uses f64 phase to avoid injecting f32 oscillator noise into the rejection test.

TDD observed missing-filter and missing-label-parser compilation failures before implementation. The filter fixture caught oscillator precision noise and passed after correcting the fixture. Additional checks cover known gain/energy values and unique overlap counting for three speakers. Graph indexing timed out after 300 seconds; caller discovery fell back to source inspection. Builds use Rust 1.95.0 with one compiler job because the installed default toolchain failed in dependencies and the shared host has memory pressure.

Full-session performance measurement remains with the parent integration task.

## Verification and review

- Full `cargo test --all-targets`: 49 passed, zero failed, three existing long-session/visual/navigation evidence checks ignored. These ignored checks belong to integration evidence, not this detector comparison.
- Final signal fixture with nonzero tone phase passed, including the 8 kHz Nyquist case. Final library regression rerun passed after iterator cleanup.
- Strict `cargo clippy --all-targets -- -D warnings` and `cargo fmt --all -- --check` passed. Clippy requested two mechanical iterator/borrow cleanups, both applied.
- Standards review of `git diff e3cd247...HEAD` found one tracker-status convention issue, corrected to a canonical triage label with completion recorded separately. No actionable code-smell findings.
- Spec review of the same diff found no missing, incorrect, or excess requirements. It verified filter alignment and edges, cache identity, measured rejection, and the authorized runner-only scope.
