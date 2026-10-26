# Memory-bounded execution guard

Status: done
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 87
Related: 85, 86

## Scope and acceptance

Implement the execution guard before any overnight builds, native hosts or deep REA. This is shared development infrastructure on development; integrate only its focused commit into both plugin branches.

Default to one heavy process tree globally across all worktrees, acquired via a named machine-local lock, not one lock per branch. Reuse scripts/local-build-settings.ps1 and the existing application-data Cargo target/config. Preserve incremental caches, --locked resolution, offline reuse, Cargo -j1 and current 16 root codegen units; do not start a second build while Cargo waits on another owner. Inventory existing compiler/analyzer jobs read-only; wait/skip rather than terminate an unrelated process.

On this 31.9 GiB machine use conservative configurable starting defaults: at least 14 GiB available physical memory before a heavy task; reserve 8 GiB while it runs; maximum 6 GiB committed memory for its owned child process tree and a 30 minute task timeout. These are starting protection settings, not measurements of the least RAM required. Sample available memory, system commit headroom and owned process-tree private/working-set bytes at one-second intervals. Report both metrics rather than conflating them. Use Windows Job Object limits/ownership where enforceable, fail before launching if containment is unavailable, and verify child cleanup. A lower-cost task may use a smaller explicit budget; never silently relax a limit to make a build pass.

For low-memory/timeout/budget failure, stop only the owned task tree, keep logs and last-good outputs, release its lock, mark resource-blocked and let other independent lightweight work proceed. Resume no more than once after memory recovers; repeated failures require a checkpoint, not an infinite retry loop. Never kill Codex/ChatGPT, browsers, speech services or user hosts to reclaim RAM.

Acceptance:
- [x] Small disposable-process tests cover competing workspaces, descendant accounting, low-memory refusal, budget crossing, timeout, crash and lock cleanup, and preserved output.
- [x] Existing build/check entry points use the guard without nested-lock deadlock. Direct-command policy enforcement in the overnight runner is tracked by dependent ticket 99; no runner enforcement is claimed here.
- [x] Record peak memory/time, guard reason, source commit and artifact hash for a representative sequential plugin and desktop check.
- [x] Explain that OS sampling is not a hard whole-machine RAM guarantee. Deep REA calls in a shared MCP server are skipped unattended unless that exact provider's resource ownership/limits can be enforced without killing unrelated server work.
- [x] Fetch current platform/API documentation before implementation, and retain useful test/build failures without wiping caches.

## Results

Claimed 2026-10-26 under the owner's implement-spec instruction. Ticket 87 is done. Implementation uses a temporary scoped worktree and integrates into development. Initial available physical memory is 6.99 GiB of 31.92 GiB; heavy checks remain refused below the accepted 14 GiB start threshold. Disposable lightweight guard tests may proceed within an explicit smaller budget. No unrelated process termination or cache cleanup is authorized by this claim.

### Guard implementation and lightweight evidence — 2026-10-26

`scripts/resource-guard.ps1` and its small native C# helper provide the global mutex, suspended child launch before Job Object assignment/resume, hard aggregate commit cap, owned-tree accounting/cleanup, one-second physical/commit sampling, bounded timeout and JSON/log receipts. Nested calls verify actual job membership before reusing outer containment/receipt. GUI builds and local checks use the guard without changing shared Cargo flags. Hosted cross-platform checks retain their existing behavior. [Guard usage and platform sources](../../../docs/resource-guard.md) describe the limits, explicit disposable-test budgets and unattended shared-REA exclusion.

Red/green: initial missing-guard test failed before implementation; low-memory refusal then passed. Actual allocation testing exposed .NET GC's earlier refusal before a kernel limit attempt; the fixture now uses the native allocator under a 128 MiB job cap. This rejection and all earlier receipts are preserved. Native peak-job accounting may include a rejected charge, so the check distinguishes that metric from sampled private/working-set bytes.

Passed `scripts/test-resource-guard.ps1`: no launch on impossible start threshold; successful output capture; descendant accounting; timeout cleanup; exit-code propagation; running-pressure stop; native job-budget rejection; nested working directory/containment; competing-workspace rejection; descendant termination after controller death; abandoned-lock recovery; preservation of last-good output. Evidence: `%TEMP%/cleanslate-guard-6d4d4ace-3175-40d2-b2fd-93348b6ea2cf/`. Longest individual fixture was 4.15 seconds; maximum sampled owned private bytes 75,415,552, working set 181,723,136. The deliberately rejected 128 MiB allocation fixture reported native peak-job commit 160,952,320; this is not a resident-memory measurement.

