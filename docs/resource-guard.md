# Local Windows resource guard

## Owner-authorized bounded plugin profile — 2026-10-27

Latest scoped adjustment: the 10/5/4 GiB profile reached a meaningful RED, then stopped its GREEN compilation at 4.59 GiB commit headroom with 1.68 GiB owned peak, 8.93 GiB physical availability and verified cleanup (receipt `20261027-105059-5c1c52f2990e4219a2f278b360beed45.json`). Under the owner's explicit request to reduce the reserve, permit one **warm-cache targeted compiler/test** attempt with `-MinAvailableGiB 8 -ReserveGiB 3 -MaxCommitGiB 2.5 -TimeoutSeconds 600`, immediate same-process native preflight physical >=8 GiB / commit >=6 GiB. The smaller owned cap bounds the task while retaining a 3 GiB physical/commit floor. This is not a lightweight-test classification, fresh dependency build, full packaging budget or new default. Do not lower the 3 GiB reserve further; stop and preserve evidence if this bounded attempt fails for resources. Successful semantic test iterations may use the same fixed profile while their immediate preflights pass. Keep one Cargo job/global compiler tree and all containment/cleanup checks.

The owner explicitly asks to continue 93/103 and permits reducing the commit reserve. For these resumed plugin checks, use an outer guarded compiler job with `-MinAvailableGiB 10 -ReserveGiB 5 -MaxCommitGiB 4 -TimeoutSeconds 1200`, Cargo `-j1`, the pinned shared development profile and existing global mutex. This is a compiler budget, not a lightweight-test classification. The preceding stopped compiler peaked at 1.65 GiB owned commit; initial measurement for this resumption was 14.05 GiB free physical / 11.74 GiB commit headroom. Require at least 9 GiB commit headroom before starting this profile (4 GiB cap plus 5 GiB reserve); a low preflight is a stop, not a polling loop.

Keep this profile fixed for the attempt. A resource-limit/pressure failure ends compilation until a new resource state or deliberate scoped adjustment is reviewed; do not repeatedly lower limits to force a pass. Log the actual policy/peak/minima and cleanup in the existing receipt. Script defaults remain 14/8/6 GiB for other callers. Native hosts keep their existing separate bounded policy. Do not change paging, services, unrelated applications or reboot as part of this authorization.

Use `OwnedTask.Memory()` in the same PowerShell process immediately before invoking the guard, rather than a separate WMI/tool-call preflight. The first resumed launch was refused at 9.60 GiB physical / 5.57 GiB commit despite an earlier preflight of 12.84/9.68 GiB; no process launched (receipt `20261027-104400-65063aac05df4a0d8e23c0a16c8469fe.json`). A subsequent one-shot side-by-side diagnostic found native GlobalMemoryStatusEx, system GetPerformanceInfo and WMI agreeing at approximately 12.75/9.85 GiB. No stable counter-mapping defect was reproduced; resource state had changed. This measured recovery permits one fresh same-profile attempt, not a threshold-lowering or polling loop. Microsoft documents [process-available commit](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/ns-sysinfoapi-memorystatusex) separately from [system commit counters](https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-performance_information); keep that distinction when interpreting measurements.

`scripts/gui-dev.ps1` and `scripts/check.ps1 -Local` run Cargo through
`scripts/resource-guard.ps1`. Hosted `check.ps1` checks retain their existing
cross-platform behavior. Locked/offline dependencies, the shared Cargo target,
incremental compilation and one Cargo job remain in `local-build-settings.ps1`.

The independent Podcast and Smart Gate workspaces also guard every Cargo command
in `scripts/build-fl-probe.ps1`. Their `scripts/test-plugin-hosts.ps1` runner wraps
the existing sequential CLAP/VST3 hosts in an explicit lightweight 0.5 GiB job,
requiring 2 GiB available before start and retaining 1 GiB while running, with a
90-second outer timeout. Each native host still has its own 30-second deadline
and isolated child cache. These smaller budgets apply to existing test binaries;
plugin compilation keeps the default heavy policy. See [ticket 88's measured
integration checks](../.scratch/offline-supervised-editor/issues/88-memory-bounded-execution.md).

The guard acquires the machine-wide `Global\CleanSlate.HeavyTask.v1` mutex,
refuses existing compiler/analyzer work, and checks available physical memory
and system commit headroom. Defaults are 14 GiB available before launch,
8 GiB physical/commit reserve while running, 6 GiB committed memory for the
owned process tree, one-second sampling and a 30-minute timeout. It never
terminates an unrelated process or automatically retries a failed task.

The child is created suspended, assigned to a Windows Job Object, then resumed.
The job applies an aggregate committed-memory limit and kill-on-close; descendants
inherit containment. A containment failure terminates the still-suspended child
and fails the command. Runtime memory pressure, a reported job limit violation
or a timeout stops only that job. Normal completion waits for all descendants.
Final cleanup verifies that the job has no active processes. Parent process
death closes its job handle and terminates the owned descendants as well.

For a bounded task outside the normal build entry points:

```powershell
& ./scripts/resource-guard.ps1 -FilePath (Get-Command pwsh).Source `
    -ArgumentList @('-NoProfile', '-File', 'C:/path/to/check.ps1') `
    -WorkingDirectory 'C:/path/to/workspace' -ReceiptPath 'C:/logs/check-001.json'
```

Arguments are passed directly with Windows quoting, without a shell command
string. Use a new receipt path for every attempt. Otherwise receipts/logs go
under `%LOCALAPPDATA%/CleanSlate/BuildReceipts`. A receipt records source commit,
executable hash, policy, exit status, stop reason, elapsed time, sampled private
bytes/working set, kernel job peak commit, minimum available physical/commit
headroom and verified cleanup. Failed attempts preserve their log and existing
outputs. The executable hash identifies the launched command, not every artifact
it may produce; record built-artifact hashes in the ticket's build receipt.

A nested guard verifies actual membership in the inherited named job before
reusing its containment, lock and outer receipt. A forged environment string does
not bypass the lock. Nested work honors its working directory, propagates failure
and must omit `ReceiptPath`. Its budget is the outer task's budget. Do not wrap a
build in a lightweight test budget: the queue runner must validate the task's
required policy before launch.

Small disposable tests use explicit budgets of 0.125–0.5 GiB and physical/commit
reserves of 1 GiB. These are separate test budgets, not relaxed build defaults.
Run `./scripts/test-resource-guard.ps1`; it retains evidence in a new temporary
directory. The memory-limit fixture deliberately fails a native allocation under
a 128 MiB cap. Kernel `PeakJobMemoryUsed` can include a rejected allocation charge;
do not interpret it as sampled resident memory or as a whole-machine guarantee.

OS sampling cannot enforce a hard whole-machine RAM guarantee: unrelated programs
may consume memory between samples. The job's commit cap is separate from physical
working set. Notification delivery for hard job limits is best effort; the kernel
still denies excess allocations even when no notification arrives, and failure
or timeout remains authoritative. Deep REA in a shared MCP server is skipped
unattended unless that exact provider can be owned, limited and cancelled without
affecting unrelated server work. No native DAW or vendor installation is started.

Platform references consulted through Context7 and Microsoft Learn:

- [Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)
- [Extended job limits](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_extended_limit_information)
- [Completion-port notification guarantees](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_associate_completion_port)
