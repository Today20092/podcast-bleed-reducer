# Local Windows execution wrapper

## Memory limits removed — 2026-10-30

The owner upgraded to 40 GB RAM and explicitly requested removal of all memory
guards. [Ticket 111](../.scratch/offline-supervised-editor/issues/111-remove-memory-limits.md)
supersedes every older RAM/commit profile, including the historical 14/8/6,
10/5/4 and 8/3/2.5 GiB instructions. There is no free-memory launch threshold,
running reserve, process/job memory cap or wait for memory recovery. Do not add
an equivalent preflight outside the wrapper. Memory measurements are informational.

`scripts/gui-dev.ps1` and `scripts/check.ps1 -Local` still use
`scripts/resource-guard.ps1` for execution coordination. Hosted checks keep their
existing behavior. The pinned toolchain, locked/offline dependencies, shared
Cargo target, incremental compilation and Cargo `-j1` are unchanged.

## Execution behavior

The wrapper acquires `Global\CleanSlate.HeavyTask.v1`, refuses an already running
compiler/analyzer, and uses a 30-minute default timeout. It never terminates an
unrelated process or automatically retries a failed command.

The root process is created suspended, assigned to a Windows Job Object and
resumed. Its job has kill-on-close enabled, with no memory-limit flags. Failure to
assign the root fails before execution. A timeout or cancellation stops the
recorded owned job; normal completion waits for its recorded processes to exit.
The sequential plugin-host runner retains its 90-second outer timeout and each
host's own 30-second deadline and isolated child cache.

[Ticket 109](../.scratch/offline-supervised-editor/issues/109-resource-guard-completion.md)
records observed Cargo/tool descendants outside the named job on this machine.
Job statistics and cleanup describe recorded members; they do not prove that
every compiler descendant was contained or stopped. Removing memory caps does
not fix this cleanup/accounting limitation. Do not claim a memory guarantee.

```powershell
& ./scripts/resource-guard.ps1 -FilePath (Get-Process -Id $PID).Path `
    -ArgumentList @('-NoProfile', '-File', 'C:/path/to/check.ps1') `
    -WorkingDirectory 'C:/path/to/workspace' `
    -ReceiptPath 'C:/logs/check-001.json' -TimeoutSeconds 1800
```

Arguments are passed directly with Windows quoting. Choose a new receipt path;
existing evidence is never overwritten. Defaults are under
`%LOCALAPPDATA%/CleanSlate/BuildReceipts`. Receipts retain source/executable
identity, exit/status, timeout policy, native job flags, observed memory/time,
process accounting and cleanup results. Built artifacts need their own hashes.

A nested call verifies actual membership in the inherited named job before
reusing the outer lock/receipt. It must omit `ReceiptPath`, honors its working
directory and propagates failure. An environment string alone cannot bypass
the lock.

When the root exits before other recorded job members, an adjacent
`.diagnostics.log` records its exit, active count and actual job PIDs. The final
receipt includes `root_exit_code` and `remaining_process_ids`.

## Verification

Run `scripts/test-resource-guard.ps1` and
`scripts/test-resource-guard-completion.ps1`. They verify native job flags have
no memory limits, output/exit propagation, timeout, nested execution, lock
exclusion, controller-death cleanup and delayed-child completion. No allocation
stress or low-memory refusal fixture remains. Evidence is retained in fresh
temporary directories.

Earlier measurements remain in historical tickets/research; they are not active
execution requirements. Native listening, publication and private signing retain
their separate acceptance boundaries.