CI/workflow, local-build-settings, packaging, release-status and action-contract script fixtures pass; all PowerShell scripts parse; `git diff --check` passes. Public entrypoint validation caught and corrected duplicate executable resolution from multiple Cargo installations. `gui-dev.ps1 -BuildOnly` and `check.ps1 -Local` now refuse safely before Cargo launches with only about 6.18 GiB available. Refusal receipts: `%LOCALAPPDATA%/CleanSlate/BuildReceipts/20261026-071200-2ed64b6c0fff4510915ec7d327d9c3bb.json` and `20261026-071201-8af4380d94ac42dd8e237071071cc61d.json`.

The implementation-stage remaining checks below were completed by the merger. Ticket 99 owns rejecting unguarded commands in the runner. No heavy build was launched or threshold relaxed, no unrelated application was terminated, no cache was deleted.

### Integration and representative checks — 2026-10-26

Shared implementation `11bd09059ba5c43b3898898cf2e24e09f4686290` is integrated on development and selectively cherry-picked as Podcast `d63dc20` and Smart Gate `d8440dc`. Product build/host wiring is Podcast `3b023f057d5cee702e770158ce0eec70295eb88c` and Smart Gate `fc15c3f`: every plugin Cargo invocation uses the default heavy guard; the existing headless host runner uses an explicit 0.5 GiB lightweight cap, 2 GiB start threshold, 1 GiB reserve and 90-second outer timeout. Its existing per-host 30-second deadline, isolated child LOCALAPPDATA, cache cleanup and PASS/exit checks remain. Nested calls share verified containment and cannot deadlock on another lock acquisition.

Representative checks reused preserved, previously checked artifacts; they do not claim a rebuild from the guard commits or new product qualification. Both completed sequentially with `reason: completed`, exit 0 and `cleanup_verified: true`:

| Check | Receipt under `%LOCALAPPDATA%/CleanSlate/BuildReceipts/` | Elapsed | Sampled peak private / working set | Kernel peak job commit |
| --- | --- | --- | --- | --- |
| Desktop `rust-app.exe --package-self-check` | `88-desktop-8d979135b7324b989232011664dc01b0.json` | 1.27 s | 1,982,464 / 9,949,184 bytes | 34,025,472 bytes |
| Actual CLAP and VST3 host checks against preserved 0.0.6 | `20261026-071531-95637980291f489e8d3fdc615f8a3ca9.json` | 3.18 s | 100,958,208 / 212,738,048 bytes | 101,302,272 bytes |

The desktop check used a separate explicit 1 GiB cap, 2 GiB start threshold, 1 GiB reserve and 90-second timeout. Sampling can miss short-lived peaks; kernel job commit and the hosts' own 10 ms working-set observations are separate metrics. Desktop passed offline model inference, analysis, project save/reopen, render and cancellation. CLAP passed 1,858 bit-exact blocks and 1,168 shared windows; VST3 passed 4,048 bit-exact blocks and 3,392 shared windows, with editor construction/release and no native window. Host-reported largest sampled peak working sets were 21.20 MiB / 25.07 MiB.

Artifact SHA256 provenance:

- `%LOCALAPPDATA%/CleanSlate/BuildCache/debug/rust-app.exe`: `e21f05b04359a6b8f01077421ea51e32180339ea51a6a120e122dd57fa0a2c0a`, matching the preserved desktop self-check binary in [Session delivery](../../../docs/research/plugin-session-delivery-2026-10-25.md). The recorded historical source was `88dda154` plus that delivery's implementation, subsequently preserved under common source baseline `44cbf432`; it is not a binary compiled from `11bd090`.
- Both existing primary `dist/fl-probe` formats and cached `debug/cleanslate_fl_probe.dll`: `94cf0dc682447677e1d942397dfc8aea929c9fe697a5a0035127968dace54aaa`, matching [navigation delivery](../../../docs/research/plugin-navigation-delivery-2026-10-25.md) and ticket 87's preserved common source `44cbf432c84de90168cbb6c25b386d40142d3f75`.
- Cached `debug/examples/host_check.exe`: `89602157dfce836652880b3a1976d2884f90cd4d83b2412d992fcb567faba9ce`; `vst3_host_check.exe`: `87de8d63212e212f6b4bd6ef4871d32d4d7afcd651ae3f22de84e6aaa6255af3`. These are the existing cached host artifacts, not newly compiled hosts.

On integrated development, the full disposable guard suite and CI/local-build/packaging/release-status/action-contract fixtures pass. Disposable evidence: `%TEMP%/cleanslate-guard-2664af49-9d62-471a-afb5-65abeef11646/`. Both product workspaces pass CI/local-build fixtures, PowerShell parsing and diff checks. Both actual product build entrypoints safely refused before Cargo at the unchanged 14 GiB requirement: receipts `20261026-071616-d537d33db00841cca7a73f49d8271466.json` (Podcast) and `20261026-071618-bf12f0a217fd430a8c66ebdcaa9bfefe.json` (Smart Gate).

Ticket 88 is done for its shared guard and entrypoint scope. Remaining native listening, release signing/publication, product implementation and full guarded rebuilds are not claimed. Ticket 99 remains required before resumable unattended queue execution.

