# CleanSlate Session candidate 0.0.5

Checked Windows x64 development candidate for [80](../../.scratch/offline-supervised-editor/issues/80-plugin-session-capture-analysis.md), implementing the confirmed [Session-first contract](../../.scratch/offline-supervised-migration/plugin-modes-interview.md). Source: development checkout at `88dda154daa9f442018302e19f1c792513fd0874` plus the recorded uncommitted implementation. No commit, push, release or vendor-plugin runtime is claimed.

## Delivered behavior

- Up to eight mono/stereo microphone inserts share source-time capture, labels, analysis and Session sound defaults. Numeric assignments are manual; host track names/colors and per-track sound overrides are follow-ups.
- A bounded callback queue sends native PCM to a disk worker. Local audio and indexed coarse waveform caches persist under `%LOCALAPPDATA%/CleanSlate/PluginSessions/<id>/<rate>/`. Loops/seeks preserve existing time coverage; replay fills holes and compares actual PCM rather than extrema. Identical covered audio skips disk/peak rewrites and does not invalidate analysis.
- Source/cache identity is separate from transport validation. Cache keys, label ranges, content fingerprints and shared settings persist in plugin state. Saved-state disagreement, duplicate members and competing cache writers fail safely. Reopening treats cached audio as unverified; replay validates retained examples against full PCM fingerprints before new analysis.
- Room/solo/arbitrary-overlap labels run the maintained desktop pipeline through its existing library: decoding/preparation, embedded Silero, acoustic/transfer evidence, independent ownership training, temporal policy and effective envelopes. No proprietary sonible model is used. Stereo analysis explicitly selects input channel 1; one envelope applies to both channels.
- Analysis snapshots contain only a verified common contiguous interval including every example. Gaps never become trusted zeros. Nonaligned interval edges are retained in analysis; partial source-validation blocks remain original. The editor reports the analyzed range and preserves original outside it.
- Live callback validation compares full native PCM against 2,048-sample SHA256 fingerprints before using cached gains. Unknown/stale/changed/invalid evidence preserves the original. Changed training content requires review. Missing clocks clear validation stamps, but continuous delayed original audio is retained; known seeks/reset clear streaming state without erasing capture history.
- Session declares a constant delay of 416 ms plus maximum host block duration, even while processing is off. Enabling processing does not request reactivation or discard analysis. Old saved states retain the legacy probe until the owner chooses the Session workspace; legacy disabled zero-delay behavior remains.
- The actual egui workspace provides eight lanes, source-time selection, Fit/zoom/navigation, labels, shared Analyze/sound, Original/Processed/Removed and cache clearing. Assignment changes show a host-reconnect state. CPU rendering was inspected at 1000×680 and 720×560; real native attachment/feel is not claimed.

## Verification

| Check | Result / boundary |
|---|---|
| `scripts/check.ps1 -Local` | Pass: 371 passing tests across root/shared/examples, 20 intentionally ignored, strict all-feature/all-target Clippy, formatting and script fixtures. |
| Plugin tests | 50 passing, including actual capture → immutable WAV → full desktop analysis → sparse member publication → PCM-validated gain. Two stereo sources use opposite-polarity channels to exercise the explicit channel policy; fixture origin is not block-aligned. |
| Source/history regressions | Loops 10–30 then 0–30; unchanged replay; changed PCM with identical extrema; saved-label validation; cache reopen; missing waveform repair; exclusive writer; queued-tail checkpoint; unknown/stopped/changed fallback; monitoring reconstruction; partial edges. |
| Missing transport metadata | A continuous independent FIFO reference verifies delayed original across variable block sizes and missing context. This replaced a block-period shortcut in the host oracle and exposed/fixed inappropriate delay resets on missing clocks. |
| Strict plugin checks/build | `scripts/build-fl-probe.ps1` passes formatting, all tests, strict all-target Clippy, DLL/examples, both actual-format hosts, old-build preservation and candidate/publication equality. |
| CLAP actual adapter | 1,858 bit-exact blocks, 1,168 shared diagnostic PCM windows; old state migration/round-trip; offline/mode restart checks. Stepped parameter migration checks display identity plus float tolerance for normalized f32 reconstruction. Legacy-state scenario, not native FL learned Session qualification. |
| VST3 actual adapter | 4,048 bit-exact delayed/original blocks, 3,392 diagnostic windows; two actual instances, mono/stereo, 44.1/48 kHz, realtime/offline, varied blocks, separate/in-place buffers, missing transport/seeks. Fixed Session latency/tail checked with processing off/on. Editor constructs/releases; no native window attached. |
| Desktop build | `scripts/gui-dev.ps1 -BuildOnly` passes, 119.36 s; no GUI opened. Device-free `--package-self-check` passes model inference, analysis, save/reopen, render and cancellation. |

