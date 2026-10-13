# Report release readiness from actual checks and artifacts

Status: done
Type: task
Assignee: Codex release-status subagent
Blocked by: none

Owner authorized this follow-up on 2026-10-13. Add a read-only PowerShell helper
that separates workflow, tests, package verification, artifact availability and
draft preparation. Cache saving never counts as package success. Cover missing,
pending, failed, expired artifacts and rerun jobs with fixture tests.

Acceptance: fixture tests pass; actual successful and failed release runs produce
accurate concise and JSON reports; no rerun, cancellation, downloads or publishing.

## Results

Fixture checks passed for failures hidden by active cache saving, failed package
without artifacts/logs, missing/pending checks, expired artifacts, build-only
success and mixed rerun attempts. Actual run 38341277070 reports the Windows
test failure with 0xc0000005/STATUS_ACCESS_VIOLATION and missing Windows artifact,
alongside successful Mac/Linux packages. Run 38366852428 reports four verified
available packages and no draft preparation (build-only run). Script/diff checks
passed. Parent integration review pending; no release mutation occurred.

Parent review added stale-failure deduplication, strict-mode missing-job coverage,
packaging/native error excerpts and failed-step detection while a job saves cache.
All fixtures passed under strict mode. Live successful and failed run checks and
PowerShell parsing passed. The fixture script is wired into the shared PR/release
check command. Readiness describes CI evidence; manual drafts, artifact contents
and public publication are explicitly outside its checks.
