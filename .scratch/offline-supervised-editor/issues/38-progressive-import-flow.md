# Make import responsive with early timelines and progressive waveforms

Status: claimed
Type: task
Assignee: Codex implement_38
Blocked by: 03, 21, 24, 33
Target branch: codex/editor-workflow-31-37
Spec: #accepted-design

## Scope

Refine import on `codex/editor-workflow-31-37` as one implementation ticket. Build on its completed editor workflow, import dialog and themes; do not restart from `main`, create another integration branch or merge into `main` as part of this ticket. The owner wants to continue refining this branch before deciding it is ready to merge.

After choosing channels, the user must see navigable timeline skeletons without waiting for full waveforms or full validation. Provide a rough overview across the whole recording first, refine it in the background, show truthful progress, reduce duplicated source work and support queued imports with independent recording readiness and cancellation.

This accepted design supersedes ticket 33's requirement to keep every import stage in a blocking centered dialog. Retain a focused channel-selection dialog, then return to the workspace for background import. Previously accepted tracks remain available. Synchronization remains the existing already-aligned-input contract; automatic alignment and drift work are deferred.

## Problem and verified evidence

Owner reproduction: `test-audio/261005-T001.WAV`, approximately 1.85 GB, three float32 channels at 48 kHz, 154,128,376 frames and 53:31 duration. The supplied development executable showed Loading waveforms without useful progress, revealed complete waveforms one at a time, then spent more time Validating audio. Its loading dialog displayed 0 selected after confirmation.

Read-only investigation of the current branch found one background worker per import request, sequential channel waveform loading, complete-waveform events only, and full validation after waveforms. Cold import of three channels performs three full-container waveform decodes, three validation decodes, three whole-file cache-key hashes and three source-identity hashes. The existing pyramid accelerates drawing after base peaks exist; it is not a progressively published preview. Cancellation prevents publication but the decode/validation calls do not receive a cancellation flag, so computation can continue.

Code entry points: `src/app.rs` import worker, source-loading polling and modal routing; `src/waveform.rs` source loading/cache/pyramid; `src/audio_io.rs` chunk decoding; `src/engine.rs` validation; `src/project.rs` source identity and atomic persistence. Rediscover current symbols through the graph before editing; these pointers are not frozen line references. Investigation notes are available locally at `C:/Users/User/.codex/import-flow-grilling-facts.md`; all required behavioral scope is recorded here.

No timing benchmark has been performed. The supplied executable is a partly optimized development build (application opt-level 1; decoding/hashing dependencies opt-level 2), not a release build. Repeated work and missing feedback are verified independently of build optimization.

## Accepted design

1. **Early workspace.** Obtain enough metadata to choose channels, names, duration and native sample grid before expensive waveform work. On confirmation, display selected track rows and timeline bounds immediately. Reserve rows for queued recordings too, with an explicit Queued state. Preserve submission/channel order as jobs finish; readiness must not reorder the timeline. Do not wait for full decoding, hashing, validation or detailed waveforms to show skeletons.
2. **Progressive overview.** Show a rough preview across the whole recording as soon as possible, then refine waveform detail in the background. Identify provisional/coarse previews honestly. Navigation and drawing remain responsive, and progressive updates use bounded work/event frequency. A coarse preview never establishes source validity or permits unsafe audio operations.
3. **Operation readiness.** Scrolling, zooming and inspecting status work immediately. Enable playback and editing only when the required source/audio is ready; waveform-detail completion alone must not gate otherwise-ready operations. Each fully validated recording becomes usable independently while other recordings continue. Selected channels from one multichannel WAV share that recording's readiness and validation outcome. Preserve full source/session compatibility checks before acceptance.
4. **Feedback.** Show overall import stage/activity and per-track status, including queued, preview/refining, validating, ready, failed and cancellation where applicable. Maintain accurate selected/existing/pending counts. Use percentages only for work with a real byte/frame denominator; otherwise show an active indicator. Do not fabricate an overall percentage or ETA. Distinguish a usable track with detail still refining from an unvalidated track.
5. **Queueing.** Allow additional recordings to be selected and queued while import work is active. Apply the existing eight-track limit to accepted and reserved pending tracks, with clear admission feedback. Use bounded source-level work and memory, not an unbounded thread per channel. Choose concurrency from measured bottlenecks; channels sharing a container should share decoding/identity work where compatible. Do not silently drop queued requests.
6. **Failure.** Keep failed provisional rows visible with actionable error and Retry/Remove controls. Do not install invalid source data or overwrite the accepted project on failure. Retry belongs to that recording and must obtain current source metadata/identity, without allowing obsolete results to publish. Failures in one recording must not discard independently completed recordings or edits.
7. **Cancellation.** Provide Cancel/Remove for an individual pending recording and Cancel all pending. Discard unfinished provisional work, stop its background computation promptly between bounded chunks and release resources. Keep already completed recordings and edits. Source replacement, New project and stale job generations must reject late results. Preserve safe atomic publication: if a short final commit cannot be canceled, indicate that honestly and do not claim a cancellation rolled it back.
8. **Playback stability.** A newly ready recording must not alter an already-playing mix or stop its playback. Keep that playback's accepted source snapshot intact; include the new recording the next time playback starts. Existing listening mode/Solo, label-loop behavior and source/job safeguards remain correct. Explain deferred inclusion where needed so a visible ready track is not mistaken for a track already in the current mix.
9. **Project and export safety.** Accept and persist completed recordings atomically against the current live project, preserving intervening edits to accepted tracks. Unfinished/failed provisional rows must not enter saved project source bindings, analysis or exports. Save/export must accurately describe and operate on the accepted recording set, without silently including pending audio. Continue to enforce native grids, alignment, finite samples, supported formats/profiles, corruption/truncation detection, source identity, cancellation and current-analysis/export safeguards.
10. **Speed and reuse.** Measure the actual stages, then reduce repeated whole-container decoding and hashing. Reuse compatible decoded metadata, identity, validation and waveform work across selected channels and downstream import stages while maintaining trust boundaries and cache invalidation. Preserve compatible shared caches and measure cold versus warm paths. Avoid multiple retained full-session PCM copies. A release build alone does not resolve this ticket's UX or duplicated-work requirements.