The first attempt linked the whole desktop application into the plugin library build and exhausted compiler memory. A default-on `desktop-ui` feature now retains the normal desktop binary/API while plugin builds disable application/widget code. Mixed cache/display modules still depend on Iced types; this is not a claim that every UI dependency was extracted. Shared local dev/test code generation is bounded at 16 units with incremental compilation retained. Desktop and headless paths were both checked after this change. No analysis recipe was changed.

## Observed resources

These are small, device-free synthetic test processes, not native FL or two-hour session measurements. Working-set samples poll the child's OS peak counter every 10 ms; compiler/PowerShell are excluded.

| Process | Elapsed | Largest sampled peak working set |
|---|---:|---:|
| Actual CLAP host | 0.49 s | 20.70 MiB |
| Actual VST3 host | 1.01 s | 24.39 MiB |
| Complete captured Session regression (two stereo microphones, four seconds, 48 kHz) | 0.63 s | 57.72 MiB |

Queues reserve about 2 MiB per group. Coarse peaks and block fingerprints grow within the supported two-hour/rate/member bounds; native whole-recording PCM is disk-backed. Eight tracks over two hours at 48 kHz need roughly 43 MiB for fingerprint bytes alone, plus peaks, containers and analysis state. High-rate/long-session CPU, RAM, disk capacity, callback deadlines and audible quality remain unmeasured. Analysis jobs currently rebuild immutable snapshots and do not persist reusable envelopes across restarts.

## Artifact identity and preservation

Both published files are **39,140,864 bytes**, SHA256:

`bcdd98812733ceb440ab61a899935631fd47d3f5ff053ca240e80848a99f1548`

- `C:/Users/User/Documents/podcast-automix-rust-gui/dist/fl-probe/CleanSlate FL Probe.clap`
- `C:/Users/User/Documents/podcast-automix-rust-gui/dist/fl-probe/CleanSlate FL Probe.vst3/Contents/x86_64-win/CleanSlate FL Probe.vst3`

The existing Common Files CLAP link and VST3 junction resolve to those exact hashes. Publication preflights both existing binaries for host locks; candidate checks finish before touching development links. Synthetic format tests redirect only their child LOCALAPPDATA and clean their owned fixture directories, leaving owner caches alone.

The original checked 0.0.4 remains in `%LOCALAPPDATA%/CleanSlate/VerifiedBuilds/fl-probe/a8aead9c0a1d12e54cf05e06b6212b72e991427887271248e66b1f658ecb281c/`. Tested intermediate 0.0.5 bundles are also retained by the preservation script. Desktop debug executable: `%LOCALAPPDATA%/CleanSlate/BuildCache/debug/rust-app.exe`, SHA256 `e21f05b04359a6b8f01077421ea51e32180339ea51a6a120e122dd57fa0a2c0a`.

Actual CPU-rendered synthetic UI captures: `%LOCALAPPDATA%/CleanSlate/SessionUiCaptures-0.0.5/session-workspace.png` and `session-workspace-compact.png`. These depict the real widget code with generated test audio, not a native FL screenshot or listening evidence.

## Owner handoff and open work

Use [the current test instructions](../../plugins/fl-probe/README.md) on a disposable FL project copy, VST3 first: reopen FL, opt old presets into the Session workspace, assign group/mics, play 10–30 then 0–30, label room and solo speech, stop/Analyze, enable and compare Original/Processed/Removed. Save/reopen/reverify and export. Report the export path and any history, readiness, UI or sound problem. No sonible installation or further vendor investigation is needed.

Native FL window/gestures, host delay compensation, source revalidation under actual playback/export, sound and save/reopen remain owner acceptance. Native CLAP export attenuation is not qualified by these checks. Shared sound automation/per-track overrides, host names/colors, reusable analysis persistence, Live Mic, VST2 and non-Windows packaging remain explicit later work. No DAW, audio device, DaVinci Resolve or sonible installer/plugin was launched during this implementation.
