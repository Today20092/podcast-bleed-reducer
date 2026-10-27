# Causal Gate detector feasibility (ticket 92)

Status: measured causal feasibility supports a conservative ticket-93 prototype;
ticket 92 remains claimed pending review. Gate audio stays pass-through. This is
speech presence, including another speaker's bleed, never microphone ownership.

## Chosen bounded experiment

Reuse the existing MIT Silero VAD v5 ONNX bytes in `assets/silero_vad.onnx`,
SHA256 `1a153a22f4509e292a94e67d6f9b85e8deb25b4988682b7e174c65279d8788e3`,
2,327,524 bytes. Reuse locked `ort=2.0.0-rc.10`, CPU execution with one intra-op
and one inter-op thread. No vendor weights, model download or trainer.

Graph discovery on primary project `C-Users-User-Documents-podcast-automix-rust-gui`
found `src.vad.silero_frames`; callers include `silero` and
`prepare_analysis_tracks_with_feedback`, then acoustic ownership analysis.
Its recurrent invocation is useful evidence; its whole-file allocations and
centered offline filtering are unsuitable as an audio callback adapter.

Context7 resolution was attempted 2026-10-26 and rejected by monthly quota.
Checked official [v5.1 wrapper](https://raw.githubusercontent.com/snakers4/silero-vad/v5.1/src/silero_vad/utils_vad.py)
and [current wrapper](https://raw.githubusercontent.com/snakers4/silero-vad/master/src/silero_vad/utils_vad.py):
16 kHz calls consume 512 new samples plus 64 past context, with recurrent state
shape `[2,1,128]`; reset zeros both. Neither wrapper proves sample-level onset
accuracy. The pinned local Rust SDK source supplies the actual Session/Tensor API.

`plugins/gate-detector` is a separate offline feasibility workspace. Preparation
constructs the ONNX session and 160 rational FIR phases. Input is finite mono
44.1/48/96 kHz. A 127-tap Blackman-windowed 7 kHz lowpass, delayed 64 native
samples, samples onto an integer 16 kHz schedule. No future native input is read.
One inference occurs per 512 model samples (32 ms), carrying context/state.
Host blocks do not reset it. Reset discards FIR/frame/context/recurrent history,
increments an epoch and restarts the sample clock. Partial EOF frames are omitted;
no manufactured zero-tail probability. Invalid input/result faults latch until reset.

The implementation deliberately allocates tensors and invokes ONNX synchronously.
It is NOT wired to the Gate process callback. Rust allocation counters exclude
native ONNX allocations; guard private/resident peaks include the owned process.

## Immutable handoff contract for 93

Recipe v1 + model digest + native rate + reset epoch identify evidence.
Each copied `Event` contains epoch, exclusive model-frame end `end_16k`, exclusive
native availability sample `available_at`, and finite probability in `[0,1]`.
The model interval is `[end_16k-512,end_16k)` at 16 kHz; subtract 64 native samples
from its rational native coordinates to describe FIR-aligned audio. Negative
startup coordinates mean initial zero history, not real recorded silence. The
64 context samples are past input, not extra lookahead. Recurrent history is
unbounded within the reset epoch. Frames are estimates, not onset guarantees.

For 93, readiness requires successfully prepared immutable model/rate identity,
current epoch and a complete fresh frame. On start/reset/seek/rate-change, missing,
stale, nonfinite, dropped or out-of-order evidence, worker failure or queue overflow:
unity/pass-through, clear stale decisions and show not-ready. Do not translate
failure into zero probability. An eventual bounded worker bridge must preserve
sample positions, epoch and ordering; it must never block or allocate on the audio
thread. That bridge and gain/attack/hold/release/PDC are 93 work, not this harness.

Algorithmic evidence age is up to one 32 ms frame plus FIR delay (1.4513/1.3333/
0.6667 ms at 44.1/48/96 kHz), plus host-block delivery and measured compute/worker
time. The current pass-through Gate declares zero host latency: this prototype
adds no plugin processing. These are different quantities. No onset preservation
or future PDC value is established without gain-path qualification.

## Reproducible measurements

Use the existing resource guard and shared Cargo settings, locked offline, `-j1`,
default 14/8/6 policy for compilation. Example `feasibility AUTHORIZED_TEST_AUDIO
NEW_REPORT_JSON` runs only against local authorized float32 mono derivatives and
creates a new report; it never writes source audio/projects. Reports contain
excerpt hashes, project hash, reference source bindings, all probability values,
event availability/tail, preparation time, push median/p95/max and Rust allocations.

Cases: constructed silence/white noise, up to three seconds of every current
human reference (including room), scaled quiet speech, 150 ms cropped speech and
artificial mixed speech. Derived cases are explicitly not naturally quiet speech,
verified interjections or natural overlap. No human laughter labels exist here.
The existing recording is development material, not an independent holdout.
Five native block partitions (1/17/64/257/1024) must produce bit-identical events
after reset at every rate. Public tests also contaminate recurrent history before
reset and assert finite probability, native availability and fault recovery.

## 2026-10-26 checkpoint: resource blocked, no feasibility verdict yet

The checkpoint below is historical. The measured follow-up at the end supersedes
its uncompiled/unmeasured status without claiming native or listening acceptance.

Source base `dda8505`; canonical claim commit `7721337`. No plugin/probe source,
installed link or preserved Gate candidate changed. This separate crate has NOT
compiled successfully and its measurement example has NOT executed. Therefore no
actual probability discrimination, CPU latency, preparation timing or runtime
memory result is claimed, and no detector is qualified for 93 yet.

The test-first public streaming/reset test initially failed to compile because
`Detector` did not exist (`E0432`). The guarded red log is
`%LOCALAPPDATA%/CleanSlate/BuildReceipts/20261026-113514-72c348f6254d4f3981a18130f9e8f19f.log`.
Dependencies compiled, then an owned VCTIP descendant kept the job alive. The
guard stopped it when commit reserve fell below 8 GiB: receipt same stem `.json`,
`resource-blocked/low-commit-running`, cleanup verified, 457.45 s elapsed,
597,155,840 byte peak aggregate job commit. This is a compile-red observation
inside a resource-blocked job, not a completed test run or useful inference timing.
An earlier command-line invocation rejected duplicate `--offline` before testing
(`20261026-113502-bf336b3d8c7b4d3ca3dcb5ae8684e5d3.json`).

After implementation, one standard locked all-target test preflight refused to
start: `20261026-114456-36e85dc340894395a6b0e386e4e3942a.json`,
`low-memory-before-start`. Point-in-time available physical/commit were about
13.92/13.30 GiB, below the unchanged 14 GiB start requirement. No repeated heavy
retry, lower reserve, helper policy change or unrelated process termination.

Noncompiling guarded formatting passed
(`20261026-114402-db9917be78424eeab71cef77b573af0f.json`); offline lock generation
passed (`20261026-114403-e2670ee54384402ba9b60d5f7e033fb2.json`), each with explicit
2/1/0.5 GiB start/reserve/owned-commit bounds. Static diff checks passed. Next:
once default headroom is available, run locked all-target tests/strict Clippy,
then compile and execute the example under the guard against the canonical
authorized fixture root with a new private report path. Inspect probability and
worst inference-bearing block results, then record a bounded feasibility decision.
Ticket 92 stays claimed; 93 remains blocked. Natural laughter, quiet speech,
interjections, overlap and listening remain qualification gaps even after the
proposed derived development cases run.

## 2026-10-26 measured follow-up and bounded decision

**Decision:** the existing model, causal resampling/state adapter and event
contract are usable for a conservative 93 worker/gain prototype. Preparation and
synchronous inference must remain off the audio callback; every missing, stale,
invalid or late decision must preserve the original. This is a feasibility
decision, not a choice of production thresholds or evidence that quiet starts,
tails, laughter and natural overlap are preserved by a gate. Ticket 92 is kept
claimed until review/integration; do not silently enable product gating.

No model or dependency was upgraded. `Get-FileHash` independently verified the
embedded model SHA256 above. Context7 became available again: exact
`/pykeio/ort/v2.0.0-rc.10` documentation confirmed the synchronous named-input
Session/Tensor API; the [pinned upstream documentation](https://github.com/pykeio/ort/tree/v2.0.0-rc.10/docs/content)
and installed source remain the contract. Strict Rust 1.99 Clippy first rejected
constant `chunks_exact(4)` in the WAV fixture reader; using `as_chunks::<4>()`
removed the lint and fallible conversion without changing the detector.

### Actual checks and resource bounds

All-target tests passed, including the public 44.1/48/96 kHz block/reset,
nonfinite-fault and recovery contract (0.53 s test execution). Strict all-target
Clippy and the example build passed. Compilation kept the default 14 GiB start,
8 GiB reserve, 6 GiB owned cap, locked/offline shared cache/profile and Cargo -j1.
The largest kernel job commit was 660,918,272 bytes; cleanup was verified.
Test/build guard elapsed times include lingering VCTIP helpers: the coordinator
confirmed each exact process inside its live named CleanSlate Job Object after
cargo/rustc/link had exited, then stopped that helper only. No compiler, unrelated
process, service or application was stopped; no guard limit was weakened.

The runtime measurement used an explicit 4 GiB start / 2 GiB reserve / 1 GiB
owned cap / 300 s budget. It completed in **13.93 s**, including streaming file
hash verification and report writing: peak private **38,187,008 bytes (36.42 MiB)**,
working set **68,612,096 bytes (65.43 MiB)**, kernel job commit **39,690,240 bytes
(37.85 MiB)**. Minimum observed physical/commit headroom was 17,987,960,832 /
17,060,339,712 bytes. These are process/job observations, not whole-host or
worst-case realtime guarantees.

| Native rate | First preparation in this run | Worst inference-bearing push across all cases/blocks | FIR delay |
| --- | ---: | ---: | ---: |
| 44.1 kHz | 124.980 ms | 0.416324 ms | 1.451247 ms |
| 48 kHz | 80.856 ms | 0.447055 ms | 1.333333 ms |
| 96 kHz | 78.913 ms | 0.534146 ms | 0.666667 ms |

Preparation is a single observation per rate, not a repeated cold-start benchmark.
Worst push spans include scheduling/resampling/model work and are measured wall
time, not a guaranteed deadline. Across all 165 case/rate/block runs there were
10,140 model calls and 6.488 s accumulated push time. Rust observed **16 allocations
per model call**, excluding native ONNX allocations: direct callback use is
explicitly rejected. A bounded worker bridge in 93 must be tested separately.

For each of 11 cases, probabilities were bit-identical across blocks 1/17/64/257/
1024 after reset at each rate. Public reset tests also contaminated state before
reset. Every model call represents 512 fresh 16 kHz samples (32 ms); frame
availability follows the exclusive native sample clock. Incomplete EOF tails
were omitted, never inferred from manufactured zero padding. The event age is
up to 32 ms plus the rate-specific FIR delay, host-block delivery and measured
compute/worker scheduling. **Current pass-through host latency remains zero** as
checked under 91; future gated-audio latency/PDC is not established by this run.

### Available probability evidence and its limits

The canonical project hash was
`cb1180b1e8bcbeb1d136d6480a2227549407fad75f66769d8f81270bf2680d78`.
All three complete float32 mono source hashes matched their saved human reference
bindings before excerpt reading. At most three seconds of each current human
reference were read for inference; sources and project were never written.

At block 257 (other blocks give exactly the same probabilities), synthetic
silence mean probability was 0.0047 at all rates, white noise 0.0085/0.0104/0.0129.
The five source-bound human speech ranges had means approximately 0.833–0.959;
scaled speech 0.877–0.884 and an artificial speech mix 0.939–0.955. These demonstrate
speech response on this development material, not calibrated confidence,
independent accuracy or natural quiet/overlap acceptance.

**The human room reference disagrees:** reference 15, active_speakers=[], begins
at 2279.618451 s. Its selected three seconds had mean probability
0.8267/0.8264/0.8248; 77/93 frames were at least 0.5 at every rate. This is an
important false-open/noise-leakage risk for a simple threshold gate. The saved
annotation describes the session's assigned speakers; it is not an independent
audit proving no untracked/distant speech in that waveform. No reference was
relabelled and no production threshold was tuned to hide this disagreement.
High probability here keeps a conservative gate open; it does not justify
attenuating wanted speech or rejecting all causal/runtime feasibility.

The 150 ms crop of reference 9 was never manually certified as speech throughout:
maximum probability was 0.2288/0.1939/0.5938. It must not be reported as a verified
interjection test or as proof of per-sample onset failure. Natural laughter,
quiet starts/tails, interjections, overlap, changing noise and subjective sound
remain qualification gaps. 93 should preserve audio on uncertainty and report
these limits rather than claiming automatic speaker isolation. No newer model
is proposed solely because it exists; any alternate model/calibration decision
requires a separate bounded comparison with source-bound evidence.

### Retained evidence

Runtime report (local only):
`%LOCALAPPDATA%/CleanSlate/GateDetector92/feasibility-5dd251afc2804a7ca82de4aec6576201.json`
(500,839 bytes), SHA256
`a262774f6ceed9cc88f2ba2469c3c251ab3d5b3044daf1ff4bb11e6b5c666195`.
Executable: `%LOCALAPPDATA%/CleanSlate/BuildCache/debug/examples/feasibility.exe`,
SHA256 `f6a16b410fa3d105c7df07444a89ee77631a01fe3f6f3772730f0432a5a550ea`.
No audio is in the report or committed; the report retains probability series,
excerpt hashes, source bindings, rate/block/tail identity and timing summaries.

Receipts under `%LOCALAPPDATA%/CleanSlate/BuildReceipts/`:

- Tests: `20261026-230839-70f78997b9ac4234b55c16ce51653aeb.json`.
- Initial Clippy lint failure: `20261026-231006-9fe5979aea504762ac6fb88843c61506.json`.
- Strict Clippy green: `20261026-231041-47c17c7e9f1e4fb5bf502392356d8302.json`.
- Example build: `20261026-231055-93495e0b04b34b3d8c80fac2fc5af476.json`.
- Runtime: `20261026-231218-b94e0e9e9c8a4669957427725637fa0a.json`.

The tested detector is checkpoint `b4e036c` plus the fixture-reader Clippy fix;
no detector algorithm, Smart Gate callback, probe source, installed link or
preserved product candidate was changed during this follow-up.

### Reference-fixture identity correction and final verification

The harness deliberately selects reviewed reference IDs 9/12, so it must not
silently accept edited projects whose reference bindings may be stale. It now
pins the exact reviewed project SHA256 above **before** JSON parsing, file hashing,
audio access or model preparation. Changed projects require deliberate reference
binding/label review and a new fixture identity; this is not a general project
importer. The actual measured project's six references were independently checked
against the same verified source fingerprints; the original measurement remains
valid.

A public CLI red/green regression uses a private fixture containing only an empty
`261005-supervised.cleanslate` (`{}`), with no source audio. Before the fix, the
expected identity-boundary assertion failed because the command reached `missing
sources`. After the fix, it explicitly rejects `reviewed reference project
identity mismatch` and creates no report. Retained evidence:
`%LOCALAPPDATA%/CleanSlate/GateDetector92/project-pin-red-guard.json` and
`project-pin-green-guard.json` plus logs and `unreviewed-project-fixture/`.
This negative check uses an explicit 2/1/0.5 GiB budget and 10 s deadline; no
inference or real source audio is required.

Final formatting and strict Clippy pass. The rebuilt final executable reran the
original reviewed fixture successfully: all **165 probability arrays exactly
match** the first report, with identical model/project identities. Final report:
`%LOCALAPPDATA%/CleanSlate/GateDetector92/feasibility-pinned-project-20261026.json`,
SHA256 `0be1fb808ee016ce7ed2d8f1cd5858d1886df1cabde7597c906e88379eeed77f`.
The second run took 16.70 s, peak private/working-set/kernel-job bytes
37,756,928 / 58,191,872 / 39,276,544, cleanup verified. Its preparation times were
171.742/75.716/75.274 ms and worst inference-bearing pushes
0.663352/0.432038/0.480089 ms at 44.1/48/96 kHz. Thus the largest observed push
across both runs is **0.663352 ms**; neither run establishes a guaranteed deadline.

Final executable SHA256:
`a255ddc70dcb97e15317a6e20cadb0f449bc5d522dd4f2746b7dd541c124d349`.
It is also hash-preserved outside the shared cache at
`%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/gate-detector/a255ddc7/feasibility.exe`.
Final receipts: formatting `20261026-231943-efc21645f68d47598f36df7abf2f330f.json`,
build `20261026-231944-913b80a1190d4ec6926c7230a2560030.json`,
Clippy `20261026-232112-72c2e47ef46a4b0680a39634b2032b4c.json`,
runtime `20261026-232116-2e1dd22d71704f219cd45d1b4c15bc0d.json`.
The rebuilt example retained the same detector algorithm; changes were the
fixture-reader lint fix and reviewed-project trust boundary.

## Recorded category follow-up

This follow-up supplies the missing **bounded category smoke evidence**. It does
not reinterpret clip-level descriptions as per-frame truth. The
[fixture acquisition record](smart-gate-detector-fixtures.md#successful-commons-acquisition-and-bounded-execution)
and [compiled metadata manifest](../../plugins/gate-detector/fixtures/public-categories-v1.json)
record successful policy-compliant Commons acquisition, licenses/attribution,
original and decoded hashes and the fixed pre-inference decoding/excerpt recipe.
Freesound GET 403 remains unresolved; no Freesound bytes are used. No audio is
distributed or committed, and no owner recording/project was changed.

The optional third CLI argument enables three exact-hash public 48 kHz mono
float32 fixtures plus a recorded natural-overlap case. Each file is read with an
expected-length cap, checked for exact bytes/SHA256 and finite PCM before model
preparation. Arbitrary external manifests cannot replace the compiled identities.
The pinned owner quick-project identity, all three complete WAV hashes and saved
reference-source bindings are also checked. Saved human reference 7 has active
speakers [1,2]; its entire 2.666208 s range from track 1 is used. Original reference
13 was solo, not overlap. The report retains the exact masks, source bindings,
project/excerpt hashes and development-data limitations.

At block 257, the representative actual recordings yielded:

| Recorded case | Complete 32 ms frames per rate | Mean probability at 44.1 / 48 / 96 kHz | Frames ≥0.5 at 44.1 / 48 / 96 kHz |
| --- | ---: | --- | --- |
| Whispered word, Mx. Granger | 80 | 0.1403 / 0.1398 / 0.1398 | 11 / 11 / 11 |
| Spoken yes, Mattadjwic/Codymator Adjwic | 19 | 0.7133 / 0.7183 / 0.7047 | 15 / 15 / 14 |
| Human laughter, ezwa | 468 | 0.0091 / 0.0091 / 0.0073 | 1 / 1 / 1 |
| Owner recorded overlap reference 7 | 83 | 0.9639 / 0.9640 / 0.9649 | 80 / 80 / 80 |

All five block partitions give exactly the same probabilities after reset at
each rate. The arbitrary 0.5 column describes the series; it is **not a selected
gate threshold or an accuracy metric**. Whole clips include unknown silent and
nonspeech spans. Low whisper/laughter response is a strong wanted-audio
preservation risk; without time masks it is not a measured false-negative rate.
The recorded-word case is an elicited affirmative, not a claimed spontaneous
interruption. The overlap mask is saved human development evidence, not an
independent acoustic audit. No thresholds, labels or excerpts were retuned to
hide any result.

**Updated bounded decision:** the existing causal/runtime path supports 93's
worker/queue/gain engineering experiment. It does not support enabling a
production gate on this probability alone. Any 93 prototype must default to
bypass, require explicit experimental arming and retain original fallback for
invalid/stale/late evidence. Per-frame preservation and owner listening remain
unqualified; these new categories do not waive 85/93's quiet/laughter/sound gates.
No newer model, ownership classifier or threshold-only substitute was introduced.

The measurement completed **225 runs** in 22.38 s with 19,890 model calls, peak private
59,162,624 bytes (56.42 MiB), working set 78,290,944 bytes (74.66 MiB), kernel job
commit 62,083,072 bytes (59.21 MiB), cleanup verified. Runtime kept explicit
4 GiB start / 2 GiB reserve / 1 GiB owned cap / 300 s bounds. Minimum observed
physical/commit headroom was 17,520,459,776 / 16,728,289,280 bytes. Preparation was
127.156/74.577/73.830 ms, and largest inference-bearing push
0.501531/0.484978/0.489727 ms at 44.1/48/96 kHz. These single-run wall-clock
observations remain neither guaranteed deadlines nor plugin PDC. Every earlier
165 probability array exactly matches the preceding report; the detector model,
algorithm and current pass-through host latency are unchanged.

All-target public causal/reset/fault tests passed (0.31 s execution), strict
all-target Clippy and formatting passed, and the example built. Compiling steps
kept default 14/8/6 GiB bounds, locked/offline shared cache and Cargo -j1. Two
lingering VCTIP descendants were each proven inside their exact named CleanSlate
job after cargo/rustc/link exit, then stopped individually by the coordinator;
guard elapsed includes that wait. No unrelated process or service was stopped.
The same-size one-byte changed public PCM fixture failed the new public CLI
boundary red before implementation (old CLI usage rejection); green rejects its
pinned PCM hash before inference or creating a report. Evidence remains at
`%LOCALAPPDATA%/CleanSlate/GateDetector92/public-fixture-red-guard.json` and
`public-fixture-green-guard.json`, plus the private tampered fixture and logs.

Category report:
`%LOCALAPPDATA%/CleanSlate/GateDetector92/category-feasibility-9da9a89b8d2248ff9a7e34cc972a9568.json`
(967,353 bytes), SHA256
`b85b692462f1f2220a2aae122f15d22ede5170facc00f959bad05c4b2936c8be`.
Compiled metadata-manifest digest in that report:
`4d850e980aaa6cf31175713a8e58a6aa15ab88ce63366373bc751446744011db`.
Tested executable SHA256:
`9cbfc2ab82c10cdd7a957a5df4cd2414833992ab326a80ddc8657137489f3541`,
hash-preserved at
`%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/gate-detector/categories-9cbfc2ab/feasibility.exe`.
Receipts under `%LOCALAPPDATA%/CleanSlate/BuildReceipts/`:

- Build: `20261026-235423-2b919568033d4a66a722d0bbedead655.json`.
- Category runtime: `20261026-235542-8c9481edf4de4ffd8c63892a40c5f0bc.json`.
- Strict Clippy: `20261026-235629-070b0fcb483d477fad580ef28ee77979.json`.
- Formatting: `20261026-235659-2b57457dec354577a52ed56c679ef509.json`.
- All-target tests: `20261026-235700-d74e271023b343a0a9ca32ce3454f270.json`.

Ticket 92 remains claimed pending independent review/integration. Category-smoke
coverage is now present, while broader sound/preservation remains an explicit
93/85 obligation rather than an invented requirement for completing this bounded
feasibility decision.
