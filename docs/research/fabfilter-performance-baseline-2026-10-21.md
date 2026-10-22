# CleanSlate editor performance baseline

Date: 2026-10-21. This is a retained baseline for the eight FabFilter-inspired performance tickets, not a FabFilter CPU comparison, release qualification, or measured improvement.

## Source and capture identity

- Native executable source: `e520273aaa4f365fd05d308cac2d9504a2ae97af`, before ticket02's selective-invalidation change.
- Executable SHA-256: `7efdaba74806ef0d516377012ce7d0e4b3efcff101f8b6e767583d42ca0283d2`.
- Application: 0.1.0-alpha.6, development build, Windows x86_64.
- Capture label: `baseline-original-invalidation-startup-idle-play-follow-zoom-edits`.
- Original project SHA-256, confirmed unchanged by the native-run operator: `20bb912825964a2ba9b717525477f6af365abb3e434084cf7836e6b5725cbd98`.
- Private development-copy SHA-256 after capture: `6016da6272c77110b32ca5a97eb9bbbc5c5e6a9ed93b5777dcb1bdd090936325`. It is a separate working project; its hash is not the original-project hash.
- Raw native JSON SHA-256: `f82222535a40b399f526033ff9abdc30477fa19a8ecdee2f4bee3a5ddeb16802`.
- Raw OS companion JSON SHA-256: `a9f73170c01c6f0b8019d3a4ae0223cb184025a46c8725dbe9637eecd6611c67`.

The native recorder's commit/hash values are launch metadata, not embedded artifact attestation. The original project was retained and a separate private development copy was used; no source recording or project contents are included here.

## Workload and observed actions

The project has three synchronized 600-second 48 kHz tracks and 18 speaker-reference examples. The native window was maximized (observed outer rectangle 2560×1392); its report's final logical client dimensions are 2560×1369. Five resize events were observed. No scale-change event was observed, so the recorder's scale factor is null rather than an assumed 1.0. Audio device identity/rate, graphics backend/adapter and monitor refresh are not established by these two artifacts.

The root agent's native run covered startup and visible stopped idle, Analyze concurrent with Original playback, wheel zoom from 600 to 480 visible seconds, gain-editor display, a warm attenuation wheel edit from −11 to −12 dB, enabled locked-follow playback, minimization, restoration with playback continuing, and graceful Alt+F4 closure. Analyze completed in approximately 22.5 seconds in this one concurrent run; that is a UI observation, not a repeated cold/warm processing benchmark. Because audition was Original, these actions do not establish audible processed-envelope response or acoustic onset latency.

The process (15520) exited normally and produced valid recorder JSON. No Cargo work ran during the OS sampling phases. Native middle-button drag was unavailable through the automation API: **no physical/native middle-drag acceptance is claimed**. The repaired read-only software pan probe supplies production waveform/gain geometry coverage, not gesture or displayed-frame coverage. Not every action/background-work combination was captured as its own isolated native run.

## Native recorder observations

This was one mixed run lasting 372.578 seconds. Each metric retains its own most recent 4096 samples, with lifetime counts and overwritten counts. Those retained windows span different clock intervals and cannot be aligned to an OS sampling phase or a particular action. They are not whole-run percentiles when older samples were overwritten. Nearest-rank percentiles and the maximum below describe each retained window.

| Observed boundary | Lifetime observations | Retained / overwritten | Median ms | p95 ms | p99 ms | Retained max ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Application update elapsed | 30,737 | 4,096 /26,641 | 0.002863 | 0.003772 | 0.005238 | 0.143454 |
| Worker-poll update elapsed | 2,364 | 2,364 /0 | 0.004330 | 0.006146 | 0.007124 | 13.271868 |
| Waveform cache-build closure elapsed | 16,848 | 4,096 /12,752 | 3.667156 | 4.174832 | 4.398114 | 5.231740 |
| Whole waveform canvas draw elapsed | 116,852 | 4,096 /112,756 | 3.699703 | 4.213594 | 4.440229 | 5.381201 |
| Gain cache-build closure elapsed | 5,928 | 4,096 /1,832 | 2.220743 | 2.551512 | 2.723740 | 3.276883 |
| Whole gain canvas draw elapsed | 72,999 | 4,096 /68,903 | 2.231778 | 2.567994 | 2.735962 | 3.289594 |
| Native redraw-request interval | 29,213 | 4,096 /25,117 | 9.773240 | 30.021693 | 31.680912 | 35.812652 |
| Latest observed input → first canvas completion | 15 | 15 /0 | 5.078928 | 155.662033 | 155.662033 | 155.662033 |

