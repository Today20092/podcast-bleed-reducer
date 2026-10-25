# Explore live and DAW CLAP/VST3 bleed reduction

Status: superseded
Consolidated into: 85-smart-gate-plugin.md, 86-podcast-plugin.md
Type: grilling
Assignee: Codex
Blocked by: none
Implementation readiness: continuing product scope and outstanding checks are owned by 85/86

2026-10-25 owner-approved split: [85](85-smart-gate-plugin.md) continues single-input live gating; [86](86-podcast-plugin.md) continues multitrack post-production. [87](87-separate-plugin-products-workspaces.md) preserves the prototype in independent product workspaces. Keep this discovery and its results as history; superseded does not mean outstanding acceptance passed. Existing 80 and blocked 81–83 remain open under Podcast. This replaces the combined Session/Live Mic product direction, not prior safety requirements.

2026-10-24 target refinement: the owner chooses broad desktop coverage first: VST3 and CLAP on Windows/macOS/Linux, retaining VST2 as a legacy follow-up subject to adapter/licensing verification. AAX/Pro Tools and AU/Logic are outside the current scope and can be considered if future users request them. [77](77-headless-plugin-checks.md) owns lightweight autonomous headless validation so routine work does not require a full DAW or repeated owner attendance. Cross-platform native packages/checks are still required before claiming those platforms work.

## Owner request — 2026-10-22

Investigate a possible CLAP or VST3 plugin that uses CleanSlate's machine-learning bleed reduction during live streaming and in post-production DAWs, including DaVinci Resolve/Fairlight and FL Studio. Determine whether bounded lookahead can make the current approach usable live, what must change, and what audio/routing/latency tradeoffs the owner accepts.

This authorizes discovery, factual research and an owner interview. It reopens the previously deferred plugin topic for design only; it does not authorize implementation, select a framework, promise host compatibility, or replace the accepted offline application architecture. Both live and post-production uses remain goals until the interview establishes their priorities.

References: [active map](../../offline-supervised-migration/map.md), [accepted offline specification](../../offline-supervised-migration/spec.md), [current architecture](../../../docs/adr/0006-offline-supervised-editable-envelopes.md), [product opportunities](../../../docs/research/podcast-bleed-product-opportunities-2026-10-22.md).

## Questions this discovery must settle

- Product priority and intended listening path: broadcast output, performer monitoring, recorded-track playback, or a combination.
- Dedicated microphone/track assumptions; how every microphone reaches the same ownership decision, host bus/sidechain support, outputs and multiple-instance coordination.
- Calibration workflow: user-confirmed soundcheck examples, pre-trained session calibration, cold start and changing speakers/microphones. Persisted calibration must not silently apply to incompatible inputs.
- Explicit end-to-end latency budgets for each intended use, including audio interface/host buffers, feature windows, lookahead, inference scheduling and output buffering; audio/video synchronization and separate monitoring paths.
- Which current acoustic features, fitted classifiers, temporal rules and gain-generation behavior can operate causally or with bounded lookahead; identify full-recording dependencies rather than calling offline processing real time.
- Live preservation/failure behavior, overlap and desired reactions, startup/warmup, missed inference deadlines, bypass, calibration changes and reset/transport discontinuities.
- CLAP/VST3 host/platform support, licensing and framework feasibility, realtime callback constraints, sample rates/block sizes, state restoration, automation and latency reporting.
- Whether post-production requires identical offline-app results or a deliberately different live-compatible policy; behavior during host offline bounce.
- Smallest measurable prototype and go/no-go checks, before accepted specification/build tickets.

## Design tree

1. First validation workflow -> target host/platform -> input/output routing proof -> host-specific latency and video/monitoring behavior.
2. Source assumptions -> available cross-microphone evidence -> single multichannel instance versus coordinated instances.
3. Preparation/calibration -> model state and source bindings -> safe start/restart/adaptation policy.
4. Audio objective -> attenuation-only versus source separation -> preservation/overlap acceptance and comparison recordings.
5. Combined decisions and factual feasibility -> bounded streaming design -> prototype success checks -> owner confirmation -> accepted specification and implementation tickets.

## Round 1 — owner answers, 2026-10-22

The owner answered through four response annotations. These answers supersede the narrower initial recommendations below; unresolved details must not be inferred as accepted.

- **Q1:** Live means broadcasting through OBS. Hearing the processed voice live is not required. Do not add a performer-monitoring requirement. DAW post-production remains a target; priority between delivering those targets is not yet settled.
- **Q2:** Support both a solo livestreamer's single microphone and multiple podcast microphones, including several instances of the plugin loaded in a DAW. One-microphone-per-speaker multitrack operation remains relevant, but must not exclude useful standalone single-microphone operation. Cross-instance information exchange/routing is a feasibility question, not a capability established by installing multiple instances.
- **Q3:** A short pre-stream soundcheck/calibration is acceptable. No automatic adaptation or reuse-across-recordings policy has been accepted yet.
- **Q4:** The product should be a better gate for a single person and a bleed/overlap reduction tool for podcasts. The owner did not accept narrowing the entire product to between-speaker attenuation. Clarify what overlap reduction and single-mic cleanup must sound like before deciding the engine scope.

