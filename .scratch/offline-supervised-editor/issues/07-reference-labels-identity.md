# Edit reference labels and resolve diarization speaker identities

Status: claimed
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
