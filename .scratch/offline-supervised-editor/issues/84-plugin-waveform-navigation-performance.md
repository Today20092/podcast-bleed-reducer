# Plugin waveform navigation and responsiveness

Status: done
Assignee: Codex
Type: task
Blocked by: none
Related: 80, 82

## Owner request

The 0.0.5 native waveform is difficult to zoom and lags. Bring navigation toward the regular application: pointer-anchored wheel zoom, middle pan, range zoom, fit/selection controls and discoverable gestures. Preserve labels and captured source-time history. Computer use in FL Studio is now authorized for testing; vendor installation remains unnecessary.

## Acceptance

- [x] Reproduce missing wheel interaction and measure current rendering work before changing it.
- [x] Add deterministic event-level navigation regressions and correct bounded viewport behavior.
- [x] Keep a useful waveform visible during capture-worker contention and measure long-history redraw cost.
- [x] Verify compact/full UI, plugin tests, strict checks and actual format hosts; preserve prior binary.
- [x] Exercise the checked editor natively in FL Studio and record evidence/remaining limits.

Scope: waveform interaction and rendering. Desktop-only file/transport operations remain host-owned; no vendor DSP extraction, algorithm change or new plugin format.

## Results

[Receipt](../../../docs/research/plugin-navigation-delivery-2026-10-25.md): 59 tests and strict actual-format checks pass. Native first-candidate wheel zoom/capture/looped playhead work in a separate FL project; fast native drag exposed and now has a passing regression/fix. Final 94CF0D candidate is checked, but development publication is blocked by FL's loaded CLAP lock. Owner asked to close FL. Keep claimed for final publication/native drag/feel checks; neither synthetic timing nor preliminary native wheel evidence establishes full desktop parity or vendor-equivalent smoothness.

Final installation: after owner confirmation, stopped only the verified separate test process still holding the binaries. Re-ran the build script successfully and verified both existing Common Files links serve final SHA256 `94cf0dc682447677e1d942397dfc8aea929c9fe697a5a0035127968dace54aaa`. First-candidate 0.0.6 was preserved. Publication is complete; keep claimed for final native drag/feel acceptance.

Final native functional check completed at owner's request: reopened the installed build in the copied FL project. Quick drag, focused F, Center, wheel zoom in/out, horizontal-wheel pan, Fit captured, Lock/Catch up and history retention during loop playback were observed working; no further code change was needed. See the receipt for exact ranges and whole-host resource observations. Closed for the scoped fixes/functional verification. Native modifier/middle-button drag injection is unsupported by the computer-input API, so those combinations retain event-test evidence only. Presented latency, subjective vendor equivalence and 80's sound/export acceptance remain unclaimed. Playback stopped; copied project remains open.
