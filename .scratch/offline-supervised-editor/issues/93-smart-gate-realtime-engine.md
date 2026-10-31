# Single-input gate DSP and actual-format checks

Status: claimed
Assignee: Codex (Gate realtime prototype; resumed arming/readiness slice)
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

### Arming/readiness continuation — 2026-10-30

Claimed on `Today20092/gate93-resume` from preserved `d03d3ab`. The first
regression uses the existing public plugin lifecycle and temporary engineering
arming interface: an inactive/deactivated instance cannot arm; a prepared
instance requires explicit consent, and deactivation clears it. This checks
readiness before native editor wiring without constructing a fake native frame.
The existing editor cannot expose consent and still advertises unconditional
pass-through; that wiring remains in this slice, separate from ticket 94 polish.
Test preparation only: no Cargo/native/indexer launch, semantic RED or production
change is claimed while the coordinator owns the execution gate.

After execution grant, shared guard diagnostics were selectively integrated as
`132bc6d` / `bfc1d46`. The default 14 GiB start / 8 GiB reserve / 6 GiB owned cap /
1800-second compiler preflight passed at 19.30 GiB physical / 18.20 GiB commit.
Guard receipt `20261030-192632-16f3ea728de744b785ce25f09c3342f6.json` then refused
before launch because unrelated Android Cargo/rustc processes were active
(Cargo 19340, rustc 19204; a different repository/toolchain). Process ID is null,
peak owned commit is zero and cleanup verified. No compiler/test ran, no semantic
RED is claimed, and no unrelated process was stopped.

The planned invocation uses pinned Rust 1.99.0's existing `rust-lld.exe` only for
the final selected unit-test target via `cargo rustc --lib --profile test` and
the ordinary locked/offline/shared settings. Cargo documents final-target-only
arguments at <https://doc.rust-lang.org/cargo/commands/cargo-rustc.html>.
`shortcut:` this scoped command avoids the MSVC helper completion blocker under
109; replace it with the ordinary product check after verified bounded MSVC
helper completion. No persistent linker setting, toolchain or dependency changes.

The unrelated compiler finished; one fresh same-policy preflight passed at
20.21 GiB physical / 19.55 GiB commit. The final-target-only lld compile completed
normally in 431.64 seconds, receipt
`20261030-192712-6402e179b21c41e49b1f02ea627c2f56.json`: root/command exit 0,
empty job PIDs and cleanup verified; peak owned job commit 989,896,704 bytes,
minimum physical 15,199,465,472 / commit 12,336,652,288 bytes. This is actual
Gate test-target evidence, not a default MSVC/native package fix.

The prepared lifecycle test produced semantic RED (unprepared instance armed)
in `20261030-193445-ecc11a58fcf544948fa0ebb5a96f4bbe.json`; nonzero exit was
propagated with empty remaining job PIDs and cleanup verified. The shared
`arm_experimental` now accepts consent only while its engine is prepared/active.
The earlier temporary-consent/reset test now activates first, preserving its
intended reset/persistence checks. No processing/UI/status behavior was changed.

GREEN compilation `20261030-193511-f5c8d7449ae0480da812e81e7e00b950.json`
completed in 16.18 compiler seconds. Targeted GREEN
`20261030-193537-857b0cab6c04420091cc71e373a9ac29.json` and all 18 tests
`20261030-193549-e7f29a8e46414a82964c846c29a95639.json` pass; the full suite
took 4.95 test seconds, peak job commit 66,461,696 bytes, root/command exit 0,
empty remaining job PIDs and cleanup verified. Inference tests and compilation
used the unchanged default guard, with same-process physical/commit >=14 GiB
preflights. Formatting `20261030-193605-8926ba7cb5ab443abc4ab920bc9b4188.json`
and `git diff --check` pass. New strict Clippy and native hosts were not run.

The tested executable and receipts are retained under
`%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/gate93/20261030-lifecycle/`.
Behavioral source is committed at `bdb94e9`; its final documentation comment was
clarified after compilation without changing behavior. Preserved test executable
SHA256: `03a78fb905f0d8092aa5904947d1427aee15c4c42a2a0ce9fa8fff4d011a5134`.
Native editor arming/readiness, fault/stale processing truth, runtime deployment,
deadlines/PDC/listening and the rest of ticket 93 remain open. Ticket 94 stays
blocked. The global compiler slot is released after this bounded slice.

