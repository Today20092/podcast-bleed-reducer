# 104: Recover from application worker-start failures

Status: ready-for-agent
Assignee: unclaimed
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

Not started. Record source, scoped verification and limitations here before closure.
