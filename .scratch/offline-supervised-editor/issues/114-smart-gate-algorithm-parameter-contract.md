# Smart Gate vocal/podcast algorithm and parameter contract

Status: open
Readiness: needs-triage
Type: task
Blocked by: 92
Related: 85, 93, 94, 115, 116

## Scope

Turn the completed [native research](../../../docs/research/smartgate-native-engine-2026-10-31.md)
into an original, implementable processing contract for spoken vocals/podcasts.
This is a draft planning ticket, not acceptance of every research suggestion or
authorization to change DSP/GUI code. No further reverse engineering is needed
to choose our design. Resolve this contract before claiming 115.

Recommended starting point: retain the hash-verified MIT Silero VAD v6.2 model
and its pretrained weights. Its speech score drives a separate sample-rate-aware
gain envelope. Do not import sonible model weights, copy its amplitude-score
calibration, train a new network, or imply speaker identification. Desktop's
supervised multi-microphone ownership engine remains a separate capability.

## Acceptance

- [ ] Record every parameter's identifier, unit, direction, range, engineering
  default, smoothing, automation and persistence behavior in one shared contract.
  Distinguish provisional engineering defaults from listening-qualified presets.
- [ ] Define threshold/sensitivity, relative soft transition (Tolerance), internal
  hysteresis if retained, level influence, maximum reduction/strength and
  Attack/Hold/Release. Specify their interactions and transfer endpoints; do not
  relabel hysteresis as the vendor's soft transition or duplicate strength knobs
  without distinct behavior. Decide whether one Depth control or separate
  Depth/Impact controls best serves this design.
- [ ] Cover three-band shaping, ordered crossovers, band enable/solo, Gate/Duck,
  internal/external sidechain, mono/stereo/Side routing, Original/Processed/Removed
  monitoring, and presets/A-B. Explicitly scope each family; do not silently drop
  advanced features or invent distinct Speech/Vocals models from one VAD model.
- [ ] Choose the audio-delay/lookahead and host-latency contract. Align dry,
  processed, bypass and Removed paths, including reset and failure handling.
  Do not adopt an unmeasured 64/80 ms value from research examples.
- [ ] Preserve safe state restoration, explicit temporary activation/readiness,
  bounded worker execution and original-audio fallback. Parameter values and
  transient detector/envelope state must have separate persistence rules.
- [ ] Publish a backend-to-GUI contract: parameter metadata, meter/history data,
  readiness/errors, monitoring and preset commands. Final GUI layout/framework
  selection belongs to 94; a temporary host parameter surface is sufficient.
- [ ] Reconcile the accepted Gate specification and 93/94/115/116 scopes with
  these choices. Mark downstream tickets ready only when their behavior is
  concrete; document any deliberately deferred family and why.

## Comments

2026-10-31: Owner requested a simple algorithm explanation and separate tickets
for finishing the backend, plugin build and GUI. Created as a draft; no work is
claimed, no production algorithm or numeric default is changed.