Next public-editor regression is prepared, unrun: opening the prepared editor is
not consent; its explicit Arm experimental gate click must set temporary consent
without clearing saved Bypass. The actual existing UI body is exposed through
`Panel::show(&mut egui::Ui)` with the native trait callback delegating unchanged,
so no native Frame fabrication or renderer dependency is introduced. No arming
button/status implementation or semantic RED is claimed before execution.

The public editor consent slice subsequently reached semantic RED for its missing
Arm control (`20261030-194214-788878c40ab2439d82540b397de5df42.json`), then GREEN
through actual pointer input while preserving Bypass. A public press/reset/release
regression exposed stale rearming (`20261030-194546-be0f340056114915b263e912b4f60a4b.json`).
One atomic consent token now carries its lifecycle generation; egui scopes the
action's identity by that generation and compares the complete observed token.
Reset and deactivate/reactivate retire old gestures, while a fresh click can arm.
Tab/Enter toggles consent through the standard focused widget; no mouse-only
handler, custom renderer or fake native Frame is used. A further RED showed that
bypass hid unavailable preparation (`20261030-194922-b8079c5a1b1b451592934e9d471df793.json`);
the status now reports unavailable preparation before bypass. Consent is never
presented as proof of actual processing; worker fault/stale status is still open.

Final selected-target lld compile `20261030-195006-897be551a91f4806ae3cbe32097daa7d.json`
completed in 12.39 compiler seconds, peak owned job commit 893,108,224 bytes.
All 20 tests pass in 4.90 test seconds
(`20261030-195027-40a07822903d4e5c9a5fae4c83aaa7cd.json`): peak owned job commit
67,395,584 bytes, minimum available physical 22,133,714,944 / commit 21,888,704,512
bytes, root/command exit 0, empty remaining job PIDs and verified cleanup.
Formatting `20261030-195040-a8035e685dbb44beb50e008102fc7fd9.json` and diff checks
pass. Every compiler/inference launch retained the default guard and same-process
>=14/14 GiB preflight. Parent source review found no actionable issue. New strict
Clippy, native editor/hosts, runtime packaging and subjective acceptance were not
run. No product integration, installed-link change or ticket 94 closure follows.

Initial headless test cleanup was corrected to clear unhandled egui texture deltas
before assertions, retaining the aborted fixture evidence without treating it as
a clean RED. Keyboard checks use actual Tab focus before Enter; pointer clicks
alone do not imply keyboard focus. Pinned egui source controls these API details
after Context7 quota and docs.rs access failed. Impeccable context/craft floor and
the existing design contract guided this functional change, without a redesign.

The final test executable and all slice receipts/logs are preserved under
`%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/gate93/20261030-editor-consent/`, SHA256
`6aed9cfcea707f193fefd0865f65cc5b15e116e1a3b859d78da0a6aadedd1194`.
After compilation, only the stale CLAP description literal and product/tracker
documentation were corrected; the tested executable retains the earlier metadata.
The next inference-linked native build must qualify its actual metadata and
runtime packaging; the retained native candidate has not been replaced. Ticket remains
claimed; compiler slot released to Podcast before any additional behavior.

Next processing-truth regression is prepared, unrun: after an explicitly armed
callback encounters invalid PCM, the actual editor must report Processing
unavailable / original audio while temporary consent remains distinct and input
bits are preserved. This uses the existing plugin callback and same public editor
body, without writing status internals or synthesizing detector output. No new
status implementation or semantic RED is claimed before execution-slot grant.

Actual-widget visual QA is also prepared, unrun: reuse the existing test-only
egui CPU mesh capture helper for three 640×440 / 1× states (unprepared, bypassed,
armed consent). Native build and capture share the unchanged extracted styling.
The opt-in `CLEANSLATE_PLUGIN_CAPTURES` directory receives PPMs from actual Panel
output, not a recreated diagram. No renderer/dependency or DPI/native claim is
added; the callback-fault regression remains the next execution priority.

### Containment audit and unrun continuation — 2026-10-30

