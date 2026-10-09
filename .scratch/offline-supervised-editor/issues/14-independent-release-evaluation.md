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

## Independent tooling checkpoint (2026-10-08)

The strict frozen-manifest seam now verifies actual source fingerprints, native grids, supplied alignment and reference identity. It rejects held-out ranges intersecting development references, contradictory labels including native-sample rounding collisions, unknown fields and false independent claims for declared previously used material. Duplicate physical truth is unioned before scoring.

Streaming applied-gain scoring decodes bounded chunks and counts wanted attenuation even where source PCM is zero. Reports retain each track's native sample denominator and measure simultaneous preservation on the common time clock. These are mechanical evaluation tools; automatic/manual/ownership separation, accepted-project CLI composition, frozen comparison integration and full performance evidence remain pending.

Public release-gate tests: 7 passed, 0 failed (0.22 s). Strict library/tests Clippy passed. Exact frozen commit `6a60cc2bfe187ea889fd2d8bcc3344311cfd706b` built offline with its unchanged lockfile in a separate managed checkout. Four original binaries and DirectML DLL were copied and hashed into external `ticket14-frozen-artifacts`, with a build receipt. The copied ownership evaluator executed all four original variants on two controlled two-second 48 kHz synthetic fixtures (exit 0, 0.62 s), exercising embedded Silero inference. This establishes executable provenance and runtime mechanics, not independent quality or baseline selection.

Ticket remains claimed. Real independent truth, listening approval and measured timing approval are not available; no release acceptance is asserted and ticket 15 remains blocked.
