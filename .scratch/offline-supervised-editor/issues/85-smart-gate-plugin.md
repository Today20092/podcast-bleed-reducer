# CleanSlate Smart Gate: single-input live plugin

Status: open
Readiness: ready-for-agent
Type: task
Blocked by: 87
Related: 67, 77, 79, 86

## Accepted product

One independent plugin for one microphone in live production: detect wanted speech, remain open while it is present and attenuate during absence. Show speech confidence, input/output activity and applied reduction with a compact, smooth interface. This is a smart gate, not automatic gain leveling. Host-independent local processing is the goal; OBS is the first intended live host.

Development belongs to `codex/plugin-smart-gate` in its own worktree. Start from the preserved 0.0.6 prototype for reusable adapter, editor, safety and test infrastructure. The prototype's Session/legacy modes are historical scaffolding, not the finished product. Do not require another instance, a session group, a whole recording, or the desktop app to be running.

## Implementation order

1. Establish a distinct plugin package, stable format identities, presets/state schema, product name, build/test output and development links. Preserve the old probe's identity and reopening behavior; no silent replacement in saved projects.
2. Separate single-input speech evidence from attenuation policy. Reuse maintained Rust components where appropriate; evaluate their causal/lookahead requirements before adapting them. Multimicrophone ownership training does not establish a single-mic gate's accuracy. Document the chosen detector, any calibration and the actual latency budget before enabling attenuation.
3. Implement bounded realtime processing, confidence/hysteresis, attack/hold/release and reduction, latency reporting, bypass and safe original fallback. Keep allocation, locks, filesystem work and inference preparation out of the audio callback as applicable to the selected design.
4. Build the focused activity/confidence/reduction UI and verify it under continuous playback without carrying the Podcast labeling workspace into the live flow.
5. Qualify format/platform/host combinations independently, retaining lightweight actual-format test hosts for routine development.

## Acceptance

- [ ] Separate identity and build coexist with Podcast and the old probe; save/reopen, bypass, automation and settings migration are checked.
- [ ] Gate works on a single input without peer discovery or offline capture, with truthful readiness/confidence and an explicit failure fallback.
- [ ] Test quiet speech, starts/ends, pauses, laughter, background noise, changing levels and incorrect/missing calibration. Record listening limitations; do not promise speaker isolation or voice authentication.
- [ ] Measure latency, callback deadline behavior and memory/CPU under stated sample rates/block sizes; no unsupported zero-latency claim.
- [ ] Unit and actual-format host checks cover silence, clipping/nonfinite input, resets, sample-rate changes, bypass/state restore and realtime safety.
- [ ] Verify the actual OBS integration path before advertising it. VST3/CLAP alone do not prove OBS compatibility; retain VST2 as requested legacy work pending a supported adapter and licensing/distribution review. Additional host integration may be needed.
- [ ] Target Windows first, then macOS/Linux with native packaging/tests. VST3 and CLAP are primary targets, VST2 is desired legacy coverage; AAX/AU remain deferred. No host/platform is advertised solely because its format builds.

## Boundaries

Automatic volume leveling, compression, multitrack ownership and vendor model reconstruction are outside this initial gate. Reuse original CleanSlate code, not extracted vendor binaries/models. No new vendor installation or research is required. Quality of the desktop detector is not evidence of live-gate quality.

## Results

Product and implementation sequence accepted; no standalone Smart Gate binary is delivered yet.
