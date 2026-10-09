# Expose ownership review and envelope editing in Iced

Status: done
Assignee: editor_workflow
Blocked by: 07, 10, 11
Spec: ../spec.md

## Scope

Extend the existing single Iced workspace rather than replacing it. Display source waveforms, automatic/effective gain, manual overrides, labels and owner estimates with clear visual distinction. Add a separate envelope-edit tool with draggable points/fades and Preserve/Attenuate actions, grouped undo, keyboard operations and clear selection feedback.

Expose required-reference readiness, explicit Analyze, worker progress/cancellation and stale-state indicators. Review navigation seeks/highlights the affected tracks and gives understandable reasons. Original/Processed/Removed comparison retains transport context. Probabilities are inspectable estimates, not promises of calibrated certainty. Keep the last completed preview visibly stale until refreshed; gain-only edits update immediately.

## Reach

src/app.rs CleanSlateApp selection/save_calibration/save_ownership/queue_preview/launch_preview, src/waveform.rs TimelineCanvas drawing/hit testing/cache, src/timeline.rs navigation, src/review.rs selection and existing house UI components.

## Acceptance

- Perform the full workflow on one window without a staged wizard: label, Analyze, seek uncertain region, correct label, batch Analyze, drag gain, undo, audition and save.
- Drag/zoom/scroll/hit testing and keyboard selection/editing stay correct at two-hour zoom extents and mixed native rates.
- Displayed labels, probabilities and envelopes cannot be mistaken for one another; protected overrides remain identifiable after reanalysis.
- Incomplete caches show loading; cancelled/obsolete jobs do not replace current UI state.
- Long-session drawing processes visible peaks/curves rather than every source sample; measure interaction latency without neural reinference.

## Completion evidence

The single Iced window now uses the complete accepted supervised pipeline rather than the retained detector. It includes explicit model acquisition/verification, Analyze/progress/cancel, structural and usable-reference checks, stale retained previews, review seek/highlighting, native Original bootstrap, comparison, labels/estimates/automatic/effective/manual display, point/fade drafts, region actions, explicit conflict replacement, grouped undo/redo, keyboard handling and project save. See [editor workflow](../../../docs/editor-workflow.md).

Six public editor tests pass, including the controlled complete pipeline workflow through review, two batched label corrections, fresh Analyze with fixed-evidence reuse, drag/undo, comparison, save/reload and cancellation. Installation rejects obsolete source/reference/alignment generations and changed runtime provenance. Failed atomic save retains the accepted cache. The subpixel display regression first lacked the required range API, then passed with a 1 ms manual excursion preserved in the viewport's shared gain bounds. Native verification-only first rejected the new command as usage, then reported the actual missing runtime; the installed pinned model and verified native package also passed offline verification without inference/download.

The two-hour test covers 44.1/48/96 kHz and 47,999 Hz sparse envelopes, full-pixel brief-activity retention, navigation and point-hit behavior. Production gain/ownership visible queries took 108 ms for 100 paired queries with navigation/hit checks. Production waveform peak selection took 1.44 ms for 300 queries with navigation across three native rates. No inference occurs in these drawing/editing paths. These are kernel measurements rather than physical window or audio-device latency claims.

The owner-authorized synchronized three-track project was created at ignored primary `test-audio/261005-supervised.cleanslate`, with neutral speaker IDs and no invented solo/room labels. Full all-target validation passed: 171 tests passed, 15 intentional opt-in tests ignored, zero failures. Strict all-target Clippy, formatting and whitespace checks passed. The final review highlight adjustment also passed strict lint; it uses the canonical grouped review selection. Owner truth/listening and release quality remain ticket 14; final native export/parity remains ticket 13.
