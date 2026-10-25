# Sequential overnight queue and durable handoffs

Status: open
Readiness: ready-for-agent
Type: task
Blocked by: 88
Related: 85, 86

## Scope and acceptance

Implement a resumable coordinator for docs/plugin-overnight-plan.md and .scratch/plugin-products/overnight-queue.json. Preparing this runner does not schedule a task or start unattended product implementation; an owner start command selects the run. Use existing execution infrastructure where sufficient; do not spawn multiple heavy workers to increase throughput.

Acceptance:
- [ ] Validate ticket existence, readiness/dependencies, expected workspace/branch/commit and exclusive claim before execution. Product umbrella tickets 85/86 are not executable leaves.
- [ ] One agent ticket at a time by default, one global guarded heavy task across all worktrees, and one REA target; never overlap Rust/LLVM/linking with deep REA or a native host. No automatic full DAW launches in the default overnight queue.
- [ ] Durable per-ticket checkpoints record source commits, edits, logs, tests, artifacts, memory/time, outstanding risks and next action before/after expensive work. Restart resumes from verified state and does not repeat completed builds blindly.
- [ ] Missing dependencies, owner listening/signing, unavailable SDK/runtime or memory pressure skip to another independent ready ticket. Timeout/budget failures get at most one resource-recovery retry; do not loop or falsely mark done.
- [ ] Worktree setup is explicit; shared-core commits land through development and are integrated selectively. No concurrent edits to the canonical planning records, unattended broad merges, push, tag, release, destructive cleanup or installed-link retargeting.
- [ ] Dry-run and small disposable command fixtures prove queue order, exclusive ownership, dependency skipping, stale claims, restart, failed-test stop, memory refusal and last-good artifact preservation without building the real products.
- [ ] End with one reviewable handoff identifying finished/blocked/pending work. Completion by morning is not guaranteed. Keep interactive/native acceptance separate from mechanical passes.
- [ ] If the host agent cannot reliably resume across tool cancellation/context/process loss, record that limitation and use the supported app automation on explicit owner scheduling rather than inventing a background chat process.

## Results

Not started. Readiness describes specification only; all listed dependencies must be done before claiming.

