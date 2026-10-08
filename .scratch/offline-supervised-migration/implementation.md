# Offline supervised editor implementation sequence

Status: accepted implementation plan, 2026-10-08

The architecture is [specified here](spec.md). The six wayfinder child tickets resolve architecture decisions; the fifteen tickets below are build work, not additional unresolved product decisions. Dependency fields, not this list alone, govern execution.

1. [Establish the accepted development baseline](../offline-supervised-editor/issues/01-establish-baseline.md)
2. [Introduce a source-aware timebase and versioned project contract](../offline-supervised-editor/issues/02-timebase-project-contract.md)
3. [Decode common mono formats and prepare aligned analysis copies](../offline-supervised-editor/issues/03-import-analysis-copies.md)
4. [Carry native rates through playback and WAV rendering](../offline-supervised-editor/issues/04-native-rate-playback-render.md)
5. [Install and verify the required Nemotron model and native runtime](../offline-supervised-editor/issues/05-model-runtime-acquisition.md)
6. [Retain complete Nemotron evidence for long recordings](../offline-supervised-editor/issues/06-nemotron-long-recordings.md)
7. [Edit reference labels and resolve diarization speaker identities](../offline-supervised-editor/issues/07-reference-labels-identity.md)
8. [Fit session acoustic evidence from trusted references](../offline-supervised-editor/issues/08-session-acoustic-evidence.md)
9. [Train independent supervised ownership models](../offline-supervised-editor/issues/09-session-ownership-model.md)
10. [Optimize ownership regions with conservative uncertainty](../offline-supervised-editor/issues/10-temporal-ownership-review.md)
11. [Compose editable exponential envelopes with protected overrides](../offline-supervised-editor/issues/11-effective-envelope-edits.md)
12. [Expose ownership review and envelope editing in Iced](../offline-supervised-editor/issues/12-editor-review-envelope-ui.md)
13. [Make audition and final exports use the same current envelope](../offline-supervised-editor/issues/13-current-preview-export-parity.md)
14. [Evaluate independent quality and long-session performance](../offline-supervised-editor/issues/14-independent-release-evaluation.md)
15. [Qualify the Windows offline release](../offline-supervised-editor/issues/15-windows-release-qualification.md)

## Existing work reconciliation

Ownership `6a60cc2` contains the responsive editor plus later calibration/cache/safety work. Prior ticket statuses are inconsistent with implemented code: six MVP implementation tickets are marked done, while responsive and guided-ownership tickets include ready/triage states despite existing implementation/evidence. Establish-baseline work reconciles their records and carries acceptance gaps forward; do not rebuild imports, transport, waveform caching, safe output handling or protection mechanics merely because an old issue is open.

The owner requested repository cleanup on 2026-10-08. The primary checkout now contains this developed code and the new roadmap on `codex/offline-supervised`. Superseded issue folders and branches have been removed; their history/records are in the external recovery snapshot. Continue only these fifteen build specifications. Baseline verification is still required before its ticket is marked done.

The historical ownership detector is a comparison baseline, not accepted truth. Preserve rejected experiments and reports as provenance, outside the shipping implementation. No build ticket is complete until its scoped checks pass.
