# 02: Detect ownership from calibrated cross-microphone evidence

**Working branch:** codex/ownership-detector
**What to build:** A runnable joint detector that reduces delayed copied speech while preserving quiet independent speakers, using optional human examples and cached compact evidence for preview/render.
**Blocked by:** None
**Status:** ready-for-agent

- [ ] Run a small comparison experiment before settling evidence/thresholds; record results.
- [ ] Calibrated speech-band levels, lag-aware similarity/copy evidence, and independent-overlap preservation work on synthetic and real cases.
- [ ] Optional room tone and solo-speech settings affect decisions meaningfully; calibration never silently applies to changed sources.
- [ ] Cold/warm APIs share decisions, settings changes reuse detector/pairwise facts, cancellation is bounded and memory does not retain duplicate PCM.
- [ ] Final render accepts same settings and reports honest mode/reasons.
- [ ] Regression tests cover gain mismatch, lag, polarity, quiet overlap, invalid calibration and preview/render equivalence.
- [ ] Relevant checks and Standards/Spec review pass.

Read the spec, decisions and interface contract. Own detector/cache/engine/report; parent wires app after parallel calibration task. Legacy API behavior remains compatible.
