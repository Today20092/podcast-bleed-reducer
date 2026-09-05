# 02: Detect ownership from calibrated cross-microphone evidence

**Working branch:** codex/ownership-detector
**What to build:** A runnable joint detector that reduces delayed copied speech while preserving quiet independent speakers, using optional human examples and cached compact evidence for preview/render.
**Blocked by:** None
**Status:** ready-for-agent
**Implementation completion:** complete on codex/ownership-detector; parent integration and real-audio acceptance remain separate.

- [x] Run a small comparison experiment before settling evidence/thresholds; record results.
- [ ] Calibrated speech-band levels, lag-aware similarity/copy evidence, and independent-overlap preservation work on synthetic and real cases.
- [x] Optional room tone and solo-speech settings affect decisions meaningfully; calibration never silently applies to changed sources.
- [x] Cold/warm APIs share decisions, settings changes reuse detector/pairwise facts, cancellation is bounded and memory does not retain duplicate PCM.
- [x] Final render accepts same settings and reports honest mode/reasons.
- [x] Regression tests cover gain mismatch, lag, polarity, quiet overlap, invalid calibration and preview/render equivalence.
- [x] Relevant checks and Standards/Spec review pass.

Read the spec, decisions and interface contract. Own detector/cache/engine/report; parent wires app after parallel calibration task. Legacy API behavior remains compatible.

## Completion evidence

Implementation commits: 0cd5f6e and ae33bc3. Full suite: 66 passed, four intentionally ignored; the bounded feature benchmark was then run separately and passed. Strict Clippy and formatting checks pass. Standards and Spec re-reviews at ae33bc3 have no remaining actionable findings. See ../evidence/02-detector.md for algorithm, red/green evidence, timing and limits.

The unchecked real-case acceptance belongs to parent task03. Synthetic ownership cases pass, including the automatic persistent-overlap regression. Automatic mode deliberately preserves nontrivial unexplained residual unless it is near background; human solo examples enable acoustic-filtered bleed decisions. App wiring, original-recording evaluation, full-session warm timing, final EXE, and human listening are not completed by this ticket branch. The canonical Status field was not changed.
