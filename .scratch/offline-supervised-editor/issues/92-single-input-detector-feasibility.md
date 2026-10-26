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

Not started. Readiness describes specification only; all listed dependencies must be done before claiming.

