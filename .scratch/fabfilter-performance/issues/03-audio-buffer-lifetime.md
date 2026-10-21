# 03: Audit and harden audio-device buffer reclamation

Status: claimed
Assignee: /root/perf03_implementer
Type: task
Blocked by: None (can start immediately)
Source proposal: P3

**What to build:** Keep the playback device path predictable by measuring heap-owning block/envelope lifetime transitions and moving demonstrated reclamation work off it without compromising bounded, nonblocking streaming.

## Acceptance criteria

- [ ] Trace and instrument allocation/deallocation at the playback output boundary for steady playback, seeks, envelope/mode changes, stale blocks and starvation; retain baseline evidence.
- [ ] Remove demonstrated avoidable heap reclamation from device iteration using established worker/queue ownership patterns, or retain a measured justification where no change is needed.
- [ ] Keep bounded memory and nonblocking device iteration; preserve generation rejection, exact output/mode/envelope behavior, silence on starvation, underrun counts and safe teardown.
- [ ] Exercise playback output and resource lifetime at confirmed seams; distinguish deterministic output/allocation probes from unperformed hardware listening or deadline guarantees.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. The owner confirmed the existing application, playback-output, shared preview/export envelope and benchmark/report test boundaries on 2026-10-21. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

### Red baseline — public playback consumer probe

The isolated integration test wraps System allocation with constant thread-local counters only around the public playback iterator's next call. Worker allocation, fixture setup, control commands, reporting, sleeping and assertions are outside the measured region. A 32,768-frame mono 48 kHz float fixture contains known constant samples. Sample oracles check Processed/Removed/Original and silence with a frozen playhead while awaiting worker data.

Baseline on the unchanged implementation: steady transitions (20,481 calls) allocated/reallocated/freed 0/0/9 times; envelope/listening edits (768 calls) 0/0/5; rapid seeks/stale blocks (159 calls) 0/0/318. The first zero-reclamation assertion failed as expected. Headless teardown separately observed 0/0/10. Observed call p95/p99 nanoseconds were steady 1,467/1,816, edits 2,374/3,353 and seeks 18,927/20,812; these are instrumented deterministic consumer-call timings under this build, not actual device callback deadlines, presentation or acoustic onset.

### Ownership change and second red/green slice

Consumed and rejected PCM/error payloads return to the existing reader through a four-item bounded channel. A failed nonblocking return retains ownership in one pending slot; iteration stops accepting new values until that slot is flushed. Full and disconnected errors never discard their owned payload. The worker drains at most four returns before reads and while waiting for forward-queue space, including stopped/finished states. A disconnected-worker error is prepared before device iteration instead of allocating its message there.

The queue capacity matches the four-item forward queue; an additional pending slot can retain the prior active block. This is a bound, not an assumption that the queue cannot fill. Once pending is occupied every consuming path returns, and the next call first retries that same ownership transfer. Normal starvation/backpressure emits silence and freezes the playhead; no worker join, file read, blocking send or unbounded spill list enters sample iteration.

PCM-only changes produced zero consumer frees for steady transitions and rapid seeks while the test next failed on three envelope frees. The completed envelope slice initializes the source from the current shared snapshot and routes both public/headless and native GUI writers through one control-side replacement helper. Source switching and replacement share the existing settings write lock. The sole source can hold only one historical snapshot; the writer retains it and prunes it only once the source no longer holds it. At most one historical envelope snapshot stays alive until the next envelope write or transport disposal. This bounded extra memory is the deliberate cost of moving its final release off iteration.

The completed probe passed: steady20,481 calls, edits11,008 calls (including40 bursts of two superseding envelope writes), and rapid seeks158 calls all observed **0 allocations, 0 reallocations and0 frees** on the consumer. Silent/frozen waits were1/0/118 respectively. Instrumented call p95/p99 nanoseconds were978/1,467;1,396/1,467;3,422/7,054. Scheduler/build-dependent tails are observations, not a controlled speedup or device deadline proof.

Headless teardown separately observed11 frees. Native rodio source destruction, channel implementation locks, actual device deadline/onset and listening quality remain outside this deterministic probe; no all-path allocation-free or hard-real-time claim is made. Existing orderly cancellation/join and source teardown semantics are retained. Scoped regression checks remain pending the coordinator's serialized Cargo window.

