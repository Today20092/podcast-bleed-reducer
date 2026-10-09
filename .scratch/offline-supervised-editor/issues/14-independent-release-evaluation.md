# Evaluate independent quality and long-session performance

Status: claimed
Assignee: release_evaluation
Blocked by: 06, 08, 09, 10, 13
Spec: ../spec.md

## Scope

Extend the existing detector-quality/release-gate machinery and headless binaries using the same library pipeline; do not add a duplicate CLI when current entry points suffice. Freeze development references/tuning separately from independent test sessions/regions. Keep historical used holdouts out of unseen claims. Include the recorded screenshot failure, quiet speech, interruptions, laughter, overlap, room tone, movement and changing delays. Store source fingerprints/splits/recipes so comparison is reproducible; keep audio local and do not publish it.

Compare the frozen historical detector and new engine on identical acceptance material. Any automatic attenuation of labeled wanted/overlap samples is a release failure; intentional manual edits are evaluated separately. Require measurable bleed improvement without preservation regression. Report ownership, uncertainty and applied-gain metrics separately. Do not tune on final acceptance results and then report them as unseen.

Exercise eight tracks/two hours on the owner's Ryzen 9 3950X / 32 GB Windows system. Measure cold load/analysis, peak memory, disk cache growth, cancellation, warm gain-edit latency and retained long-recording model frames. A hard cold-time target is not agreed; obtain the owner's acceptability decision from actual measurements. Fail resource exhaustion or unusable transport rather than hiding them behind averages.

## Reach

src/detector_quality.rs read_labels/compare_ownership/ownership_metrics, src/release_gate.rs, existing src/bin/{ownership_diagnose,ownership_evaluate,preview_benchmark,release_gate}.rs fixed-rate assumptions and library calls, meaningful quality fixtures, tests/release_gate.rs and local evidence reports.

## Acceptance

- Frozen independent labels are excluded from references, forced optimizer truth and training/tuning.
- Wanted/overlap unity checks and bleed comparisons use effective applied gain on source samples, not raw classifier success alone.
- Old failing cases remain visible; unseen/generalization claims identify what was actually held out.
- Two-hour/eight-track CPU evidence includes early model probabilities after native compaction and final-tail completeness.
- Measured performance and listening-ready Original/Processed/Removed regions are recorded for owner sign-off. A failed result blocks release.
