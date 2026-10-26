# Desktop and updater next work

Audit date: 2026-10-26. Source: `development` at `486a6ca`, isolated branch `codex/desktop-readiness-98`. [Ticket 98](../../.scratch/offline-supervised-editor/issues/98-desktop-update-readiness.md) owns this documentation audit. Read the [active map](../../.scratch/offline-supervised-migration/map.md), [accepted specification](../../.scratch/offline-supervised-migration/spec.md), [branch workflow](../branch-workflow.md), [release policy](../releasing.md) and [1.1 receipt](stable1.1-release-receipt-2026-10-23.md) together: dated amendments supersede historical planning statements.

The desktop updater is implemented under completed [73](../../.scratch/offline-supervised-editor/issues/73-github-release-in-place-updates.md). The remaining release problem is final-package acceptance and private descriptor signing/publication under [74](../../.scratch/offline-supervised-editor/issues/74-stable-1.1-release.md), not another updater implementation. Owner-closed 14 and retired 15/25 remain closed; this audit creates no new quality benchmark requirement.

## Remaining actions and readiness

Readiness below classifies the action, not the entire ticket. “Agent-ready” does not authorize signing, publication or deletion and does not bypass the memory guard.

| Record / remaining action | Dependency / stop condition | Evidence | Classification |
| --- | --- | --- | --- |
| 74: obtain the four final package artifacts from run 39983829792, compare container digests, extract safely, compare inner package checksums/manifests/source identities, inspect Windows descriptor | Artifact access; preserve checked originals; use 88 for execution and do not substitute a current development rebuild | Four unexpired containers and successful package jobs verified below; no local final packages in named retention location | Agent-ready bounded artifact verification; actual executable/native checks remain resource-blocked here |
| 74: perform actual downloaded Windows offline self-check and record physical checks omitted | Final Windows artifact identity; >=14 GiB available before guarded heavy work | Prior development self-check and CI pass are separate from downloaded final acceptance | Agent-ready when resources permit; native listening/desktop judgment owner-only if required |
| 74: inspect actual draft assets/status, verify final descriptor byte identity and public-key binding | Authorized release access; current API returns 404 for v1.1.0; do not infer draft deletion | Draft-preparation job succeeded; public latest remains 1.0.0 | Agent-ready read-only reconciliation; draft visibility currently needs access/context |
| 74: sign exact checked Windows descriptor privately, then independently verify signature/package binding | Completed package checks; owner-controlled encrypted key/password stay outside repo, CI and chat | Public key pinned; no production signature verified in this audit | Owner-only private signing; never ask for a password or copy private material |
| 74: attach checked signature, publish verified assets, verify public downloads, merge checked generated download update and synchronize branches | Exact signed descriptor, final acceptance and explicit current release authorization | Historical shipping authorization recorded; audit explicitly excludes all remote mutations | Owner release lane; agent may perform authorized public post-publication checks later |
| 72: define manual-editing baseline, formula and counting/reset/reopen/dedup rules for savings feedback | Real comparison evidence or explicit owner assumptions; cancelled/failed/repeated work cannot inflate counters | Optional purchase stage accepted; no measured manual-work baseline supplied | Needs-info; do not implement numeric estimates or silently close the whole feature |
| 72: complete remaining relevant preference/keyboard/theme/failure-flow acceptance, keeping accepted optional purchase wording | Existing implementation; actual GUI/native authorization and resources if needed | Recorded software pass and owner appearance acceptance do not establish every unchecked item | Agent-ready preparation; attended native/checkout review owner-only; no payment/account action |
| 22: disposition of exact retired recovery target, then preservation checks and reclaimed-byte receipt | Historical automatic approval denial unresolved; fresh authorization/state must permit cleanup | Fresh streaming inventory below; no deletion attempted | Owner/external-authorization barrier, not unattended agent-ready cleanup |
| Recurring desktop release checks from 23/41/42 and per-release procedure | Select only checks relevant to a changed behavior/release; disclose unperformed hardware checks | Existing closed tickets retain unchecked matrices; owner waived more computer control for accepted GUI | Future change/release acceptance, not reopened feature tickets |
| 75/76/78 residual plugin checks | Carry through Podcast 80/86 and host matrix; preserve old identity/artifacts | Detailed disposition below | Product work, separate from desktop; native/listening and CLAP blockers remain |

Ticket 74's header says “none (72 and 73 done)”. Treat this as stale shorthand for the accepted **purchase stage** of 72 and completed 73; whole-ticket 72 is still claimed/needs-info. It is not a reason to invent savings evidence or block the already accepted 1.1 purchase/updater batch. No dependency/status was changed to make a queue runnable. Historical paths in branch policy/old tickets are provenance; the current primary checkout is `C:/Users/User/Documents/podcast-automix-rust-gui`.

