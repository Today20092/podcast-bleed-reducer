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

### 2026-10-27 bounded gain-policy checkpoint

The next test-first batch adds a public immutable-Event-to-gain module without
connecting it to the callback. [Policy/checks/limits](../../../docs/research/smart-gate-gain-policy-93.md)
record actual RED/GREEN evidence: fresh absence reaches a finite -6 dB floor,
expired evidence is exact unity, and confidence 0.14 opens a previously reduced
gate within five milliseconds after lowering the demonstrated 0.5 threshold
failure to 0.10. All eight library tests pass. The owner-authorized fixed warm
8/3/2.5 GiB, 600-second profile retains one compiler and native same-process
preflight. The existing 92 path dependency/lock adds no new package identity
outside its locked graph and changes no existing Gate package versions.
No worker, callback gain, inference preparation, actual-format/PDC/offline
processing, loader qualification or UI was implemented. Laughter/whisper
preservation remains unqualified; default bypass and explicit arming remain.
This supersedes the description of the gain test as unrun, not the remaining
acceptance checks. Ticket 93 stays claimed and 94 stays blocked.

### 2026-10-27 resumed state-safety checkpoint

After the coordinator verified the authorized reboot and granted the sole heavy
slot, public state tests were executed before their implementations. New Gate
instances now default to bypass. Every saved state (including schema 1 and the
current schema 2) restores to bypass; restore is never consent to experimental
attenuation. Explicit experimental arming is an atomic, nonpersistent control,
and reset clears it without allocation. Product and bypass parameter IDs remain
unchanged. The editor does not expose arming yet, and the process callback remains
exact pass-through. No detector/runtime dependency or DSP change is included.

Default-bypass RED: `20261027-100331-cf248b739ce84a759f4321b8097603ae`;
GREEN: `20261027-100541-b0ca389701f94c2a88c6fedc202c1ed8`.
State-restore RED: `20261027-100733-6c26ddbc35324b1189b4d65fc3eedb5e`;
GREEN (all five tests): `20261027-101040-b2a0f5ab356442ff99d1d77e9f71ae05`.
Arming/reset public API RED (missing methods):
`20261027-101146-642bb270c4f74b0583e42764c8419e9e`;
GREEN (all six library tests): `20261027-101159-5c74e1b9b9804a589e3e67bd42874f48`.
Guarded formatting passed: `20261027-101309-2b260b28176846199a98fa3ac4ec8e39`.
Receipts/logs are under `%LOCALAPPDATA%/CleanSlate/BuildReceipts/`.
Compilation used the unchanged default 14/8/6 GiB guard, locked/offline Cargo,
one job and the shared pinned profile. A premature GREEN start while the prior
guard still owned a lingering helper was refused before any process launched:
`20261027-100424-1a473574f7a1415a8466c0f7f72700c2` (heavy-task-busy).
The coordinator proved and stopped only the owned VCTIP helpers after compiler
exit. No reserve failure, resource-policy relaxation or reboot was used here.

Graph search again returned no Gate coverage; no index ran. Context7 rejected
the request at quota, so installed pinned nice-plug/core 0.4.2 state/lifecycle
sources supplied the API contract. The canonical primary packaging amendment
and runtime audit were read directly without merging desktop source.

Remaining: bounded worker/epoch queues, gain policy, actual-format state and
automation/PDC/offline/lifecycle checks, allocation/deadline evidence and isolated
runtime staging/loading. The zero-added-latency design is a prospective causal
engineering prototype, not qualified speech-onset preservation. Whisper/laughter
risk remains and default bypass/explicit arming are mandatory. Full host/Clippy
checks have not run for this state-only checkpoint; ticket 93 remains claimed.

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

