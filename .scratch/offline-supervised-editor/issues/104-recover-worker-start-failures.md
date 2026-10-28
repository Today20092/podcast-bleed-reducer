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

RED evidence: hosted run 40627066393, macos-arm64 job 122614435707, source e4f598e8d248ddef123b10d214fa25f6e3407814. The deterministic import start-failure regression reached the actual panic in the shared launcher: "Could not start background worker ... injected thread-start failure". Library result: 214 passed, 1 failed, 11 ignored. This is the intended failed behavior, not a compiler/formatting failure.

Prepared correction: transient launch returns the OS I/O error; a closure-owned registration guard rolls back failed starts and releases successful/unwinding work. All nine transient launch call sites now settle their existing error/state paths. Persistent playback uses the same fallible OS boundary without global transient registration, retaining its separate cancel/join owner so ordinary close is not blocked by an idle playback thread. Synthetic failure injection is thread-local and test-only. Added regression coverage for import retry, UI project/pending states, render/export retry, audition preparation/playback, and captured-work completion/unwinding. The original import regression drives the shared root correction; subsequent caller checks broaden acceptance coverage and are not claimed as independently observed RED cycles.

No local compiler, GUI, audio device, package publication or resource-limit reduction ran. Context7 was quota-limited; the Builder::spawn I/O-error contract was checked against official Rust documentation. GREEN verification remains pending on this source; do not close the ticket before it passes.

First GREEN candidate 6d54c35af4b1219d3f9582d2ecccc33b8e2c2fb7, hosted run 40628271310, stopped in rustfmt before compiling on all four platforms. The macos-arm64 log (job 122618347172) identifies only the new manual-link-feedback assertion layout; corrected without changing behavior. This is not a behavioral test failure or a passing GREEN. The follow-up also adds a process-isolated current-test-binary check for exact active-registration state after failed starts, completion and worker unwinding; ordinary parallel test workers cannot interfere with that fixture. All follow-up runtime verification remains pending.

Second GREEN candidate 3707ba07a93e16e549f73d5c6eabafe6765c877d, hosted run 40628677305, reached library-test compilation but macos-arm64 job 122619656989 reported E0509 in the new audition-preparation fixture: struct-update syntax attempted to move fields from a Drop type. Changed that fixture to a complete direct initializer, preserving the production Drop implementation and avoiding default-then-field-reassignment lint churn. Other new fixture initializations were checked against Drop implementations. No runtime GREEN is implied by fixing this compiler error; verification remains pending.
