# Memory-bounded execution guard

Status: claimed
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
- [ ] Existing build/check entry points use the guard without nested-lock deadlock; direct unguarded heavy invocations are rejected by the overnight runner.
- [ ] Record peak memory/time, guard reason, source commit and artifact hash for a representative sequential plugin and desktop check.
- [x] Explain that OS sampling is not a hard whole-machine RAM guarantee. Deep REA calls in a shared MCP server are skipped unattended unless that exact provider's resource ownership/limits can be enforced without killing unrelated server work.
- [x] Fetch current platform/API documentation before implementation, and retain useful test/build failures without wiping caches.

## Results

Claimed 2026-10-26 under the owner's implement-spec instruction. Ticket 87 is done. Implementation uses a temporary scoped worktree and integrates into development. Initial available physical memory is 6.99 GiB of 31.92 GiB; heavy checks remain refused below the accepted 14 GiB start threshold. Disposable lightweight guard tests may proceed within an explicit smaller budget. No unrelated process termination or cache cleanup is authorized by this claim.

### Guard implementation and lightweight evidence — 2026-10-26

`scripts/resource-guard.ps1` and its small native C# helper provide the global mutex, suspended child launch before Job Object assignment/resume, hard aggregate commit cap, owned-tree accounting/cleanup, one-second physical/commit sampling, bounded timeout and JSON/log receipts. Nested calls verify actual job membership before reusing outer containment/receipt. GUI builds and local checks use the guard without changing shared Cargo flags. Hosted cross-platform checks retain their existing behavior. [Guard usage and platform sources](../../../docs/resource-guard.md) describe the limits, explicit disposable-test budgets and unattended shared-REA exclusion.

Red/green: initial missing-guard test failed before implementation; low-memory refusal then passed. Actual allocation testing exposed .NET GC's earlier refusal before a kernel limit attempt; the fixture now uses the native allocator under a 128 MiB job cap. This rejection and all earlier receipts are preserved. Native peak-job accounting may include a rejected charge, so the check distinguishes that metric from sampled private/working-set bytes.

Passed `scripts/test-resource-guard.ps1`: no launch on impossible start threshold; successful output capture; descendant accounting; timeout cleanup; exit-code propagation; running-pressure stop; native job-budget rejection; nested working directory/containment; competing-workspace rejection; descendant termination after controller death; abandoned-lock recovery; preservation of last-good output. Evidence: `%TEMP%/cleanslate-guard-6d4d4ace-3175-40d2-b2fd-93348b6ea2cf/`. Longest individual fixture was 4.15 seconds; maximum sampled owned private bytes 75,415,552, working set 181,723,136. The deliberately rejected 128 MiB allocation fixture reported native peak-job commit 160,952,320; this is not a resident-memory measurement.

CI/workflow, local-build-settings, packaging, release-status and action-contract script fixtures pass; all PowerShell scripts parse; `git diff --check` passes. Public entrypoint validation caught and corrected duplicate executable resolution from multiple Cargo installations. `gui-dev.ps1 -BuildOnly` and `check.ps1 -Local` now refuse safely before Cargo launches with only about 6.18 GiB available. Refusal receipts: `%LOCALAPPDATA%/CleanSlate/BuildReceipts/20261026-071200-2ed64b6c0fff4510915ec7d327d9c3bb.json` and `20261026-071201-8af4380d94ac42dd8e237071071cc61d.json`.

Remaining before done: merger integration into both plugin branches and their product build/check scripts; representative sequential plugin/desktop guarded checks with source/artifact provenance; ticket 99 owns rejecting unguarded commands in the runner. No heavy build was launched or threshold relaxed, no unrelated application was terminated, no cache was deleted. Keep this ticket claimed until the remaining scoped evidence is recorded.

