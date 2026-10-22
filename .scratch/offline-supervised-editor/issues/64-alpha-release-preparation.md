# Prepare and ship the accepted development release

Status: claimed
Type: task
Assignee: Codex
Blocked by: None
Labels: ready-for-agent

## Owner authorization

2026-10-22: the owner accepted the current development GUI and requested updated documentation, help, README, screenshots, feature/performance summary, roadmap/ticket reconciliation, promotion to main and shipping the release. Parallel agents are authorized. This supersedes prior statements withholding main/release authorization. Preserve truthful alpha/platform limits and required mechanical package checks.

## Scope and acceptance

- [ ] Audit the development-to-main diff, current release/version and outstanding release checks. Record remaining physical/platform limitations without inventing acceptance.
- [ ] Refresh representative native screenshots, usage/help/README, release notes and measured performance summary. Integrate scoped documentation work from temporary worktrees.
- [ ] Reconcile roadmap/tickets with current owner acceptance and retain future work explicitly.
- [ ] Choose the next unused alpha version, update release metadata consistently, run combined local and hosted/package checks, retain executable/package identities.
- [ ] Promote development to main through a PR with a merge commit, attach the PR, synchronize release metadata back to development.
- [ ] Tag the reviewed source, inspect the four-platform draft/package workflow and verify release artifacts/checksums and available native acceptance. Publish only at the verified boundary required by the release checklist; record any unavailable checks and exact remaining actions.

## Results

Documentation agents integrated through f8b8865/2a0d7e1 and release candidate 6f02015. Seven actual Windows alpha.7 GUI captures include manual gain dragging and preservation; temporary demo edits were undone. README, user guide, portable help, release guide/notes, performance evidence and roadmap are refreshed. 89 edited-document local references passed; five badge endpoints returned SVG successfully. [Promotion PR 17](https://github.com/Today20092/podcast-bleed-reducer/pull/17) is open with all four required native checks running. Local checks hit E0463 loading a stale application crate after the version bump; the tested screenshot executable/runtime were preserved outside the repository, and only this package's shared-cache artifacts were cleared before retrying. The retry's Clippy stage passed; final tests are pending. Native package/download verification and publication remain pending.

Claimed on development at 6160da9 before release edits. The current application is version 0.1.0-alpha.6; release/version availability audit is in progress. Documentation and release review may proceed in parallel; Cargo builds remain coordinated in the shared target.
