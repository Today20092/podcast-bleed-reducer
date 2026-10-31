# Remove local build memory guards

Status: claimed
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: none
Related: 88, 99, 109, 110

## Accepted scope

Owner instruction, 2026-10-30: “remove any memory gaurds, i have 40gb of ram now installed compared to before.”

Remove free-RAM/commit launch thresholds, running reserves, owned-job memory
caps and memory-recovery waiting from active build/test execution. This owner
instruction supersedes every older fixed memory profile, including historical
research and roadmap instructions. Keep execution timeouts, cancellation,
single-compiler serialization, output/exit reporting and owned cleanup.
Memory accounting may remain informational.

Update shared guard/helper APIs, callers, overnight runner and fixtures, product
workspaces and active documentation. Preserve historical measurements as history.
Do not replace the removed limits with another hidden threshold or cap.

## Acceptance

- [ ] Guard CLI and native helper apply no RAM/commit threshold, reserve or cap.
- [ ] Actual native job flags confirm memory-limit flags are absent.
- [ ] Retained timeout, lock, exit, nested execution and cleanup fixtures pass.
- [ ] Runner policies, retry handling and fixtures no longer gate on memory.
- [ ] Desktop and product callers use the revised API; active docs agree.
- [ ] Review and record checks, remaining limitations and integration commits.

## Results

Claimed before implementation. Shared guard and runner changes use separate
worktrees; product callers receive the focused tested shared changes. Ticket 109's
observed compiler membership gap is distinct from memory limits and is not claimed
fixed by their removal.
