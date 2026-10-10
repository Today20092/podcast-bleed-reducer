# Offline supervised CleanSlate architecture

Status: accepted architecture, 2026-10-08

2026-10-09 portable-alpha owner override: Nemotron is removed entirely from the
application, GUI and package, superseding all required/optional native diarization
language below. Acoustic ownership uses human labels and embedded Silero only.
Legacy native-analysis results become stale; human edits and labels remain intact.
The owner authorized an unsigned Windows 11 x64 public `v0.1.0-alpha.1` portable ZIP
with explicit incomplete quality qualification. Tickets 14/15 still gate stable
qualification; ticket 25 owns packaging and clean-machine checks. Historical native
experiments remain only in Git history and `docs/research/nemotron.md`.

This is the target architecture, not a description of completed code. Decision detail lives in the [map's child tickets](map.md); documentation findings live in [feasibility notes](feasibility-notes.md). Implementation begins after planning is settled.

## Product and release scope

2026-10-09 confirmed attenuation display refinement: uniform muted red background marks effective gain below unity, including fades and manual attenuation, in all audition modes. Full-volume spans remain clear; the waveform, purple gain curve, labels, protection and blue selection stay visible above it. This supersedes the earlier removal of attenuation backgrounds, while uncertainty decoration/hatching remains removed. Red means applied reduction, not absence of speech.

CleanSlate is a standalone, local editor for synchronized mono podcast tracks. Each speaker has one dedicated track. The first release targets Windows, up to eight speakers, and recordings up to two hours for acceptance testing. Preserve portable Rust boundaries for later macOS/Linux releases.

Use ownership baseline `6a60cc2bfe187ea889fd2d8bcc3344311cfd706b` and retain Iced. Keep earlier responsive code and rejected experiments recoverable; do not promote existing detector quality by inheritance. Start a fresh versioned project format. No existing saved project migration is required.

Required workflow: import and validate, assign speakers, mark solo references for each speaker and room tone, Analyze the whole recording, audition and adjust attenuation, correct labels or gain where needed, save an editable project, and export individual processed WAVs. Uncertainty review remains available without dominating the editing flow. Overlap labels are optional. Labels can include any active speaker set. One calibration setup covers a recording, with multiple examples across the show to handle small microphone movements. Poorly explained acoustic relationships become review warnings.

Acoustic-first supervised ownership is the default. Nemotron is an explicit per-project option, default off. When selected, its native runtime and pinned model are verified and run locally on CPU without Python or a GPU requirement; a failed selected model cannot silently switch to acoustic-only analysis. Acoustic-only analysis requires no Nemotron download or verification.

## Architectural boundaries

2026-10-09 owner refinement: remove the uncertainty review queue, review navigation/counts, orange dotted bands and red attenuation hatching from the editor. The user reviews the waveform, auditions and applies labels, attenuation or protection directly. Internal missing-evidence protection and export validation remain enforced. Provide a persisted **Ignore gaps shorter than** gain control, 10–5000 ms with a 768 ms default. It bridges only brief, bounded automatic attenuation gaps between preserved spans; it cannot erase explicit room-tone truth or protected manual edits. Changing it updates audition and export envelopes immediately from cached ownership, without rerunning analysis. Old analysis with fixed temporal bridging must be reanalyzed once before the new control can expose shorter gaps.

Keep the existing Rust library and Iced application. Add focused modules only where a real responsibility is missing; do not pre-create the ten-crate workspace sketched in the source chat. The analysis API does not depend on GUI widgets. The same library supports the editor and a small headless evaluation entry point.

The pipeline is:

1. Native-rate source decoding and validation.
2. Cached waveforms and 16 kHz analysis copies.
3. Per-track fixed speech/acoustic measurements, with optional Nemotron speaker activity.
4. Human-reference speaker mapping, room calibration and directed bleed evidence.
5. A small supervised session classifier producing independent local-speaker probability estimates per track.
6. Whole-timeline temporal ownership with human truth constraints and mode-specific evidence handling.
7. Automatic gain-envelope generation.
8. Protected manual edit composition into one effective envelope.
9. Display, Original/Processed/Removed audition and final WAV rendering.

Nemotron describes who speaks when. Per-track ownership describes whether a microphone's assigned person is speaking locally rather than being heard as bleed. Multiple tracks can own speech simultaneously. VAD probabilities are speech-presence evidence, not owner probabilities. Model estimates are not presented as proven calibrated confidence.

The original signal is never replaced by resampled analysis audio or bleed-subtracted audio. This product applies attenuation envelopes to decoded native-rate sources. Monitoring headroom remains a playback-only adjustment.

## Inputs and time coordinates

Support mono audio at decoded native sample rates, including 44.1, 48 and 96 kHz and mixed-rate sessions. Initial format profiles are WAV/BWF PCM, AIFF PCM, FLAC, MP3, M4A/MP4 AAC-LC or ALAC, and Ogg Vorbis. Use Symphonia with the required features for general decoding while retaining the existing WAV/BWF metadata path. Do not advertise Opus, HE-AAC or arbitrary codecs. Validate actual decoded channels/profile, not extensions. Reject corrupt/truncated data, invalid rates, nonfinite samples and unsupported profiles clearly.

The owner confirmed multichannel WAV/BWF import on 2026-10-09 in [ticket 21](../offline-supervised-editor/issues/21-multichannel-wav-import.md). Present the channels initially selected and allow deselection and naming. Each selected channel becomes a mono project track referencing the original container and its interleave index. Use iXML recorder track names with Channel N fallback; allow later renaming. Preserve native rates, frame counts and timing, and isolate source-bound decisions and caches by file plus channel. No split source WAVs are written on import. Channels from one container share a sample grid and establish internal synchronization even without BEXT; combining them with mono files retains the existing timing and supplied-alignment checks. Keep the eight-speaker limit. Processed individual WAVs remain an export operation.

Sources are supplied synchronized. There is no automatic alignment. Compare durations and timing metadata in physical time, not equal sample counts or raw BEXT integers. Matching BEXT values at different rates must normalize to matching times. Contradictory available timing blocks analysis/export. Missing metadata cannot establish synchronization: show alignment unverified and require the supplied-alignment declaration. Use at most one native-sample rounding tolerance for duration agreement; never silently stretch, trim or pad delivered sources to match.

Use a common integer time coordinate in the project and analysis API, with checked conversions to each source's sample grid. Native rates/sample counts stay explicit. Preserve half-open intervals and conservatively round preserved-region edges so quantization cannot shorten wanted speech. Display may use seconds. No module may use 48 kHz as an implicit global timebase.

Analysis copies are antialiased resamples at 16 kHz. Account for resampler startup delay, flush the tail and verify reference/feature positions against source time. Playback preparation resamples tracks to a common device rate outside the audio callback. Exports use each source's native rate and decoded frame count. Preserve WAV/BWF timing metadata where present and verified. Lossy inputs require decoded onset/duration checks; container support is not a gapless-timing guarantee.

## References and ownership analysis

A reference label is a source-bound time interval with an active speaker set. Solo labels identify one speaker; overlap labels identify multiple; room-tone references identify no speaker and supply noise calibration. Validate membership, bounds, source bindings, duration, usable speech/noise and finite feature values. A contradictory overlap between human labels requires an explicit replacement/edit; do not silently select the newest truth.

One valid solo reference per assigned speaker and a room-tone reference are required to run supervised analysis. Multiple references are encouraged when acoustic evidence changes. Insufficient or inconsistent examples block training with a specific correction request. Fixed neural feature extractors are not fine-tuned from labels. Do not automatically turn predictions into training truth in this release.

When Nemotron is selected, use human solo references to map its first-arrival speaker channels to assigned speakers. Require one-to-one evidence with ambiguity checks; unmatched or inconsistent channels remain unresolved and cause review/preservation, not a guessed assignment. Keep one native diarization stream across the recording. Copy probability frames after bounded input pushes using global frame positions before the runtime compacts old values. Persist its complete probability history, including the final tail. Acoustic-only analysis learns directly from assigned tracks and human labels; it neither requires nor fabricates anonymous speaker channels. Full-attention one-shot native inference is not the two-hour processing route.

Build a deterministic nonclipping analysis-only mono mix. Compare candidate downmix recipes on development fixtures and freeze the chosen recipe before independent acceptance evaluation. The documentation does not establish one optimal multitrack mix.

Reuse existing room-floor, directed scalar leakage and pairwise evidence as the initial acoustic baseline. Add regularized offline FIR/transfer evidence only if development validation shows improvement without wanted-speech loss. A poor or changing acoustic fit must reduce trust in that feature. It must not suppress a speaker simply to make bleed prediction fit. No acoustic estimator subtracts from delivered audio.

The first learned ownership model is independent binary logistic regression per assigned track. Use a maintained pure Rust implementation for nontrivial training rather than writing a new optimizer. Features combine mapped Nemotron activity, raw VAD, local noise-relative energy, relative dominance, bleed explanation/residual evidence and fit reliability. Freeze normalization, feature recipe and regularization through development validation. More complex classifier families and blind source separation are outside this release.

Temporal ownership uses whole-track context while treating human active-speaker labels as constraints. It may retain multiple owners. It must not override a label to satisfy a minimum-duration heuristic. Acoustic-only analysis uses independent fitted ownership evidence throughout every track; native mapping ambiguity and a poor bleed-fit feature cannot blanket-veto acoustic absence predictions. Missing or invalid evidence remains preserved/actionable. Selected Nemotron retains explicit uncertainty for unresolved identity and contradictory evidence. The more aggressive acoustic policy requires recording-specific listening and independent preservation qualification.

## Cache, jobs and invalidation

Cache independently: source identity/decoded metadata and waveform; fixed 16 kHz/VAD/Nemotron evidence; reference-derived calibration and model; owner probabilities; optimized regions; automatic envelopes; effective edited envelopes. Keys include source identity, rates/decoded lengths, decoder/resampler recipe, model/runtime hashes, analysis mix, label set, speaker mapping and relevant parameters.

Changing labels or speaker assignment marks analysis stale and waits for explicit Analyze. Reuse compatible fixed evidence; retrain the small model and regenerate derived results. Gain/fade/protection/manual-edit changes update the effective envelope immediately without neural reinference. Source or fixed-model changes invalidate their dependent evidence. Do not silently upgrade model versions for an existing project.

Keep the last completed preview marked stale while a replacement is pending or required. Original playback and editing remain available. Final rendering/export requires current analysis and effective envelope. Cancelled, failed or obsolete jobs cannot install their results, invalidate a newer result or overwrite an accepted deliverable. Use bounded worker processing and check cancellation between chunks. Avoid retaining multiple full-session PCM/resampled/residual copies.

## Editable envelopes and persistence

Labels teach detection. Manual gain edits control final sound and remain authoritative until explicitly removed, even when later analysis disagrees. Represent automatic and manual data separately. A unity protection is a manual preserve constraint; it is not an arbitrary curve edit. Whole-track protections override automatic attenuation. Creating a conflicting manual edit or protection requires explicit replacement of the overlapping manual constraint, rather than an invisible precedence rule.

Provide draggable points and fades plus Preserve/Attenuate region actions. Group one drag or region action as one undoable command. The default is attenuation-only, with unity as the maximum; boosting and full DAW automation are not added. Empty overrides expose the regenerated automatic envelope. Finite/bounded values, ordered times, endpoint behavior and source binding are validated on load and edit.

2026-10-09 owner transition refinement: use finite S-shaped automatic fades with explicit start/end times and exact target endpoints, superseding exponential-shaped fades. Quintic smootherstep has zero slope and curvature at both endpoints to soften transitions into and out of constant gain. The goal is transparent attenuation without audible clicks, pumping or obvious ducking; numerical smoothing does not guarantee inaudibility, which requires listening approval. Defer a fade-shape selector. Preserve owned speech, explicit human speech/overlap and invalid-evidence protection at exact unity absent manual overrides. Treat requested fade lengths as maximum durations; fit them proportionally inside shorter detected non-speaking gaps instead of merging those gaps into unity. Never shorten a preserved span to fit a fade. Manual edits may intentionally change labeled speech and retain linear interpolation.

Store project schema version, stable track/speaker identities, source fingerprints/metadata, reference labels, settings, manual edits/protections, review state, analysis recipe/model provenance, current or stale result identity and render identity. Derived cache files are versioned and rebuildable; a project must remain editable if a cache is missing. Save atomically and retain prior data on failure. A moved identical source can relink; a changed source requires revalidation and explicit review of bound labels/edits before they apply. Never silently transfer decisions to different audio.

One effective envelope evaluator drives waveform/gain display, audition, removed-signal calculation, headless evaluation and final rendering. Final output is a deliberate safe transaction; keep existing destination collision checks, cancellation, validation and atomic completion.

## Editor behavior

2026-10-09 owner follow-up: per-track Solo controls select any combination for live audition in Original, Processed and Removed. No active solos means all tracks. Solo is temporary monitoring state, resets on session loading, and never changes project gain edits or export output. Continuous Lock to center scrolling must be measured with waveform and gain drawing together; gain-strip-only timings cannot establish full-frame smoothness.

2026-10-09 owner refinement: provide Off, Page-at-edge and center-locked playback scrolling at the current zoom. Manual horizontal panning disables following. Group reference/analysis, sound and export controls separately and collapse model/source setup while keeping blockers visible. Remove whole-track-protection creation buttons from the GUI; existing saved protections retain their meaning. Improve Ctrl-wheel redraw latency without changing its travel distance, exact gain bounds, source time or export behavior.

Retain one synchronized Iced workspace, waveform cache and transport. Add speaker assignment, label selection with active-speaker checkboxes, reference readiness, Analyze progress/cancellation, stale-state display, ownership estimates, uncertainty review navigation and a separate manual gain tool. Do not make users navigate a staged wizard. Model acquisition is an explicit first-use action with progress, expected identity/license information and actionable errors.

Review items show affected time/tracks and reasons such as uncertain identity or poor bleed fit. Clicking seeks/selects without losing transport context. Show numerical estimates as analysis evidence rather than a claim of guaranteed confidence. Retain Original, Processed and Removed comparison. Distinguish automatic envelopes, protected manual edits and active labels visually. Provide keyboard operations for core selection/label/edit actions.

## Confirmed owner workflow refinement

2026-10-09 feedback after the first GUI candidate supersedes the required-Nemotron default above for development and subsequent implementation: persist an explicit Use Nemotron option, default off. Off runs acoustic-first supervised ownership across every track for the whole session, trained only from human labels and fixed per-track speech/acoustic measurements; it requires no Nemotron verification or anonymous-channel mapping. On explicitly selects the verified Nemotron-assisted path. Analysis mode participates in source/result/cache identity, and changing it requires Analyze. No missing/corrupt selected model may silently switch modes.

The label popup is one active-speaker multiselect: no selected speaker means room tone, one means solo, several means overlap. Show the complete bounded menu without its own scrollbar. Numeric keyboard shortcuts toggle selections; Enter saves, R clears, Escape/outside dismiss. Analysis shows the actual stage and advancing elapsed time instead of a fabricated total percentage. Existing measured stage markers may support the display but cannot claim an inference completion estimate.

Ownership prediction and attenuation run across the entire source timeline, including unlabeled spans. Labels are training examples and explicit truth constraints, not processing limits. Acoustic-only decisions use the fitted per-track ownership evidence without an unresolved Nemotron mapping or poor-fit feature automatically vetoing every absence prediction. Invalid/missing evidence remains actionable; trusted human speech/overlap and explicit manual edits remain authoritative. This more aggressive development policy requires owner listening and independent preservation/bleed qualification before release.

2026-10-09: The owner confirmed [the GUI and attenuation design](gui-design-summary.md). [Build ticket 16](../offline-supervised-editor/issues/16-gui-editing-workflow.md) consolidates the UI work: ordinary click seeks; dragging selects a track-bound range and immediately opens pointer-adjacent solo/overlap/room label actions, with captured input, dismissal and keyboard access. No selection mode should prevent seeking. Labels describe the session-wide active-speaker set. Use concise readiness/next-step guidance and compact installed-model status with expandable details.

For compatible validated tracks whose start timing is unverified, Original audition is available before supplied-alignment confirmation, model setup or references. Actual timing conflicts remain blocked; Analyze/export still require the existing alignment contract. Start cancellable playback preparation when validated tracks open, reuse compatible prepared audio and show progress. Share per-source waveform caches across projects using validated source identity and recipe/version checks; retain user projects, sources and legacy cache files.

The owner authorized acoustic-first supervised analysis with optional Nemotron, using a few clear representative references and adjustable attenuation. Implement this mode within existing ticket 14, preserving explicit labels and source/output safeguards. Independent wanted/overlap, bleed and listening release checks remain required; the new default does not claim those gates have passed.

## Release acceptance

Acceptance data is local and source-authorized; do not publish recordings. Freeze development/training/tuning data separately from independent test sessions/regions, including the historical screenshot failure, quiet speech, interjections, laughter, overlap, room tone, movement and changing bleed delay. Adjacent frames from the same labeled episode cannot create an artificial train/test split.

On independent labels, wanted speech and overlap retain exact unity unless intentionally manually edited. Any detected loss is a release failure. Compare bleed attenuation against the frozen current detector on the same test material and require measurable improvement without trading away preservation. Keep numerical metrics separate from listening approval. The owner signs off on transitions, removed-signal audition and natural overlap before release. This is a bounded acceptance claim, not proof of perfect behavior on every recording.

Verify every advertised input profile; mixed native rates; metadata agreement/conflict/missing cases; first/last sample timing; per-source WAV frame/rate integrity; shared envelope parity; label invalidation; source relinking; protected edits and undo/save/reload; stale/cancelled jobs; corrupt/missing models; native runtime loading; and network-disabled analysis after the first download.

Measure cold/model-load/full analysis time, peak memory, disk cache growth, cancellation response and warm gain-edit latency on the owner's Ryzen 9 3950X / 32 GB Windows machine. Exercise eight tracks and two hours, including native Nemotron probability compaction and long-gap speaker recurrence. No hard cold-analysis deadline has been agreed; obtain the owner's timing acceptance from measured results before release. Warm gain edits must reuse evidence and avoid inference; long-session UI/transport must remain usable.

## Deferred work

Live/DAW plugins, automatic alignment, shared microphones or speaker track switching, more than eight speakers, mixed-master/DAW automation export, macOS/Linux release qualification, a fade-shape selector, alternative classifier families, blind source separation and neural fine-tuning are deferred. Unlisted codecs are not implicitly supported. These boundaries do not prohibit extending the shared engine after this release is accepted.

## Implementation sequence

The [implementation index](implementation.md) links individual build tickets. They identify affected modules, dependencies and meaningful acceptance checks; they do not mark any implementation as complete.
