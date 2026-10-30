# Restore bounded resource-guard completion

Status: claimed
Assignee: Codex guard implementer
Readiness: ready-for-agent
Type: task
Blocked by: none
Related: 88, 93, 103

## Scope and acceptance

Diagnose the two post-command guard stalls recorded in [the resumption receipt](../../../docs/research/implement-spec-resumption-2026-10-30.md). Preserve containment, actual descendant waiting, limits, cleanup and exit-code propagation. Reuse ticket 88's public CLI/receipt test seam.

- [x] Capture root exit and actual job PID/count evidence in a bounded disposable reproduction.
- [x] Add a failing diagnostic regression before the diagnostic implementation; no root-cause fix without reproduction.
- [x] Pass the existing disposable resource-guard suite and the regression.
- [x] Record remaining limitations and scoped evidence; representative product check remains pending.

## Results

Claimed 2026-10-30 from integration `6877e73`. Scripts are excluded from graph indexing, so direct source inspection is used for this graph coverage gap.

The ordinary pipe-launched completion probe and all existing disposable fixtures passed before any change. Historical stalled logs/artifacts were absent at their recorded paths. A PTY probe was refused by the execution provider before launch (Access denied). No historical-stall reproduction or root cause is claimed.

Added one bounded public CLI regression: a root exits 17 while its real child sleeps and writes a completion marker. RED was missing `root_exit_code` receipt field; GREEN captures root exit17 with one actual job member (PID4780), waits for its marker, propagates17 and finishes with zero remaining PIDs and verified cleanup. Evidence `%TEMP%/cleanslate-guard-completion-ba8269ba-848b-4faf-aebe-9d8f148e3744/`; elapsed3.17s, peak job commit28,344,320 bytes. Initial fixture incorrectly required two simultaneous sampled processes; removed that assertion because short-lived root/child overlap can fall between samples, while the marker and persisted membership prove actual waiting.

The guard now persists one root-exit diagnostic containing the root code, active job count and actual job PIDs in a separate `.diagnostics.log`, avoiding concurrent writes against inherited child output handles. Receipts carry `diagnostic_path`, `root_exit_code` and final `remaining_process_ids`. Existing descendant completion, timeout, containment and limits are unchanged. `WaitForSingleObject` distinguishes an exited root from a running process even when its exit code is259.

Full existing disposable suite passes after the change: `%TEMP%/cleanslate-guard-72e8109e-1e23-48e8-b3a9-a8d2a74bb3ac/`. Budget fixture's intentional native allocation failure remains expected. `git diff --check` passes. Ticket remains claimed: a representative product run under these diagnostics must establish whether the historical stall reproduces before any semantic fix or closure.

Standards review found that the fixture's fixed two-second child lifetime could expire before the guard sampled root exit on a busy machine. Replaced that timing assumption with a bounded handshake: the actual child waits up to six seconds for its unique public root-exit diagnostic before writing its completion marker; the guard retains its ten-second timeout. Targeted fixture passes in `%TEMP%/cleanslate-guard-completion-6a19326d-48a6-4d78-9736-4e20b2f22998/`; diff check passes. This test-only adjustment does not require repeating the unchanged full suite. Historical-stall diagnosis remains open.

### Actual persistent helper reproduced — 2026-10-30

The Podcast targeted regression subsequently passed after its compile, but the root exited0 while actual job member4112 remained: MSVC14.44.35207 Hostx64/x64 `VCTIP.EXE`. The coordinator cancelled only that owned job and verified process absence and global mutex release. A synthetic no-CRT source `void entry(void) {}` now reproduces the same completion failure without Cargo or application behavior:

- Guarded14.44 `cl.exe /nologo /c entry.c /Foentry.obj`: compiler root0, actual retained VCTIP PID25328; outer15s timeout, cleanupverifiedtrue, peakjob43,720,704 bytes, elapsed15.40s. Receipt `%LOCALAPPDATA%/CleanSlate/BuildReceipts/20261030-192303-0ba1ecaf73b54a1784f54a2a1321702e.json`, with adjacent diagnostic/log.
- Guarded same-version `link.exe /nologo /nodefaultlib /entry:entry /subsystem:console /out:entry.exe entry.obj`: linker root0, retained jobPID23152; outer15s timeout, cleanupverifiedtrue, peakjob38,985,728 bytes, elapsed15.39s. Receipt `20261030-192326-be2c1cc00f18460786b93ed893fc0ecb.json`. Tool output exists but guarded success is not claimed.

Both tiny existing-tool probes used the explicit fixture budget2GiBstart/1GiBreserve/0.5GiBowned/15s, one compiler globally. Help-only `link.exe /?` completed promptly with its nonzero help exit; no telemetry switch was listed. Source/object/output remain under `%TEMP%/cleanslate-msvc109/`. No VCTIP/compiler remained afterward and the global mutex was acquired/released. No unrelated process was stopped, telemetry policy changed or completion requirement relaxed.

No documented invocation-local VCTIP prevention was found. Context7's official C++ docs describe [CL/_CL_ option transport](https://learn.microsoft.com/en-us/cpp/build/reference/cl-environment-variables), but not such a switch. Installed `VsDevCmd.bat` checks `VSCMD_SKIP_SENDTELEMETRY` only around its PowerShell developer-shell telemetry command; it is not evidence of compiler/linker VCTIP suppression. [Microsoft-hosted moderator guidance](https://learn.microsoft.com/en-us/answers/questions/6021264/supported-invocation-local-zero-child-and-temporar) likewise found no published invocation-local zero-helper contract for14.50; that is supporting research, not a14.44 guarantee. No guessed environment-variable loop followed.

Potential scoped alternative, still unverified: the already-installed pinned Rust1.99 ships `lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe`. [Rust's codegen docs](https://doc.rust-lang.org/rustc/codegen-options/index.html#linker) document invocation-local `-C linker=<path>` and stable `-C linker-flavor=lld-link`; the latter supplies LLD's Windows driver flavor. [LLVM Windows support](https://lld.llvm.org/windows_support.html) documents MSVC-compatible options/import libraries/PDBs, except `/DEBUG:FASTLINK`. Use a source-only final rustc override rather than changing all Cargo RUSTFLAGS/dependency fingerprints. Tiny-link and actual artifact checks must prove it before adoption; this does not prevent cl.exe-launched helpers in dependency rebuilds. Ticket remains claimed.
