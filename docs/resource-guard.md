# Local Windows resource guard

## Owner-authorized bounded plugin profile — 2026-10-27

The owner explicitly asks to continue 93/103 and permits reducing the commit reserve. For these resumed plugin checks, use an outer guarded compiler job with `-MinAvailableGiB 10 -ReserveGiB 5 -MaxCommitGiB 4 -TimeoutSeconds 1200`, Cargo `-j1`, the pinned shared development profile and existing global mutex. This is a compiler budget, not a lightweight-test classification. The preceding stopped compiler peaked at 1.65 GiB owned commit; initial measurement for this resumption was 14.05 GiB free physical / 11.74 GiB commit headroom. Require at least 9 GiB commit headroom before starting this profile (4 GiB cap plus 5 GiB reserve); a low preflight is a stop, not a polling loop.

Keep this profile fixed for the attempt. A resource-limit/pressure failure ends compilation until a new resource state or deliberate scoped adjustment is reviewed; do not repeatedly lower limits to force a pass. Log the actual policy/peak/minima and cleanup in the existing receipt. Script defaults remain 14/8/6 GiB for other callers. Native hosts keep their existing separate bounded policy. Do not change paging, services, unrelated applications or reboot as part of this authorization.

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
