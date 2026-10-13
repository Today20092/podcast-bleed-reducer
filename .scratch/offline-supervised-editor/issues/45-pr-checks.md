# Run native checks before release tags

Status: done
Type: task
Assignee: Codex pr_checks
Blocked by: none

## Scope

Owner authorized a subagent implementation on 2026-10-13. Run formatting,
strict Clippy, Rust tests and packaging-script regressions on pull requests
across the same four native release platforms. Reuse the same command in the
release workflow. Pull requests cannot publish packages or write release caches.

## Acceptance

- PR and release use one check script; command failures stop the job.
- Native Windows, Linux, Apple Silicon and Intel Mac jobs run on PR updates.
- PR jobs use read-only permissions, no persisted checkout credentials and no release steps.
- Static workflow regression checks and packaging regressions pass locally.
- Hosted native PR results remain pending until the parent opens the PR.

## Results

2026-10-13 implementation adds `.github/workflows/pr-checks.yml` with the same
four native runners as release packaging. Both workflows invoke `scripts/check.ps1`.
The shared command includes formatting, strict all-features Clippy, all-target
tests, packaging regressions and workflow invariants. PR jobs use `pull_request`,
read-only contents permission and checkout without persisted credentials. They
cannot save release caches or publish artifacts. No dependency was added.

`scripts/test-ci.ps1`, `scripts/test-packaging.ps1`, PowerShell syntax parsing
and `git diff --check` pass. A mocked Cargo invocation verified the shared script
runs all three commands on success and immediately stops on formatting, Clippy
or test failure. Actual Rust suites are delegated to the parent integration pass.
Hosted native PR checks and branch protection configuration remain pending.

Parent integration verification passed workflow and packaging regressions,
strict-mode release-status fixtures and PowerShell parsing. The shared command now
includes the release-status regression suite. No Rust/application behavior changed.
Branch protection is not changed by this ticket; hosted PR results remain pending.

Completion: run 38374588209 passed all four native PR jobs (Windows, Linux and
both Mac architectures) after normalizing workflow CRLF input for Windows.
The actual regression script also passed with forced CRLF input locally.
Owner authorized merging after checks passed; PR 9 merged into main as 5b148a0.
Branch-protection settings remain unchanged.
