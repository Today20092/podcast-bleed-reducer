# Google Rust audit corrections

Status: implemented (tickets 104–107 done)
Owner authorization: 2026-10-27 conversation approving the four audit corrections and invoking to-tickets / implement-spec.
Integration branch: codex/rust-guidelines-corrections
Baseline: development at 6a0760c2225c27c5eb4ba95ed39c28849ddc3065.

This is a scoped extension of the sole active offline-supervised migration roadmap, not a second product roadmap. Preserve the accepted specification, ADR-0006, source validation, cancellation, safe publication, exact envelope evaluation and serialized compatibility. Independent Podcast/Gate worktrees remain outside this desktop correction batch.

## Accepted corrections

1. Recover from OS worker-start failures through the application's existing error paths. A failed launch must not panic, leave a phantom active worker or permanently busy state, lose project edits, or publish output. Real workers remain registered before they can run and release registration exactly once after completion or unwinding. Cover both shared launches and the separate playback launch.
2. Reuse immutable completed derived acoustic evidence without cloning recording-length frame vectors. Validation-only rechecks must retain all disk/source identity, label/recipe and cancellation safeguards without materializing a discarded independent copy. Preserve previous accepted evidence on cancellation/failure and identical probabilities/envelopes.
3. Document the real safety obligations and technical need of the Windows updater FFI boundaries. Clarify buffers, NUL termination, lengths/results, parent identity and handle lifetime without changing behavior.
4. Document the non-obvious exported project-time, gain-coordinate and ownership-model feature contracts. Explain units, rounding/ranges/boundaries, feature order/dimension/normalization and unavailable inference at their API seams. Preserve signatures, representations and model recipes.

The full source audit is retained at C:/Users/User/Documents/cleanslate-google-rust-audit-2026-10-27.md. It screened 94 tracked Rust files and identified these four findings; source/caller evidence must be rechecked against implementation changes. Its five optional policy/style choices are not authorized migrations in this batch.

## Task graph

- [104: recover worker-start failures](../offline-supervised-editor/issues/104-recover-worker-start-failures.md): no blockers.
- [105: share completed acoustic evidence](../offline-supervised-editor/issues/105-share-derived-acoustic-evidence.md): no blockers.
- [106: document updater unsafe contracts](../offline-supervised-editor/issues/106-document-updater-unsafe-contracts.md): no blockers.
- [107: document timing and model APIs](../offline-supervised-editor/issues/107-document-timing-model-contracts.md): no blockers.

All four are independently verifiable. Coordinate any overlapping files at integration. One guarded compiler process tree may run globally; parallel source work is permitted.

## Verification boundaries

The owner accepted the audit's scoped acceptance checks by directing implementation of these four tickets. Use the existing application job-start / failure-state and worker-lifetime boundaries for 104, and the existing acoustic/ownership cache consumer and source-validation boundaries for 105. Failure injection must be deterministic, not actual machine exhaustion. The externally visible invariants are actionable launch errors, recoverable UI state, truthful worker exit, immutable evidence reuse, unchanged results, and rejected stale/corrupt/cancelled evidence.

Apply TDD to behavior changes at those seams. Documentation-only 106/107 need contract/source consistency, links and unchanged behavior, not artificial tests matching comment text. Record RED/GREEN evidence truthfully: unrun tests are not passing acceptance.

Run appropriate focused checks and repository-required combined checks through existing guarded entrypoints. The smaller plugin-only warm budgets in resource-guard.md do not automatically authorize lower budgets for this desktop batch. Stop heavy work on resource refusal without retry loops or configuration relaxation. Existing hosted CI may provide additional evidence on an authorized branch; report its source and platform separately from local checks.

Complete both Standards and Spec review of the integration branch, resolve findings, test combined features in the primary development checkout when policy permits, and record limitations. Tickets close only after their scoped acceptance checks pass. No release promotion, tag, signing, plugin deployment or quality/listening claim is included.

## Completion — 2026-10-28

All four tickets passed scoped acceptance and are done. The combined source 618aed8 passed the existing four-platform check matrix; independent Standards and Spec reviews each found zero actionable issues. Integration merge 31e2c33 preserves the tested production/test tree. [Combined receipt](../../docs/research/rust-guidelines-corrections-2026-10-28.md) records exact runs, failed intermediate checkpoints, the refused primary local-check attempt, and residual app-blocked cleanup. Three merged implementer worktrees remain preserved because the app reports another active task still uses them; 107 is archived. No optional style migration or release action is included.
