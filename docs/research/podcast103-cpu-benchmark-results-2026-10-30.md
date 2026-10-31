# Podcast 103 fresh matched CPU measurements

The unchanged waveform benchmark and the separate current gain/event cases pass.
Warm waveform-only totals are similar in this short matched run. Gain refresh
costs more than retaining the cached projection. All measured warm stage totals
were below 16.667 ms; this is not native presentation or 60 Hz evidence.

Current benchmark source: `b5052b22737fe4e24d01283f9ec6758bb9757ceb`, test-only
extension of production `b41d51e89a3bc5d413f83c0193b859e2d1a27f27`. Fresh baseline:
untouched `d2b30c91fa5a7f078ca75cdbf6999defc69ed513`. Both are scoped managed Orca
children of the stable Podcast checkout. No production branch, DLL, installed
file, native editor or owner audio was changed by this benchmark tranche.

## Method and reproducibility

Both checkouts used actual `plugins/podcast/Cargo.toml`, pinned Rust 1.99 Cargo/
Rustc and LLD, the CURRENT timeout-only wrapper and current shared settings:
locked/offline, one compiler job, shared incremental target, identical
`gui-dev.toml`. Compiler-artifact JSON confirms both normal-feature test targets
have opt-level 1, line-table debug info, debug assertions and overflow checks.
Lockfile/profile/fixture-source/tool executable hashes and exact commands are in
[the preparation record](podcast103-cpu-benchmark-plan-2026-10-30.md).

The eight-track 8 kHz varied-PCM fixture, label ranges, selection, playhead, 1000
peak columns and 564px clipped scrolling lanes are unchanged. Logical sizes are
720×560 and 1000×680. The old raw log and old executables remain missing from their
documented machine-local paths: these are **fresh matched runs**, separate from
the historical published summary table. The baseline worktree remains clean.

Each row has one cold and 40 warm samples. Tables show milliseconds as
**p50 / p95 / p99**, using the existing `ceil((n-1)*p)` warm-sample convention.
The correlated total is each raw row's query + draw + tessellation span, followed
by percentiles of those totals. It is not a sum of independent percentiles.
Only one serialized run per scenario was taken; OS scheduling, thermal state,
timer resolution and observer overhead limit small differences and tail claims.

## Unchanged waveform-only matched pair

Gain is disabled in this historical comparison; the new optional gain argument
is None. The full current workspace controls and gain are separate measurements.

| Source / logical size | Peak query | Geometry | Tessellation | Correlated total |
| --- | --- | --- | --- | --- |
| Baseline / 720×560 | .224/.255/.285 | .339/.427/.450 | .251/.344/.410 | .812/.948/.991 |
| Current / 720×560 | .224/.307/.308 | .333/.411/.411 | .231/.288/.292 | .792/.904/.939 |
| Baseline / 1000×680 | .189/.258/.296 | .437/.564/.593 | .463/.615/.630 | 1.107/1.303/1.355 |
| Current / 1000×680 | .197/.283/.316 | .499/.621/.660 | .351/.541/.583 | 1.083/1.255/1.259 |

There were **0/40 warm correlated-total CPU-budget exceedances** at each size
for both sources. Cold rows and every raw shape/vertex/index/capacity observation
remain in the logs. Small total differences do not establish an improvement;
the current full-size geometry stage was slower while tessellation was faster.

## Separate current gain cases

Draw includes the actual clipped-lane gain query/projection and geometry. All
eight source slots are mapped; only visible lanes are queried/painted. Refresh
is forced; cached trials force a fresh retained timestamp and assert that no
refresh occurred. These distinguish costs, not production scheduling frequency.
Gap removes verified source coverage from 16000–24000 on every member. Dense
uses the existing real manual-ramp EffectiveEnvelope fixture and a two-hour
view, explicitly refusing detail; it is not comparable to the eight-second view
as a speed improvement. Stale invalidates after the cold frame and paints no gain.

