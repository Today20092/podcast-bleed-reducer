# Smart Gate 93 resumption

## Current callback checkpoint

Source `af4d9214c7a0884c9d319bebe08d3eafac01264b` is committed and clean on the
retained `codex/smart-gate-realtime-93` branch, after worker `64c5525`. It is not
integrated into the permanent Gate lane or desktop. The callback now connects
prepared inference to the shared finite gain policy; the earlier pass-through
statements below describe the preceding worker-only/preparation checkpoints.

The actual offline callback test reached semantic RED (gain remained 1), then
GREEN at the -6 dB floor on a synthetic absence fixture for mono and stereo.
Unarmed/bypassed/reset audio remains original; nonfinite input preserves exact
payload bits and latches an original-audio fallback. Both stereo outputs use one
gain. Fixed 256-sample input chunks and source-time event application produce
identical offline audio with 127/1536/32768 host block sizes. Future evidence is
not applied early. Activation prepares resources and deactivation joins workers;
process/reset do not allocate model/session/queue storage or create/join threads.
No claim of measured callback allocations/deadlines follows from code inspection.

All **17 library tests**, formatting and strict library/test Clippy pass:

- Callback semantic RED: `20261028-181909-fcc004acf5d24c61a636e26ee3b78011`.
- First mono/stereo callback GREEN: `20261028-182049-241a574a1c5a42ffa6d97009fe11ed3f`.
- Final 17-test GREEN: `20261028-182205-dba42bf392524f4eafb4aa44cc386e83`.
- Final strict Clippy GREEN: `20261028-182304-6e43b8120a4947c9aa8aaba04dd7a279`.

These receipts verify cleanup and unchanged warm guard settings. The callback
GREEN peaked at 0.854 GiB owned job commit; its minimum physical/commit headroom
was 7.83/7.51 GiB, above the fixed 3 GiB running floor. Final 17-test peak was
0.831 GiB; minimum physical/commit 7.96/7.52 GiB. No dependency changed.

Preserved test executable:
`%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/gate93/20261028-182205/gate-library-tests.exe`,
SHA256 `f278b3b1c43ef26f8d3b7e4c80c8245e7a8be050986073340f1b9777fce11e68`.
This protects tested bytes; it is not a standalone inference runtime package.

To prepare native compilation, fourteen further unused Astro/shadcn tool-server
processes were verified as descendants of the same Codex server and stopped by
exact identity. PIDs: 20184, 29344, 21144, 25444, 25816, 14960, 37316, 38716,
29332, 31084, 15924, 31524, 36200 and 28572. Physical availability rose
8.82 -> 9.64 GiB; commit headroom changed 8.36 -> 8.03 GiB concurrently, so
process private-byte totals are not claimed as a net system-memory saving.
The final native-build preflight was 9.76 GiB physical / 7.82 GiB commit, below
the separate 10 GiB physical / 9 GiB commit start requirements. No native
compilation/host or further resource retry ran, and no larger build was relabeled
as a targeted warm test. No user application or Windows service was stopped.

Remaining: realtime allocation/deadline/underrun and onset-loss measurements,
all-rate/native CLAP/VST3 processing/state/automation/PDC/offline checks,
candidate-local runtime/import/notices qualification, explicit native arming and
truthful readiness controls, and human listening. Arming is currently Rust
API/test-only; the existing editor and checked native candidate remain unchanged.
Zero inserted audio delay does not mean zero detector/delivery delay. Whisper
and laughter preservation remain a strong risk. 93 stays claimed and 94 blocked.

## Progress after owner-authorized memory cleanup

This section supersedes the preparatory/unrun state below. The owner explicitly
authorized freeing memory. Seven unused REA MCP server processes were verified
as descendants of the current Codex app-server, with no native analysis or other
compiler running, then stopped by exact identity. PIDs: 27820, 5632, 22052,
22992, 3096, 36560 and 26676. Their private memory totaled approximately 1.7 GiB;
available physical memory rose 7.77 -> 8.67 GiB and commit headroom 6.53 -> 8.55
GiB. Wrapper exit and unrelated allocation changes can affect the net delta.
No Windows service, user application, source, build cache or app session was
closed. Those tool-server connections may need reinitialization if used later.

