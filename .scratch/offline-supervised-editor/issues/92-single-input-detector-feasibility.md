# Choose and validate the causal single-input speech detector

Status: claimed
Assignee: Codex (Gate detector feasibility)
Readiness: ready-for-agent
Type: task
Blocked by: 91
Related: 85, 86

## Scope and acceptance

In the Gate workspace, evaluate the maintained speech detector and feature path through the graph before changing it. Determine causal inputs, lookahead/window/hop, preparation cost, sample-rate conversion and model-state/reset behavior. Distinguish speech presence from speaker identity and multi-mic ownership. This is a bounded decision/prototype, not a claim that offline ownership already solves live gating.

Acceptance:
- [ ] Deliver a decision record with exact reused model/code identity, current documentation, one chosen causal inference design and explicit measured algorithmic/host latency.
- [ ] Provide a small offline harness for speech probability and resets at 44.1/48/96 kHz and varied block sizes, without a DAW.
- [ ] Test silence, noise, quiet/normal speech, interjections, laughter and overlap using authorized fixtures; label available objective evidence and owner-listening gaps.
- [ ] Measure per-block/tail latency, resident/private memory and preparation/inference work under 88. If the existing detector is unsuitable, document the concrete blocker and a bounded alternative requiring no vendor weights; do not enable a misleading threshold-only substitute.
- [ ] Define the immutable detector-to-gain event/result contract, readiness and conservative fallback needed by 93. Stop that dependency if no usable causal detector is demonstrated.

## Results

Claimed after 91 qualification on `dda8505`; canonical claim commit `7721337`.
See [bounded detector decision/checkpoint](../../../docs/research/smart-gate-detector-92.md).
The separate `plugins/gate-detector` prototype reuses the exact embedded Silero
v5 bytes and pinned ort runtime, with causal rational-rate FIR, recurrent reset,
immutable copied events and an offline source-bound measurement harness.

Public streaming/reset tests were written first; initial compilation failed at
the missing Detector API. The guarded job later stopped an owned lingering MSVC
helper on low commit reserve, with verified cleanup. After implementation, the
standard 14/8/6 test preflight refused below 14 GiB; no green test or measurement
execution is claimed. Noncompiling formatting and lock generation passed under
explicit small bounds. No Gate audio, probe, installed links or candidate output
changed. Source remains a reviewable checkpoint; 92 is claimed and 93 blocked
until actual all-target tests, causal measurements and a usable-detector decision.
Human category/listening gaps are explicitly recorded; artificial scaled/cropped/
mixed development speech does not replace missing real-category labels.

