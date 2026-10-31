# Remove local build memory guards

Status: done
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

- [x] Guard CLI and native helper apply no RAM/commit threshold, reserve or cap.
- [x] Actual native job flags confirm memory-limit flags are absent.
- [x] Retained timeout, lock, exit, nested execution and cleanup fixtures pass.
- [x] Runner policies, retry handling and fixtures no longer gate on memory.
- [x] Desktop and product callers use the revised API; active docs agree.
- [x] Review and record checks, remaining limitations and integration commits.

## Results

Claimed before implementation. Shared guard and runner changes use separate
worktrees; product callers receive the focused tested shared changes. Ticket 109's
observed compiler membership gap is distinct from memory limits and is not claimed
fixed by their removal.

Core `ce046a3` removes the three memory CLI options and constructor limit,
startup/running pressure failures and native memory-limit notification plumbing.
The native flags regression produced RED at `0x2200`, then GREEN at `0x2000`
(kill-on-close only). Retained guard and delayed-descendant suites pass; evidence:
`%TEMP%/cleanslate-guard-80808c55-ad54-45b4-9d47-587fc9a2f2f4/` and
`%TEMP%/cleanslate-guard-completion-43aa3815-9b73-46cb-9f8d-97762c92516e/`.

Runner `2c91266` removes memory policies/refusal/recovery waits. Its public suite
and five source-identity scenarios pass in
`%TEMP%/cleanslate-runner-eace638c-9f30-4176-85db-4addcadf9679/` and
`%TEMP%/cleanslate-source-review-48021424-13e0-4f4f-93b2-5294564d02e6/`.
Integrated through `1b213e6`. Docs/queue policy update is `259db7c`.

Podcast caller `76599f1` and Gate equivalent `bc95a5a` remove the host runner's
memory options while retaining product-specific arguments and its timeout.
Podcast/Gate integrated the exact four tested shared guard files. All active
PowerShell scripts parse: Desktop 25, Podcast 26, Gate 27. Removed memory arguments
and admission/pressure checks are absent from active script sources. Product
coordinator files intentionally absent remain centralized rather than copied in.

Independent Standards and Spec reviews each found zero actionable issues. No
product/native build was needed to verify this infrastructure removal. Earlier
RAM-cap qualification is no longer a launch requirement; 109 remains a distinct
cleanup/accounting limitation, with no claim that every compiler descendant is
recorded. No replacement memory gate was introduced.
