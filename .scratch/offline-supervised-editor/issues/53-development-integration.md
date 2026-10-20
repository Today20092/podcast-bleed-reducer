# Combine completed work in development

Status: done
Type: task
Assignee: Codex
Blocked by: none

Owner requested on 2026-10-20 a shared development branch, an inventory of today's chats/branches/worktrees, combined application checks, and a clear path to a future main merge and release.

## Scope

- Start `development` from current main; merge completed analysis feedback, removable-media export, walkthrough documentation and repository-protection verification records.
- Preserve dirty working copies, retained build caches, frozen baselines and existing releases.
- Resolve integration conflicts and ticket-number collisions without changing feature behavior.
- Retain accepted future-work specifications 41/42 and today's maintainability guidance; leave the wider unfinished documentation reconciliation separate.
- Document the branch/worktree/build inventory and feature → development → main → version-tag workflow.
- Run repository checks, compile a combined GUI build through `scripts/gui-dev.ps1`, and run its offline self-check.

## Acceptance

- Every selected feature tip is an ancestor of development; main and release tags remain unchanged.
- Formatting, CI/package/release fixtures, strict Clippy, tests, GUI compilation and offline self-check pass, or concrete blockers remain recorded without claiming completion.
- The report distinguishes implemented, merged, released, uncommitted and planned work and records remaining hardware/listening qualification.
- Future agent instructions use development as the feature starting point while preserving main as the release-ready integration target.

## Results

- Started from main `0b9622a88643093fc4496cd4729be1d441b3a27c`. Merge commits `5a6e26c`, `7ec9b2f` and `2274ddf` integrate analysis feedback/walkthrough history, removable-media export and repository-protection verification. Every selected branch tip is an ancestor of development. Only two roadmap-note conflicts required resolution; application code merged automatically.
- Renamed analysis-feedback ticket 50 to 52, retaining its scope and validation, because main already uses 50 for publication. Preserved accepted future-work specifications 41/42 without implementing them. Carried the owner's maintainability section into AGENTS.md.
- Updated AGENTS.md, CONTRIBUTING.md and developer documentation: scoped feature branches/worktrees → GitHub PRs to development → pull/build/test the combined checkout → batch PR to main → deliberate tag and draft-package verification → publication. Temporary integration branches cover local pre-PR combination tests. Explicitly documented build-only Actions artifacts and the four native package formats.
- At the owner's follow-up request, updated only the branch-policy section of primary-checkout AGENTS.md and added a short workflow pointer there. Its previous maintainability edits and other unfinished documentation remain intact; its branch was not switched or committed.
- PowerShell 7 CI/workflow, packaging/download and release-status fixtures passed. Formatting and diff-whitespace checks passed. Strict all-target/all-feature Clippy passed with the locked offline dependency set, one Cargo job, retained owner-ui target and scripts/gui-dev.toml.
- Combined `cargo test --all-targets --locked --offline -j 1 --target-dir <retained>/owner-ui --config scripts/gui-dev.toml`: 278 passed, 0 failed, 18 ignored across 32 suites. Ignored checks include owner-recording/device benchmarks, external frozen-baseline evidence and the known quiet-overlap quality gate; these are not passing release evidence.
- `scripts/gui-dev.ps1 -BuildOnly` passed; final GUI build took 0.92 seconds after the test build populated the cache. The combined binary passed offline model inference, analysis, project save/reopen, render and cancellation self-checks.
- Copied the tested executable and its DirectML companion into ignored `target/development/`. `CleanSlate-development.exe` SHA-256 is `aff0fc07c268c6c4fe504f3a22c0bb4abedcb3f3e5f30bb37b329abb18a0e5f5`. Its own self-check was explicitly awaited with Start-Process and exited 0. Adjacent build receipt and logs retain provenance and evidence.
- Initial fixture execution used Windows PowerShell 5 and failed on the PowerShell 7 LinkTarget API; the documented PowerShell 7 retry passed. Context-mode became unavailable during its background test build, so that incomplete attempt was not counted; the complete native PowerShell 7 run above passed.
- Changed documentation links passed. All selected branch tips are ancestors of development. Main and published release tags remain unchanged. The integration is not a stable release qualification; no release was created or published.
- Desktop visual/listening, physical SD write-lock/removal, full owner-recording analysis speed and fresh native Mac/Linux qualification remain unverified. Existing quality/long-session gates remain open. Development protection settings were not changed.