There were 18 observed input events; the recorder coalesces the latest pending input and obtained 15 subsequent first-canvas-completion measurements. This small mixed sample cannot identify which edit caused its tail. It does not measure whole-window feedback, hardware input-to-photon latency or audio onset.

Update/draw durations are wall-clock elapsed spans on their executing thread, including scheduling delays. A cache-build timer covers its draw-closure body; subsequent renderer/cache conversion is included only in the whole-canvas span. Counts refer to individual canvas invocations, including ruler/track canvases, and are not frame counts. Recorder timer/lock overhead can perturb results.

Iced redraw-request timestamps are **not presentation acknowledgments**. No displayed FPS, GPU busy time, monitor refresh rate, physical input latency or callback deadline guarantee is inferred from these values.

## Separate OS process samples

Each phase below is a single approximately five-second sample with 10 working-set observations at about 500 ms intervals. CPU percentage is total process CPU seconds divided by elapsed wall seconds, expressed in units of one logical processor; it can exceed 100% because it sums process threads. The machine exposes 32 logical processors. These percentages are neither total-machine utilization nor proof that one particular thread is saturated.

| Phase | Observed seconds | Process CPU, one-core units | Maximum sampled working set MiB |
| --- | ---: | ---: | ---: |
| Visible, stopped | 5.165 | 0.30% | 189.76 |
| Visible locked follow, gain editor enabled | 5.188 | 97.29% | 233.73 |
| Minimized locked follow, gain editor enabled | 5.200 | 105.47% | 233.79 |

A sampled working-set maximum is **not peak process memory**, committed memory or allocator use. The minimized sample does not show a cost reduction and motivates ticket06's focused check; a single sequential sample cannot establish a statistically meaningful regression or its cause. Visible stopped idle provides a useful check against an observer-induced continuous redraw loop, without proving zero observer overhead.

## Software geometry companion

The repaired `probe-pan.ps1` used production waveform/gain/timeline drawing with synthetic waveforms/envelopes for the same three-track source-grid shape: 1900 columns, 40 viewport moves. It passed at median 5.72 ms and max 9.62 ms, on implementation861a29a before the native integration build. This is CPU/software geometry preparation, not native presentation. Its measurement window and renderer differ from the mixed native run and historical probes; no numerical speedup is claimed.

## Reproduction and retained artifacts

Use `scripts/profile-gui.ps1 -Project <absolute-project> -Scenario <one-scenario-label> -Report <new-absolute-json>`, exercise the chosen state and close the capture normally. Use fresh reports per isolated scenario, fixed viewport/display/device settings, matched builds, repeated OS sampling and no compilation to make a causal before/after comparison. Pair the recorder with the separately labeled OS sampler and playback/graphics diagnostics where needed. Preserve source/project hashes and distinguish original files from working copies.

The raw `baseline-native.json`, `baseline-os.json`, private `native-baseline.cleanslate` and preserved software-probe executable/PDB remain outside the repo under the shared investigation directory recorded in the feature specification. The hashes and portable summaries above allow their identity and meaning to be checked without publishing private recordings or relying on absolute local links.

Ticket01's instrumentation, representative baseline and source-preservation receipt are complete. Full scenario matrices, repeated causal comparisons, physical middle-drag, processed-audio listening/onset, GPU presentation and cross-platform/hardware acceptance remain outside this baseline. Existing GUI acceptance records retain their separate physical-input scope.
