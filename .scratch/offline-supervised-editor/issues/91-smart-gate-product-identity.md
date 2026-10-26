# Separate Smart Gate package and state identity

Status: claimed
Assignee: Codex
Readiness: ready-for-agent
Type: task
Blocked by: 87, 88
Related: 85, 86

## Scope and acceptance

In codex/plugin-smart-gate, establish the separate CleanSlate Smart Gate package/artifact identity and focused editor entry point using the existing adapter patterns. Retain the probe source/state for reopening old projects. Do not enable the legacy paired ownership DSP under a new single-mic name.

Acceptance:
- [ ] Separate package/crate, product name, unique CLAP ID and VST3 class identity, state version and product-specific build/test/dist/link paths; deterministic IDs documented.
- [ ] Gate and probe coexist in the actual-format test hosts; probe IDs/state remain unchanged.
- [ ] Product output is pass-through until 93 is qualified, visibly marked as such; no Session group/capture prerequisite.
- [ ] Extract/reuse only demonstrated common host/editor helpers without copying the ownership trainer or moving unrelated desktop UI.
- [ ] Build/check with 88, preserve old artifacts, and verify locked dependency graph, format enumeration, state round trip and bypass.
- [ ] Package names/signing/version maturity follow docs/releasing.md; no release or installed-link change in this ticket.

## Results

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

