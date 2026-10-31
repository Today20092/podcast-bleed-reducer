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
- [x] Record remaining limitations and scoped evidence. Direct execution of all 74 Podcast tests passes with a finalized guarded receipt; a complete product build using the alternative linker remains pending.
- [ ] Prove named-job membership for the actual Cargo/compiler chain before another heavy run. Receipt statistics alone do not establish descendant containment.

## Results

### Actual compiler containment gap — 2026-10-30

Ticket 110's selected root-library budget test produced the expected semantic RED on source `a27daac`: `Bounds(0.25, 0.75)` instead of `TooDense`. Compilation took 7 minutes 48 seconds and the outer receipt `20261030-195338-aa9aeffff95149bfa6fb863f5634232f` finalized with root exit 101 and cleanup of its recorded job. However, it recorded only one active process and 36,892,672 bytes peak job commit while a read-only observation saw rustc using about 1,849 MiB private memory. Therefore its compiler owned-memory cap and cleanup are **not proven**. Global minimum physical/commit availability remained about 16.8/16.3 GiB, but that does not repair the missing ownership proof. All further heavy execution stopped; ticket 110's prepared GREEN source remains unrun.

Tiny self-membership probes now establish a real gap, beyond a sampling anomaly:

- Direct current-host PowerShell payload: `IsContained` is true for the exact inherited named job. Its executable is the same regular file as the receipt (`Program Files/WindowsApps/.../pwsh.exe`); the PATH aliases are zero-length reparse points, but were not used in this launch. Receipt `20261030-200353-dcef404a9f78482f8fab2ff2201f45ca`.
- Contained PowerShell root launching the same real PowerShell child directly: both report named membership true; the receipt records two active processes and 154,439,680 bytes peak job commit. Receipt `20261030-200427-304a0ccad14a45f3b2a49dcd633e0dff`.
- The same root through `rustup.exe run 1.99.0` to the same tiny child: root membership true, child membership false, despite inheriting the identical job name. Receipt `20261030-200505-0fdba295bd944199a3dadb61288d90d1`. Launching real rustup directly as the guarded root repeats it (`20261030-200659-3ef6c536b52c4f049789ae6a69b74124`): child is in some job, the named job opens successfully, but `IsProcessInJob` for that named job is false. This is not merely an inaccessible named-job handle.
- Bypassing rustup is not sufficient: guarded pinned actual Cargo with `RUSTC` set to a disposable `.cmd` version-probe stub launches that stub's actual PowerShell child outside the named job too. The stub exits 23 during Cargo's `-vV` check, before any Rust source compilation. Receipt `20261030-200804-3727cce7a15a4c9d95f32829ac662731`.

Each child only printed membership and slept three seconds; the probes used 2 GiB start / 1 GiB reserve / 0.5 GiB owned cap / 15 seconds. Probe children exited normally and the mutex is free. An unknown VCTIP PID 19516 was observed afterward, without Cargo/rustc; it was not terminated because ownership is not established. No further compiler or guessed resolver fix followed. The disposable scripts remain at `%TEMP%/cleanslate109-membership.ps1`, `%TEMP%/cleanslate109-rustc-probe.cmd` and `%TEMP%/cleanslate109-cargo/`.

