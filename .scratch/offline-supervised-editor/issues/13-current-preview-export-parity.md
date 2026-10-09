# Make audition and final exports use the same current envelope

Status: done
Assignee: export_parity
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

## Implementation and scoped evidence

Final rendering consumes the accepted temporal result and current sparse effective envelopes directly. It never runs the retained detector or reconstructs its fades. The versioned snapshot binds source identities/native grids, model/runtime/recipe, labels, ownership/temporal keys and effective gain/manual/protection settings. Monitoring and comparison mode stay outside delivery identity. See [editor workflow](../../../docs/editor-workflow.md).

Rendering and delivery use exclusively owned, unique private directories. Publication rechecks payload hashes/native metadata, source identities, persisted sound/delivery identity and the shared generation mutex before rename. Cancellation takes the same generation gate as sound edits. Prior nonempty outputs cannot be overwritten. Published attempts that cannot be accepted remain recovery material; discarding one on the GUI thread performs no payload scans or deletion. Strong verification and cancellable 64 KiB delivery copies run in workers; the GUI exposes cancellation and keeps its readiness checks inexpensive.

Public tests compare every exported frame against the shared evaluator at 44.1, 48, 96 and 47.999 kHz, conservative gain bounds and device-rate monitored playback. They reject obsolete generation, cancellation, changed runtime recipe, saved label removal, manual gain edits, changed source bytes, same-grid rendered WAV replacement, changed staged copies and legacy report-only delivery claims. They preserve source bytes, prior outputs and modified recovery material. Canonical supervised warnings and their analysis-specific review keys reopen from the new report; legacy reports remain readable. The legacy mid-render failure injector was updated to discover unique private attempts rather than assume the removed PID-only directory.

Final validation passed: 177 all-target tests passed, 15 intentional opt-in tests ignored, zero failures. Strict all-target Clippy, formatting and whitespace checks passed. Actual installed pinned model/native runtime verification passed offline without inference or download. The graph was refreshed after implementation (2090 nodes, 11806 edges). The branch includes the current integration tip. Owner-labeled independent speech/overlap quality and listening approval remain ticket 14; these controlled parity checks do not substitute for them.
