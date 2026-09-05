# Delegated grilling

The user authorizes the full Matt workflow, automatic answers, separate implementation tasks/worktrees, integration, and a Windows test build. They will open the application themselves. Base: 135fbb1. Integration: codex/cleanslate-ownership.

## Round 1: Product

1. Fully automatic or guided? Automatic initial estimates with optional, editable human calibration.
2. What does room tone teach? A particular microphone's background floor. It cannot identify which speaker owns speech.
3. What additional example helps? A selected interval where one participant speaks alone; compare their assigned microphone with all other simultaneous tracks.
4. Is calibration destructive? No. Store labeled ranges and source identities, not processed audio or copied samples.
5. Required examples? None to import/analyze. Recommend room tone and solo speech for each microphone when automatic discrimination is insufficient.
6. Defaults and compatibility? New projects enable joint ownership. Old projects with no ownership settings keep legacy behavior until the user enables the visibly labeled control. Persist the choice.

## Round 2: Decisions and safety

7. Calibrate raw microphone amplitude only? No: calibrated relative speech-band levels plus delayed-copy evidence. Preserve quiet independent speech even when another microphone is louder.
8. Correlation alone decisive? No. Combine relative level/leakage expectations and independent residual evidence. Handle polarity, modest lag and acoustic filtering conservatively.
9. Overlap? Multiple owners are valid. Never force a single winner or constant-total-gain normalization.
10. Temporal behavior? Smooth evidence/hysteresis then apply existing preservation padding and one-pole gain. Padding must not reintroduce bleed ownership from another microphone.
11. Human annotation mistakes? Validate intervals/tracks/source identity, explain room tone versus solo speech, make examples visible and removable. Do not silently trust examples after sources change.
12. Source changes? Invalidate calibration for changed/reordered/relinked sources and keep preview/audio identities strict.
13. Audio contract? Preserve wanted samples through established gains/protections; attenuation only, no waveform subtraction or neural reconstruction.
14. Controls? Keep sensitivity and gain controls; distinguish own-floor speech evidence from cross-microphone bleed separation. Add only controls justified by the implemented classifier.

## Round 3: Performance and implementation

15. Recompute expensive features on every edit? No: retain compact pairwise evidence with detector facts. Calibration/settings edits derive from cached features without rereading full PCM or repeating inference.
16. Cache PCM for ownership? No full-session duplicate. Compute features in bounded windows while source PCM is already loaded; discard it afterward.
17. Dependencies? Existing Rust/Iced/ORT/stdlib first. A small deterministic joint classifier before a new neural stack.
18. Parallel work? Guided project/editor calibration and joint detector/cache/render are independent behind the shared settings contract. Evaluation follows their merge.
19. Test seams approved? Project round-trip/source invalidation, editor selection-to-example behavior, cold/warm preview equivalence, joint ownership on synthetic bleed/overlap, actual source evaluation, and rendered gain/output integrity.
20. Prototype uncertainty? Detector task must run a bounded comparison experiment before selecting thresholds; the actual labeled corpus gates promotion, not intuition.

## Round 4: Acceptance

21. Screenshot labels? User identifies top/middle circles as bleed and bottom as direct speech. Prior full screenshot order is A03/A02/A01. Inner 828–831 seconds is a provisional interval inferred from the ruler, not newly verified sample-exact human annotation. Keep this provenance explicit; do not train and score on the same interval.
22. Other references? Existing 36 human labels on 00005603_A01/A02/A03.wav; retain quieter speakers, laughter and 2.737 seconds of known overlap.
23. Success? Material bleed reduction on single-speaker copied-signal cases and supplied examples, no additional meaningful wanted-speech loss relative to baseline, overlap preserved, source integrity maintained, warm p95 below100ms on three full stems if feasible. Report all measured tradeoffs.
24. Exact-unity gate? Existing gate fails on tiny one-pole recovery tails. Keep it unchanged and report exact counts plus gain-depth/duration and baseline deltas; do not present numerical tails as proven audible loss or waive new speech damage.
25. Human listening? Remains for the user. No automatic listening sign-off.
26. Deliverable? One integrated branch and identified EXE with brief calibration instructions. Do not launch or operate the user's app.

## Reference locations

Full sources are C:/Users/User/Downloads/chopshow podcast 155_A03.wav, chopshow podcast 155_A02.wav, chopshow podcast 155_A01.wav. The typed backslash before the underscore was formatting; these actual paths were verified previously. Each has239660221 samples at48kHz.

Original research exists in the saved project checkout: docs/automixing-research.md and docs/research/{competitive-landscape,cleanslate-rust-target-architecture}.md. Those documents propose cross-channel ownership; 135fbb1 still derives owners using per-track VAD and own-floor RMS. Prior quality evaluation is .scratch/responsive-preview/evidence/06-detector-quality.md. Screenshot: C:/Users/User/AppData/Local/Temp/codex-clipboard-279f2d7c-506d-4bfd-a185-f0fdc68c045c.png.