### Initial questions (retained for provenance)

**Q1 — First validation workflow.** Keep live streaming and DAW post-production as goals, but validate delayed broadcast output first, with the speakers able to hear a separate dry/direct monitor. Recommendation: yes; settle performer-monitoring latency separately rather than assuming broadcast buffering is acceptable in headphones.

**Q2 — Recording arrangement.** Design the first version for one dedicated microphone per speaker, with all microphones available separately at the same time. Recommendation: yes; shared microphones and already-mixed stereo are separate source-separation problems.

**Q3 — Preparation.** Allow a short soundcheck to provide user-confirmed solo examples for each speaker and room tone before going live, then use that fitted session calibration. Recommendation: yes; automatic cold-start learning and continual adaptation require separate evidence and decisions.

**Q4 — First audio objective.** First reproduce CleanSlate's ownership-based microphone attenuation while preserving wanted speech and natural overlap; investigate removing bleed inside an actively speaking person's microphone separately. Recommendation: yes; do not require a new source-separation engine in the first plugin experiment.

## Round 2 — open frontier

These recommendations are pending, not implementation requirements. Questions now concern independent audio objectives, OBS installation experience and delay tolerance. Detailed host routing and model architecture depend on these answers.

**Q5 — Overlap objective.** When two people talk simultaneously, should each person's own track retain their voice, while the plugin tries to remove only the other person's leakage? Recommendation: yes; never solve overlap by automatically silencing one participant. This is an audio objective, not a claim that the current gain-only engine can isolate simultaneous voices. Whether isolation is mandatory for the first deliverable remains a later prioritization decision.

**Q6 — Single-mic objective.** For solo streaming, should the first target be a speech-aware gate that reduces keyboard/fan/room sound between phrases and preserves quiet speech, word beginnings/endings and wanted reactions? Recommendation: yes. Continuous removal of noise while talking is a separate objective that must be explicitly accepted if desired; speech detection plus gain does not selectively remove noise mixed with speech.

