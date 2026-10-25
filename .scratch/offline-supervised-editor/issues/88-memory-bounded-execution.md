# Memory-bounded execution guard

Status: open
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
- [ ] Small disposable-process tests cover competing workspaces, descendant accounting, low-memory refusal, budget crossing, timeout, crash and lock cleanup, and preserved output.
- [ ] Existing build/check entry points use the guard without nested-lock deadlock; direct unguarded heavy invocations are rejected by the overnight runner.
- [ ] Record peak memory/time, guard reason, source commit and artifact hash for a representative sequential plugin and desktop check.
- [ ] Explain that OS sampling is not a hard whole-machine RAM guarantee. Deep REA calls in a shared MCP server are skipped unattended unless that exact provider's resource ownership/limits can be enforced without killing unrelated server work.
- [ ] Fetch current platform/API documentation before implementation, and retain useful test/build failures without wiping caches.

## Results

Not started. Readiness describes specification only; all listed dependencies must be done before claiming.

