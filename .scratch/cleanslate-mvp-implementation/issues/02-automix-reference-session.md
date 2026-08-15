# 02 — Automix the labeled reference session

**What to build:** A user can process the labeled Chopshow session without supplying decisions; the embedded CPU detector produces conservative ownership decisions, inspectable attenuation regions, smooth rendered stems, and a complete report using the validated engine path.

**Blocked by:** 01 — Render deterministic sessions end to end.

**Working branch:** `feature/cleanslate-mvp`

**Status:** done

- [x] Run the pinned embedded Silero model on CPU and combine its evidence with calibrated perceptual energy.
- [x] Produce only `Owned(track set)`, `Uncertain`, or `NoSpeech` across the engine seam; ambiguous human material remains at unity.
- [x] Convert decisions into categorized attenuation regions with timing, affected track, applied attenuation, evidence reasons, and review warnings.
- [x] Render the labeled Chopshow reference stems at fixed −18 dB attenuation for non-owners and `NoSpeech` while preserving overlap.
- [x] Record fixed thresholds and all settled input, output, region, warning, and protection evidence in the JSON report.
- [x] Keep lag-aware correlation out unless a labeled failure demonstrates that VAD plus calibrated energy cannot satisfy the contract.

## Verification

- Red-green integration slices cover CPU automix/report evidence, empty audio, and invalid ownership sets at the public render seam.
- `cargo test`: 6 passed; `cargo fmt -- --check`, strict all-target Clippy, and `cargo build --release`: passed.
- Labeled `00005603_A01.wav` through `A03.wav` reference run: passed in 226.07 seconds; report contained `Owned`, `Uncertain`, and `NoSpeech` regions.
- Scoped standards/spec review completed; trust-boundary findings were fixed. Release measurements and listening gate remain ticket 03 scope.
