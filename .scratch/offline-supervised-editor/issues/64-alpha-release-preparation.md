# Prepare and ship the accepted development release

Status: claimed
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
- [ ] Choose the next unused alpha version, update release metadata consistently, run combined local and hosted/package checks, retain executable/package identities.
- [ ] Promote development to main through a PR with a merge commit, attach the PR, synchronize release metadata back to development.
- [ ] Tag the reviewed source, inspect the four-platform draft/package workflow and verify release artifacts/checksums and available native acceptance. Publish only at the verified boundary required by the release checklist; record any unavailable checks and exact remaining actions.

## Results

Packaging correction: alpha.7's run 39734525594 failed on every platform at the license collector (`zune-core 0.4.12`); no draft was created. The same audit also found missing `zune-jpeg 0.4.21` coverage. Alpha.8 adds the identical upstream Zlib notices from the exact crate revisions; a four-target metadata audit reports no remaining missing coverage. Preserve alpha.7's failed tag and rebuild/reverify alpha.8 under this ticket.

[Alpha.7 source CI](https://github.com/Today20092/podcast-bleed-reducer/actions/runs/39732866350) passed on all four platforms: Windows 353 tests, each Mac/Linux target 349, all with 20 ignored and zero failures, plus strict Clippy, formatting and script fixtures. PR 17 merged as `e5c37aa29856da002f148f9475ca171e8ad98674`; annotated `v0.1.0-alpha.7` identifies that main commit. Development is synchronized. The [release receipt](../../../docs/research/alpha8-release-receipt-2026-10-22.md) records verification and the existing low-severity transitive `atty` alert.

Documentation agents integrated through f8b8865/2a0d7e1 and release candidate 6f02015. Seven actual Windows alpha.7 GUI captures include manual gain dragging and preservation; temporary demo edits were undone. README, user guide, portable help, release guide/notes, performance evidence and roadmap are refreshed. 89 edited-document local references passed; five badge endpoints returned SVG successfully. [Promotion PR 17](https://github.com/Today20092/podcast-bleed-reducer/pull/17) is open with required native checks running. Local formatting, script fixtures and strict Clippy passed. The local test compile could not complete: after clearing only this package's stale cache artifacts, Windows reported paging-file exhaustion (OS 1455); a command-only four-codegen-unit retry also exhausted rustc allocation. These are host compiler failures, not passing tests. The prior application-source check passed 353 tests with 20 ignored; fresh hosted checks must verify the release candidate. The screenshot executable/runtime and local logs are preserved outside the repository. Native package/download verification and publication remain pending.

Claimed on development at 6160da9 before release edits. The current application is version 0.1.0-alpha.6; release/version availability audit is in progress. Documentation and release review may proceed in parallel; Cargo builds remain coordinated in the shared target.
