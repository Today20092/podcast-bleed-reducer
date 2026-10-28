# Single-input gate DSP and actual-format checks

Status: claimed
Assignee: Codex (Gate realtime prototype)
Readiness: ready-for-agent
Type: task
Blocked by: 92
Related: 85, 86

## Scope and acceptance

Implement the detector/result contract accepted in 92 in the Gate workspace. Keep confidence estimation separate from gain policy. Single-input mono/stereo must apply one consistent envelope without peer dependence; document channel-selection/downmix semantics.

Acceptance:
- [ ] Bounded preparation/worker inference and gain application with no unbounded callback waits, filesystem work or allocations; explain scheduling and missed-deadline fallback.
- [ ] Confidence open/close hysteresis, attack/hold/release and finite attenuation, state-safe bypass and truthful readiness; choose/document conservative defaults from measured speech fixtures, not vendor parameter guesses.
- [ ] Preserve input on invalid/missing/stale evidence; no random gain jumps, NaNs or discontinuities on resets/state/rate changes.
- [ ] Report actual latency/tail consistently to each adapter and validate sample alignment/PDC in the lightweight hosts.
- [ ] Deterministic envelope tests cover silence, short speech, quiet starts/tails, chatter, rapid parameter changes and mono/stereo; actual CLAP/VST3 hosts exercise lifecycle/state/automation/offline processing.
- [ ] Record memory/deadline/audio-integrity evidence and remaining subjective listening needs. Do not advertise OBS/VST2 support until 97 verifies the integration path.
- [ ] Inspect the inference-linked artifact's actual native imports, stage verified runtime prerequisites/applicable notices, and validate CLAP/VST3 loading/inference from candidate paths without Cargo-cache, developer PATH or incidental host-module assistance. Embedded model bytes alone do not make the native runtime self-contained.

## Results

Latest source checkpoint is `bb9e1a4efcceec168e461813cf3373fd8d882ed2`, not yet integrated: eight library tests and formatting pass for state safety and a finite gain policy with .10 open/.01 close, .14 quiet-confidence opening, bounded ramps and exact original at evidence expiry. [Policy receipts/limits](../../../docs/research/smart-gate-gain-policy-93.md). Callback still passes through; worker/inference/actual-format/offline/packaging/UI acceptance remains unchecked. A single worker epoch-reset RED is prepared but unrun, backed up outside the repo; no worker implementation. Further compilation stopped on the shared 3 GiB commit reserve during 103; [resource recovery checkpoint](../../../docs/research/cleanslate-memory-recovery-2026-10-27.md) records the owned tool-server footprint and protected app-recovery plan.

2026-10-27 post-reboot partial implementation: tested state-safety checkpoint `c66f8ba5dfc77a1aa8d1232e27c2c5fb77d2884d` on the scoped branch makes default and restored states bypassed, adds nonpersistent explicit arming, and clears it on reset. Three recorded RED/GREEN slices end with six passing library tests and formatting. The callback is still unity; worker/gain, inference deployment, format/PDC/offline checks and native UI are not implemented or qualified. A single subsequent gain-policy RED is prepared but unrun; retain it separately from this green checkpoint. No permanent Gate integration or ticket closure.

All heavy work stopped after the shared-resource failure in 103: receipt `20261027-102017-498887ecffda405da9a8ede99fe30316.json` reports 7.74 GiB commit headroom below the unchanged 8 GiB running reserve, peak owned job commit 1.65 GiB and verified cleanup. No further compiler, retry or reboot. Continue only after a new safe resource state permits the default guard; do not count proposed gain behavior as tested.

2026-10-27 bounded deployment audit is complete; no implementation/source change. [Verified runtime findings](../../../docs/research/gate93-runtime-packaging-2026-10-27.md) establish static ORT 1.22.0 in the retained 92 build, with ordinary DirectML/MSVC imports and no onnxruntime.dll import. The model is already embedded. The preserved detector executable folder lacks DirectML.dll; hash preservation is not standalone deployment qualification. Current pass-through Gate is not linked to ORT and remains unaffected. The current Gate staging script omits the future native prerequisites and existing model/runtime/dependency notices. DirectML-specific redistribution terms and actual DAW DLL search behavior remain unverified; no dependency/link strategy was selected and no GPU-provider execution is claimed.

The latest default guard readiness attempt refused startup before launching any process: `20261027-093309-2983749bfe4e46f383b2142619d5ba65.json`, 3.59 GiB available physical / 1.30 GiB commit headroom, cleanup verified. No compiler, native host, deep engine/index, heavy retry, service restart or additional reboot ran. Retain `codex/smart-gate-realtime-93` at doc-only `d734bbd`; resume test-first state/arming, worker/DSP and isolated packaging checks only when safe resources permit. 93 stays claimed and 94 stays blocked.

Post-reboot preservation supersedes that earlier doc-only tip: the scoped branch is clean at `c66f8ba` and its [state checkpoint report](../../../docs/research/gate93-state-checkpoint-2026-10-27.md) records three RED/GREEN slices and six library tests. The next unrun gain test is outside the worktree at `C:/Users/User/.codex/project-recovery/gate93-unrun-gain-red-2026-10-27.patch`, SHA256 `3a7bb8ddaa7d39909f371e2d5a210c5728431d69cddb703df93da0ed647b1776`; patch application check passes, compilation/execution never occurred. No worker/gain production source was written. All acceptance remains unchecked and permanent Gate remains `c7513c8`.

Claimed 2026-10-27 after 92 acceptance, scoped branch
`codex/smart-gate-realtime-93` from permanent Gate `c7513c8` in the retained Gate
worktree. Read-only discovery found that current Gate state is schema 1 with
Bypass default false and the callback is still pass-through; both state migration
and actual-format assertions must change deliberately before introducing DSP.
The first test-first seam is the public `PluginState` restore/parameter boundary:
new instances and every v1/unknown/malformed state must remain bypassed/unarmed;
saved state must never implicitly enable experimental gating. Product IDs and
probe compatibility remain stable. No test or implementation source was written
in this checkpoint.

The 92 inference path allocates synchronously and must stay on a bounded worker,
with preallocated epoch/source-time queues and finite callback work. Stereo must
avoid opposite-phase cancellation and apply one envelope; exact channel policy,
worker-delivery budget, fixed host latency/PDC and tail need explicit engineering
tests before being selected. Invalid, missing, stale, overflowed or late evidence
must preserve original audio. Whisper/laughter response from 92 remains a strong
preservation risk, requiring default bypass and explicit experimental arming.

Graph search found no Smart Gate coverage in the indexed primary project; no
index was started while another product owned the heavy slot. Named Gate files
were read for the coverage gap. ORT runtime packaging/link mode has **not** been
inspected or established; do not assume DLL mode or standalone deployment.

Further source/test work is deferred after the coordinator's verified standard
guard denial at 13.8205 GiB available physical memory, below the unchanged 14 GiB
start threshold: receipt
`%LOCALAPPDATA%/CleanSlate/BuildReceipts/20261027-004212-5e39d1247dc343ac804ddb95ca04accd.json`.
No compiler, test, index, native host, resource re-poll, dependency change,
callback/DSP edit or installed artifact action was attempted for 93. This is a
claim/design checkpoint, not implemented or qualified gating; 94 remains blocked.

