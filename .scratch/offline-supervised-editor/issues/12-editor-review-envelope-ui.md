# Expose ownership review and envelope editing in Iced

Status: ready-for-agent
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
