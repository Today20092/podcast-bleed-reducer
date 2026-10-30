# Restore bounded resource-guard completion

Status: claimed
Assignee: Codex guard implementer
Readiness: ready-for-agent
Type: task
Blocked by: none
Related: 88, 93, 103

## Scope and acceptance

Diagnose the two post-command guard stalls recorded in [the resumption receipt](../../../docs/research/implement-spec-resumption-2026-10-30.md). Preserve containment, actual descendant waiting, limits, cleanup and exit-code propagation. Reuse ticket 88's public CLI/receipt test seam.

- [ ] Capture root exit and actual job PID/count evidence in a bounded disposable reproduction.
- [x] Add a failing diagnostic regression before the diagnostic implementation; no root-cause fix without reproduction.
- [x] Pass the existing disposable resource-guard suite and the regression.
- [x] Record remaining limitations and scoped evidence; representative product check remains pending.

## Results

Claimed 2026-10-30 from integration `6877e73`. Scripts are excluded from graph indexing, so direct source inspection is used for this graph coverage gap.

The ordinary pipe-launched completion probe and all existing disposable fixtures passed before any change. Historical stalled logs/artifacts were absent at their recorded paths. A PTY probe was refused by the execution provider before launch (Access denied). No historical-stall reproduction or root cause is claimed.

Added one bounded public CLI regression: a root exits 17 while its real child sleeps and writes a completion marker. RED was missing `root_exit_code` receipt field; GREEN captures root exit17 with one actual job member (PID4780), waits for its marker, propagates17 and finishes with zero remaining PIDs and verified cleanup. Evidence `%TEMP%/cleanslate-guard-completion-ba8269ba-848b-4faf-aebe-9d8f148e3744/`; elapsed3.17s, peak job commit28,344,320 bytes. Initial fixture incorrectly required two simultaneous sampled processes; removed that assertion because short-lived root/child overlap can fall between samples, while the marker and persisted membership prove actual waiting.

The guard now persists one root-exit diagnostic containing the root code, active job count and actual job PIDs in a separate `.diagnostics.log`, avoiding concurrent writes against inherited child output handles. Receipts carry `diagnostic_path`, `root_exit_code` and final `remaining_process_ids`. Existing descendant completion, timeout, containment and limits are unchanged. `WaitForSingleObject` distinguishes an exited root from a running process even when its exit code is259.

Full existing disposable suite passes after the change: `%TEMP%/cleanslate-guard-72e8109e-1e23-48e8-b3a9-a8d2a74bb3ac/`. Budget fixture's intentional native allocation failure remains expected. `git diff --check` passes. Ticket remains claimed: a representative product run under these diagnostics must establish whether the historical stall reproduces before any semantic fix or closure.