| Scenario / width | Peak query | Draw including gain | Tessellation | Correlated total |
| --- | --- | --- | --- | --- |
| Refresh / 720 | .257/.294/.306 | 1.081/1.183/1.200 | .662/.844/.848 | 1.962/2.250/2.274 |
| Refresh / 1000 | .255/.291/.325 | 2.263/2.442/2.456 | 1.269/1.803/1.982 | 3.797/4.466/4.686 |
| Cached / 720 | .249/.300/.301 | .754/1.047/1.194 | .684/.877/.883 | 1.682/2.199/2.206 |
| Cached / 1000 | .228/.302/.390 | 1.433/1.541/1.566 | 1.203/1.425/1.453 | 2.890/3.197/3.214 |
| Gap refresh / 720 | .232/.376/1.490 | 1.049/1.738/2.448 | .701/1.077/1.300 | 1.953/3.757/4.528 |
| Gap refresh / 1000 | .237/.339/.348 | 1.975/2.528/2.588 | 1.127/1.806/3.400 | 3.342/4.590/6.253 |
| Dense refresh / 720 | .190/.238/.238 | .065/.148/.169 | .021/.037/.043 | .276/.392/.395 |
| Dense refresh / 1000 | .190/.221/.226 | .100/.130/.200 | .024/.029/.060 | .317/.352/.465 |
| Stale / 720 | .209/.272/.299 | .331/.442/.454 | .228/.413/.472 | .791/.983/1.151 |
| Stale / 1000 | .182/.240/.247 | .455/.640/.700 | .362/.468/.525 | 1.006/1.271/1.301 |

All ten cases have **0/40 warm correlated-total CPU-budget exceedances**. The
largest observed warm total was 6.253 ms in the full-size gap case. Missing/dense
gain was never fabricated, and no failed cache/behavior sample was accepted.

## Actual egui dispatch to observed state

After an untimed layout/hover prime, actual wheel events update the viewport in
the measured `run_ui` call while preserving selection. The span ends after
reading the changed source viewport. It includes geometry and is an upper bound
on CPU dispatch-to-observed-state, excludes tessellation, and has no OS/device/
compositor timestamp. No 250 ms tween simulation is reported as input latency.

| Logical size | Dispatch-to-observed-state p50/p95/p99 | Warm CPU-budget exceedances |
| --- | --- | --- |
| 720×560 | .409/.538/.548 ms | 0/40 |
| 1000×680 | .576/.708/.787 ms | 0/40 |

## Preserved evidence and checks

All files are outside the managed worktrees under
`%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/podcast103/20261030-cpu-b5052b2/`:

- `current-tests.exe`: SHA256
  `9eb6b973b78c17384a089046a30efbd5b4614606d379739766deed5fa8d69abd`.
- `baseline-tests.exe`: SHA256
  `fde130db447b078bec8d447700247f95837f1a6e7883e8252cf87e439de69044`.
- `current-artifact.json` / `baseline-artifact.json`, exact compiler output and
  normal-feature/profile metadata; corresponding `*-compile.log` raw JSON.
- `current-waveform-retry`, `current-gain`, `current-dispatch`, and
  `baseline-waveform` receipt/log pairs. Each exact filter passed one test.
- `matched-summary.json`: SHA256
  `e67aa6ae6d5935debd47c0d6b16bf10e0390eec5314dab61a52f95641beaa708`;
  contains all stage percentiles, maxima and budget counts with raw-log paths.
- `summarize.ps1`: SHA256
  `b40571de15948f8865c827a8bf2b284e59ec4c43363aac9df01196c8a2f3e919`;
  retained postprocessor for independently recalculating correlated totals.
- `current-gain.log` also records the eight actual source PCM SHA256s outside
  measurement. The baseline fixture function's whitespace-normalized source
  hash is identical; no runtime hash probe was added to the untouched baseline.
- Current strict library/test Clippy passes (12.65 s compiler), formatting and
  diff checks pass. This is three targeted current tests plus one baseline test,
  not a new full-suite claim.

Successful receipts record root exit 0, cleanup verified and no remaining process
IDs. Cargo compile times were 3m32s current and 2m55s baseline; these are not UI
measurements. Task elapsed times also include lingering owned MSVC VCTIP helpers.
Their exact job-reported PID, parent and executable path were checked before
stopping only those telemetry helpers: current 6940/parent24416, baseline
12840/parent9464, Clippy8412/parent26216. Current waveform's initial attempt was
rejected by the still-held own compile lock before process launch; its busy
receipt is retained and is not timing evidence. The retry followed finalized
compile cleanup and uses a distinct receipt. No memory guard was applied.

## Remaining acceptance

These timers exclude fixture/model-envelope preparation, JSON/summary work,
untimed assertions and some texture/mesh destruction. They are measured stage
totals, not whole-frame or whole-window costs. Replacement during a query can
leave final Arc destruction on the UI thread; this fixture does not exercise
that concurrency cost. Allocation-call counts remain unavailable; mesh storage
is not an allocation counter. Per-instance/process CPU and memory attribution,
simultaneous instances, sustained idle/playback/resize, native DPI/wake behavior,
audio underruns, actual presentation misses and input-to-photon remain open.
These results do not close 103, native listening/PDC/export acceptance or release
qualification.
