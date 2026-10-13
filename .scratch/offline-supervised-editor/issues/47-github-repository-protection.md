# Protect main and enable repository maintenance

Status: claimed
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
