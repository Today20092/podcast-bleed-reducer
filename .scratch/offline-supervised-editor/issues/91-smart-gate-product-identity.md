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

Claimed 2026-10-26 after 87/88 completion. Implement only the separate pass-through product identity in the Smart Gate lane, preserving probe source/state and installed artifacts. Current available memory remains below the default 14 GiB heavy-work requirement; code and locked metadata may progress, but build/format/state acceptance stays pending until guarded checks can actually run. Do not mark done on static checks alone or enable legacy paired DSP.

### Reviewable source preparation — 2026-10-26

Worktree/branch: `smart-gate-identity-91` / `codex/smart-gate-identity-91`, based on product integration `3bbda73010143f25a36de78a8bac56e5dac89bf3`. Selected canonical ticket/design/host-matrix/guard-policy records come from `bb356cb`; the shared guard was already identical. No whole-development merge or probe source edit was made. Source stays on the temporary feature branch until actual acceptance, rather than being promoted into the permanent product branch.

Prepared separate `plugins/smart-gate` package `cleanslate-smart-gate` 0.0.1, CLAP ID `org.cleanslate.smart-gate`, deterministic VST3 bytes `CleanSlateGate01`, namespaced version-1 state and a single `smart_gate_bypass` parameter. Missing, foreign, future or malformed state resets to this product's bypass. The process callback leaves input-filled buffers untouched; it contains no peer, model, trainer, Session or gain-policy path. The focused editor says **Pass-through prototype**, explains that gating is unavailable and exposes saved Bypass. UI/native behavior remains unverified; the accepted 90 design is visual authority, not a claim that 94 has shipped.

Unit contract sources were written before implementation for stable IDs/state serialization, foreign-state fallback and exact mono/stereo/bypass audio bits without host work. Two uniquely named actual-ABI host examples (`smart_gate_clap_check`, `smart_gate_vst3_check`) load Gate and preserved probe libraries, check independent factory IDs/state, and test Gate unity/latency/bypass without opening a DAW or native window. These tests have **not executed**. Existing `plugins/fl-probe` sources, IDs and state are unchanged (`git diff --exit-code codex/plugin-smart-gate -- plugins/fl-probe` passed).

`scripts/build-smart-gate.ps1` uses guarded locked/offline Cargo steps and only creates fresh `dist/smart-gate-candidates/<unique-id>` outputs after checks pass. Product test entry `test-smart-gate-hosts.ps1` reuses the maintained host runner's deadlines, PASS contract and isolated per-child application-data cache, with an explicit SmartGate case; original Probe defaults remain intact. Reserved future Common Files names are **CleanSlate Smart Gate.clap** and **CleanSlate Smart Gate.vst3**; no link creation, installation, signing or publication occurs. The package has no `rust-app`, ownership-model or probe dependency.

Current API evidence: Context7 resolved the NIH-plug predecessor and egui 0.37.0 docs; installed nice-plug 0.4.2 / nice-plug-egui 0.6.0 / vst3 0.3.0 sources determined actual fork/editor/stream signatures. The prototype UI uses the accepted brief and token contract; no new owner interview or visual-world invention was needed.

Guarded checks (all receipts under `%LOCALAPPDATA%/CleanSlate/BuildReceipts/`):

- `20261026-085415-f850df1fc4db4235a533306923271f7b.json`: default 14/8/6 GiB Rust test **refused before launch**, with 3,645,349,888 bytes available. This is neither semantic red/green nor a passing Rust test.
- `20261026-085416-355bdf7900e94ce489d9c3afb3404f99.json`: noncompiling offline lock generation passed, 7.21 s; peak owned private 90,144,768 bytes, working set 125,186,048, kernel job commit 113,266,688.
- `SmartGate-91-metadata-c5b180a70c1a4fd2a6e2e330b52a37d9.json`: full locked offline metadata passed, 273 packages including this root; no desktop/trainer/probe dependency, 4.22 s, kernel peak commit 69,111,808 bytes.
- `20261026-091431-5e816064a8d5420293312fd3c116c658.json`: noncompiling Cargo/rustfmt completed, 1.16 s, kernel peak commit 26,279,936 bytes.
- `SmartGate-91-final-metadata-ea86b8f013cf40fe910b0fb09c601101.json`: locked final target enumeration passed (Gate library and both unique host examples), 1.08 s, kernel peak commit 26,214,400 bytes.

Noncompiling jobs used explicit 2 GiB start / 1 GiB reserve / 0.5 GiB owned budgets with 90-second deadlines. A temporary machine headroom drop paused further jobs; work resumed only in the coordinated safe slot, without changing policy or retrying the unchanged heavy check. PowerShell parsing, probe-source preservation and diff checks passed. No Rust compiler/build/test, Clippy, actual host executable, GUI render or native acceptance ran; no existing artifacts/caches/installed links were changed. New unit/ABI code still requires real type-checking and runtime verification. Keep 91 **claimed**, and 92 blocked, until guarded compilation and format/state/bypass plus relevant probe regression checks pass.

### Candidate output parent review correction — 2026-10-26

The specification review found that existing `dist` or `dist/smart-gate-candidates` junctions could redirect candidate writes outside the Gate tree. The build entrypoint now rejects linked and non-directory output parents before helper/build setup, and rechecks those parents after Cargo steps before creating output directories. Inspection failures other than a genuinely missing path fail closed. No shared framework or probe source was changed.

Public test `scripts/test-smart-gate-output-paths.ps1` copies the actual entrypoint into a marked disposable workspace without local build helpers or a Cargo project. This ensures even a regression cannot compile real products. Its red run failed because the original script reached the missing helper instead of the intended link rejection: receipt `20261026-093235-5c0aea155c2e4b3d85fb40fcbd5b5bfe.json`, evidence `%TEMP%/cleanslate-gate-output-6638fa68-95f2-4987-ad38-3417ba13987a/`.

Green receipt `20261026-093317-33bcc7a80c054bb9a1d4b5d12e9e1e06.json` verifies all four cases: linked `dist`, linked candidate parent, regular-file `dist`, and regular-file candidate parent. Each is rejected before the helper boundary; each owned outside directory retains only its unchanged SHA-256 sentinel. Full evidence remains `%TEMP%/cleanslate-gate-output-400228b0-eea8-44ef-a611-d7e54ac37fa9/`, including `results.json` and owned junctions. No junction, target or unrelated files were deleted. Both runs used an explicit lightweight 2 GiB start / 1 GiB reserve / 0.5 GiB job / 60-second policy; no actual Gate build was wrapped in that budget. Focused PowerShell parsing, diff checks and exact probe-source preservation pass. Ticket 91 remains claimed and unqualified on its feature branch; no Rust heavy retry, GUI or installed-link action occurred.

