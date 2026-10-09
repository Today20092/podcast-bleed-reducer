# Compose editable exponential envelopes with protected overrides

Status: claimed
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
