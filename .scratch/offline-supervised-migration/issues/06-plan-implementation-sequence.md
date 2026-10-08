# Turn the settled architecture into migration tickets

Type: grilling
Labels: wayfinder:grilling
Status: resolved
Assignee: Codex with repository owner
Parent: ../map.md
Blocked by: 02, 03, 04, 05

## Question

What ordered implementation tickets migrate the chosen baseline to the settled architecture while keeping an operable editor at each stage?

Reconcile existing effort tickets. Each new implementation ticket needs exact scope, affected callers and persisted formats, dependencies, acceptance checks and relevant architecture decisions. Cover the shared analysis API, optional headless entry points where justified, correction UI, project migration, preview/audition/render consistency and validated individual WAV export. Add documentation research and prototypes as prerequisite decision tickets when unresolved facts appear; do not invent algorithm choices to make the sequence look complete.

## Answer

2026-10-08: The owner confirmed the accepted architecture and fifteen individually scoped implementation tickets. Their [sequence and links](../implementation.md) cover baseline preservation/reconciliation; common timebase/versioned project; broader import and 16 kHz analysis copies; native-rate playback/render; pinned model acquisition/runtime; continuous long-recording Nemotron evidence; reference/identity mapping; calibrated acoustic evidence; independent session models; temporal ownership/review; protected effective envelopes; Iced editing; preview/export parity; independent evaluation; and Windows qualification.

Each build ticket names affected callers/configuration/fixtures, dependencies and meaningful checks. Existing command-line diagnostics/benchmarks must lose fixed-rate assumptions too. Render entry points are in engine, not a nonexistent standalone render.rs. Gain evaluator changes reach direct/transitive consumers, sequential and bounds evaluators, padding and sample-edge parity tests.

All build specifications are `ready-for-agent`, with dependency fields controlling execution order. The only initial unblocked build ticket is [Establish the accepted development baseline](../../offline-supervised-editor/issues/01-establish-baseline.md). No application code was changed or implementation marked complete. Native runtime speed, numerical detector qualification and listening acceptance are implementation/release gates with specified evidence, not silently resolved facts.
