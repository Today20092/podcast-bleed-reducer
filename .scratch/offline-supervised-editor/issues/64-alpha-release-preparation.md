# Prepare and ship the accepted development release

Status: done
Type: task
Assignee: Codex
Blocked by: None
Labels: ready-for-agent

## Owner authorization

2026-10-22: the owner accepted the current development GUI and requested updated documentation, help, README, screenshots, feature/performance summary, roadmap/ticket reconciliation, promotion to main and shipping the release. Parallel agents are authorized. This supersedes prior statements withholding main/release authorization. Preserve truthful alpha/platform limits and required mechanical package checks.

## Scope and acceptance

- [x] Audit the development-to-main diff, current release/version and outstanding release checks. Record remaining physical/platform limitations without inventing acceptance.
- [x] Refresh representative native screenshots, usage/help/README, release notes and measured performance summary. Integrate scoped documentation work from temporary worktrees.
- [x] Reconcile roadmap/tickets with current owner acceptance and retain future work explicitly.
- [x] Choose the next unused alpha version, update release metadata consistently, run applicable source and hosted/package checks, retain executable/package identities. Record unavailable local checks separately.
- [x] Promote development to main through a PR with a merge commit, attach the PR, synchronize release metadata back to development.
- [x] Tag the reviewed source, inspect the four-platform draft/package workflow and verify release artifacts/checksums. Record available native acceptance and its limits.
- [x] Complete Windows export acceptance and publish the verified release downloads at the release-checklist boundary; reconcile release documentation and the README download table.

## Results

Published on 2026-10-22: [v0.1.0-alpha.8](https://github.com/Today20092/podcast-bleed-reducer/releases/tag/v0.1.0-alpha.8), tag source `24361a2aa201fb7e22258f44a2a1be4626d58ce1`, promoted by [PR 18](https://github.com/Today20092/podcast-bleed-reducer/pull/18). Main and development contain the reviewed release metadata. [Alpha.8 source CI](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39741751147) passed on all four platforms: Windows 353 tests and each Mac/Linux target 349, zero failures and 20 ignored per target, plus formatting, strict Clippy and script fixtures. [Alpha.8 packaging](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39742979614) completed successfully on all four targets and prepared the now-published prerelease.

All eight release assets, four native packages and four SHA-256 files, were downloaded before publication. Checksums and package identity against the tested Actions artifacts passed for every platform; those verified assets are now published. Windows Sandbox interactive export passed for three 600-second, 48 kHz mono 32-bit float WAVs, with RIFF headers independently inspected. Existing solo/room-tone classifications were resaved; save/reopen retained 18 references. The actual host package analyzed and advanced in Original, Processed and Removed, including an overlap region, without device errors. This is functional acceptance, not a new subjective sound-quality claim.

After publication, unauthenticated HTTP HEAD checks returned 200 for all eight public assets. README download-table reconciliation is carried by [PR 19](https://github.com/Today20092/podcast-bleed-reducer/pull/19), together with the final documentation record; duplicate bot PR 20 was closed.

The [release receipt](../../../docs/research/alpha8-release-receipt-2026-10-22.md) owns package/executable identities, final public-download reconciliation and detailed native checks. Documentation and seven actual Windows screenshots are complete. Release preparation is done; broader qualification remains open in 23/41/42, and future features 60/63 remain outside this release. Queue reconciliation leaves six active tickets and two deferred proposals.

### Earlier candidate history

Packaging correction: alpha.7's run 39734525594 failed on every platform at the license collector (`zune-core 0.4.12`); no draft was created. The same audit also found missing `zune-jpeg 0.4.21` coverage. Alpha.8 adds the identical upstream Zlib notices from the exact crate revisions; a four-target metadata audit reports no remaining missing coverage. Preserve alpha.7's failed tag and rebuild/reverify alpha.8 under this ticket.

[Alpha.7 source CI](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39732866350) passed on all four platforms: Windows 353 tests, each Mac/Linux target 349, all with 20 ignored and zero failures, plus strict Clippy, formatting and script fixtures. PR 17 merged as `e5c37aa29856da002f148f9475ca171e8ad98674`; annotated `v0.1.0-alpha.7` identifies that main commit. Development is synchronized. The [release receipt](../../../docs/research/alpha8-release-receipt-2026-10-22.md) records verification and the existing low-severity transitive `atty` alert.

Documentation agents integrated through f8b8865/2a0d7e1 and release candidate 6f02015. Seven actual Windows alpha.7 GUI captures include manual gain dragging and preservation; temporary demo edits were undone. README, user guide, portable help, release guide/notes, performance evidence and roadmap are refreshed. 89 edited-document local references passed; five badge endpoints returned SVG successfully. [Promotion PR 17](https://github.com/Today20092/podcast-bleed-reducer/pull/17) later merged after its four native source checks passed. Local formatting, script fixtures and strict Clippy passed. The local test compile could not complete: after clearing only this package's stale cache artifacts, Windows reported paging-file exhaustion (OS 1455); a command-only four-codegen-unit retry also exhausted rustc allocation. These are host compiler failures, not passing local tests. Fresh hosted source and package checks subsequently passed for alpha.8 as recorded above. The screenshot executable/runtime and local logs are preserved outside the repository.

Initially claimed on development at 6160da9 before release edits, when the application was version 0.1.0-alpha.6. The audit selected alpha.7, then preserved its failed packaging tag and advanced to alpha.8 after adding the required dependency notices. Documentation and release review proceeded in parallel; Cargo builds remained coordinated in the shared target.