Execution is stopped at the shared ticket 109 containment blocker. All Gate
compiler commands above launched `C:/Users/User/.cargo/bin/cargo.exe`, currently
a symbolic link to `rustup.exe`. No explicit `RUSTC` override was supplied; the
compiler path selected by the historical proxy invocation was not recorded.
Only the final linker was explicitly resolved to the actual pinned 1.99.0
`rust-lld.exe`. Subsequent tiny shared-guard probes found that a rustup launch
chain can escape the intended named job; no such mechanism is inferred solely
from Gate's receipts.

The first Gate compile receipt observed five active job processes and peak job
commit 989,896,704 bytes; the final editor compile observed five and 893,108,224
bytes. Root exit 0, empty final job PIDs and cleanup verification describe the
observed job, not proof that every compiler descendant belonged to it. Historical
per-compiler PID membership was not captured. The earlier compiler/accounting
figures therefore remain observations, not complete containment qualification.
The directly launched test executables did not use rustup; their 18/20 passing
behavioral results and actual job cleanup evidence are unchanged.

Verified consent behavior remains source `146b057` and the separately preserved
20-test executable/hash above (with the documented later metadata-only change).
The following continuation drafts are explicitly **uncompiled and unrun**:
the callback-fault/editor fallback RED; test-only reuse of the existing CPU mesh
capture helper; unchanged extraction of native style setup; and actual 640×440 / 1×
unprepared/bypassed/armed capture fixtures. No callback status implementation,
new capture output, native host/editor qualification, strict Clippy or further
compiler job followed the stop. Keep this incomplete worktree and all preserved
artifacts; 93 remains claimed and 94 remains blocked. Resume only after the shared
containment path is repaired and proved, followed by a new execution-slot grant.

### Owner memory override — 2026-10-30

The owner reports 40 GB RAM and explicitly removes memory guards under accepted
ticket 111 (`df9d707`). This supersedes every older RAM/commit preflight, reserve,
cap and memory-recovery wait above; historical receipts retain their original
measurements. Active Gate instructions/README now state the override. Gate's
build and host wrapper contain no explicit memory arguments or preflight refusal.
The common product `scripts/test-plugin-hosts.ps1` still has the three memory
arguments in this checkpoint and is reserved for the coordinator's shared caller
patch, alongside the revised guard API. Timeouts, global compiler serialization,
cancellation and owned cleanup remain. Shared files/runner and prior UI drafts
were not changed by this policy documentation patch. No build/test/native/capture
execution occurred. Compiler-membership investigation 109 is a separate remaining
blocker; removing memory caps does not establish containment or close 93/94.

Memory-removal integration is now present: core guard `791a5ed` (focused
`ce046a3`), shared host caller `bc95a5a`, and active docs/queue `b13cd31`
(focused `259db7c`). The host caller preserves Gate's `ProbeBinary` arguments and
90-second timeout while dropping the three memory arguments. Runner patch
`2c91266` was skipped because coordinator scripts are intentionally absent from
this product branch; it did not reintroduce them. Coordination remains centralized
in the desktop checkout. The resource-policy documentation conflict retained the
accepted memory-removal text and its separate containment caveat.

Static checks: all 27 product PowerShell scripts parse; no obsolete memory CLI
arguments or memory admission/pressure checks remain in active `scripts/*.ps1`;
`git diff --check` passes. All four shared guard/helper/fixture files have exactly
the coordinator root's SHA256 hashes, so its passing common tests apply to these
identical files without a redundant suite run. No product compiler, inference,
native host, UI capture or new test executed; the prepared UI/callback drafts and
their unverified status are unchanged. Ticket 93 remains claimed, 94 blocked.

### Resumed all-target check — 2026-10-30

Fresh preflight recovered to 26.26 GiB physical / 28.69 GiB commit with no
compiler processes. On unchanged source `0e8de06`, the guarded all-target test
log reports 17 library tests passed and both example test targets passed (zero
tests). The guard did not finalize after test completion; scoped Ctrl-C ended
its session. No finalized success/cleanup receipt exists. Guard PID 17488 and
its observed conhost 18364 were gone afterwards, no compiler remained, and the
global heavy-task mutex was successfully acquired/released. The next build
attempt had safely refused that still-held mutex; no native candidate was built.
See [check evidence](../../../docs/research/gate93-check-2026-10-30.md).
Status remains claimed; native arming, runtime packaging, host/PDC/deadline and
listening acceptance remain open, and 94 remains blocked.

### Resource preflight — 2026-10-30

