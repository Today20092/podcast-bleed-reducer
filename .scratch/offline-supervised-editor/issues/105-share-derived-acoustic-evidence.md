# 105: Share immutable completed acoustic evidence

Status: done
Assignee: Codex rust_ticket105
Readiness: ready-for-agent
Type: task
Blocked by: None (can start immediately)

## What to build

Cache reuse and final source revalidation reuse completed immutable acoustic frame evidence without independent recording-length copies. Keep all source fingerprints, label/recipe identity, cancellation and accepted-result protection; analysis probabilities and effective envelopes remain unchanged.

## Specification

[Accepted Google Rust audit corrections](../../offline-supervised-migration/rust-guidelines-corrections.md).

## Acceptance criteria

- [x] Unchanged source/label cache hits share the completed derived payload rather than deeply copying frame vectors; a validation-only recheck does not materialize a discarded independent payload.
- [x] Deterministic consumer/cache tests demonstrate shared reuse or equivalent bounded allocation evidence, without a machine-specific timing threshold.
- [x] Same-length source tampering is still rejected, label changes invalidate the appropriate derived fit, and cancelled/failed refits retain prior accepted evidence.
- [x] Existing acoustic, ownership and editor cache consumers remain correct, with unchanged probability and envelope results.
- [x] Focused RED/GREEN evidence, relevant combined checks and any unmeasured long-session limits are recorded under the existing resource policy.

## Results

Claimed 2026-10-27 on codex/rust-audit-105 from integration source 5a321767b09cd42eb836c996979da5bea05b714c. The approved verification seam is the existing public acoustic-cache consumer and its source/label validation boundary.

RED checkpoint 34d698c0a53928e1ca65ed78d35331df7c36f881 adds unchanged_acoustic_requests_share_completed_fit. Hosted run 40626870961 / macos-arm64 job 122613807500 reached the regression and failed at its shared-fit identity assertion (2026-10-28T03:33:35); session_acoustics reported 10 passed / 1 failed. The other checks, including strict Clippy, had reached test execution. This is actual failing behavior, not a compile-only or inferred RED.

The production correction stores and returns one Arc<SessionAcoustics> per completed derived fit. Cache hits and the existing final ownership revalidation now clone only that handle, not the recording-length vectors. All source fingerprints, alignment/readiness checks, reference/recipe identity, cancellation and atomic completed-cache replacement remain in place. Existing consumers borrow through Arc dereferencing; no algorithm, source-validation call, model recipe or serialized field changed.

GREEN source 954e0cf3cec3ae28678349ddfb5d65337d58cda7 (production correction f312169) passed hosted run 40627689255 on all four configured platforms: Linux job 122616644660, Windows job 122616644907, macOS ARM job 122616644992 and macOS Intel job 122616645051. Each job ran the existing scripts/check.ps1, including formatting, strict all-target/all-feature Clippy, the full all-target tests and ownership-model tests. The macOS ARM log explicitly records all 11 session_acoustics regressions passing, including the new shared-fit test, same-length payload tampering, cancelled-refit/label invalidation, known scalar/FIR results, alignment and terminal-tail cases; warm/cold cache-result parity also passes in prepared_analysis. Thus the observed behavior RED is followed by actual GREEN, not merely source inspection.

The latest integration tip 2968487 was merged before handoff; its additional 107 changes are documentation-only. No local Cargo/compiler ran because the desktop resource policy is unchanged. Primary-checkout combined integration and independent branch review are parent-owned batch checks; the all-platform evidence above identifies its exact pre-closure source. No long-session time or peak-memory improvement is measured or claimed. No new executable was built locally or needs preservation from this worktree.
