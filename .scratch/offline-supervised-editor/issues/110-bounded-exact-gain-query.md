# Bound exact gain-envelope queries before evaluating dense edges

Status: claimed
Assignee: Codex bounded gain-query implementer
Readiness: ready-for-agent
Type: task
Blocked by: none
Related: 103

## Scope and acceptance

Provide the shared query prerequisite for ticket 103's accepted bounded Analysis gain display. The [accepted bounded-gain seam](../../../docs/research/podcast-ui-polish-2026-10-27.md#visual-red-launch-resource-stop) requires refusing dense detail rather than shifting boundaries or allocating unbounded geometry. This ticket adds no GUI or displayed processing claim.

- [ ] Add an explicit exact bounds / empty source range / too-dense result at the existing public `GainEnvelope` query seam.
- [ ] Count indexed effective-envelope edges and legacy retarget points before evaluating extrema; refuse work over the caller's edge budget.
- [ ] Preserve existing APIs, exact extrema, half-open boundaries, source clipping, manual overrides and protections.
- [ ] Observe semantic RED before production implementation, then relevant tests, formatting and independent review.

## Results

Claimed 2026-10-30 from integration `56d5d82` in an isolated managed worktree. Code graph transport is unavailable; direct source and caller searches cover the affected query paths. Public API/source proposal sent to the coordinator before test or production edits. No compiler, tests or production implementation started; Gate owns the sole execution slot, followed by Podcast's prepared gesture regression.

Coordinator approved the additive `BoundedGainBounds::{Bounds(f32, f32), Empty, TooDense}` result and `GainEnvelope::gain_bounds_bounded(start, end, max_edges)` at the existing public query seam. Existing methods and callers stay unchanged. The budget counts exactly the indexed candidate slice used by the evaluator: effective edges at or after the start tick through the ceil-converted end tick inclusive; legacy retargets strictly after the start frame and before the clipped end frame. Counting uses two partition points before evaluation or allocation.

Prepared, unrun RED assertions cover legacy budget refusal and exact threshold, half-open retarget ownership, source clipping/empty/reversed ranges, and effective manual/protection extrema against a sample-by-sample oracle. The coordinator explicitly approved a temporary boundary scaffold to obtain behavioral RED rather than an absent-method compiler error: it delegates to the existing exact evaluator and deliberately ignores the budget. This scaffold is not an implemented or shippable bounded API. No candidate counting is implemented before the semantic `TooDense` failure runs. The compiler slot remains reserved for other work.
