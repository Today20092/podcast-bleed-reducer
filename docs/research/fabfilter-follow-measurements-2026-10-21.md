# Continuous-follow drawing measurements

Date: 2026-10-21. Base source: `e82448233be10d867f0df79c29f59ba0754855f4`, with ticket07's exact-flat curve change. This is performance evidence for the existing renderer, not release or display qualification.

## Workload and boundary

The retained private baseline project has three 600-second 48 kHz tracks, 18 references and 823 analyzed ownership regions. The probe uses actual retained source waveform peaks, actual automatic/effective envelopes and project constraints. It prepares analysis in its own external cache before timing and verifies the saved project is byte-identical afterward. Parameters are attenuation −11 dB, open550ms, close1656ms and minimum gap768ms. Input SHA-256: `6016da6272c77110b32ca5a97eb9bbbc5c5e6a9ed93b5777dcb1bdd090936325`.

The timed boundary contains all three waveform and all three gain canvas draws, including their overlays, in the existing software renderer. Canvas sizes are2176×180 and2176×20 logical units, respectively. Each of240 observations moves a locked viewport forward1/60 second in source time; the visible span is480 seconds. This is an unpaced preparation loop, not a60Hz native application or presented-frame observation. Real peak amplitudes, curve density, labels and source grids are retained; GPU/raster work, transport/device output, whole-window layout and physical feedback are excluded. The separate native [application baseline](fabfilter-performance-baseline-2026-10-21.md) reports redraw requests/CPU and its different retained-window limitations; its per-canvas medians cannot be added into an observed frame percentile.

## Measured decision

Initial separate-process2176-column follow runs measured complete medians15.144 and10.223ms, compared with0.111ms for unchanged cached drawing. A4096-column control measured17.649ms median and failed the inherited16.7ms diagnostic budget. A later candidate4096 run still failed that budget at24.487ms. These differing process/thermal observations establish substantial redraw work and variability, not a controlled speedup or a4K60Hz acceptance claim.

The smallest implemented improvement reuses exact horizontal curve spans: a run of equal gains keeps its first/last source-time points and omits redundant interior vertices before the existing path conversion. Every changing sample is retained, with no epsilon, time quantization, new retained cache, custom widget or renderer change. A full-source constant0.25 curve at1000 columns falls from1001 vertices to3: both flat endpoints and the existing outside-source unity boundary. At most4097 temporary points remain allocated, as before; no persistent memory is added. Modest measured savings do not justify tiled/translated geometry complexity in this ticket.

For a closer comparison, the diagnostic alone retains the original `e824482` dense sampler and alternates dense/candidate draw order on every observation in the same process and renderer. Revision changes force rebuilding both passes. The baseline selector and raw sample output exist only in the diagnostic example; production has no option or benchmark branch. Two runs gave:

| Complete waveform + gain, ms | Candidate median / p95 / p99 / max | Dense median / p95 / p99 / max |
| --- | --- | --- |
| First interleaved2176-column run |13.831 /14.745 /15.070 /15.200 |14.093 /15.066 /15.213 /15.621 |
| Repeated interleaved2176-column run |9.535 /12.218 /13.454 /14.039 |9.665 /12.052 /14.240 /14.284 |

All-gain medians were5.111 versus5.353ms in the first comparison and3.491 versus3.592ms in the repeat. Complete median reductions were approximately1.9% and1.35%; gain reductions approximately4.5% and2.8%. Complete p95 was slightly worse in the repeat. These are small CPU geometry savings in two runs, not established tail, OS CPU, GPU or displayed-frame improvements. Waveform drawing remains dominant.

## Reproduction and retained evidence

Run `scripts/probe-pan.ps1 -Project <private-baseline.cleanslate> -Follow -AnalysisCache <isolated-cache> -Width 2176 -Compare`; use1900 for the historical-width control or `-Cached` without `-Compare` for unchanged drawing. No Cargo/probe/background compilation ran concurrently with timed observation loops. The script uses `scripts/local-build-settings.ps1`; the pinned Rust1.99.0 build retains the configured application/dependency optimization profiles.

External evidence directory: `C:/Users/User/Documents/Codex/Investigations/cleanslate-performance-implementation-2026-10-21/`. `perf07-interleaved-2176-raw.txt` retains all six240-sample arrays in acquisition order, summaries and workload labels from the repeat. Earlier `perf07-baseline-cached.txt`, `perf07-baseline-follow-2176.txt`, `perf07-baseline-follow-4096.txt`, and `perf07-flat-follow-{1900,2176,4096}.txt` retain console summaries only; their individual samples were not retained. Final diagnostic executable: `%LOCALAPPDATA%/CleanSlate/BuildCache/debug/examples/pan-drawing-probe.exe`, SHA-256 `4c75a7a2add0c189a76cf545ce574c53d2a4136049f393883b3a16b19be421ce`.

## Checks and remaining acceptance

TDD RED observed1001 points instead of compact endpoints at the existing public visible-curve seam. GREEN retained the full-source unity boundary and passed the mixed-rate/two-hour drawing check. Scoped integration targets passed26 tests with one existing ignored owner-preparation test. Additional manual-envelope checks retain sampled linear gain, source endpoints and the half-open return to automatic gain; existing preview/export parity and source-boundary checks passed. Strict Clippy passed for the library, both changed integration targets and the Cargo example; formatting and diff checks passed.

No additional cross-frame cache exists, so source/edit/zoom/fit/resize/theme/scale invalidation still belongs to the unchanged renderer/application paths. Root's combined338-test/19-ignored suite and native locked-follow/live attenuation/gain-editor/zoom, compact↔wide resizing, recording-end seek, minimized completion and restoration passed before the final software acceptance follow-up. The new application real-canvas test passed1/1 across Dark/Light at1x/2x logical sizes: waveform and gain retained geometry equal fresh-state geometry; unchanged draws reuse primitive Arcs; theme/size changes produce new geometry; gain clipping and exact source endpoints remain correct. Strict library Clippy, formatting and diff checks passed after integrating1190b91. This closes the scoped software theme/scale gap without changing production behavior.

Ticket42, selection, playback, source validation and audio evaluation are unchanged. Logical-size geometry checks do not establish physical DPI changes, raster/presentation equality or physical middle-button gestures. Those observations and listening remain unclaimed. The wide4096-column software control remains above its diagnostic budget; this change does not claim to solve every follow cost. Root retains the final combined check/build receipt.
