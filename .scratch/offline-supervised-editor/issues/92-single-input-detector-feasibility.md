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

### Measured follow-up, pending review

Actual all-target tests (public causal/reset/fault contract at 44.1/48/96 kHz),
strict Clippy, example compilation and two complete 165-run measurement passes
now succeed. See the linked report's measured decision and final verification;
earlier blocked/unrun statements below are historical. Compilation kept standard
14/8/6 GiB limits and Cargo -j1; the inference harness used explicit 4/2/1 GiB
bounds and a 300 s deadline. Peak runtime private memory was 36.42 MiB (first run),
kernel job commit 37.85 MiB; cleanup was verified. The largest inference-bearing
push across both runs was 0.663352 ms, not a guaranteed callback deadline.

All probability arrays matched exactly across block partitions and the second
run. Model/source/project hashes are recorded; modified reference projects are
rejected before audio/model work, with a public CLI red/green rejection check.
The tested final executable is hash-preserved outside the shared cache.

Bounded decision: existing Silero/causal adapter is usable for a conservative 93
worker/gain prototype with original fallback and explicit readiness. It remains
allocating synchronous inference and is not wired to the audio callback. The
source-bound room annotation disagrees with high model probability (a false-open/
noise-leakage risk); it is not an independently audited no-speech dataset. Natural
quiet starts/tails, interjections, laughter, overlap and listening are unqualified;
the report does not invent those labels or tune a production threshold to hide
the disagreement. Future gate/PDC quality is separate from current pass-through
host latency. Keep 92 claimed until standards/spec review and integration; no
product callback, installed link, probe or existing candidate was changed.

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

