# Edit reference labels and resolve diarization speaker identities

Status: done
Assignee: reference_labels
Blocked by: 02, 06
Spec: ../spec.md

## Scope

Extend the existing Iced calibration selection to solo, room-tone and arbitrary active-speaker-set labels. Require usable solo references for each assigned speaker and room tone before supervised analysis. Multiple references may span a recording; overlap examples are optional. Validate contradictions and require explicit replacement of conflicting human truth.

Resolve first-arrival Nemotron channels to stable assigned speakers from trusted solo references. Ambiguous, missing, conflicting or unstable identity mappings remain unresolved and preserve affected audio for review. Label changes mark analysis stale and wait for explicit Analyze; do not rerun fixed inference unnecessarily. Include undoable label operations and persistence.

## Reach

src/calibration.rs kinds/examples/validation, src/app.rs save_calibration/save_ownership/selection/Analyze, src/project.rs labels/source bindings, new diarization-to-speaker mapping and prepared cache identity.

## Acceptance

- Solo, room tone, overlap, contradictions, invalid membership, unusable examples and missing required references behave explicitly.
- Swapped arrival order, duplicate identity, long-gap recurrence and ambiguous solo evidence cannot guess a microphone owner.
- Label undo/save/reload and source invalidation retain the right truth and mark exactly the dependent results stale.
- Repeated label edits batch until Analyze, preserve source/fixed-evidence caches and cancel obsolete derived work safely.

## Implementation evidence

Authoritative source-bound labels use stable IDs and transactional save/replace/remove/undo/redo. Conflicting intervals require explicit whole-label replacement. Label undo restores truth without making an invalidated analysis current. Current-source checks refuse historical truth on changed audio until explicit review/rebinding.

The public reference readiness boundary requires usable solo signal for every assigned speaker and room tone across all tracks; optional overlap labels retain arbitrary active-speaker sets. Readiness signal checks are AC-energy checks, not independent speech-quality validation. The identity resolver reads verified complete ten-millisecond Nemotron history in bounded blocks and maps trusted solos conservatively. Its versioned derived key includes current label/source assignments, fixed evidence, model/runtime and downmix provenance. Ambiguous, duplicate and disagreeing recurring channels stay unresolved.

Iced reuses waveform/time selection with active-speaker checkboxes, room tone when none are active, explicit conflicting-label replacement, stable-ID removal, undo/redo and explicit batched preview Analyze. Label edits invalidate generations without discarding reusable fixed facts; old jobs cannot install derived results. Retained-detector preview is provisional: it never declares the new supervised recipe current or enables final rendering after label changes. Composition of the new supervised jobs and commit gate belongs to ticket 12; shared effective-envelope export belongs to ticket 13.

Controlled test evidence does not label the owner's recording or establish model listening quality. Ticket 14 retains independent speech/overlap quality and listening acceptance.

Checks passed on the integration base `b39c4d2`:

- Public reference tests: 5 passed, covering explicit replacement, save/reload/undo/redo, missing/unusable references, swapped arrival/duplicate identity, long-gap disagreement, ambiguity, cancellation, fixed hashes and derived cache identity, membership and changed-source dormancy.
- Full all-target suite: 137 passed, 0 failed, 14 ignored. Ignored tests require native runtime/model, private audio, subprocess fixtures, visual or benchmark evidence.
- Final Iced checks after the authoritative reload/source-install gate: 22 passed, 0 failed, 1 ignored. The first explicit Analyze can start without a prior preview; pending/obsolete/provisional preview never enables supervised export. Source installation and freshly reopened project labels cannot reset this gate.
- Final public reference confirmation after the lint-only bounded-chunk iteration change: 5 passed, 0 failed.
- `cargo clippy --all-targets --locked -j 1 --config 'profile.test.package.rust-app.debug=0' -- -D warnings`, `cargo fmt -- --check` and `git diff --check` passed.
- Integration tip merged before completion (already up to date).

No source recording was relabeled, no native model accuracy was inferred from controlled probability fixtures, and no release/listening approval is claimed.
