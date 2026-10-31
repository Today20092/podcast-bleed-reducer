# Bound exact gain-envelope queries before evaluating dense edges

Status: done
Assignee: Codex bounded gain-query implementer
Readiness: ready-for-agent
Type: task
Blocked by: none
Related: 103

## Scope and acceptance

Provide the shared query prerequisite for ticket 103's accepted bounded Analysis gain display. The [accepted bounded-gain seam](../../../docs/research/podcast-ui-polish-2026-10-27.md#visual-red-launch-resource-stop) requires refusing dense detail rather than shifting boundaries or allocating unbounded geometry. This ticket adds no GUI or displayed processing claim.

- [x] Add an explicit exact bounds / empty source range / too-dense result at the existing public `GainEnvelope` query seam.
- [x] Count indexed effective-envelope edges and legacy retarget points before evaluating extrema; refuse work over the caller's edge budget.
- [x] Preserve existing APIs, exact extrema, half-open boundaries, source clipping, manual overrides and protections.
- [x] Observe semantic RED before production implementation, then relevant tests, formatting and independent review.

## Results

### Completed shared query prerequisite — 2026-10-30

Behavioral source `4df16d4` passes the prepared budget regression (one passed, 106 filtered out), all 100 root-library tests (seven existing ignored cases), all 14 effective-envelope integration tests and all three timebase integration tests. The tests cover legacy exact budget/refusal, half-open retarget ownership, empty/reversed/clipped source ranges, manual/protection extrema against a sample-by-sample oracle, offscreen edges and an exclusive endpoint that consumes candidate budget without contributing its gain. Existing APIs and callers remain unchanged.

The implementation counts candidate edges with indexed partition points before invoking the existing exact evaluator. This bounds candidate-dependent query work without changing gain curves or introducing a sampling approximation. Effective edges include the same start/end candidates the existing evaluator inspects; legacy retargets are strictly inside the clipped source range. `BoundedGainBounds` explicitly distinguishes exact lower/upper gain, empty range and too-dense detail. No display or audio claim is added.

Checks use the owner-authorized ticket 111 policy: timeout and one compiler, no RAM/commit preflight, reserve or cap. Pinned Rust 1.99 and LLD select only the final test targets while retaining locked/offline shared build settings. Receipts under `%LOCALAPPDATA%/CleanSlate/BuildReceipts/`:

- `20261030-202820-df9c410749f24254b02233986815a859`: budget GREEN and full library suite, completed in 99.06 seconds.
- `20261030-203020-f9dd0bda2c024d0b984412401fd8098c`: effective-envelope and timebase suites, completed normally.
- `20261030-203343-08cef1a4dfae46509e1c673051ba630f`: retained no-default-feature strict Clippy failure caused by six existing unused Sound preset declarations after disabling their UI consumers; no changed query-code finding.
- `20261030-204051-6a2d21c9dd6d402591bdd4f53e653b3b`: standard default-feature strict library Clippy (`-D warnings`) and changed-file formatting pass in 8.27 seconds; no warning suppression or unrelated Sound changes.

Independent Spec review found no issue; Standards review requested `#[must_use]` on the pure query result. Final source `1106d3e` applies that lint-only attribute and passes the final strict Clippy/format checks. Functional tests and preserved executable hashes refer to `4df16d4`; runtime checks were not repeated solely for this attribute. The coordinator independently reviewed the candidate counting and scope.

Preserved test-only executables, checksums, logs and receipts: `%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/ticket110/20261030-green-4df16d4/`, including `TEST-ONLY.md`. Library executable SHA256 `01a5ac40d6e116b0ee68ceabc714fdc912163f83b698dcf3dc567f5452ed38bd`; effective-envelope executable `b0d715e37efac87f9c123bcd7293d974fa6f8e8cb588c7a0e0ae1e67c38c5a2b`; timebase executable `18a045625472bfc4f3c560f9adce614e7b5a79317e4ee78fdf150d23c90d067a`.

