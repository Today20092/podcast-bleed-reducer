# Gate 93 confidence-to-gain checkpoint

2026-10-27. This is a tested policy module, **not a working Gate engine**.
The callback remains exact pass-through; no worker, gain application, inference
preparation, product UI, offline gating or standalone loader is implemented.
The preceding state checkpoint is `c66f8ba`; this scoped batch remains on
`codex/smart-gate-realtime-93`. Ticket 93 stays claimed with acceptance unchecked.

## Policy and evidence limits

`plugins/smart-gate/src/gain.rs` consumes the unchanged public detector `Event`
contract. It prepares only fixed-size policy state, never audio storage, host
work, filesystem access or a model. Supported rates match 92: 44.1/48/96 kHz.
It validates finite probabilities, epoch, consecutive 512-sample frame ends and
the corresponding rational native availability; invalid evidence latches a fault
until reset and immediately returns exact unity. A reset begins a new stream at
unity. Same-source fault switching is deterministic but **not claimed click-free
or continuous**; exact original preservation takes priority over a fault fade.

Chosen engineering values: open at probability >=0.10, close at <=0.01, retain
the previous confidence decision between them. The initial/open hold is 300 ms,
closing takes 250 ms and opening/ordinary disable returns within 5 ms, with an
exact final unity sample. Maximum reduction is finite -6 dB (gain 0.5011872).
Evidence expires after 100 ms; its final 5 ms proactively returns to unity while
still valid, and missing/expired evidence returns exact unity. Durations are
integer sample counts; the five-millisecond count rounds down at 44.1 kHz.
This policy does not buffer audio or add host latency, but it is not connected
to any adapter and establishes no product latency/tail/PDC qualification.

The initial 0.5 opening proposal demonstrably retained attenuation for synthetic
confidence 0.14. The test-first correction to 0.10 intentionally favors original
audio/false-open behavior around 92's measured whisper mean (~0.14). A category
mean does not establish per-frame probability or speech preservation. Laughter's
mean (~0.009) remains below the close threshold: **strong preservation risk**.
No threshold, limited reduction, numerical test or synthetic confidence stream
qualifies whisper/laughter quality. Default bypass and temporary explicit arming
from the preceding state checkpoint remain mandatory for eventual integration.

## Executed public tests and resource receipts

Receipts/logs: `%LOCALAPPDATA%/CleanSlate/BuildReceipts/`.

- Prepared gain RED executed: `20261027-110108-62fda180318647ee854be45ecd76ec3c`.
  The missing policy module and Event dependency produced compilation failure.
- Offline lock update: `20261027-110222-88985eacedfe425cb5fa015f91453e8a`.
  Direct path dependency reuses the 92 crate. All 56 added package identities
  already occur in 92's lock; zero existing Gate identities changed or disappeared.
  No download occurred. Reusing the Event type also introduces the existing ORT
  dependency graph; it does not establish inference preparation or deployment.
- Fresh-absence/expiry GREEN: `20261027-110303-2605f5c59bc346bc9247ee2b14887bfd`.
  Fresh 32 ms zero-probability events reach the -6 dB floor, then delivery stops
  and gain is exact unity at and after the 100 ms expiry boundary.
- Quiet-confidence RED: `20261027-110627-18ff09224b494350a0a70d0c5921daa5`.
  A previously attenuated gate stayed at 0.5011872 under 0.14 evidence with the
  initial 0.5 threshold, instead of reaching unity.
- Quiet-confidence GREEN plus all eight library tests:
  `20261027-110814-97fa912244e043b6a7dcfc761809cb11`.
  With 0.10 open and a counted exact-endpoint ramp, 0.14 reaches exact unity
  within five milliseconds; all preceding state/pass-through checks also pass.
- Guarded formatting passed:
  `20261027-111108-620ffa93aeba4a82909b07b2bc8fc74a`. `git diff --check` passes.
  Strict Clippy, all-target/native-host and deployment checks have not run for
  this bounded policy-only checkpoint.

This owner-authorized warm batch uses an unchanged per-attempt 8 GiB physical
start / 3 GiB physical-and-commit running reserve / 2.5 GiB owned commit cap /
600-second timeout. Same-process native `OwnedTask.Memory()` immediately before
each launch requires >=8 GiB physical and >=6 GiB available commit. Cargo stays
locked/offline, `-j1`, on the pinned shared profile/cache and global mutex.
The first GREEN rebuilt existing dependency feature-union nodes; actual compile
was 2m02s, job elapsed 181.02s including the lingering helper. It peaked at
819.81 MiB owned job commit, with minima 12.10 GiB physical / 8.90 GiB commit.
Quiet RED peaked at 821.36 MiB, minima 8.44/3.95 GiB, above the fixed reserve.
The final eight-test GREEN peaked at 799.96 MiB, minima 11.85/8.95 GiB, with
completed status and verified cleanup.
The coordinator proved exact named-job membership and stopped only lingering
VCTIP helpers after compiler exit. Completed receipts verify cleanup; no service,
pagefile, unrelated process, installed artifact, reboot or budget lowering action.

## Remaining acceptance

Tests currently cover the declared public policy traces and state safety, not
the full confidence/fault/parameter/rate/mono/stereo matrix. Worker scheduling,
epoch queues, host-block-independent stereo without phase cancellation,
sample/latency alignment, real-time callback allocation/deadline evidence and
actual CLAP/VST3 state/automation/lifecycle checks remain. Fast offline rendering
must demonstrate actual non-unity processing, or remain explicitly unmet; an
all-original worker-miss fallback is not equivalent to a working offline gate.
Runtime prerequisites, candidate-local DLL resolution/import checks and notices
remain governed by the runtime audit. No callback, native format, sound/listening,
OBS/VST2, UI or product completion is claimed by this checkpoint.