The prepared epoch regression executed with a real missing-module RED, then
passed against a new worker using the existing embedded detector. A subsequent
stereo RED rejected two channels, followed by a passing stereo implementation.
Fourteen library tests passed, including mono/direct-detector equality across
127/512/1536 host block sizes, all three mono rates, repeated/reused-epoch resets,
queued-result invalidation, invalid samples, input overflow, right-only and
opposite-phase stereo, FIFO wrap/full/empty and concurrent record publication.
Strict library/test Clippy passed after its single collapsible-if correction.
An additional result-overflow/realtime-no-wait regression is being verified;
its outcome is recorded in the final receipt section when available.

The bridge prepares its model/queues off the callback, uses one fallibly-created
worker and fixed storage, and joins that worker on off-callback teardown. Its
private SPSC queues use atomic payloads and release/acquire cursors, with no
unsafe shared-memory code or CAS retry loops. Existing crossbeam queue source
was considered but uses retry/backoff loops; no dependency was added. Stereo
uses independent recurrent detector states and the greater confidence, without
summing signed audio. Generation IDs are independent of externally supplied
epochs; monotonic fault markers prevent an old inference failure clearing a
new stream's fault. Input/result overflow and invalid input hide evidence until
reset. Offline synchronization has a 500 ms bound; realtime mode refuses that
wait. The 1 ms worker polling interval, dual-detector CPU cost, shutdown timing
and callback deadlines are engineering choices awaiting product measurements.

The product callback is still pass-through. No audio gain, editor, host-format,
latency/PDC, standalone runtime, installed link or listening qualification is
claimed. Tests use the development runtime environment; they do not close the
candidate-local DirectML/MSVC packaging gate. Ticket 93 stays claimed and 94
blocked. Next: callback/arming integration, then actual-format and deployment
checks. Whisper/laughter preservation remains unqualified.

Resource receipts under `%LOCALAPPDATA%/CleanSlate/BuildReceipts/`:

- Worker missing-module RED: `20261028-180531-89229d20a1ce41be9aefcaa3cafb8f1b`.
- First mono/reset GREEN: `20261028-180801-e4a90b328c9c45d4a68a94ee088d79b2`.
- Stereo RED, other five worker tests GREEN:
  `20261028-181150-e59289d8bb514a2cb2354be8d4528be7`.
- Fourteen-test GREEN: `20261028-181259-3f34d7731aa6410786ea3db6c0aaff2b`.
- Strict Clippy GREEN: `20261028-181529-bd90156bb15d48a68e26e964c61360b5`.

All completed receipts verify cleanup. Compilation retained the fixed warm
8/3/2.5 GiB / 600-second profile, locked/offline dependencies, pinned toolchain
and one Cargo job. Completed Cargo/rustc/link processes left compiler telemetry
helpers; exact newly-created VCTIP identities were checked and stopped only
after recorded test completion, allowing the guard to finish descendant cleanup.

## Earlier preparation checkpoint

The owner requests continued Smart Gate development and permits additional
sonible investigation if needed. Existing manual/static evidence is sufficient
for the current worker/DSP work. No further vendor analysis, installation,
activation or execution was performed.

## Source and verification boundary

Product worktree:
`C:/Users/User/.codex/worktrees/smart-gate-identity-91/podcast-automix-rust-gui`,
branch `codex/smart-gate-realtime-93`, HEAD `bb9e1a4`.
Canonical [ticket 93](../../.scratch/offline-supervised-editor/issues/93-smart-gate-realtime-engine.md)
remains claimed; 92 is done and 94 remains blocked by 93.

