# Separate Smart Gate package and state identity

Status: done
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 87, 88
Related: 85, 86

## Scope and acceptance

In codex/plugin-smart-gate, establish the separate CleanSlate Smart Gate package/artifact identity and focused editor entry point using the existing adapter patterns. Retain the probe source/state for reopening old projects. Do not enable the legacy paired ownership DSP under a new single-mic name.

Acceptance:
- [x] Separate package/crate, product name, unique CLAP ID and VST3 class identity, state version and product-specific build/test/dist/link paths; deterministic IDs documented.
- [x] Gate and probe coexist in the actual-format test hosts; probe IDs/state remain unchanged.
- [x] Product output is pass-through until 93 is qualified, visibly marked as such; no Session group/capture prerequisite.
- [x] Extract/reuse only demonstrated common host/editor helpers without copying the ownership trainer or moving unrelated desktop UI.
- [x] Build/check with 88, preserve old artifacts, and verify locked dependency graph, format enumeration, state round trip and bypass.
- [x] Package names/signing/version maturity follow docs/releasing.md; no release or installed-link change in this ticket.

## Results

### Post-reboot acceptance — 2026-10-26

Ticket 91 is **done** for the separate Windows pass-through identity. Earlier blocked/unrun statements below are historical. Owner-authorized ordinary reboot changed boot time to 2026-10-26T15:02:24.5000000Z; available RAM recovered to 18.53 GiB and commit headroom to 20.04 GiB. The temporary restart heartbeat resumed this thread and was deleted before compilation. No additional reboot or service termination occurred.

Tested clean source: `10180cb756c4f769c51e5e14df7b5a5516c27907`. Standard `scripts/build-smart-gate.ps1` passed formatting, all-target Rust tests (3 unit tests), strict all-target Clippy, library/examples build and actual CLAP/VST3 hosts. Heavy stages kept the default 14 GiB start / 8 GiB reserve / 6 GiB owned cap, one Cargo job and locked/offline shared cache/profile. Maximum kernel job commit across these stages was 933.57 MiB; cleanup was verified throughout. Test command compilation took 5m14s; its guard elapsed 506.35s includes a lingering MSVC VCTIP helper. Two VCTIP processes were each independently confirmed inside the named CleanSlate Job Object after compiler exit, then ended by exact process identity. No unrelated process, compiler or service was stopped; no guard policy was weakened.

Both actual hosts passed independent Gate/probe factory IDs and state round trips; Gate exact unity, bypass, mono/stereo, 44.1/48/96 kHz and zero latency. CLAP covered in-place/separate buffers; VST3 covered realtime/offline. Hosts took 0.42 / 0.37s, sampled process peak working sets 16.00 / 17.04 MiB. Preserved-probe regression hosts separately passed 1,858 CLAP / 4,048 VST3 bit-exact blocks, legacy state/control and shared-window checks. Exact probe source comparison against the product baseline passed.

Preserved candidate: `C:/Users/User/.codex/worktrees/smart-gate-identity-91/podcast-automix-rust-gui/dist/smart-gate-candidates/ec71fbd055914e61ae74ef1dd9cba37f/`; CLAP and VST3 SHA256 `3f609ef3a211df4c317f9827e7a610c0310edd9dc4c3c627051fcd703208e8e2`. Its build-receipt.json records source/paths/hashes and installed=false. Existing installed probe outputs/links remain unchanged.

Receipts under `%LOCALAPPDATA%/CleanSlate/BuildReceipts/`: fmt `20261026-111221-a1c468c45d0f45eb82483152bb5ca107.json`; tests `20261026-111226-75a6e8a28268423da97f8b559b6ec2ee.json`; Clippy `20261026-112052-0e663f7034544801b59ab380255033f9.json`; build `20261026-112057-ecdde45c90f543b3933e70bc89524579.json`; Gate hosts `20261026-112143-c83a1480283c456e93d2ac1f4344b370.json`; probe regression `20261026-112226-d330bd8b3bd445bab3880bdeb2b968e9.json`.

This unblocks detector feasibility 92, not detector/DSP/UI completion. Native GUI, DAW/listening, non-Windows platforms, installation, signing and release acceptance remain outside this ticket.

### Bounded compiler measurement follow-up — 2026-10-26

After the owner's request to investigate further progress, the configurable lower-cost-task provision in 88 was used for a narrow `cargo check --lib` measurement. The standard heavy policy was not changed. This explicit compiler budget used one Cargo job, the existing locked/offline shared cache/profile, 4 GiB physical start threshold, 2 GiB physical/commit reserve, 1.5 GiB hard owned-tree commit cap and 600-second timeout. It was described before execution; it was not labeled a noncompiling or lightweight metadata check.

