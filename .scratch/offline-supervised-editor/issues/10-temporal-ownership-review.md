# Optimize ownership regions with conservative uncertainty

Status: claimed
Assignee: temporal_ownership
Blocked by: 09
Spec: ../spec.md

## Scope

Turn independent whole-track owner probabilities into coherent multi-owner, no-speech and uncertain regions using offline context. Start with deterministic smoothing and duration/hysteresis rules only where development evidence supports them; avoid adding HMM/Kalman machinery without a demonstrated need. Honor explicit human active-speaker sets as truth constraints.

Preserve uncertain regions at unity, record actionable reasons and group them into review items. Brief interruptions, quiet speech and overlap must not be deleted to satisfy a smoothing rule. Freeze optimizer parameters and its uncertainty policy before independent acceptance testing. Keep optimization separate from gain settings and retain probability evidence for inspection.

## Reach

New focused ownership timeline stage, src/engine.rs regions/result contract, src/review.rs warning grouping, src/prepared_analysis.rs optimized-result cache, src/detector_quality.rs diagnostics and project review identities.

## Acceptance

- Fixtures cover short independent interjections, long gaps, overlapping speech, isolated noisy estimates and contradictory/ambiguous evidence.
- Human overlap constraints retain all marked owners; room-tone truth and manual overrides retain their separate roles.
- Uncertain regions stay preserved and are navigable review warnings, not render blockers unless a trust-boundary failure exists.
- Changing gain settings cannot alter ownership/probability results; label or optimizer changes invalidate the correct layers.
- Independent acceptance labels never participate in smoothing/tuning or forced truth during evaluation.
