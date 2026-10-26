# Causal Gate detector feasibility (ticket 92)

Status: measurement pending; Gate audio stays pass-through. This is speech
presence, including another speaker's bleed, never microphone ownership.

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