## Acceptance checks

- [ ] Metadata/channel selection appears before full waveform work; confirming selections displays correctly named/ordered skeletons and native timeline bounds on the next usable GUI update, without waiting for a full source scan. Selected/pending counts remain accurate.
- [ ] A whole-recording coarse preview is published before detailed waveform loading completes on the owner reproduction; subsequent refinement updates existing rows without blocking navigation, resetting selection/playhead or changing semantic waveform/gain/label colors. Cache hits also provide early usable presentation.
- [ ] Overall and per-track feedback visibly update throughout decoding, hashing/validation and refinement. Known byte/frame progress is measured; unknown work shows activity. Both Dark/Light and narrow/wide views remain readable and keyboard accessible.
- [ ] Separately imported recordings become ready independently, with shared readiness for channels from one container. Validated/audio-ready tracks can be used while detail or other imports remain pending; invalid/pending tracks cannot perform unsafe operations.
- [ ] Additional imports queue in stable order while loading; reservation/admission checks preserve the eight-track limit. Ready events do not reorder rows or overwrite edits made during import.
- [ ] Failed rows retain actionable Retry/Remove controls. Tests cover malformed/truncated/nonfinite/unsupported inputs, source changes on retry, incompatible timing and one recording failing while another succeeds. Accepted project data and persisted bytes remain safe.
- [ ] Individual cancellation and Cancel all pending stop work promptly at bounded checkpoints, remove unfinished provisional rows and retain completed recordings/edits. Regression tests cover cancel during preview, validation and refinement, queued cancellation, New project/source replacement, late results and atomic publication boundaries.
- [ ] Import readiness during playback preserves the active mix/source snapshot and transport/playhead. New recordings participate on the next start. Tests include Original/Solo and ticket-31 label-loop/stale-preparation interactions.
- [ ] Save/analysis/export use only valid accepted recordings and current identities/envelopes, with accurate feedback about pending audio. Theme changes and editing of accepted tracks during import preserve project/audio behavior.
- [ ] Record before/after cold/warm stage timings, time to channel choices, skeletons, whole-recording coarse preview and usable recording, total import time, decode/hash traversal counts, peak memory and cancellation latency for `261005-T001.WAV`. Record build profile and compare the same source/settings; include a release-profile comparison before attributing performance to threading. Demonstrate reduced duplicate work and improved readiness latency, without weakening validation. The owner recording is local and must not be committed or uploaded.
- [ ] Add meaningful state-machine/worker/cache/progress regressions; run appropriate Rust tests, formatting and strict all-target/all-feature Clippy. Build and visually review through `scripts/gui-dev.ps1`, including queue/failure/cancel/readiness and playback scenarios. Record unavailable physical checks and baseline differences honestly.

## Out of scope

Automatic synchronization, drift correction/checking, new audio formats, split source WAVs, analysis algorithm changes, mixing/master export, updater/diagnostic features, waveform automation-overlay redesign, arbitrary worker settings and stable release qualification. Do not persist resumable unfinished import queues across application restarts in this scope. Tickets 14/15 retain their independent release gates.

## Results

Implementation in progress on `codex/editor-workflow-31-37`, claimed by Codex. Not done; acceptance checks remain open.

2026-10-12 implementation checkpoint: added transient recording jobs and ordered provisional timeline rows, sampled whole-recording WAV previews, bounded progress/refinement events, shared selected-channel decoding and container hashing, live-project atomic acceptance, retry/remove/cancel controls and reservation limits. Pending/failed sources remain outside the project. Readiness invalidates stale preparation/analysis without stopping active playback, and envelope replacement rejects a changed source count. Metadata inspection hands its results to GUI skeleton creation; retry rechecks metadata in its worker. Added worker/state regressions and an opt-in before/after benchmark.

Verification before interruption: `cargo check` passed and the existing app suite passed 45 tests with 2 ignored. A subsequent broader intermediate run passed 102, failed 4 and ignored 7. Three failures were new test fixtures passing a channel count where the fixture expects a WAV format tag; these fixtures were corrected. The other failure was an existing shared-cache publication race during concurrent tests. A serial suite was started after corrections, but its final result has not been reviewed. Later media/preview/playback changes still require final compilation, tests, formatting and strict all-target/all-feature Clippy.

GUI verification was stopped by the user through the Computer Use physical Escape control. No further GUI actions were attempted. `scripts/gui-dev.ps1 -BuildOnly` refused to overwrite the running development executable as designed. The existing CleanSlate window remains open. Full recording cold/warm timings, release comparison, peak memory, cancellation latency and requested visual/playback scenarios remain unverified. No source recording was committed or uploaded, no merge was performed, and unrelated workspace edits were preserved.

## Comments

2026-10-12: Codex claimed implementation on the existing target branch after verifying dependencies 03, 21, 24 and 33 are done. Existing unrelated workspace edits will be preserved.

2026-10-12: Owner confirmed the import-flow interview choices Q1–Q11, including queued imports, independent recording readiness, cancellation retaining completed work and stable current playback. Owner explicitly requested one build ticket on top of `codex/editor-workflow-31-37`, with continued refinement there before merging into `main`. This record creates no implementation or merge claim.