Resumption at clean `af4d921` stopped before heavy work: same-process native
memory reported 4.77 GiB available physical / 3.66 GiB commit, below the fixed
warm 8/6 and native 10/9 GiB preflight requirements. No compiler, indexer,
inference, host, retry or threshold change ran. Existing tested source remains
preserved; 93 stays claimed and 94 blocked. [Checkpoint](../../../docs/research/gate93-resume-2026-10-30.md).

### Callback checkpoint — 2026-10-28

The offline mono/stereo callback regression reached real semantic RED (gain
remained 1), then GREEN after activation/preparation and shared envelope
integration. Explicitly armed absence reaches the finite -6 dB floor; one gain
is applied to both channels. Unarmed/bypassed/reset/faulted input stays original,
including invalid sample payload bits. Processing uses fixed 256-sample chunks
and never applies future evidence early. Offline renders match exactly for
127/1536/32768 host blocks. No delay is added to audio; this does not establish
speech-onset preservation or zero detector latency. Activation/preparation and
joining teardown stay outside process/reset; no new dependency.

All 17 library tests pass in `20261028-182205-dba42bf392524f4eafb4aa44cc386e83`;
formatting and strict library/test Clippy pass
(`20261028-182304-6e43b8120a4947c9aa8aaba04dd7a279`). Cleanup is verified.
The tested executable is preserved at
`%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/gate93/20261028-182205/gate-library-tests.exe`,
SHA256 `f278b3b1c43ef26f8d3b7e4c80c8245e7a8be050986073340f1b9777fce11e68`.
It is not a standalone runtime package. Worker checkpoint `64c5525` precedes this
callback slice. Native compilation preflight stopped at 9.76 GiB physical /
7.82 GiB commit, below the separate 10/9 GiB requirements; no native build/host
started. Arming is currently Rust API/test-only; no editor/native arming control,
new candidate, installed link, PDC/deadline or listening qualification is claimed.
93 remains claimed and 94 blocked.

### Worker checkpoint — 2026-10-28

Owner-authorized memory cleanup stopped seven verified unused REA server
processes. Available physical/commit headroom rose 7.77/6.53 -> 8.67/8.55 GiB;
no application/service/cache change or threshold reduction. The pending worker
test reached a missing-module RED, then GREEN; a separate stereo RED preceded
independent-channel/max-confidence implementation. Fixed-capacity atomic SPSC
queues use no unsafe shared storage, mutexes or CAS retry loops. Generation
invalidation rejects queued/in-flight results across repeated/reused public
epochs; input/result overflow and nonfinite input latch a fault until reset.
Model preparation, fallible worker startup and joining teardown stay off callback.
Offline synchronization is bounded to 500 ms and unavailable in realtime mode.

All 15 library tests pass in receipt
`20261028-181617-9484ac1a3da643a4be4279e39a556426`; formatting and strict
library/test Clippy pass (`20261028-181729-24069531cf5f470cb782677fe33e3fa0`).
Cleanup is verified. No dependency changed. This is worker-level evidence:
callback remains unity, and no native-format/PDC/packaging/listening/UI acceptance
is claimed. Ticket stays claimed; 94 stays blocked. Canonical report:
`C:/Users/User/Documents/podcast-automix-rust-gui/docs/research/smart-gate-resumption-2026-10-28.md`.

### Resumption — 2026-10-28

The owner requests continuing 93 and permits sonible research if needed; the
existing evidence is sufficient for this worker/DSP slice. Canonical ownership
remains in the primary development tracker. Current branch is
`codex/smart-gate-realtime-93` at tested policy checkpoint `bb9e1a4`.

The already-applied pending worker regression now covers 44.1/48/96 kHz and
repeated resets with undrained evidence. It is unrun and requires the missing
worker module. Native preflight measured 7.51 GiB available physical / 6.52 GiB
commit, no compiler processes; no compile/index/inference/host followed below
the fixed 8 GiB warm-build start threshold. No new RED/GREEN, callback integration
or product qualification is claimed. No dependency or production source changed.

Continue with the prepared worker RED once memory permits the unchanged
8/3/2.5 GiB / 600-second warm guard and immediate >=6 GiB commit preflight.
Canonical details and next slices are in
`C:/Users/User/Documents/podcast-automix-rust-gui/docs/research/smart-gate-resumption-2026-10-28.md`.

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

