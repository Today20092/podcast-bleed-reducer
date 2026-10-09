# Compose editable exponential envelopes with protected overrides

Status: done
Assignee: effective_envelopes
Blocked by: 02, 10
Spec: ../spec.md

## Scope

Generate attenuation-only automatic envelopes from optimized ownership. Confirmed fade contract: exponential-shaped finite fades with explicit endpoints; preserve wanted/overlap/uncertain intervals at exact unity absent deliberate manual edits. Fit transitions in surrounding time and keep short gaps preserved rather than reducing wanted samples to make fades fit. Defer a shape selector.

Add sparse source-bound manual points/segments and Preserve/Attenuate actions. Manual edits remain authoritative across reanalysis until removed. Represent them separately from automatic envelopes; whole-track/range unity protections are manual preserve constraints. Conflicting manual constraints require explicit replacement. Validate ordered times, finite bounded gains and endpoint semantics. Each drag/action is one undoable command.

Provide one effective-envelope evaluator in source-aware time for all consumers. Persist edits and reconstruct their effect identically after save/reload. Removing an override reveals current automatic gain; changing a label never silently erases a manual edit.

## Reach

src/engine.rs GainEnvelope/GainPoint/Protection/gain_envelope and GainEnvelope::{gain_at,gain_at_sequential,gain_bounds}, src/project.rs manual edit/undo persistence, src/prepared_analysis.rs effective gain cache, engine render wrappers, detector_quality, waveform/audition consumers, embedded gain edge/parity tests and joint_detector padding tests. Trace all current direct/transitive callers before editing rather than relying on a frozen caller count.

## Acceptance

- Unity is exact throughout preserved spans, including first/last samples and rounded mixed-rate boundaries.
- Curves have correct endpoints, duration, continuity and retarget/short-gap behavior; manual edits have the same numeric interpretation when displayed and evaluated.
- Overlapping manual edits/protections cannot create hidden precedence; replacement and undo restore exact previous data.
- Reanalysis, parameter changes, save/reload and cache loss preserve manual edits and unity protections.
- Gain-only updates reuse expensive evidence and can be compared against a direct sample scan at every transition/protection edge.

## Completion evidence

Implemented the sparse finite exponential automatic layer, source-bound linear manual edits and unity protections, one shared effective evaluator, separate automatic/effective caches, transient drag previews, and atomic grouped undo/redo. See [the envelope contract](../../../docs/effective-envelopes.md) for half-open endpoints, source binding and workflow boundaries.

Eight public integration tests passed: exact endpoints and short-gap preservation; outward boundaries on 44.1/48/96 kHz and 47,999 Hz grids; explicit conflicting-constraint replacement and grouped undo; reload/reanalysis/manual removal and direct-scan bounds parity; protection membership after adding a source; failed atomic saves and obsolete drafts; touching edits/protections and whole-track unity; cancellation and changed-source dormancy until review plus fresh analysis. The global-protection regression first failed with an unintended unity gain on the newly added track, then passed after projection and conflict validation both honored binding membership.

`cargo test --all-targets --locked -j 1 --config 'profile.test.package.rust-app.debug=0'` completed successfully with no failed tests. Strict all-target Clippy, formatting and diff whitespace checks passed. The existing ignored tests cover opt-in native inference, local recording evidence, visual checks and benchmarks; this run does not claim those gates. The eight new tests were also run directly and passed.

Manual edits preserve the probability/temporal evidence and reuse automatic curves. Changed-source identities reject old curves; label-only changes expose an explicit stale-analysis flag. Full supervised UI composition and final export gating remain tickets 12 and 13. Real recording quality, long-session performance and owner listening approval remain ticket 14.
