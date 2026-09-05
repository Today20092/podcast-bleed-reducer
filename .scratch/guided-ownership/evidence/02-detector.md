# Detector experiment and implementation

Branch starts at shared contract 10f2cd0. Test seams are pre-approved in decisions.md, round 3, item 19. Implementation is separate from parent editor wiring and actual-session quality acceptance.

## Bounded comparison before implementation

A deterministic 4096-sample harmonic/modulated signal was compared with a copy delayed by 13 samples. Search covered signed lags -48 through 48. Squared normalized correlation removes polarity and scalar gain. Least-squares residual fraction is 1 minus squared correlation.

| Case | Recovered lag | Explained power | Residual fraction |
| --- | ---: | ---: | ---: |
| Copy at -0.25 gain | 13 | 1.000 | 0.000 |
| Same copy plus independent signal | 13 | 0.434 | 0.566 |
| Copy at 1.6 gain | 13 | 1.000 | below 0.000001 |

Conclusion: lag-aware evidence distinguishes the constructed overlap, but correlation cannot establish direction under recording gain mismatch. Solo examples must learn directional level relationships. These synthetic results do not establish real-recording quality.

Parent independently probed the provisional 828–831 second screenshot interval in the full originals. Median absolute speech-band correlation was 0.675 for A03/A01 and 0.585 for A02/A01, with level differences -15.72 and -18.45 dB. This excludes requiring near-perfect correlation as the only useful rule. The interval remains provisional and is not a training label here.

## Implemented algorithm

- Two Butterworth biquads form an approximate 150–3000 Hz sidechain at 48 kHz, then decimation by six supplies 8 kHz samples. This is a lightweight sidechain, not the separate 127-tap experimental anti-alias VAD path. Source samples and legacy VAD preprocessing stay unchanged.
- Filter state continues across 256 ms blocks. Each unordered microphone pair searches signed lags within 12 ms using sparse samples once per block. Each 32 ms decision stores squared correlation over up to 96 ms at the chosen lag and its neighboring samples. Windows truncate at block/file boundaries.
- Retained facts contain speech-band level per track/frame and explained power per unordered pair/frame. Full filtered audio is never retained. Three tracks use six floats per frame plus small calibration summaries. The existing VAD probabilities and broadband energy remain cached separately.
- Automatic floors use the tenth percentile. Directional level/residual medians use frames with at least 8 dB level separation, speech-band energy above floor, and squared correlation at least 0.25. Automatic suppression also requires residual energy below the receiving floor plus the selected energy margin, or squared correlation at least 0.9999. This independent bound prevents persistent overlap from teaching its own suppression threshold.
- Room-tone examples replace the selected track's floor with their median band level. Solo examples replace directional leakage and residual medians, including reversed recording gain. Example windows use only wholly contained analysis frames. The source-bound API rejects missing/stale stamps, invalid tracks/ranges, and intervals shorter than 250 ms. Near-silent solo examples fail with an explicit error.
- A copy must fit learned level and residual within 3 dB, have squared correlation at least 0.25, and have an adjacent copied frame. Multiple owners remain possible. Padding extends direct ownership but cannot override current stable copy evidence. Existing global attenuation and one-pole protections/gains are used without normalization or waveform subtraction.

## Review correction and limits

The first Spec review found that learning median residual from unlabelled frames could absorb continuous independent speech into the bleed model. A new regression reproduced this failure. Automatic residual acceptance now has the independent noise/purity bound described above. Explicit solo examples can establish an acoustic residual baseline; automatic overlap cannot.

The first Standards review found duplicated range validation. Range/source checks now live at the engine/cache boundary. The detector only checks whether a declared solo has usable energy. Parent integration may consolidate the local engine validator with ticket01's calibration helper.

Automatic mode intentionally preserves filtered/reverberant copies whose unexplained energy cannot be distinguished from independent speech. Guided examples enable those cases. Incorrect solo labels, independent speech below the learned acoustic residual, lag changes inside a block, delays beyond 12 ms, and heavy filtering remain limits. No numeric confidence or listening acceptance is claimed.

## Verification

Red/green tests reproduced legacy double ownership, missing public APIs, and the automatic persistent-overlap flaw before their fixes. Synthetic checks cover delayed/inverted copies, guided filtered copies, reversed gain, independent quiet overlap, a quiet third speaker, meaningful room tone, and cancellation. Public API checks cover cold/warm equality without re-preparation, invalid calibration/parameters, and rendered samples matching preview gains. Full checks and bounded timing results follow below. Actual labelled recordings and the provisional screenshot interval remain for parent task03; they were not used to tune these thresholds.

- `CARGO_BUILD_JOBS=1 cargo test`: 66 passed, zero failed, four intentionally ignored checks. The ignored checks are the gain benchmark, bounded joint benchmark, reference waveform visual evidence, and reference timeline responsiveness.
- `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: passed after the review correction.
- `cargo test --lib bounded_joint_feature_benchmark -- --ignored --nocapture`: passed. On 30 seconds of deterministic three-track synthetic audio in the unoptimized test build, feature preparation was 361.04 ms, numeric feature payload was 23,320 bytes, and p95 of 20 warm ownership derivations was 1.50 ms. This excludes PCM I/O, Silero inference and gain construction. It is not a full-session or release-build benchmark.
- Three full 239,660,221-sample stems require 156,029 analysis frames. The numeric feature count is six floats/frame, or 3,744,696 bytes before Vec spare capacity, floor values and small calibration summaries. `PreviewCache::joint_diagnostics()` reports measured payload capacity and feature preparation time for the parent benchmark.
- Standards re-review and Spec re-review at ae33bc3: zero remaining actionable findings. The original automatic-overlap P1 was reproduced, fixed, and re-reviewed.

No source WAV was copied or modified. No application was launched. Parent integration must replace editor worker/render calls with the named ownership APIs and may consolidate the engine's source/range validator with ticket01's matching calibration validator. Real-session quality and listening acceptance remain pending.
