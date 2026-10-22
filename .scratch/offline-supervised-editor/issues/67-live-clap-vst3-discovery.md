# Explore live and DAW CLAP/VST3 bleed reduction

Status: claimed
Type: grilling
Assignee: Codex
Blocked by: none
Implementation readiness: needs-info

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

## Factual investigation — in progress

A delegated read-only investigation is checking current code dependencies and official plugin/host documentation. Findings and source links will be recorded here. Format support, routing and realtime throughput are not established merely by naming a plugin format.

## Completion criteria

- [ ] Record primary-source format/host/routing findings and graph-backed current-engine streaming constraints.
- [ ] Record owner answers round by round; advance only decisions whose prerequisites are settled.
- [ ] Agree on routing, calibration, latency, preservation, failure and host/platform requirements.
- [ ] Define a bounded prototype and measurable go/no-go checks, without claiming unmeasured realtime performance.
- [ ] Owner confirms shared understanding before implementation planning or implementation.

## Comments

2026-10-22: Created and claimed for the requested interview. No plugin code, dependency, audio behavior or release scope changed. Round 1 awaits owner answers; factual research proceeds independently.

2026-10-22: Recorded all four owner annotations. Expanded the desired product to standalone single-mic speech-aware gating and multiple-instance multitrack podcast use; OBS broadcasting is explicit, processed self-monitoring is unnecessary, and soundcheck is accepted. Overlap/continuous-noise objectives, direct OBS integration and delay tolerance remain open in round 2. Parent verified the current official OBS filter page; no realtime throughput or compatibility test has been performed.
