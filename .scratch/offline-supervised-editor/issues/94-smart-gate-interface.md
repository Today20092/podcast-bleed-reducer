# Implement the focused live-gate interface

Status: open
Readiness: ready-for-agent
Type: task
Blocked by: 90, 93
Related: 85, 86

## Scope and acceptance

Apply docs/design/plugin-ui-contract.md to Smart Gate. Reuse the established editor stack and proven drawing/cache patterns. Display input/output, speech confidence, gate state, applied reduction, readiness and the selected detector/gain controls. Keep detector learning/calibration only if 92 demonstrates it is necessary.

Acceptance:
- [ ] Original CleanSlate styling with clear hierarchy, responsive controls, predictable keyboard/focus and discoverable help; no Podcast session setup or hidden dependency.
- [ ] Bounded refresh/geometry, idle throttling, transient UI smoothing independent of DSP, reduced-motion behavior and no callback UI contention.
- [ ] Deterministic compact/full/DPI/error renders and targeted interaction regressions; compare measured frame work/memory with the pre-change baseline.
- [ ] Native physical interaction/feel and listening are explicitly pending if not observed. Build with 88 and preserve old binaries; do not start a heavy DAW unattended.

## Results

2026-10-27 owner quality amendment: apply FabFilter/sonible-level visual craft and fluid motion using original CleanSlate styling, the [102 visual study](../../../docs/design/plugin-ui/visual-study.md) and the contract's explicit native measurement targets. Large confidence/input/output displays, clear gate/reduction state and precise numeric controls are the focal surface. Active 60 Hz presentation and independently bounded meter refresh are targets to verify, not passed performance. Preserve the default-bypass/explicit-arm engineering boundary and stale-evidence original fallback from 92/93. Existing dependency on 93 remains; the design study does not implement or qualify gating.

Not started. Readiness describes specification only; all listed dependencies must be done before claiming.