Receipt cleanup/statistics describe the recorded job only, retaining ticket 109's actual compiler-descendant membership caveat. That accounting limitation is not a RAM qualification gate. No GUI/native interaction, performance benchmark, listening, plugin integration or release acceptance is claimed. Ticket 103's Analysis gain display remains separate work. Earlier unverified checkpoint descriptions below are historical and superseded by these results.

Owner override: completed ticket 111 removes the earlier RAM-cap qualification
gate. Resume this prepared GREEN under the timeout/single-compiler wrapper,
without memory preflight, reserve or cap. Ticket 109's cleanup/accounting caveat
remains separate. Earlier stopped-checkpoint instructions below are historical;
they do not authorize reintroducing a memory gate.

Claimed 2026-10-30 from integration `56d5d82` in an isolated managed worktree. Code graph transport is unavailable; direct source and caller searches cover the affected query paths. Public API/source proposal sent to the coordinator before test or production edits. No compiler, tests or production implementation started; Gate owns the sole execution slot, followed by Podcast's prepared gesture regression.

Coordinator approved the additive `BoundedGainBounds::{Bounds(f32, f32), Empty, TooDense}` result and `GainEnvelope::gain_bounds_bounded(start, end, max_edges)` at the existing public query seam. Existing methods and callers stay unchanged. The budget counts exactly the indexed candidate slice used by the evaluator: effective edges at or after the start tick through the ceil-converted end tick inclusive; legacy retargets strictly after the start frame and before the clipped end frame. Counting uses two partition points before evaluation or allocation.

Prepared, unrun RED assertions cover legacy budget refusal and exact threshold, half-open retarget ownership, source clipping/empty/reversed ranges, and effective manual/protection extrema against a sample-by-sample oracle. The coordinator explicitly approved a temporary boundary scaffold to obtain behavioral RED rather than an absent-method compiler error: it delegates to the existing exact evaluator and deliberately ignores the budget. This scaffold is not an implemented or shippable bounded API. No candidate counting is implemented before the semantic `TooDense` failure runs. The compiler slot remains reserved for other work.

### Functional RED and unverified GREEN checkpoint

On clean source `a27daac`, immediate native preflight measured 21.03 GiB available physical memory and 20.96 GiB commit headroom. The selected root-library unit target used pinned Rust 1.99, existing locked/offline single-job shared arguments, `--no-default-features --profile test` and final-target-only pinned LLD. After prerequisite compilation (7 minutes 48 seconds), the budget test failed semantically: expected `TooDense`, actual `Bounds(0.25, 0.75)`; one failed, 106 filtered out. Receipt `20261030-195338-aa9aeffff95149bfa6fb863f5634232f` finalized with exit 101 and no remaining recorded job members.

This proves functional RED, **not contained compiler execution**: the receipt recorded only one active process and 36,892,672 bytes peak job commit, while a read-only observation saw rustc at about 1,849 MiB private memory. Ticket 109's subsequent tiny actual-membership probes reproduce children outside the named job even with actual Cargo and matching no-breakaway job flags. No heavy GREEN run is authorized until that prerequisite is resolved. Global minimum physical/commit availability stayed about 16.8/16.3 GiB, which does not establish the missing owned-process cap.

The minimal GREEN source is preserved but unrun: clip and reject empty ranges, count legacy retargets or effective sorted edges with partition points, return `TooDense` before evaluation when over budget, otherwise delegate unchanged exact extrema. Added effective tests cover offscreen edges consuming no budget and an exclusive endpoint consuming candidate budget without contributing its gain. No existing caller, old API, dependency, UI, format or audio behavior was changed. Formatting and GREEN checks remain unrun. The exact RED executable, receipt and log are preserved outside the worktree under `%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/ticket110/20261030-red-a27daac/`; it is a failing test executable, not an application build.

RED executable SHA256 is `7dfa2fcccd9d699f012640d3ef135b19d6aad7ec422882df142f33068925ef4c`, matching the copied artifact. Prepared GREEN commit `5f6cc42` is deliberately unverified and must not be integrated as completed behavior. Keep this incomplete worktree and branch until 109 proves actual compiler-chain containment and the scoped GREEN checks can run. Status remains claimed; no resource threshold, generic guard semantics or release claim was changed.
