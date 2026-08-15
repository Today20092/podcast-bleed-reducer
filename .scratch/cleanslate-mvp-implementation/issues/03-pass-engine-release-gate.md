# 03 — Pass the engine release gate

**What to build:** The engine produces repeatable evidence that the MVP preserves intended material, reduces bleed, survives full-session use, and fails or cancels safely, so desktop work begins only after the processing contract is validated.

**Blocked by:** 02 — Automix the labeled reference session.

**Working branch:** `feature/cleanslate-mvp`

**Status:** attention

- [x] Demonstrate zero attenuation in labeled direct-speech, laughter-breath, and uncertain ranges.
- [x] Demonstrate unity gain for every intended track during genuine overlap and measurable bleed reduction versus unprocessed input.
- [x] Verify structural and relevant metadata preservation, absence of introduced clipping or invalid samples, and atomic failure/cancellation behavior.
- [ ] Record human listening sign-off that transitions are artifact-free on deterministic fixtures and the labeled reference session.
- [ ] Complete the full 83-minute reference session, exercise safe cancellation, and record a gross listening spot-check without requiring exhaustive annotation.
- [x] Judge release against the CleanSlate contract rather than Python numerical parity.

## Verification

- Labeled evidence: `../evidence/labeled-reference-4/release-gate.md` and `report.json`. The gate independently verified integrity, measured 2,565,744 preservation samples and 131,376 genuine-overlap samples with zero changes, and reduced bleed RMS from 0.004544658 to 0.003691280.
- Full-session evidence: `../evidence/full-session-2/release-gate.md` and `report.json`. All three 239,660,221-sample stems completed atomically in 586.300 seconds and passed independent format, BEXT, sample-count, finite-sample, and peak verification.
- Deterministic integration coverage exercises in-flight cancellation and forced output failure; neither leaves the destination directory.
- Automated verification: `cargo test`, `cargo fmt --all -- --check`, strict all-target Clippy, and `cargo build --release`.

## Attention needed

Listen to the processed stems beside their same-named sources at several attenuation boundaries listed in each `report.json`; include labeled overlap/preservation ranges and at least the start, middle, and end of the full session. If transitions are artifact-free and the gross full-session spot-check is acceptable, tick the two open criteria and change Status to `done`.
