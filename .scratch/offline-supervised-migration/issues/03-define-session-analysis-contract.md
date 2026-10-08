# Define labels and the offline session-analysis contract

Type: grilling
Labels: wayfinder:grilling
Status: resolved
Assignee: Codex with repository owner
Parent: ../map.md
Blocked by: 01

## Question

What data and boundaries connect reference labels, calibration, whole-session evidence, local-speaker probabilities and temporal optimization?

Define speaker-to-track assignment, room-tone and solo/overlap labels, missing references, conflicting labels, microphone movement and invalidation after source changes. Distinguish speaker activity from local ownership and allow simultaneous owners. Decide the role of optional diarization, fixed feature extractors and a trainable session classifier. Keep source audio separate from analysis copies and gain application. Reconcile global-first assumptions in existing domain records without treating earlier chat proposals as proven algorithms.

## Comments

2026-10-08 human decisions: one dedicated mono track per speaker; require solo references for every assigned speaker plus room tone; optional overlap labels; one calibration setup per recording with examples from multiple times and poor-fit review flags for small microphone movement. Preserve uncertain speech and overlap at unity. Nemotron is required in the first release, not optional. Support up to eight speakers and two-hour recordings. Windows ships first with portable architecture for later macOS/Linux. Download the pinned model once on first use, then run locally offline. Explicit Analyze follows label changes; gain adjustments update preview immediately.

Fact finding: existing `JointFacts::regions` yields categorical results from VAD and scalar acoustic evidence, not learned owner probabilities. Native Nemotron frame probabilities must be copied incrementally before long-session compaction; use one continuous runtime stream rather than resetting speaker identities per chunk. Full-attention one-shot inference is unsuitable for two-hour input. See [feasibility notes](../feasibility-notes.md).

## Answer

2026-10-08: The owner confirmed the full architecture. Accept common supported mono codecs and mixed native rates, retain synchronized source timing, resample analysis copies to 16 kHz and render native-rate WAVs. Use one dedicated track per speaker, up to eight speakers, with two-hour acceptance tests. One setup permits references from multiple points; unsupported/missing/contradictory evidence is preserved for review rather than forced into an owner.

Required reference truth consists of source-bound solo examples for every speaker and room tone, with optional arbitrary overlap sets. Validate bounds, identity, usable examples and conflicts. Labels require explicit Analyze; fixed evidence is cached and reused. Source/model recipe changes invalidate their descendants. No self-training or neural fine-tuning is included.

Nemotron is mandatory, downloaded once on first use, then local CPU/offline. Its first-arrival identities are mapped through human references with ambiguity checks. Use one continuous native stream and retain probability frames incrementally before compaction. Combine that evidence with existing per-track VAD/noise/directed bleed measurements in independent binary Rust logistic session models. Acoustic estimator qualification and numerical tuning use development data, then freeze before independent acceptance. Separate whole-track optimization from gain generation; uncertainty and overlap are preserved.

The exact pipeline, input profiles, time conversions, cache boundaries and failure rules are recorded in [the accepted specification](../spec.md). Current capability evidence and unmeasured limits are in [feasibility notes](../feasibility-notes.md). This supersedes categorical-only ownership/global-first inference assumptions, not non-destructive source/output safety. Runtime/quality success remains required implementation acceptance.
