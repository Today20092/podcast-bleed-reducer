# Protect main and enable repository maintenance

Status: done
Type: task
Assignee: Codex
Blocked by: none

Owner authorized all five recommended settings on 2026-10-13:
require PRs and four native checks for main, disallow force pushes/deletion and
enforce administrators with zero mandatory reviewers; change published-download
automation to open checked PRs; enable secret scanning/push protection; enable
Dependabot security updates; automatically delete merged branches.

## Acceptance

- Download-link updates push a new branch and open a PR, never push main.
- Bot PRs explicitly dispatch native checks using scoped GITHUB_TOKEN permissions.
- Workflow regression tests and native PR checks pass before integration.
- Verify all live protection/security/cleanup settings through GitHub APIs.
- Preserve read-only default workflow permissions and no runtime/application changes.

## Completion

PR 10 merged as 3caf3f5 after native run 38377735535 passed all four checks.
Live API assertions verified strict main protection with four checks bound to
GitHub Actions app 15368, administrators enforced, zero required reviewers and
force pushes/deletion disabled. Secret scanning, push protection, Dependabot
alerts/security updates and merged-branch cleanup are enabled. Actions defaults
remain read-only; automation PR creation is permitted.

Manual download workflow run 38379238075 passed under the new protection.
README was already current, so its PR-creation branch was not exercised in that run.
All workflow/packaging/status fixtures passed; no application behavior changed.
