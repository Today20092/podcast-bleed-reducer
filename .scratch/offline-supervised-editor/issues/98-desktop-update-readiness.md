# Desktop and updater remaining-work readiness audit

Status: claimed
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 88
Related: 85, 86

## Scope and acceptance

Audit the existing desktop queue and updater delivery without inventing another updater or reopening owner-waived quality work. Keep development changes separate from plugin branches. Read 22, 72, 73, 74, the 1.1 release receipt and versioning/branch policy; verify local records against available authoritative release evidence when needed.

Acceptance:
- [ ] Produce docs/research/desktop-next-work.md with each remaining action, dependency, evidence and whether agent-ready, needs-info or owner-only.
- [ ] Verify package/hash/descriptor records and identify the exact private-signing/publication barrier in 74. No private-key access, password request, automatic signing, tag, push or release publication in this audit.
- [ ] Review current updater tests for source/project preservation, shutdown cancellation, interrupted update recovery, clean/mixed install rejection and offline/manual fallback; add only concrete uncovered regressions as separate ready tickets.
- [ ] Keep 72 savings-stage copy/claims needs-info unless real evidence resolves it; do not invent measured time savings.
- [ ] Treat 22's historical auto-review cleanup denial as unresolved external authorization/state; inventory exact residual paths, but no overnight deletion or cache wiping. Preserve active Cargo caches, recoveries, media and tested executables.
- [ ] Execute only change-appropriate existing checks under 88; do not turn the audit into a full repeated build matrix or a new desktop redesign.
- [ ] Classify obsolete 75/76/78 checks against 80/86 without dropping unverified acceptance or changing dependencies merely to make the queue runnable.

## Results

Claimed 2026-10-26 after ticket 88 completion. Audit source/test coverage and existing release/package evidence without rebuilding, publishing, private-key access or recovery deletion. Preserve owner-only acceptance, needs-info savings claims and older native-host obligations.

