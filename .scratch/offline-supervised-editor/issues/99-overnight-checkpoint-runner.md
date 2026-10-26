# Sequential overnight queue and durable handoffs

Status: done
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 88
Related: 85, 86

## Scope and acceptance

Implement a resumable coordinator for docs/plugin-overnight-plan.md and .scratch/plugin-products/overnight-queue.json. Preparing this runner does not schedule a task or start unattended product implementation; an owner start command selects the run. Use existing execution infrastructure where sufficient; do not spawn multiple heavy workers to increase throughput.

Acceptance:
- [x] Validate ticket existence, readiness/dependencies, expected workspace/branch/commit and exclusive claim before execution. Product umbrella tickets 85/86 are not executable leaves.
- [x] One agent ticket at a time by default, one global guarded heavy task across all worktrees, and one REA target; never overlap Rust/LLVM/linking with deep REA or a native host. No automatic full DAW launches in the default overnight queue.
- [x] Durable per-ticket checkpoints record source commits, edits, logs, tests, artifacts, memory/time, outstanding risks and next action before/after expensive work. Restart resumes from verified state and does not repeat completed builds blindly.
- [x] Missing dependencies, owner listening/signing, unavailable SDK/runtime or memory pressure skip to another independent ready ticket. Timeout/budget failures get at most one resource-recovery retry; do not loop or falsely mark done.
- [x] Worktree setup is explicit; shared-core commits land through development and are integrated selectively. No concurrent edits to the canonical planning records, unattended broad merges, push, tag, release, destructive cleanup or installed-link retargeting.
- [x] Dry-run and small disposable command fixtures prove queue order, exclusive ownership, dependency skipping, stale claims, restart, failed-test stop, memory refusal and last-good artifact preservation without building the real products.
- [x] End with one reviewable handoff identifying finished/blocked/pending work. Completion by morning is not guaranteed. Keep interactive/native acceptance separate from mechanical passes.
- [x] If the host agent cannot reliably resume across tool cancellation/context/process loss, record that limitation and use the supported app automation on explicit owner scheduling rather than inventing a background chat process.

## Results

Claimed 2026-10-26 after ticket 88 completed at development a4dc16c. The owner's implement-spec instruction authorizes this sequential implementation run; no automation schedule or unattended DAW launch is created. Disposable runner fixtures use explicit lightweight budgets; product builds retain the accepted resource policy.

### Scoped completion — 2026-10-26

Implemented [the explicit command coordinator](../../../scripts/overnight-runner.ps1), [usage/manifest contract](../../../docs/overnight-runner.md), fixed disposable helper and public script fixtures. The default dry-run is read-only; explicit start serializes coordinators and canonical claims globally, validates exact branch/commit and planning revision, and runs each reviewed command through 88. Builds retain 14/8/6 GiB policy; arbitrary shell strings, caller budget overrides, direct host executables, publishing build scripts, DAWs and uncontained shared REA are excluded. Maintained plugin-host checks retain their isolated child cache and PASS contract; desktop acceptance uses the actual `--package-self-check` entrypoint.

Atomic run/ticket checkpoints retain source/claim/dependency/plan hashes, tracked and untracked source fingerprints/edit paths, commands, input/output hashes and last-good backups, receipts/logs, timestamps, memory/time, risks and next action. Passed steps resume without repetition after source/artifact verification. Known failed or interrupted outcomes stop the whole run. Resource blocks can skip independent steps, with at most one later retry and no retry consumed before memory recovers. Command success leaves the canonical ticket claimed for actual acceptance review; it never fabricates a done state or a resumed background agent.

TDD began with the missing-runner dry-run failure. Successive public slices proved dry-run order/dependency/claim handling, guarded execution and non-repeating restart, tracked/untracked source-change refusal, failed-test stop and stopped restart, deterministic memory refusal with independent progress, retained retry until recovery, exactly two timeout attempts maximum, last-good artifact hash/backup preservation, wrong branch/commit, stale claims, forbidden lightweight Cargo/override flags, and machine-wide coordinator exclusion. Complete fixture evidence: `%TEMP%/cleanslate-runner-0bdeda27-6f8e-4c3e-9238-d912d81eb8ca/`; its per-run `handoff.md`/`run.json` and per-ticket JSON retain all results. Maximum single command 5.15 seconds; maximum sampled owned private bytes 36,225,024, working set 100,352,000 and native peak job commit 36,229,120. Fixtures preserve all logs/outputs and neither build real products nor claim real tickets.

One earlier suite attempt stopped correctly at `low-commit-running` under the fixed 1 GiB reserve; its evidence remains `%TEMP%/cleanslate-runner-142e9773-7c48-4147-8f48-feecf7c22dbf/`. A single bounded retry after observed recovery to 5.53 GiB physical/4.99 GiB free virtual passed; no budget was lowered. Later fixture changes added the required validation/exclusive-owner checks and passed. CI and local-build-settings fixtures, recursive PowerShell parsing and diff checks pass. Actual canonical queue dry-run reports done 88, claimed 99, dependency blocks and agent-action-required leaves without writing execution state.

Host chat cancellation/context/process loss cannot autonomously resume this agent. The supported recovery is another active session invoking the unchanged explicit plan after inspecting checkpoints. Any future app schedule requires a separate owner scheduling request. No unattended queue, release, vendor installation, installed-link publication, destructive cleanup, new product build or native/listening qualification was performed. Product implementation continues on the ordinary ticket frontier independently of whether a command manifest has been selected.

### Integrated verification — 2026-10-26

Fast-forwarded tested `534c4c956f2644eff33bb5e552f5604d93e5e312` into primary development from `8e612b5b90ef0f0b6413f74024449b6a57311d7c`. The focused disposable suite passed again with 5.48 GiB physical memory available at the initial observation and the unchanged fixed lightweight guard policy. Evidence: `%TEMP%/cleanslate-runner-339d0781-f159-47ba-8bac-106322389eb2/`. CI/local-build settings fixtures, recursive PowerShell parsing and `git diff --check 8e612b5...HEAD` pass. A brief integration review found no scoped acceptance blocker. This verifies the coordinator using disposable commands only; no real queue execution, schedule, build or product branch change was made.

