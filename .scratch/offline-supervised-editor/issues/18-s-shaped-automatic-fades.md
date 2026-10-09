# Use S-shaped automatic fades for transparent attenuation

Status: done
Assignee: codex
Labels: ready-for-agent
Blocked by: 11, 13
Spec: ../../offline-supervised-migration/spec.md

## Scope

Owner authorized on 2026-10-09 replacing exponential automatic fades with finite S-shaped fades, aiming for no audible clicks, pumping or obvious ducking. Use quintic smootherstep in the shared evaluator: exact endpoint gains and zero first/second derivatives at each endpoint. Preserve existing fade durations, short-gap fitting, speech unity, source validation, manual linear edits and protections. Bump the envelope recipe and update current documentation. No new controls, dependencies or detection changes.

## Reach

src/effective_envelope.rs FadeSegment::gain and EFFECTIVE_ENVELOPE_RECIPE; src/project.rs sound_identity; tests/effective_envelopes.rs and tests/supervised_delivery.rs; shared drawing, audition, native rendering and headless evaluation consumers; accepted specification, ADR and envelope/workflow documentation.

## Acceptance

- Both fade directions are monotonic, bounded, S-shaped and exact at endpoints, including zero; flat segments remain exact.
- Endpoint slope and curvature approach zero; nearby samples do not exhibit an abrupt gain step.
- Existing short-gap, mixed-native-rate speech preservation, manual edit persistence, bounds/sequential access and preview/export parity checks pass.
- A render snapshot created under the previous sound identity is rejected without publishing output; existing exponential renders require a fresh render.
- GUI development build and repository formatting checks pass.
- Record numerical evidence and leave real-recording listening acceptance with ticket 14. A smooth gain curve cannot guarantee inaudibility or eliminate the audible effect of removing background/bleed.

## Comments

2026-10-09 owner documentation follow-up: claim the unblocked ticket again to update the README, ownership explanation and domain glossary wherever current gain behavior is described. Acceptance: descriptions agree with the S-shaped automatic evaluator, distinguish linear manual edits, preserve listening limitations and link to the envelope contract; validate changed Markdown links and diff whitespace. No runtime change or new build is needed.

Dependencies 11 and 13 are done. Existing unrelated GUI edits are present in the shared checkout and are outside this ticket.

## Verification

Implemented quintic smootherstep in the shared automatic evaluator and bumped the effective-envelope recipe to v4. Clamped normalized gain progress to [0, 1] against floating-point roundoff. The envelope recipe now participates in project sound identity, so prior exponential renders become stale without deleting their payloads or invalidating source/manual data. Current spec, ADR, map and workflow/envelope documentation record the owner refinement; historical ticket 11 describes the original implementation.

The new unit test checks both directions, zero and nonzero attenuation, exact endpoints/midpoint/quarter shape, 10,001 monotonic bounded samples per direction, finite-difference endpoint slope/curvature and constant gain. Existing short-gap assertions were updated from the old exponential's <0.3 interior gain to <0.55 for the gentler S-curve; exact unity/attenuation endpoints, minimum-gap behavior and authoritative manual edits remain checked. A legacy sound-identity snapshot is rejected before rendering/publishing output.

`cargo test --locked --offline -j 1 --target-dir C:/Users/User/.codex/worktrees/offline-timebase/podcast-automix-rust-gui/target/owner-ui --config scripts/gui-dev.toml --lib --test effective_envelopes --test supervised_delivery --test native_delivery --test native_playback --test export --test project` passed: 104 passed, 6 opt-in ignored. Coverage includes per-frame native export/evaluator parity at 44.1/48/96/47.999 kHz, speech unity, sequential/bounds access, manual persistence, cancellation, stale renders and source integrity. Formatting and diff whitespace checks passed. Source is stable; the owner's authorized parallel GUI chat is running the combined workflow/timeline checks and final gui-dev build/launch without a duplicate window.

Real-recording listening approval remains ticket 14. The numerical checks do not establish inaudible background changes, pumping or distortion; existing manual linear edits may still contain deliberate discontinuities.

### Completion evidence

Documentation follow-up completed: README now explains the automatic S-curve, maximum durations, preserved speech, linear manual points, listening requirement and regeneration of older renders. CONTEXT glossary replaces exponential fade terminology with S-shaped fade and correctly describes fade durations as maxima. The ownership explanation describes gain multiplication, gentle endpoints and composition; the envelope contract includes the exact quintic formula, example progress, sample-wise application and Removed/monitoring semantics. Existing current workflow/spec/ADR descriptions were checked for consistency; historical decisions remain labeled as superseded. All 31 local Markdown links in the four updated docs resolve, and diff whitespace checks passed. Documentation-only follow-up; runtime tests were not repeated.

The authorized shared GUI chat completed its combined library/timeline/native-playback/editor-workflow/effective-envelope check on the final source: 99 passed, 8 documented opt-ins ignored. Final strict all-target/all-feature Clippy with `-D warnings`, formatting and diff checks passed after the sound-identity changes. `scripts/gui-dev.ps1` built the candidate in 1.28 seconds and opened the saved development copy; the process responded with the expected development title. No second GUI was launched by this ticket. All scoped numerical/build acceptance checks are complete. Owner listening and release approval remain ticket 14; GUI/Solo acceptance remains separate ticket 16 work.
