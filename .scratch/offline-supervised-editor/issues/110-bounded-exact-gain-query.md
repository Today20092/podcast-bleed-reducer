# Bound exact gain-envelope queries before evaluating dense edges

Status: claimed
Assignee: Codex bounded gain-query implementer
Readiness: ready-for-agent
Type: task
Blocked by: 109
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

### Functional RED and unverified GREEN checkpoint

On clean source `a27daac`, immediate native preflight measured 21.03 GiB available physical memory and 20.96 GiB commit headroom. The selected root-library unit target used pinned Rust 1.99, existing locked/offline single-job shared arguments, `--no-default-features --profile test` and final-target-only pinned LLD. After prerequisite compilation (7 minutes 48 seconds), the budget test failed semantically: expected `TooDense`, actual `Bounds(0.25, 0.75)`; one failed, 106 filtered out. Receipt `20261030-195338-aa9aeffff95149bfa6fb863f5634232f` finalized with exit 101 and no remaining recorded job members.

This proves functional RED, **not contained compiler execution**: the receipt recorded only one active process and 36,892,672 bytes peak job commit, while a read-only observation saw rustc at about 1,849 MiB private memory. Ticket 109's subsequent tiny actual-membership probes reproduce children outside the named job even with actual Cargo and matching no-breakaway job flags. No heavy GREEN run is authorized until that prerequisite is resolved. Global minimum physical/commit availability stayed about 16.8/16.3 GiB, which does not establish the missing owned-process cap.

The minimal GREEN source is preserved but unrun: clip and reject empty ranges, count legacy retargets or effective sorted edges with partition points, return `TooDense` before evaluation when over budget, otherwise delegate unchanged exact extrema. Added effective tests cover offscreen edges consuming no budget and an exclusive endpoint consuming candidate budget without contributing its gain. No existing caller, old API, dependency, UI, format or audio behavior was changed. Formatting and GREEN checks remain unrun. The exact RED executable, receipt and log are preserved outside the worktree under `%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/ticket110/20261030-red-a27daac/`; it is a failing test executable, not an application build.

RED executable SHA256 is `7dfa2fcccd9d699f012640d3ef135b19d6aad7ec422882df142f33068925ef4c`, matching the copied artifact. Prepared GREEN commit `5f6cc42` is deliberately unverified and must not be integrated as completed behavior. Keep this incomplete worktree and branch until 109 proves actual compiler-chain containment and the scoped GREEN checks can run. Status remains claimed; no resource threshold, generic guard semantics or release claim was changed.
