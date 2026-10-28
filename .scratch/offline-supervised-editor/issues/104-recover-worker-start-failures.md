# 104: Recover from application worker-start failures

Status: claimed
Assignee: Codex rust_ticket104
Readiness: ready-for-agent
Type: task
Blocked by: None (can start immediately)

## What to build

User actions that need a background worker report an actionable start failure and remain usable when the OS refuses thread creation. Preserve actual-worker exit accounting, project/source data, cancellation and safe output transactions across import, analysis, audition/playback, waveform/update work and final rendering.

## Specification

[Accepted Google Rust audit corrections](../../offline-supervised-migration/rust-guidelines-corrections.md).

## Acceptance criteria

- [ ] Every production launch handles an OS thread-start error without an uncaught panic, phantom active-worker registration or permanently busy operation.
- [ ] Deterministic failure injection at the existing launcher/job boundaries proves failed launch, retry and user-facing recovery without exhausting the machine.
- [ ] Normal completion, worker unwinding and cancellation/close retain exactly-once lifetime accounting and never permit publication or exit while an actual owned worker remains alive.
- [ ] Affected callers and focused tests are updated together; source validation, project edits and safe export behavior remain intact.
- [ ] Focused RED/GREEN evidence and relevant repository checks are recorded under the existing resource policy.

## Results

Claimed 2026-10-27 on codex/rust-audit-104 from integration checkpoint 5a321767b09cd42eb836c996979da5bea05b714c. Behavior changes use TDD at the approved launcher/job and worker-lifetime boundaries. Local compilation waits for parent-owned guarded execution; the plugin-only smaller budgets are not applicable.
