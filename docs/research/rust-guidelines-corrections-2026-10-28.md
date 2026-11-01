# Google Rust audit correction batch

Status: implemented; three managed worktree archives remain app-blocked
Date: 2026-10-28
Baseline: development at 6a0760c2225c27c5eb4ba95ed39c28849ddc3065
Integration: codex/rust-guidelines-corrections
Specification: [accepted corrections](../../.scratch/offline-supervised-migration/rust-guidelines-corrections.md)

Source promotion: [PR 24](https://github.com/Today20092/podcast-bleed-reducer/pull/24), merge `f0ea4f02e092d3e8f0c674a5489051f2474a4ba8`, after exact PR head `e1919e9` passed [all four platform checks](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/40635018559). [Ticket 108](../../.scratch/offline-supervised-editor/issues/108-rust-guidelines-promotion.md) records skill installation documentation, development synchronization and remaining release gates. Source promotion did not publish packages or qualify fresh physical playback/listening.

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
- Combined candidate: 618aed8058ffff79b8037a1ab1cb8d9e7c850543. [Run 40630060652](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/40630060652) passed all four platforms: macOS ARM job 122624166212, Linux job 122624166575, Windows job 122624166579, and macOS Intel job 122624166621. The Linux log explicitly passes all eight new regressions, with 380 passing tests and 20 existing ignored tests across its successful test targets.
- Independent Standards review: zero actionable findings through 618aed8, including the final test-only delta.
- Independent Spec review: zero actionable findings through 618aed8, including corrected failure injection before the actual Analyze command.

Documentation tickets independently verified source contracts, unchanged non-comment implementation and links. Tickets 104–107 are all done. Merger review verified production code and tests byte-identical to the combined CI source, then merged the final worker branch at 31e2c33216dd8667f8138b528b058c95ff578712.

## Standards

Zero actionable findings against the documented repository rules and Google Rust adaptation. The independent review covered the whole changed source and the final test-only correction. Mechanical checks ran in CI; source review is separate evidence.

## Spec

Zero actionable findings against the accepted four-ticket scope. The independent review traced launch recovery, lifetime ownership, source validation, immutable evidence reuse and the documented API contracts. The corrected test reaches the real Analyze command's launch boundary.

Review summary: Standards 0, Spec 0; neither axis has an unresolved finding.

## Primary checkout and cleanup

The primary development checkout was fast-forwarded to 31e2c33. A single default-policy outer guard attempted the combined scripts/check.ps1 -Local command there, containing any compiler used by its fixture scripts as well as Cargo. It refused before starting a process at 11.08 GiB available physical memory and 7.82 GiB commit headroom. Receipt rust-audit-33b073001b6e45f38a4899e174f0addb.json is retained under the local CleanSlate BuildReceipts directory; reason low-memory-before-start, cleanup_verified true, process_id null. Defaults stayed 14 GiB start / 8 GiB reserve / 6 GiB owned cap / 1800 seconds. This is a local resource refusal, not a local passing check; no local retry or threshold reduction followed.

The managed archive of the 107 implementer worktree is confirmed. Archives of 104, 105 and 106 were refused by the app with “Another active task still uses this worktree,” including owner-side attempts. Those clean, merged worktrees and their branches remain preserved; no manual removal bypassed the in-use guard. The integration worktree is also retained for the reviewed branch. Existing unrelated worktrees and the primary checkout's pre-existing untracked attachments/screenshots remain intact.

## Evidence limits

Immutable payload reuse establishes avoided copying by ownership and regression evidence; no speedup or long-session peak-memory measurement is claimed. Hosted checks are not local native-device or listening qualification. Source/project validation, cancellation, safe output publication, shared envelope behavior and current compatibility promises remain required. This batch includes no release promotion, tag, signing, publication or plugin deployment.
