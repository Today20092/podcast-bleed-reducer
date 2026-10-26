# Desktop and updater remaining-work readiness audit

Status: done
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 88
Related: 85, 86

## Scope and acceptance

Audit the existing desktop queue and updater delivery without inventing another updater or reopening owner-waived quality work. Keep development changes separate from plugin branches. Read 22, 72, 73, 74, the 1.1 release receipt and versioning/branch policy; verify local records against available authoritative release evidence when needed.

Acceptance:
- [x] Produce docs/research/desktop-next-work.md with each remaining action, dependency, evidence and whether agent-ready, needs-info or owner-only.
- [x] Verify package/hash/descriptor records and identify the exact private-signing/publication barrier in 74. No private-key access, password request, automatic signing, tag, push or release publication in this audit.
- [x] Review current updater tests for source/project preservation, shutdown cancellation, interrupted update recovery, clean/mixed install rejection and offline/manual fallback; add only concrete uncovered regressions as separate ready tickets.
- [x] Keep 72 savings-stage copy/claims needs-info unless real evidence resolves it; do not invent measured time savings.
- [x] Treat 22's historical auto-review cleanup denial as unresolved external authorization/state; inventory exact residual paths, but no overnight deletion or cache wiping. Preserve active Cargo caches, recoveries, media and tested executables.
- [x] Execute only change-appropriate existing checks under 88; do not turn the audit into a full repeated build matrix or a new desktop redesign.
- [x] Classify obsolete 75/76/78 checks against 80/86 without dropping unverified acceptance or changing dependencies merely to make the queue runnable.

## Results

Claimed 2026-10-26 after ticket 88 completion. Audit source/test coverage and existing release/package evidence without rebuilding, publishing, private-key access or recovery deletion. Preserve owner-only acceptance, needs-info savings claims and older native-host obligations.

Completed 2026-10-26: [desktop readiness audit](../../../docs/research/desktop-next-work.md) records action-level dependencies, evidence and agent-ready/needs-info/owner barriers. Connected GitHub run/job evidence confirms all four final 1.1 package jobs and draft preparation succeeded; four unexpired artifact-container hashes/source identities are recorded separately from unverified inner distributable hashes. Public latest remains v1.0.0; v1.1.0 lookup returns 404 through connected/public routes, so current draft visibility is unresolved rather than assumed deleted. The preserved 1.1 development executable freshly matches its recorded SHA256; final-package/descriptor/signature bytes are absent from the documented local retention location. Final artifact verification and exact owner-private descriptor signing/publication remain under 74; public 1.0.0 still requires manual bootstrap.

Graph-first and focused current-source/test review covered shared close/worker cancellation acknowledgment, package identity/inventory/signature rejection, helper parent verification, journaled replacement/recovery, empty candidate health checks and explicit cleanup. Existing nine-case packaged receipt is preserved evidence, not a rerun. No demonstrated uncovered updater regression justified a new ticket. 72 savings stays needs-info; no second updater or waived quality project is created. 75/76/78 residual native, CLAP scheduling, sound/PDC/export/reopen checks are explicitly mapped to 80/86 without closing them or clearing 81–83 dependencies.

Exact retired recovery target streaming inventory: 9,549 files, 13,541,383,452 bytes; zero errors/encountered reparse points. Recovery manifest hash matches prior receipt. No deletion/cache wiping or workarounds for historical approval denial; reclaimed 0 bytes. Read-only memory snapshot: 31.92 GiB total / 6.43 GiB available. Documentation-only verification used local link existence, hash/record/source consistency and git diff whitespace; no application build/tests, native host, fixture execution, private-key/password access, signing, settings changes, remote mutation, tag/push/publication or cleanup occurred.

Readiness clarification: 88 already measured `--package-self-check` as a lightweight bounded task (1.27 seconds; 34,025,472-byte kernel peak job commit) under an explicit 1 GiB cap, 2 GiB start threshold, 1 GiB reserve and 90-second timeout. After final artifact/source/hash verification, that CLI check is agent-ready with safe guard refusal; it does not inherently require the 14 GiB heavy-build threshold. Builds and unmeasured native GUI/DAW/listening work retain their existing gates. This clarification downloads/executes no package and leaves 98 done.

