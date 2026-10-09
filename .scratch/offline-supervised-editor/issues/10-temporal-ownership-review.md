# Optimize ownership regions with conservative uncertainty

Status: done
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

## Implementation evidence — 2026-10-08

- Added `TemporalOwnershipCache`, immutable whole-track probability/speech/activity evidence, independent `Owned`/`Absent`/`UncertainUnity` states and exact native-time regions. A one-frame direct overlap keeps both owners. No winner selection, minimum speech duration, or duration-based deletion is present. Bidirectional bounded interior-gap context can only preserve additional uncertainty.
- Every ticket 09 preservation reason, missing estimate and speech/ownership contradiction becomes unity/review evidence. Empty owner sets require quiet raw and global speech evidence before becoming no speech. Original probabilities remain inspectable and are not replaced by categorical decisions.
- Every current human active-speaker set applies at exact half-open project ticks after statistical context, including unsampled frames. Consistent overlaps unite; contradictory truth blocks installation. Room-tone truth and manual edits remain separate. Source fingerprints, synchronization declarations and label bindings are checked before reuse; cancellation, source mutation or stale supervised inputs retain the last completed result.
- Cache identities include immutable numeric evidence, upstream supervised recipe/provenance, current source/reference identity, optimizer settings and the versioned temporal recipe. Gain and protected manual settings are excluded. Warning clipping uses ordered interval overlap lookups and a label-union sweep with cancellation checks, avoiding quadratic work on a long alternating timeline.
- `ReviewWorkspace::from_ownership` groups reasons and affected tracks into navigable warning spans. Warning keys include analysis identity, physical interval, affected tracks and reason. The retained renderer adapter preserves the whole region whenever any track remains uncertain; the detailed result retains per-track states for ticket 11.
- Public red→green: absent temporal API; one-frame independent overlap, contradictory low/high estimates and isolated noisy ownership; short preserved gaps versus long silence; cancelled/changed-setting reuse; 4,096 alternating intervals, exact off-grid overlap and room boundaries, gain reuse, label invalidation, source mutation and withdrawn alignment. The supervised integration fixture checks retained numeric probability evidence and preservation of quiet overlap through the real adapter. The project label API's existing 0.25-second minimum remains intact; off-grid boundaries are exact rather than shorter references being invented.
- Checks: broad all-target run **159 passed, 14 ignored, zero failed**. Final scoped run after the provenance/cancellation strengthening: **3 temporal + 5 supervised integration tests passed**. Final strict all-target Clippy, formatting and whitespace checks passed; integration tip was already included. Independent release/native benchmarks remain intentionally separate.

## Handoff and release boundaries

Development settings and the conservative policy are frozen in `docs/temporal-ownership.md`; synthetic qualification is not independent real-recording accuracy. Acceptance truth is never read by this stage unless deliberately supplied as human project references, which is excluded during unassisted evaluation. Ticket 11 consumes per-track states when composing gain envelopes. Ticket 12 owns full GUI job composition and current-generation installation; this implementation does not clear the provisional export gate or promote the retained legacy Analyze job. Ticket 14 owns independent speech/overlap preservation, bleed improvement, actual long-session measurements and owner listening approval.
