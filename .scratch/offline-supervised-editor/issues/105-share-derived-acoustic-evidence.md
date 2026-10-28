# 105: Share immutable completed acoustic evidence

Status: ready-for-agent
Assignee: unclaimed
Readiness: ready-for-agent
Type: task
Blocked by: None (can start immediately)

## What to build

Cache reuse and final source revalidation reuse completed immutable acoustic frame evidence without independent recording-length copies. Keep all source fingerprints, label/recipe identity, cancellation and accepted-result protection; analysis probabilities and effective envelopes remain unchanged.

## Specification

[Accepted Google Rust audit corrections](../../offline-supervised-migration/rust-guidelines-corrections.md).

## Acceptance criteria

- [ ] Unchanged source/label cache hits share the completed derived payload rather than deeply copying frame vectors; a validation-only recheck does not materialize a discarded independent payload.
- [ ] Deterministic consumer/cache tests demonstrate shared reuse or equivalent bounded allocation evidence, without a machine-specific timing threshold.
- [ ] Same-length source tampering is still rejected, label changes invalidate the appropriate derived fit, and cancelled/failed refits retain prior accepted evidence.
- [ ] Existing acoustic, ownership and editor cache consumers remain correct, with unchanged probability and envelope results.
- [ ] Focused RED/GREEN evidence, relevant combined checks and any unmeasured long-session limits are recorded under the existing resource policy.

## Results

Not started. Record source, scoped verification and limitations here before closure.