The first attempt made real dependency-checking progress, then stopped with `low-commit-running` after 35.35 seconds. Peak sampled private bytes were 731,590,656, working set 668,438,528 and kernel peak job commit 727,347,200. Minimum observed physical availability was 2,311,557,120 bytes; commit availability 1,762,484,224. Receipt: `%LOCALAPPDATA%/CleanSlate/BuildReceipts/gate91-bounded-typecheck-aa055f5f0da84bfb84e74177fa7f69cf.json`.

After resource recovery above that budget's start threshold, the single allowed retry reused completed dependency checks and progressed through Windows, egui and into the nice-plug dependencies. It also stopped with `low-commit-running`, after 45.48 seconds. Peak sampled private bytes were 912,064,512, working set 826,511,360 and kernel peak job commit 963,411,968. Minimum observed physical availability was 2,756,050,944 bytes; commit availability 2,075,189,248. Receipt: `%LOCALAPPDATA%/CleanSlate/BuildReceipts/gate91-bounded-typecheck-retry-43fddd656d824f818b26755b5c09b73a.json`. Cleanup was verified for both attempts. The hard commit cap was not reached; the system allocation reserve stopped the work. No further identical retry is permitted without a material resource/state change.

The dependency versions match the preserved probe lockfile; no dependency or compiler-profile change was made. Context7 quota was exhausted, so current official Cargo documentation was consulted: <https://doc.rust-lang.org/cargo/commands/cargo-check.html>. A read-only host snapshot identified one shared Windows service host holding about 10.5 GiB private committed memory / 8.09 GiB resident; no individual hosted service was attributed and no service was stopped. A complete Gate type-check, Rust tests, linking and format/GUI acceptance remain outstanding; status stays claimed.

Claimed 2026-10-26 after 87/88 completion. Implement only the separate pass-through product identity in the Smart Gate lane, preserving probe source/state and installed artifacts. Current available memory remains below the default 14 GiB heavy-work requirement; code and locked metadata may progress, but build/format/state acceptance stays pending until guarded checks can actually run. Do not mark done on static checks alone or enable legacy paired DSP.

### Isolated source checkpoint and output-parent correction — 2026-10-26

Retained candidate: `codex/smart-gate-identity-91` at `10180cb756c4f769c51e5e14df7b5a5516c27907`, comprising source preparation `320745a28f0cf1b9d2ea64880e64d1fd240cba56` and its focused output-parent fix, based on permanent Gate revision `3bbda73`. Workspace: `C:/Users/User/.codex/worktrees/smart-gate-identity-91/podcast-automix-rust-gui`. The candidate's `plugins/smart-gate/README.md` documents the product boundary. Source remains off Desktop and the permanent Gate branch until qualified.

The independent `cleanslate-smart-gate` 0.0.1 package defines CLAP `org.cleanslate.smart-gate`, VST3 raw bytes `CleanSlateGate01`, namespaced version-1 state, saved Bypass and a visibly marked pass-through editor. The callback leaves input unchanged; there is no legacy ownership/group/capture/trainer path or root GUI dependency. Unit and actual CLAP/VST3 coexistence/state/bypass test sources are prepared, with uniquely named host examples. Existing probe source, IDs and state remain unchanged.

Full locked offline metadata passed with 273 packages and no desktop/trainer/probe dependency; target enumeration, formatting, PowerShell parsing, exact probe-directory preservation and diff checks also passed. The default 14/8/6 GiB compilation/test preflight refused before launching a child. Rust compilation, Clippy, unit/event tests, actual ABI/format/state/bypass checks, executable GUI and native acceptance remain **unrun**; static review is not a substitute for these checks.

Specification review found that linked output parents could redirect candidate writes outside the Gate output tree. Fix `10180cb` rejects reparse points and non-directory parents at the workspace/`dist`/candidate-parent boundary before helper/build setup, and rechecks before creating outputs. Read-only re-review confirms that the original P2 is resolved. The public entrypoint fixture covers linked `dist`, linked candidate parent, regular-file `dist` and regular-file candidate parent, proving refusal before helpers/builds and unchanged outside sentinel hashes.

Retained receipts under `%LOCALAPPDATA%/CleanSlate/BuildReceipts/`: red `20261026-093235-5c0aea155c2e4b3d85fb40fcbd5b5bfe.json`; green `20261026-093317-33bcc7a80c054bb9a1d4b5d12e9e1e06.json`. The green lightweight fixture completed in 1.13 seconds with 37.7 MiB kernel peak owned commit; all four cases passed without a Rust build. Evidence: `%TEMP%/cleanslate-gate-output-400228b0-eea8-44ef-a611-d7e54ac37fa9/`. No candidate binary, installed link or product-source integration is claimed.

All acceptance boxes remain unchecked; ticket 91 stays **claimed** and ticket 92 remains blocked by it. Next: once the normal heavy-work memory requirement is met, run the standard guarded Gate build and actual format/state/bypass/coexistence checks before promoting the source checkpoint.

