# Make audition and final exports use the same current envelope

Status: ready-for-agent
Blocked by: 04, 11, 12
Spec: ../spec.md

## Scope

Route final effective envelopes through existing live audition, waveform gain bounds, removed-signal comparison and native-rate renderer. Version the entire render snapshot by sources, required model/recipe, labels, optimizer settings, gain/manual edits and completed result identity.

After label/source changes, last completed Processed playback may continue clearly marked stale; Original/editing remain available. Final render/export is blocked until analysis and effective envelope are current. A slider release, edit, relink or new Analyze generation cannot leave an older background completion installable as current. Keep output validation, nonempty destination rejection, cancellation and atomic completion.

## Reach

src/app.rs RenderSnapshot/PreviewProcessing/start_processing/results, src/audition.rs LiveAudition::replace_envelopes, src/waveform.rs bounds, src/engine.rs run_with_ownership/render wrappers, src/export.rs, src/project.rs render staleness and tests/{render,export,review,project}.rs.

## Acceptance

- The same source/envelope sample positions produce matching gain in UI bounds, audition, headless evaluation and each WAV export.
- Label, model, source, gain and manual-edit changes stale exactly the required snapshots; stale output cannot be exported as current.
- Seek/toggle/replacement preserves transport and does not introduce unintended discontinuities or monitoring changes into exports.
- Late completions after cancellation/new edits cannot replace current results or overwrite valid deliverables.
- Original files and prior valid outputs survive every blocked/failed/cancelled operation.
