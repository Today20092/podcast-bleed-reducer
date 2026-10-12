# Make ready-session playback and selection audition start promptly

Status: ready-for-agent
Type: task
Assignee: unassigned
Blocked by: 04, 31, 38
Target branch: codex/editor-workflow-31-37
Spec: #accepted-design

## Scope

Remove avoidable startup work when playing or changing a selection in an already prepared session. Preserve selection/label-loop behavior and the accepted-source playback snapshot introduced by ticket 38. Continue implementation on `codex/editor-workflow-31-37`; do not restart from `main`, create another integration branch or merge into `main` as part of this ticket. Dependencies 04 and 31 are done; 38 is still claimed, so implementation is blocked until its acceptance checks pass.

## Problem and local evidence

The owner sees Preparing audio when auditioning selections and expects the prompt response of a DAW. Current code evidence, not a timing measurement:

- CleanSlateApp::audition creates a pending audition range and requests source preparation when no current request is outstanding.
- AuditionPreparation::request_sources cancels the previous request generation and dispatches a worker. Its warm path can reuse PreparedAudio when source stamps and the output rate match; the message does not establish that the recording was decoded again.
- PreparedAudio::prepare_sources_for_rate checks caches and prepares compatible playback data when needed. Its key covers sources and output rate, not the selected range.
- LiveAudition::start opens the default output stream, creates a Sink and starts a new bounded source-reading worker for each playback instance. The instance holds the stream lifetime.
- GUI Preparing audio text also covers a pending start from cached data, before poll_audition receives the worker result and starts output.

The relative contribution of worker dispatch, GUI polling, source-buffer priming and device opening has not been measured. Do not describe an assumed bottleneck as verified or promise physical zero latency.

## Accepted design

1. Measure the existing path first. Record cold preparation and repeated warm play/stop/selection/seek/loop-start timings separately, including input dispatch, source cache lookup, conversion, GUI result delivery, output stream opening and first output-buffer availability. Use the same source, settings, build profile and output device for comparisons. Record median, p95, maximum, underruns and retained memory. Separate application startup time from the hardware/device buffer contribution and physical audible onset.
2. Reuse validated prepared source audio for a ready session when only selection start/end, playhead, loop range or monitoring settings change. Warm starts must not rescan/hash/decode/resample the full recording or rerun analysis. Preserve identity/stamp, source-generation and output-rate safeguards; a changed source/device rate requires appropriate invalidation or preparation.
3. Keep the output stream available while the prepared session is open, across ordinary stop/start and range changes. Use the existing Rodio/CPAL stack and the smallest reusable controller that removes repeated device initialization and unnecessary warm preparation handoffs. Reuse the existing transport/source worker where suitable; rebuild a lightweight source only if measured necessary. This ticket does not require a new audio graph framework or a dependency change.
4. Ready-session playback/selection commands should update position, range and loop controls without waiting for the GUI's preparation-result polling path. Disk reads and buffer priming stay off the GUI and audio callback; storage work and queues remain bounded. Keep callback work predictable and avoid blocking device output on GUI locks or file I/O. Initial preparation remains cancellable and honest.
5. Show Preparing audio only during actual missing/stale audio preparation or conversion. Distinguish buffer startup if a visible delay genuinely remains, and delay short-lived status presentation to avoid misleading flashes. Do not hide a long wait or claim readiness before the source/buffers can be used safely.
6. Preserve Original, Processed and Removed monitoring, Solo combinations, monitor gain, sample-grid/range boundaries, selection-once playback and ticket-31 popup-bound Original loops. Retain popup close/restore, Escape, pause/resume and stale-preparation rejection. Selecting a new range must retain existing product behavior; this ticket does not change when playback starts automatically.
7. New recordings becoming ready must leave the active mix/source snapshot and playhead intact. They participate on the next start as required by ticket 38. New project, source replacement, device loss, preparation failure and obsolete job generations must stop or invalidate the affected output safely and release resources. Failed device output offers actionable retry instead of an infinite restart loop or a false Ready state.
8. Evaluate the [primary-source research](../../../docs/research/selection-playback-latency.md). Reuse installed APIs and architectural patterns first. If copying third-party source is actually proposed, record the exact revision/file, its applicable license, attribution obligations and compatibility with this MIT application before copying. A functioning demo, repository popularity or Rust implementation does not establish latency, stability or licensing suitability.

## Research references

The [research note](../../../docs/research/selection-playback-latency.md) cites version-pinned Rodio APIs, CPAL, Firewheel's realtime command architecture, Creek's cached seek/loop buffering and Dropseed as a Rust DAW example. Firewheel and Creek identify MIT/Apache licensing; Dropseed identifies AGPL and is a reference only, with no copying proposed. These examples do not establish measured performance or justify switching this application's engine.

## Acceptance checks

- [ ] Before/after stage timings are recorded for cold start and repeated warm selection playback on the owner machine; software first-buffer and physical audible-start measurements are distinguished.
- [ ] A proposed target is warm command-to-first-output-buffer p95 of at most 100 ms on the owner's current device configuration. This is an engineering target, not an owner-approved universal latency guarantee. Report any miss with stage evidence and seek owner acceptance of measured responsiveness; do not quietly weaken the target or tune global OS/device settings.
- [ ] Warm commands incur zero whole-recording decode/hash/resampling traversals and zero new device-stream opens while source identity/output configuration remains current. Verify counts through meaningful instrumentation or a fake backend, not only a timing assertion.
- [ ] Tests cover cache hits/misses, source changes, mixed native rates, device-rate changes/failures and cold cancellation/stale results. No unbounded queue or retained full-session PCM duplication is introduced.
- [ ] Transport tests verify exact selection endpoints, repeated seek/play/stop, popup loop replacement and close/restore, pause/resume, monitoring mode/Solo/gain and obsolete source generations. Import-ready events preserve the current playback snapshot; the next start uses the new accepted sources.
- [ ] Preparing audio accurately reflects actual preparation; repeated ready-session selection starts do not flash that status solely because of cached worker/poll delivery.
- [ ] Run appropriate Rust tests, formatting and strict all-target/all-feature Clippy. Build through scripts/gui-dev.ps1 after the owner closes the running development executable.
- [ ] Ask the owner to test repeated selections, Original/Solo, Processed/Removed where ready, label-loop changes and stop/start, reporting perceived delay and any clicks/dropouts. The owner is using the computer; do not use Computer Use unless subsequently requested. Mark unavailable physical checks explicitly; automated first-buffer evidence does not establish listening acceptance.

## Out of scope

Recording, live input monitoring, plugins, ASIO/backend selection, user buffer-size settings, a new DAW/audio graph, new codecs, synchronization/drift work, analysis changes, export changes, dependency migration and stable release qualification. Tickets 14/15 retain their independent gates.

## Results

Pending. Specification created at the owner's request; no playback implementation or latency benchmark is claimed. Implementation must wait for ticket 38 and be claimed before editing. Research references are collected separately; no third-party code has been copied.

## Comments

2026-10-12: Owner requested a ticket for reusable ready-session playback and invited Rust FOSS DAW/audio research and clarification if necessary. Existing selection and popup semantics are retained, so no new product interview is needed for this scope. The proposed 100 ms software target and physical listening check remain to be validated against measured device behavior.
