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

## Results

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