## Release, package and signature evidence

Read-only GitHub checks on 2026-10-26 used the connected GitHub tools for run artifacts/jobs and release lookup. `gh` was unavailable on PATH; unauthenticated GitHub REST supplied public release metadata. Both connected release-by-tag and public REST returned 404 for v1.1.0. This means no visible v1.1.0 release through those routes, not proof no private draft exists.

- [Public latest](https://github.com/Today20092/podcast-bleed-reducer/releases/latest) resolves in GitHub REST to **v1.0.0**, stable, published `2026-10-22T17:28:36Z`. Eight visible assets comprise four native packages and their four checksums. Windows ZIP metadata: 18,288,883 bytes, digest `c42c99ed2df88f35ef698e2ea7d89bb0f824373df8b3b1ba0ae20673f130f748`. This audit did not redownload/retest it.
- [Tagged package run 39983829792](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39983829792): all four package jobs and **Prepare one draft with all desktop packages** completed successfully. Package checking/build steps succeeded for each native lane. This updates the older receipt's pending workflow state, but does not establish downloaded-package or private-signature acceptance.
- All returned containers name head SHA `88dda154daa9f442018302e19f1c792513fd0874`, branch `v1.1.0`, matching ticket 74's reviewed merge. Expiration metadata is `2027-01-21T16:05:23Z`, and all were `expired=false` when read. Do not replace these packages by rebuilding today's development source.

The following hashes are **Actions artifact ZIP-container digests**, not hashes of the inner distributable ZIP/DMG/AppImage or descriptor:

| Container | Artifact ID | Bytes | GitHub SHA256 |
| --- | --- | --- | --- |
| windows-x64 | 12514913854 | 18,756,007 | `ea7c367112bfa6a57bca3bcff1eda886c2b01cb50fad75739711b4fda7995bb3` |
| macos-arm64 | 12514827995 | 21,838,524 | `de017554ec0af75ae29f9627985f1892902500ae5a6ba2293e39f054517221c9` |
| macos-x64 | 12516605309 | 22,284,104 | `eaf770f9d1986d8c97aefd5bc65cc0c08fe168901e6b8b55d776095d8b41f42f` |
| linux-x64 | 12515187642 | 21,467,873 | `d68c3abf729d0d43049007aceac6ad7c0af9e61519cd3eacd6daf2d750b9ecb8` |

Local named retention folder `C:/Users/User/AppData/Local/CleanSlate/ReleaseChecks/1.1.0/` contains only the development executable, JSON receipt, prior self-check text and local-check log. Fresh SHA256 of `CleanSlate-development-1.1.0.exe` is `5aca340e79856abb19e9d6d5f02d3a206a3e3d5b1e5a60e2bb60a69bbf7be446`, matching the JSON/release receipt; 63,113,216 bytes, receipt source `79797ce46dae556843fafa4f7236135677348907`. The saved self-check says PASS for offline inference/analysis/save/reopen/render/cancellation. These are revalidated file/hash records, not a newly executed self-check or final distribution package.

Narrow primary `dist/` and `.scratch/release-checks/` top-level inventories expose old alpha/probe outputs, not final 1.1 packages. No whole-disk search was performed; another unrecorded storage location remains possible. No final 1.1 descriptor/signature/package bytes were available at the documented retention path, so their exact hashes and signature validity remain **unverified**.

The public-only trust root, confirmed from `assets/update-public-key.txt`, matches the receipt:

`RWQMaOa5uWhvVOJCDE5I0zi1ybM7aIPZW9D8FZpNyOb7aA6ibb9VsJ0c`

The workflow uploads unsigned `CleanSlate-1.1.0-windows-x64.zip.update.json`; it does not produce a production `.minisig`. [Signing procedure](../update-signing.md) and `scripts/update-metadata.ps1` require the owner to sign exact reviewed bytes privately. Acceptance must bind descriptor version/tag/platform/archive, ZIP hash/size, expanded budgets and inner manifest SHA256 to the downloaded checked Windows package, then verify `.minisig` against the pinned key. Adjacent SHA256 files and container digests establish integrity records, not publisher authentication. Do not sign a regenerated descriptor whose byte identity differs unnoticed from the checked draft.

**Bootstrap remains manual:** public 1.0.0 has no updater. A trusted complete-package manual replacement is required once; a future signed descriptor cannot add an updater to a binary that lacks it. Automatic installation is Windows x64 clean portable-tree/fixed NTFS only. Mixed/modified/legacy folders, unsupported storage and macOS/Linux retain manual guidance. Minisign metadata does not supply Windows Authenticode or macOS notarization; keep those existing limits visible.

## Current updater safety and test coverage

Graph discovery succeeded for the indexed primary checkout: symbol search, inbound trace for `request_safe_close` (ordinary `update` and `update_action` callers), outbound updater trace, and snippets for close guards/recovery/test. Focused current-source reads covered the bodies and test fixtures below. Updater modules, `worker_lifetime.rs` and packaged fixture script have no diff from reviewed `88dda154`; current app guards were read directly. No code or dependency changed.

| Behavior | Current owner layer / coverage reviewed | Evidence boundary |
| --- | --- | --- |
| Preserve unsaved project and in-progress edits | `src/app.rs::request_safe_close` checks workers and edit/dialog states, then Save/Discard/Cancel; save/helper failure leaves session open. Ordinary close and update restart use it | Source review; no owner's unsaved project manipulated. Discard is deliberate, not automatic |
| Cancellation is not worker completion | `close_workers_busy` checks global `worker_lifetime::active`, analysis/export/preview/import/audition work and update `busy`. `CancelUpdate` sets a flag; `UpdateFinished` acknowledges outcome | Test `close_waits_for_export_acknowledgment_and_updater_completion` retains export and busy cancellation; source shows worker guard released on actual thread exit |
| Source/project/preference/cache preservation | `package::validate_tree` requires exact owned inventory; mixed files/directories and modified/hash-mismatched package contents reject. Health mode uses self-check/empty startup, not owner projects | `scripts/test-updater-package.ps1` compares hashes of synthetic project/source/export/preferences/cache files in each case; old delivery receipt records nine passing packaged cases |
| Interrupted replacement / candidate failure | `transaction::prepare` writes recovery instructions/journal before publishing index. `apply` authenticates trees/descriptor, journals same-volume old/new moves, then self-check and nonce/PID/version/manifest GUI handshake. `wait_parent` verifies and awaits actual parent handle | Packaged success, self-check failure, crash, timeout, native lock, ACL denial and forced exit after old rename recorded in [73 receipt](portable-updater-delivery-2026-10-23.md); not physical power-loss proof |
| Recovery / safe cleanup | `recover`/`restore` lock and validate identities, restore only established old tree, refuse ambiguity; committed cleanup is explicit and hash-limited | `preparation_write_failure_never_publishes_a_dangling_installation_index`, `recovery_refuses_unrelated_or_ambiguous_roots`, `cleanup_preserves_unrelated_files_and_retries_an_interrupted_acknowledgment`; latter preserves unrelated data and retries incomplete cleanup |
| Clean/mixed install gate and unsupported storage | `transaction::eligible` checks Windows x64, exact executable/package version, clean tree, storage and pending transaction. `storage_check` bounds capacity; no silent reorganization/elevation | Packaged mixed-folder/corrupt/permission/lock cases plus `disk_space_preflight_refuses_an_unfulfillable_budget_without_mutating_the_installation`; local fixed NTFS checks reviewed |
| Signed package / archive integrity | `Descriptor::verify` verifies pinned key/exact signed bytes and strict identity/budgets; archive validates hashes/paths, bounds, aliases, links and truncation before live writes | `signed_descriptor_requires_pinned_identity_and_exact_bytes`, `extraction_rejects_links_aliases_truncation_and_resource_overruns_without_live_writes`, `unsafe_archive_names_are_rejected_before_extraction` |
| Offline/manual fallback / release selection | No startup check; registered manual action. `transfer` bounds network/time/redirects and reports offline/rate-limit failures; `manual_asset` covers advertised platforms; non-Windows/install-ineligible states use manual route | `malformed_channels_drafts_missing_architecture_and_redirects_fail_closed`, `release_selection_uses_semantic_precedence_and_stable_default`; actual future newer-release discovery used synthetic metadata, not fabricated publication |

Coverage is a source/test and preserved-receipt review, not a rerun. No demonstrated uncovered updater regression was found in this scope, so no speculative ready regression ticket was created. Network timeout/rate-limit code inspection is not a new live fault-injection result. Mechanical tests do not establish native listening, hardware failure behavior, or every possible installation environment.

## Cleanup 22: fresh read-only receipt

Exact residual target:

`C:/Users/User/AppData/Local/CleanSlate/Recovery/2026-10-20-worktree-consolidation/retired-model-runtime-residue/target`

A streaming traversal at 2026-10-26 08:27 EDT counted **9,549 files / 13,541,383,452 bytes (12.611 GiB)**; zero traversal errors and zero encountered reparse points. This matches ticket 22's retired-target inventory. It is file length, not promised reclaimable disk allocation. No cleanup preflight/authorization was inferred and nothing was deleted.

Recovery root `untracked-manifest.json` rehash is `97108e98ea528b8cafc3959ba1b2ec1b809ece6a835be809914630b61722174a`, matching recorded preservation evidence. The separate `validated-development-build/CleanSlate-development.exe` remains present (56,158,720 bytes), current hash `aff0fc07c268c6c4fe504f3a22c0bb4abedcb3f3e5f30bb37b329abb18a0e5f5`; this is a separate preserved artifact, not the 1.1 candidate or the historical active executable hash. No recovery manifest or executable was changed.

Historical automatic approval rejected deleting this target with “blocked by policy”; this audit neither retries nor works around that denial. The enclosing residue source/native-runtime/snapshots, sibling recoveries, active `%LOCALAPPDATA%/CleanSlate/BuildCache`, tested binaries, original media/projects and caches stay preserved. Prior Git-temporary cleanup is already owner-verified complete; old combined 14.03 GiB pending totals in the ticket are historical. Current scope remains the named 12.611 GiB target plus preservation checks. Reclaimed by this audit: **0 bytes**.

## Old plugin obligations retained under Podcast

These are outside desktop acceptance and are not closed by documentation. [Product separation](../plugin-products.md) keeps current plugin source/builds on their product worktrees. [Host matrix](../plugin-host-matrix.md) records legal/format/process-isolation limits. Old `plugins/fl-probe` links in historical tickets now refer to source preserved in those workspaces, not a reason to restore it into desktop.

| Record | Evidence already established | Remaining obligation / continuing owner |
| --- | --- | --- |
| [75](../../.scratch/offline-supervised-editor/issues/75-fl-studio-plugin-prototype.md) | Native owner discovery of VST3/CLAP, parameter access, saved assignments and matched-timeline callbacks; later 77 adds actual VST3 harness | Full seek/loop/bypass/Smart-disable/project-reopen and independently inspected dry/offline export remain unqualified. Exercise applicable behaviors through current 80 candidate / 86 product acceptance; saved FLP contents are not successful reopen evidence |
| [76](../../.scratch/offline-supervised-editor/issues/76-fl-studio-shared-audio.md) | Bounded pass-through PCM queue; native playback and VST3 paired shared-window export; valid WAV containers | Native CLAP export had no Ready windows and 1.518–1.527-second peer timestamp separation. Later synthetic rejection protects audio but does not resolve host scheduling. Carry explicit CLAP export/attenuation barrier through 80/86 and 97; no larger buffer assumed to fix it; retain original FLP/WAV/logs |
| [78](../../.scratch/offline-supervised-editor/issues/78-supervised-plugin-listening-build.md) | Shared trainer, compiled inference, delayed safe fallback/state tests; 0.0.4 waveform candidate; later 84 validates selected native navigation | Historical no-waveform/form UI and paired-streaming product direction are superseded by 80/86. Real setup, quiet speech/overlap, PDC, learned-mode listening, state reopen and native export remain explicit owner acceptance. 84 navigation is not sound acceptance; CLAP attenuation remains unqualified |
| [80](../../.scratch/offline-supervised-editor/issues/80-plugin-session-capture-analysis.md) / [86](../../.scratch/offline-supervised-editor/issues/86-podcast-plugin.md) | Persistent source-time Session capture and maintained offline analysis candidate; 86 coordinates separate Podcast identity/workflow | 80 remains claimed for actual FL sound/export/PDC/save/reopen. 81–83 remain blocked by 80 until done; 95/96 are scoped product leaves, not permission to drop earlier checks or relabel dependencies |

Use current product artifacts/identities deliberately; do not reopen obsolete 0.0.1/0.0.3 delivery designs or silently overwrite old saved projects. VST2, OBS native adaptation, CLAP qualification and macOS/Linux hosting remain separate product gates. No old prototype is counted as a completed Smart Gate.

## Audit validation and limits

Only this report and ticket 98 change. Checked relative Markdown targets, source/release/hash consistency and diff whitespace. Fresh local worktree was clean and branched from claimed-ticket source `486a6ca`; the latest development tip is merged before handoff. No application tests/builds were necessary for documentation-only work. At 08:27 EDT memory was 31.92 GiB total / 6.43 GiB available, below 88's heavy start threshold; no native host, updater fixture, signing, provider rebuild, SDK or full matrix ran.

Next executable desktop action is bounded final-artifact verification under 74, followed by the owner-only signature/release lane. 72 still needs savings assumptions; 22 still needs resolved external cleanup authorization/state. No private key/password, payment, release mutation, cache cleanup, application settings, owner project or media was touched.