The prior pending worker regression was found already applied in `src/tests.rs`;
its original recovery patch was preserved. This pass extends that regression
across 44.1/48/96 kHz and consecutive stream resets. It checks immediately after
reset that no previous result is exposed, then checks real detector evidence has
the new epoch and exact rational source-sample availability. Each repeated reset
also starts with an undrained result. These assertions are prepared, **unrun**,
and depend on the not-yet-implemented `WorkerBridge`. No semantic RED is claimed.
No production implementation changed; the callback remains pass-through.

## Resource stop

Native `OwnedTask.Memory()` measured 7.51 GiB physical availability / 6.52 GiB
commit headroom; no Cargo/rustc/link/rust-analyzer process was present. The fixed
warm profile requires at least 8 GiB physical and 6 GiB commit immediately before
launch, a 3 GiB running physical/commit reserve, 2.5 GiB owned cap and 600-second
deadline. No compiler, index, model, native host, resource-threshold reduction or
automatic retry followed. This was a read-only preflight, not a guarded task
receipt. The owner was asked to free memory using an application of their choice.

The graph's primary-project Smart Gate query returned no symbols; this separate
product worktree is absent from the project inventory. Indexing was not started
below the resource boundary. Named product files supplied the coverage gap.
Context7 resolution reached monthly quota; official Rust atomic and fallible
thread-builder documentation was consulted. No dependency was added or selected.

## Next test-first slices

1. Execute the prepared worker-reset test under the unchanged warm guard and
   confirm the missing worker API produces a real RED. Add the smallest bounded
   bridge, then verify real detector events and repeated resets go GREEN.
2. Test bounded input/result storage, nonfinite input, queue overflow, worker
   failure and stale/in-flight epoch rejection. Callback operations must not
   allocate, lock, create/join a thread, perform I/O or wait for inference.
   Prepare model/session/storage off the audio thread and make worker creation
   failure recoverable. No detached worker may outlive plugin teardown.
3. Resolve stereo with tests for right-only and opposite-phase speech before
   selecting channel analysis. A signed average can erase opposite-phase audio;
   both output channels must receive one envelope.
4. Connect valid source-ordered evidence to the existing gain policy. Preserve
   default bypass, explicit temporary arming, reset disarming and exact original
   fallback. Measure realtime deadline behavior separately from bounded offline
   synchronization; offline all-original output does not demonstrate gating.
5. Update CLAP/VST3 hosts to prove real non-unity processing, state/automation,
   reset, varied block sizes, mono/stereo, reported latency and sample alignment.
   Retain pass-through checks for unarmed/bypassed/faulted states.
6. Qualify the newly inference-linked artifact's imports and candidate-local
   runtime resolution, stage verified prerequisites/notices, then perform the
   isolated loader checks in the existing runtime audit. Only then preserve a
   checked engineering candidate. Native listening and ticket 94 stay separate.

## Files and consumers in scope

- `plugins/smart-gate/src/tests.rs`, a focused worker module and the existing
  `SmartGate` lifecycle/process implementation and gain policy.
- Existing CLAP/VST3 check examples and product build/host scripts, when actual
  processing and native dependencies change their current pass-through contract.
- Product README/state behavior and the canonical ticket/check receipts.
- Reuse `plugins/gate-detector` without copying or modifying vendor payloads.
  A shared detector change requires its own caller/tests checks.

No new editor controls, multiband processing, leveling, product identity change,
installed links, desktop/probe changes or release publication is part of this
slice. The prior eight-test policy checkpoint remains the latest tested source;
whisper/laughter preservation, worker/DSP and native packaging remain unqualified.

References: [gain checkpoint](smart-gate-gain-policy-93.md),
[runtime audit](gate93-runtime-packaging-2026-10-27.md),
[sonible findings](smartgate-followup-2026-10-25.md),
[resource guard](../resource-guard.md),
[Rust atomic memory model](https://doc.rust-lang.org/std/sync/atomic/index.html),
[fallible thread creation](https://doc.rust-lang.org/std/thread/struct.Builder.html).
