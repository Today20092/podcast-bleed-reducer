# Google Rust audit correction batch

Status: verification pending
Date: 2026-10-28
Baseline: development at 6a0760c2225c27c5eb4ba95ed39c28849ddc3065
Integration: codex/rust-guidelines-corrections
Specification: [accepted corrections](../../.scratch/offline-supervised-migration/rust-guidelines-corrections.md)

## Delivered scope

- [104](../../.scratch/offline-supervised-editor/issues/104-recover-worker-start-failures.md): fallible OS thread startup, transient lifetime rollback and caller-specific recovery, preserving playback's separate cancel/join ownership.
- [105](../../.scratch/offline-supervised-editor/issues/105-share-derived-acoustic-evidence.md): shared completed derived evidence; source/label/recipe validation, cancellation, final boundary revalidation and numerical operations retained.
- [106](../../.scratch/offline-supervised-editor/issues/106-document-updater-unsafe-contracts.md): precise Windows updater safety contracts and technical rationale; no runtime change.
- [107](../../.scratch/offline-supervised-editor/issues/107-document-timing-model-contracts.md): exported time/sample/gain and ownership-feature contracts; no signature, serialization or recipe change.

The five optional style/policy choices in the source audit remain outside this batch. Independent Podcast and Smart Gate source is not merged or re-qualified.

## Verification record

The machine's initial native snapshot was about 10.92 GiB available physical memory and 7.04 GiB system commit headroom, below the existing desktop build start threshold. The smaller previously authorized plugin profiles were not applied to this desktop batch. No local compiler, GUI, native host or listening test has run for these corrections. Existing hosted Pull request checks provide source-identified verification without weakening local resource policy.

- Worker recovery meaningful RED: [run 40627066393](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/40627066393), source e4f598e8d248ddef123b10d214fa25f6e3407814, macOS ARM job 122614435707. The injected launch error panicked instead of recovering. Remaining redundant RED jobs were cancelled after preserving the actual failure.
- Acoustic sharing meaningful RED: [run 40626870961](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/40626870961), source 34d698c0a53928e1ca65ed78d35331df7c36f881, macOS ARM job 122613807500. Shared-fit identity failed; the suite had ten passing cases and one expected failure.
- Acoustic GREEN: [run 40627689255](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/40627689255), source 954e0cf3cec3ae28678349ddfb5d65337d58cda7. Windows x64, Linux x64, macOS ARM64 and macOS x64 all passed the existing full check script.
- Intermediate worker candidates stopped at one formatting mismatch, a Drop-type test-fixture initialization error, then a regression injection-ordering error. These failures and their precise sources/results remain in ticket 104; they are not represented as successful checks.
- Combined candidate: 618aed8058ffff79b8037a1ab1cb8d9e7c850543. [Run 40630060652](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/40630060652) is pending final qualification.
- Independent Standards review: zero actionable findings on the production change at 17feca9; final test-only delta review pending.
- Independent Spec review: zero actionable findings through 618aed8, including corrected failure injection before the actual Analyze command.

Documentation tickets independently verified source contracts, unchanged non-comment implementation and links. Final integration/primary-checkout verification and implementer cleanup are pending. Ticket 104 remains claimed until scoped acceptance passes.

## Evidence limits

Immutable payload reuse establishes avoided copying by ownership and regression evidence; no speedup or long-session peak-memory measurement is claimed. Hosted checks are not local native-device or listening qualification. Source/project validation, cancellation, safe output publication, shared envelope behavior and current compatibility promises remain required. This batch includes no release promotion, tag, signing, publication or plugin deployment.
