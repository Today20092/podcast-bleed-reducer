# Confirmed plugin direction — 2026-10-24

# Product separation amendment — 2026-10-25

The owner accepts [two separate plugin products](../../docs/plugin-products.md), replacing Session/Live Mic as modes of one shipping plugin. Session answers below continue under [Podcast 86](../offline-supervised-editor/issues/86-podcast-plugin.md); single-input live answers continue under [Smart Gate 85](../offline-supervised-editor/issues/85-smart-gate-plugin.md). Desktop stays independent. No live multitrack podcast recording is required; automatic volume leveling is not initially included in Smart Gate. Existing implementation evidence and unmet acceptance stay intact.

Owner confirmed Q1–Q19 across the design interview. This is the agreed product contract and replaces the prototype's rolling-history-only/two-member product direction. It is not a claim that these capabilities are implemented or qualified. The checked binary remains 0.0.4; ticket 78 remains open. Desktop release/quality acceptance is unchanged.

## Session mode first

- One CleanSlate plugin offers Session and Live Mic modes. Session is the next priority. The first Session version handles prerecorded DAW playback/export, not real-time multi-microphone podcast recording.
- Up to eight dedicated speaker microphone tracks, with independent ownership so overlap can remain open on multiple tracks. Use the desktop application's maintained analysis approach; the prototype's shared trainer alone does not establish equivalent features or results.
- One capture/play-through is acceptable before unseen material can be displayed/analyzed. Use a persistent source-time timeline: keep 10–30 seconds when looping; later 0–30 playback fills 0–10 and validates/updates overlapping content. Transport discontinuity must not itself erase source history. Similar waveform bins are not proof of unchanged audio.
- Retrospective selection/labeling of Room, individual speakers and overlap remains primary. Trustworthy room/solo examples are required. Label changes require explicit Analyze to regenerate cached processing envelopes. First scope excludes point-by-point manual gain-envelope editing.
- Preserve original audio wherever valid analysis is missing or stale; clearly distinguish readiness, unprocessed and unverified coverage. Do not infer export readiness solely from host-reported offline mode.
- Local project-specific cache files, including captured audio necessary for analysis, are authorized; bound RAM and preserve original media. Save labels/settings/cache references with the DAW project. Include a clear-cache action. If moved to another computer without cache, rebuilding through capture is acceptable; validate retained labels against recaptured source before reuse. Portable cache packaging is later scope.
- Refresh changed source-time regions and invalidate affected analysis. Labels over changed training content require review. Detection is limited to audio the host delivers: unplayed edits remain unverified until received again.

## Shared instance control

- Discover CleanSlate instances automatically, display host-provided track name/color when available, and provide manual fallbacks. Owner accepts confirming group membership once. Names and track metadata are not speaker identities or stable source/project identities.
- Open any participant to view the shared session, label/analyze and adjust members centrally, inspired by Pro-Q 4's instance list. Discovering our own instances is distinct from enumerating all DAW tracks.
- Reduction/fade defaults are linked across the group, with per-track overrides editable from any instance. Remote changes must preserve each target instance's host parameter/automation/save behavior. Separate project/group identity from streaming safety generations and display metadata.

## Live Mic later

- Independent single-microphone speech gate. Owner accepted detecting any human speech, not enrolled-person verification. Nearby speech or TV speech is therefore not guaranteed to be rejected.
- Works initially with a pretrained speech detector; room/noise calibration is optional, not mandatory enrollment. A speech probability, ownership estimate and speaker identity are distinct concepts.
- Prioritize cleaned stream/recording output, without an initial near-zero-latency headphone-monitoring promise. Latency and speech preservation require measurement.
- Default gentle attenuation between speech and preserve uncertain/quiet speech conservatively. Separate opening/closing thresholds and smooth transitions; stronger suppression/full mute are deliberate settings. Displayed activity/probability is an estimate, not guaranteed confidence.

## sonible reference investigation

2026-10-25 owner correction: end the sonible reference investigation at the supplied binaries/manuals. Do not require installation, activation or vendor runtime tests to proceed with CleanSlate. The [first static pass](../../docs/research/sonible-static-pass-2026-10-24.md) and [smart:gate follow-up](../../docs/research/smartgate-followup-2026-10-25.md) preserve the findings and unknowns. The previously proposed installation step is withdrawn. No further broad design interview is needed before the accepted Session work; Q1–Q19 remain confirmed. Sonible's exact network and measured behavior remain unknown and are not prerequisites for our implementation.

- [smart:gate manual](https://www.sonible.com/wp-content/uploads/2023/09/manual-smgate_EN.pdf), pp. 6–11: separates selected target activity, level bias, activity threshold/tolerance, attack/hold/release and impact; shows original/processed/gate activity separately. Source-category targeting is relevant to the accepted any-speech Live Mic goal, not evidence of personal voice verification. Its [product specification](https://www.sonible.com/smartgate/) lists approximately 80 ms latency; that is a vendor specification, not a local measurement or CleanSlate target.
- [smart:comp 3 manual](https://www.sonible.com/wp-content/uploads/2025/12/manual-smartComp3.pdf), pp. 7–9 and 19–21: learning builds a source-adapted parameter space; groups offer remote control, linked parameter adjustments and batch learning. The manual explicitly states that grouping does not make instances adapt their audio processing to one another: each analyzes/compresses independently. CleanSlate additionally needs cross-microphone evidence for ownership.
- Relevant questions for the installed-build pass: documented activity versus actual gate gain; quiet speech/noise/competing speech response; transitions and measured latency/CPU; learning/state persistence; group discovery and remote host parameter behavior. Do not treat a compressor's group GUI as proof of joint multitrack inference or adopt another detector's claims without testing.

The interview is complete. No software change, new build, installation, purchase, activation or release was performed while recording these decisions and reviewing public references.
