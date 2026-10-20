# Make ready-session playback and selection audition start promptly

Status: superseded
Consolidated into: 23
Type: task
Assignee: Codex
Blocked by: 04, 31, 38
Target branch: codex/editor-workflow-31-37
Spec: #accepted-design

## Consolidation — 2026-10-20

Retained ready-session playback and recorded software verification are integrated into development. Remaining GUI/listening/device scenarios and unresolved timing/measurement obligations now live in [23](23-gui-frontend-polish.md#current-remaining-work--2026-10-20). Preserve the original specification, measurement record and limitations below; first-valid-PCM timing is not hardware onset or whole-buffer qualification. This closes a duplicate queue entry without claiming physical acceptance. The historical dependency on 38 is retained as provenance, not an active execution dependency for this closed record.

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

Final software verification, 2026-10-12: implementation committed on `codex/editor-workflow-31-37`. Ready starts reuse validated prepared audio, the output stream and bounded worker; range/seek commands bypass preparation polling. All 263 tests passed across 33 suites, with 18 ignored. Strict all-target/all-feature Clippy, formatting, GUI BuildOnly and Release build passed. Both reviews have zero remaining source findings after completion/position and pending-seek fixes. The opt-in device benchmark passed, including a simulated CPAL error signal and exactly one stream open on explicit retry.

The [measurement record](../../../docs/research/selection-playback-40-measurements.md) distinguishes source-callback readiness from hardware audible onset and unavailable GUI/conversion stages. Quiet Release warm p95 was 21.57 ms versus baseline 102.57 ms, meeting the proposed software target at this marker. No steady-state underruns occurred in the sampled 350 ms intervals; startup missing-buffer counts and measurement limits are recorded. Physical listening/device-unplug checks remain pending, so keep this ticket claimed rather than declaring release or listening acceptance.

The owner requested help cleaning disk space. Cargo package-scoped debug cleanup removed 16.8 GiB, then another 9.2 GiB after verification; final C: free space measured 72,055,238,656 bytes, about 67.1 GiB. These are Cargo's artifact-size counts, not a claim that every change in volume free space came from this cleanup. Source, recordings and projects were preserved. Old and new debug/Release executables were backed up under `C:/Users/User/.codex/build-preserved/podcast-automix-rust-gui`; current debug and Release paths remain available. No further cleanup approval is pending.

Current test executable: `C:/Users/User/.codex/worktrees/progressive-import-38/podcast-automix-rust-gui/target/release/rust-app.exe`. It contains ticket 40; the worktree name reflects the retained build cache, not a different implementation branch. The original ticket-38 Release binary remains backed up as `import-38-release.exe`.

2026-10-12: Claimed after the owner tested ticket 38's latest Release executable, reported "its good so far" and explicitly requested implementation of ticket 40. This authorizes proceeding from that tested import baseline; it does not claim exhaustive GUI/listening or release qualification. Use the accepted prepared-audio and transport interfaces for regression tests of reuse, ranges, cancellation and source snapshots. Research references are collected separately; no third-party code has been copied.

## Comments

### Earlier implementation checkpoints

Implementation checkpoint: ready cache hits bypass the preparation worker and GUI result poll; output and the bounded source worker are retained across stop/start/range changes. Complete range commands are applied by the device iterator, obsolete blocks are rejected, and completion waits for command acknowledgment. Pending ranges report their new start using the full prepared grid. Device errors invalidate output and offer explicit Play retry. Preparation/buffering messages are distinguished and delayed by 150 ms. No dependencies or third-party source were added.

Verification so far: audition and app suites passed (10 audition tests and 48 app tests, with opt-in benchmarks ignored); strict all-target/all-feature Clippy passed before the final review fixes. Both review findings were fixed with persistent-transport regression coverage; reviewer source rechecks found no remaining findings. Later rate-cache regression and final changes still require the complete verification run. The first full debug attempt hit Windows LNK1318 in `ownership_diagnose`; the documented GUI-profile retry then hit disk exhaustion. C: has roughly 148 MB available and this task's generated debug symbols total 11.14 GB. Direct symbol cleanup was rejected by automatic approval review as "blocked by policy"; no cleanup occurred and owner approval for package-scoped Cargo cleanup is pending. Existing EXEs, audio and projects remain intact.

Measured on the owner's Focusrite USB Audio output at 96 kHz, Release profile, original three-channel `261005-T001.WAV`, muted output and existing disk caches: baseline 23 repeated warm starts had software first-buffer median 89.42 ms, p95 102.57 ms, maximum 103.03 ms. Retained transport's 23 warm starts measured median 20.06 ms, p95 29.17 ms, maximum 29.27 ms. Instrumented warm work counted zero full-source preparation entries, one initial stream open and one initial worker across all 24 range starts; subsequent starts opened neither stream nor worker. Initial stream start measured 130.77 ms baseline versus 237.33 ms retained; initial in-memory preparation measured 6.13 s versus 7.53 s with existing on-disk caches. These are not cold-disk conversion benchmarks. Timings overlap build activity and need a quiet final repeat. The retained run counted 23,986 missing-buffer sample requests during repeated startup/priming; steady-state underruns were not isolated. GUI input/worker-result delivery and physical audible onset were not instrumented, so neither is inferred from these headless software measurements. Final owner listening checks remain unavailable.

2026-10-12: Owner requested a ticket for reusable ready-session playback and invited Rust FOSS DAW/audio research and clarification if necessary. Existing selection and popup semantics are retained, so no new product interview is needed for this scope. The proposed 100 ms software target and physical listening check remain to be validated against measured device behavior.