**Q7 — OBS setup.** Should ordinary OBS users be able to add CleanSlate directly as a microphone filter, without running a DAW or separate plugin host? Recommendation: yes as the usability target; keep the OBS integration mechanism open instead of assuming CLAP/VST3 alone provides it. [OBS's native filter documentation](https://obsproject.com/kb/vst-2-x-plugin-filter) explicitly supports VST2.x and excludes VST3; it does not document CLAP support. This does not select VST2 or an OBS-native implementation.

**Q8 — Broadcast delay.** Is up to 100 ms of additional processing delay a reasonable initial prototype target, provided audio/video synchronization is addressed? Recommendation: yes as a tentative budget to test, not a promised measured result. No low-latency performer-monitoring requirement is implied. If this is too much or more is acceptable, record the owner's bound before designing lookahead and gap handling.

## Round 2 — owner clarification, 2026-10-23

The owner separates two products/workflows rather than asking for the offline engine to become a live multitrack processor:

- **Solo OBS:** one plugin on one microphone is a smart voice gate for livestreaming.
- **DAW podcast:** multiple instances on separate tracks (example: three in DaVinci Resolve) should provide the application's reference-section labeling and multitrack algorithm. Full-recording analysis is acceptable because this is post-production; live processing is not required for this mode.
- No numerical latency bound was accepted. The earlier 100 ms suggestion remains unanswered for OBS and does not constrain offline DAW analysis.
- Do not interpret this answer as accepting source separation during simultaneous speech, continuous noise removal while talking, a particular host format, automatic inter-instance access, or an audio-capture workflow. Those details require factual investigation and the next design round.

Updated design branches: (1) solo OBS gate installation/calibration/preservation/latency; (2) offline DAW source acquisition, instance grouping, shared references/analysis, source-change invalidation and playback/bounce parity. Prioritization remains open.

## Factual investigation — in progress

The [2026-10-23 feasibility report](../../../docs/research/live-plugin-feasibility-2026-10-23.md) records official host/API sources and graph/source inspection. OBS direct filtering needs a separate integration path; VST3/CLAP alone does not supply it. DAW instances do not automatically gain all-track/full-file access. Reference-local supervised fitting appears feasible to extract, but existing APIs still prepare full recordings, and identical playback/seek results remain unproven.

### Round 3 — owner clarification, 2026-10-23

The owner wants **both** prototype directions, without selecting a first priority. An hour-long acquisition pass is undesirable. For DAW work, use the existing host waveforms to find and play representative sections, capture those sections and label who is speaking. The plugin need not duplicate the DAW timeline/waveform display. The owner expressed uncertainty about the best mechanism; this is a workflow preference, not acceptance of a specific capture/inference design.

The next technical proof is a reference-fit/inference split plus synchronized evidence across instance callbacks during ordinary playback/render. Never wait inside one audio callback for another instance: host scheduling can be serial. Matching block/timeline identity, bounded scheduling skew, reset/preroll after seeks and missing-instance behavior need measured host evidence. Do not treat short calibration capture as complete analysis, or claim the unmodified offline pipeline already works this way. Optional host-fast acquisition is an alternative to investigate, not a mandatory realtime hour-long pass.

### Confirmed workflow summary — 2026-10-23

The owner explicitly confirmed: OBS gets a solo speech-aware gate; DAW users use host waveforms to play and label short examples, grouped instances share calibration, and processing runs during ordinary playback/render without a required hour-long capture pass. Preserve the original audio when valid synchronized evidence is unavailable. Both prototypes remain goals, subject to measured host feasibility.

This confirms the workflow, not a framework, format/platform matrix, measured latency or completed implementation design. Bounded next proofs: (1) solo-gate callback/preservation/latency in direct OBS integration; (2) reference-only fit plus bounded timestamped cross-instance delivery, seeks and offline bounce in the target DAW. Require explicit session groups, whole-group reset/invalidation, fail-open missing evidence and no callback wait on another instance. Do not silently substitute file selection or full realtime capture when the desired workflow cannot be supported.

The delegated factual investigation is recorded in the linked report. Format support, routing and realtime throughput still require implementation proofs; documentation research alone does not establish them.

## Completion criteria

### Renewed feasibility discussion — 2026-10-24

The owner wants to advance toward a working plugin and asks what it requires and whether the current algorithm can work. Existing confirmed OBS and DAW workflows remain goals. The next-proof report is [here](../../../docs/research/live-plugin-next-proof-2026-10-24.md); its prototype suggestions are proposals, not accepted host/framework/latency choices.

Current source inspection confirms that reference-based ownership fitting is reusable in principle, but `analyze_acoustic_with_feedback` still couples fitting to full-source acoustic/VAD preparation. `SessionAcoustics::features_at` requires same-time competing-track levels and pair evidence; `JointFacts::prepare_blocks` uses 256 ms blocks and neighboring frames, while automatic envelope opening depends on future preserved speech. A plugin therefore needs a separate calibration/application boundary and a bounded processing/seek policy; unchanged callback use is unsuitable. Exact desktop result parity and realtime throughput remain unproven.

Proposed DAW experience: explicitly group instances, assign each microphone's speaker, capture synchronized short room/solo/overlap examples while using the host timeline, fit off the audio thread, then process playback/render with reduction meters and Original/Processed/Removed audition. Missing or mismatched group evidence preserves original samples at the declared processing delay. Do not silently substitute a full realtime acquisition pass or file selection.

The first host proof should establish grouped input timestamps, callback scheduling and bounded delivery during playback, seek/loop and offline bounce before substantial plugin UI or DSP extraction. A single multichannel instance can simplify synchronized input acquisition, but changes the requested per-track workflow and remains an alternative rather than an accepted replacement. Host for the first working test is requested from the owner; both product goals remain open.

Attenuation-only ownership cannot isolate another speaker's bleed from wanted speech mixed into the same microphone samples during overlap. Solo OBS gating can reuse speech detection and gain-transition concepts, but lacks the multitrack ownership evidence and needs separate qualification. No plugin code, dependency or measured compatibility is claimed by this update.

The owner selected **FL Studio on Windows** for the first working plugin test. OBS and other DAW targets remain goals. Scope the first host proof to two grouped pass-through inserts on separate mixer tracks, measuring timestamp alignment and bounded delivery through playback, seeks, loops, smart disable, and offline export. Then connect short-reference calibration and ownership inference after the delivery contract is established. This selects the first test host; plugin format/framework and numerical latency remain unsettled.

The owner subsequently authorized building that first prototype and selected **VST2, VST3 and CLAP** as audio-plugin product targets. [Ticket 75](75-fl-studio-plugin-prototype.md) claims the unblocked Windows FL pass-through proof, using pinned nice-plug 0.4.2 for VST3/CLAP. VST2 needs a separate adapter and verified licensing path; no VST2 support is claimed. This authorization supersedes the earlier discovery-only boundary for ticket 75's explicit scope, not the later ML/calibration or other-host scope. The owner requests to perform FL Studio testing themselves; further native host automation is not part of this delivery.

- [x] Record primary-source format/host/routing findings and graph-backed current-engine streaming constraints.
- [x] Record owner answers round by round; advance only decisions whose prerequisites are settled.
- [ ] Agree on routing, calibration, latency, preservation, failure and host/platform requirements.
- [ ] Define a bounded prototype and measurable go/no-go checks, without claiming unmeasured realtime performance.
- [x] Owner confirms shared understanding before implementation planning or implementation. Confirmed workflow above; detailed prototype contracts/host proofs remain outstanding.

## Comments

2026-10-22: Created and claimed for the requested interview. No plugin code, dependency, audio behavior or release scope changed. Round 1 awaits owner answers; factual research proceeds independently.

2026-10-22: Recorded all four owner annotations. Expanded the desired product to standalone single-mic speech-aware gating and multiple-instance multitrack podcast use; OBS broadcasting is explicit, processed self-monitoring is unnecessary, and soundcheck is accepted. Overlap/continuous-noise objectives, direct OBS integration and delay tolerance remain open in round 2. Parent verified the current official OBS filter page; no realtime throughput or compatibility test has been performed.