[Microsoft's job documentation](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) says ordinary CreateProcess children inherit job membership absent permitted breakaway, and explicitly identifies brokered WMI launches as an exception. [Pinned Rust 1.99 process source](https://github.com/rust-lang/rust/blob/1.99.0/library/std/src/sys/process/windows.rs) calls CreateProcessW, while [rustup's command source](https://github.com/rust-lang/rustup/blob/main/src/command.rs) calls `cmd.status()`. Those sources do not establish the cause of this machine's observed escape. Do not label it intentional rustup breakaway or assume a generic guard fix. Context7 quota was already exhausted, so primary sources were consulted directly. The actual chain must be made fail-closed and requalified before heavy work resumes; checking only the wrapper's own membership is insufficient.

Final bounded identity/flags probe `20261030-201034-8e00397c85ca40eea54f2b632f1b42f7` confirms both processes receive exactly `Local\CleanSlate.OwnedTask.e380f9b70a154feb84a8c6015a9a0687`. The root reports named membership true, the child false; both can open that same job and query `LimitFlags = 0x2200` (kill on close plus job memory limit, neither breakaway permission). Thus the observation is not a mismatched inherited GUID or an accidentally enabled breakaway flag. The existing three-second child exited normally. No real compiler-chain build followed: no supported root-cause fix or proven complete containment exists at this checkpoint. Keep ticket 109 claimed and all heavy checks externally blocked pending platform/launch diagnosis; preserve ticket 110's unrun GREEN source.

Historical selected-target functional result: Gate lifecycle source `bdb94e9` was compiled with
the pinned Rust linker and passed all 18 library tests. Initial selected-target
compile receipt `20261030-192712-6402e179b21c41e49b1f02ea627c2f56.json`
recorded root/command exit 0, no remaining recorded job PIDs and cleanup of that
recorded job. Actual compiler descendant containment is not proven by this receipt.
GREEN compilation and full-suite receipts are
`20261030-193511-f5c8d7449ae0480da812e81e7e00b950.json` and
`20261030-193549-e7f29a8e46414a82964c846c29a95639.json`.
The executable SHA256 is
`03a78fb905f0d8092aa5904947d1427aee15c4c42a2a0ce9fa8fff4d011a5134`,
preserved in `%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/gate93/20261030-lifecycle/`.
This supersedes the earlier pending functional selected-target check below. It
records a successful scoped unit-test link/runtime workaround, not proof of the
compiler's owned-memory cap or complete cleanup. Ordinary MSVC builds, fresh native C
dependencies and native plugin packaging retain their helper-completion blocker.
No default linker, containment or completion policy changed. Ticket stays claimed.

The actual membership failures above require auditing prior Gate and Podcast
compiler receipts as observed job statistics, not proof that all compiler
descendants were contained or covered by the reported cap and cleanup. Their
source-level RED/GREEN and direct test-executable results remain valid functional
evidence. No new native, listening, resource-safety or release qualification follows.
All further heavy execution is blocked until the complete launch chain is proved.

Claimed 2026-10-30 from integration `6877e73`. Scripts are excluded from graph indexing, so direct source inspection is used for this graph coverage gap.

The ordinary pipe-launched completion probe and all existing disposable fixtures passed before any change. Historical stalled logs and artifacts were absent at their recorded paths. A PTY probe was refused by the execution provider before launch (Access denied). Those initial probes did not reproduce the stall; the later compiler/linker reproduction is recorded below.

Added one bounded public CLI regression: a root exits 17 while its real child sleeps and writes a completion marker. RED was missing `root_exit_code` receipt field; GREEN captures root exit17 with one actual job member (PID4780), waits for its marker, propagates17 and finishes with zero remaining PIDs and verified cleanup. Evidence `%TEMP%/cleanslate-guard-completion-ba8269ba-848b-4faf-aebe-9d8f148e3744/`; elapsed3.17s, peak job commit28,344,320 bytes. Initial fixture incorrectly required two simultaneous sampled processes; removed that assertion because short-lived root/child overlap can fall between samples, while the marker and persisted membership prove actual waiting.

The guard now persists one root-exit diagnostic containing the root code, active job count and actual job PIDs in a separate `.diagnostics.log`, avoiding concurrent writes against inherited child output handles. Receipts carry `diagnostic_path`, `root_exit_code` and final `remaining_process_ids`. Existing descendant completion, timeout, containment and limits are unchanged. `WaitForSingleObject` distinguishes an exited root from a running process even when its exit code is259.

Full existing disposable suite passes after the change: `%TEMP%/cleanslate-guard-72e8109e-1e23-48e8-b3a9-a8d2a74bb3ac/`. Budget fixture's intentional native allocation failure remains expected. `git diff --check` passes. The later product run and tiny tool probes establish the persistent helper; an alternative product link remains required before closure.

Standards review found that the fixture's fixed two-second child lifetime could expire before the guard sampled root exit on a busy machine. Replaced that timing assumption with a bounded handshake: the actual child waits up to six seconds for its unique public root-exit diagnostic before writing its completion marker; the guard retains its ten-second timeout. Targeted fixture passes in `%TEMP%/cleanslate-guard-completion-6a19326d-48a6-4d78-9736-4e20b2f22998/`; diff check passes. This test-only adjustment does not require repeating the unchanged full suite.

### Actual persistent helper reproduced — 2026-10-30

The Podcast targeted regression subsequently passed after its compile, but the root exited with code 0 while actual job member 4112 remained: MSVC 14.44.35207 Hostx64/x64 `VCTIP.EXE`. The coordinator cancelled only that owned job and verified process absence and global mutex release. A synthetic no-CRT source `void entry(void) {}` now reproduces the same completion failure without Cargo or application behavior:

- Guarded MSVC 14.44 `cl.exe /nologo /c entry.c /Foentry.obj`: compiler root exit code 0, retained VCTIP PID 25328; outer 15-second timeout, verified cleanup, peak job commit 43,720,704 bytes, elapsed 15.40 seconds. Receipt `%LOCALAPPDATA%/CleanSlate/BuildReceipts/20261030-192303-0ba1ecaf73b54a1784f54a2a1321702e.json`, with adjacent diagnostic and log.
- Guarded same-version `link.exe /nologo /nodefaultlib /entry:entry /subsystem:console /out:entry.exe entry.obj`: linker root exit code 0, retained job PID 23152; outer 15-second timeout, verified cleanup, peak job commit 38,985,728 bytes, elapsed 15.39 seconds. Receipt `20261030-192326-be2c1cc00f18460786b93ed893fc0ecb.json`. Tool output exists but guarded success is not claimed.

Both tiny existing-tool probes used the explicit fixture budget: 2 GiB start threshold, 1 GiB reserve, 0.5 GiB owned cap and 15-second timeout, with one compiler globally. Help-only `link.exe /?` completed promptly with its nonzero help exit; no telemetry switch was listed. Source, object and output remain under `%TEMP%/cleanslate-msvc109/`. No VCTIP or compiler remained afterward and the global mutex was acquired and released. No unrelated process was stopped, telemetry policy changed or completion requirement relaxed.

No documented invocation-local VCTIP prevention was found. Context7's official C++ docs describe [CL/_CL_ option transport](https://learn.microsoft.com/en-us/cpp/build/reference/cl-environment-variables), but not such a switch. Installed `VsDevCmd.bat` checks `VSCMD_SKIP_SENDTELEMETRY` only around its PowerShell developer-shell telemetry command; it is not evidence of compiler/linker VCTIP suppression. [Microsoft-hosted moderator guidance](https://learn.microsoft.com/en-us/answers/questions/6021264/supported-invocation-local-zero-child-and-temporar) likewise found no published invocation-local zero-helper contract for MSVC 14.50; that supports the research but does not establish a guarantee for 14.44. No guessed environment-variable loop followed.

Potential scoped alternative, not yet verified for a complete product link: the already-installed pinned Rust 1.99 ships `lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe`. [Rust's codegen docs](https://doc.rust-lang.org/rustc/codegen-options/index.html#linker) document invocation-local `-C linker=<path>` and stable `-C linker-flavor=lld-link`; the latter supplies LLD's Windows driver flavor. [LLVM Windows support](https://lld.llvm.org/windows_support.html) documents MSVC-compatible options, import libraries and PDBs, except `/DEBUG:FASTLINK`. Use a source-only final rustc override rather than changing all Cargo RUSTFLAGS and dependency fingerprints. Actual product artifact checks must pass before adoption; this does not prevent cl.exe-launched helpers in dependency rebuilds. Ticket remains claimed.

Tiny LLD integration proof now passes: the same MSVC-produced `entry.obj`, pinned Rust 1.99 `rust-lld.exe -flavor link` and the same no-CRT link arguments produced `entry-lld.exe`. The guard finalized as `completed`, with root exit code 0, verified cleanup and no remaining PIDs, in 2.02 seconds; peak job commit was 6,168,576 bytes. Receipt `20261030-192500-392de530bdfe4193962d7c588f3415d4.json`; output SHA256 `ec5d38936599f8cb08056e48275a8a94dcc49fbcbdac062c037f07f1493ed8c9`. The same explicit tiny fixture budget applied; no compiler or helper remains and mutex release was verified. This proves this tiny COFF input only, not the plugin's complete link or runtime.

The saved Podcast test executable was also run directly, without a linker or compiler: all 74 tests passed under the unchanged guard. Receipt `844f49a11ce642b69b72908ad31d801c` finalized in 4.299 seconds with root exit code 0, verified cleanup and no remaining PIDs. Tested source is `595d0aa`; executable SHA256 is `fab1a2c6127ec80fd47f5a3e1826abffd516ffa415488a84b700cb89dba35883`, preserved under `%LOCALAPPDATA%/CleanSlate/Podcast/Workflow103Checkpoints/20261030-seek-green-owned-helper/`. Podcast evidence commit `2afd150` records this result. This validates the full test runtime and seek fix; it does not establish that a new product compile/link can finish under the guard.

[Cargo's official rustc command](https://doc.rust-lang.org/cargo/commands/cargo-rustc.html) documents `--lib` selecting the single library, `--profile test` enabling unit-test mode and test cfg, and extra arguments after `--` applying only to the final compiler invocation. Therefore the next scoped candidate is `cargo rustc --lib --profile test <existing shared locked/offline arguments> -- -C linker=<pinned rust-lld path> -C linker-flavor=lld-link`, followed by running and preserving the actual test artifact. No extra rustc `--test`, RUSTFLAGS, dependency, installation or default-policy changes are needed. Context7 was consulted for MSVC and version-specific Rust; its quota then exhausted, so the Cargo fact was verified against the primary documentation directly. The alternative product compile/link remains pending coordinator execution; direct execution of the existing 74-test artifact has passed as recorded above.
