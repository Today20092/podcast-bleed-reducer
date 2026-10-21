# 07: Measure and improve continuous-follow drawing reuse

Status: claimed
Assignee: perf07_implementer
Type: task
Blocked by: 01 — Measure application frame pacing and input feedback
Source proposal: P6

**What to build:** Use complete-editor and geometry measurements to decide whether continuously following the locked playhead needs additional retained geometry reuse, and implement the smallest justified improvement.

## Acceptance criteria

- [x] Measure waveform plus gain drawing together, separating CPU preparation and application frame observations from actual presented-frame metrics.
- [x] If follow geometry is a demonstrated bottleneck, add bounded reuse within the existing renderer; otherwise record the measured decision and preserve the current implementation.
- [ ] Verify clipping, source time, edits, zoom/fit, resize, themes/scales, recording bounds and middle-pan release reanchoring; bound additional memory.
- [x] Coordinate with the existing edge-catch-up proposal without implementing its separate animation behavior or changing playback/selection semantics.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. The owner confirmed the existing application, playback-output, shared preview/export envelope and benchmark/report test boundaries on 2026-10-21. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

### Preparation checkpoint — 2026-10-21

Claimed by perf07_implementer after fast-forwarding integration a530f4f and verifying ticket01 done. Read the required roadmap/architecture/GUI/tracker documents, the portable native baseline and external editor-notes.md, and applied TDD plus ponytail full. Own checkout graph is indexed as `cleanslate-perf-07`.

No product code changed and no Cargo or probe execution started. Exclusive Cargo lease is still required. The native mixed retained windows cannot be added into a complete follow-frame number or assigned to a follow interval; the earlier isolated software-pan measurement was5.72ms combined but is not follow evidence.

Prepared diagnostic-only changes extend the existing production waveform+gain software probe with `scripts/probe-pan.ps1 -Project <saved-project> -Follow`: three existing project tracks,2560 logical columns, one600→480-second zoom,240 locked-follow updates at1/60-second source-time increments, with combined/all-waveform/all-gain count and nearest-rank median/p95/p99/max. It uses existing production modules/software renderer and synthetic waveform/envelope data, excludes raster/GPU/presentation and writes no source/project/cache data. Existing pan mode is retained. This preparation has not been compiled or measured.

Resume plan: obtain root's exclusive lease; save identical src/lib.rs through apply_patch before the pinned/offline/locked/-j1 shared-target library build; run the prepared probe on the retained600-second three-track project with no concurrent compilation, repeat to expose spread and retain source/build identity. Compare complete geometry preparation with the documented60Hz16.7ms diagnostic budget and root's separate native application observations. If no material bottleneck is established, retain the renderer and document the measured conditional no-change result. If a material bottleneck is established, write one real failing application/production-canvas seam test before any product change, then add only bounded existing-renderer reuse with all invalidation keys and prove fresh-versus-reused output for clipping/source-time/edit/zoom/fit/resize/theme/scale/bounds/reanchor. The existing Iced0.14 canvas cache does not directly expose translated cached Geometry through Program::draw; Context7's fetched canvas docs describe ordinary Cache/Frame transformations, so tiled rendering would need justified complexity rather than an assumed free translation API.

Relevant existing correctness checks to run under the lease: waveform/gain/editor-drawing/timeline tests; app middle-pan, manual-pan/follow, zoom/fit and selective-invalidation real-canvas tests. Root retains native presentation/gesture/scale and combined integrated checks. Ticket42 and audio/selection semantics remain unchanged. Missing native/physical checks must remain explicit.

Root checkpoint correction: the synthetic constant effective envelope is a control, not sufficient to reject the observed native workload's cost. Before the conditional decision, match the actual3×600-second48kHz/18-reference analyzed-envelope density and480-second viewport at1900–2176 columns. The existing public `EditorAnalysisWorker::analyze` plus `EffectiveEnvelopeCache::update` (see editor_benchmark.rs) can recreate real automatic/effective envelopes from the retained project and isolated/reused analysis cache without saving the project. Add this optional real-analysis input to the diagnostic probe, or obtain equivalent real app capture, before deciding. Do not infer a no-change result from a low-density synthetic run or invent a threshold after seeing measurements. Root can repeat the native optimized capture later.

Resume preparation merged integration dd8df122 (ticket05 per-track effective reuse). Probe now accepts `-Follow -AnalysisCache <isolated-cache> -Width 2176`, creates actual analysis/effective envelopes before timing, loads all three matching retained waveform peak caches read-only (all exist), retains real18 references/protections/manual constraints, and asserts saved-project bytes unchanged. Actual private input: `C:/Users/User/Documents/Codex/Investigations/cleanslate-performance-implementation-2026-10-21/native-baseline.cleanslate`; own cache destination: sibling `perf07-analysis-cache`. Source grid/fingerprint assertions guard peak-cache selection. No Cargo/rustc/timing or production optimization has run; ticket06 owns the current lease. Width1900 is the historical control;2176 matches the native canvas. Existing16.7ms assertion remains the inherited diagnostic budget, not a newly established whole-editor or presentation acceptance threshold.

### Scoped implementation and checked handoff — 2026-10-21

Merged integration e824482 before delivery. The complete [measurement receipt](../../../docs/research/fabfilter-follow-measurements-2026-10-21.md) records real peaks/actual823-region envelopes/18 references, source/artifact identity, isolated-cache preparation,240-observation follow/cached controls, nearest-rank distributions, variance and limitations. Follow preparation is substantial; the smallest justified product change removes exact-flat interior curve vertices before existing path conversion. Equal runs retain both source-time endpoints; changing samples and the existing outside-source unity boundary remain. The same4097-point allocation bound applies and no persistent cache, new renderer/widget, epsilon or time quantization exists.

The existing manual-rustc probe failed because newest retained rlibs could represent mismatched renderer features. It now uses a Cargo example plus shared pinned/offline/locked/single-job settings; no package/version was added. The example excludes its duplicated imported unit tests under cfg(test); production module tests remain owned by the library. Diagnostic-only interleaved dense/current sampling control and bounded raw sample output add no production options. Real analysis may populate only the explicitly supplied isolated cache; source/project/waveform caches remain unchanged.

TDD RED: the public mixed-rate two-hour curve emitted1001 points instead of compact endpoints. GREEN retains3 points for a full-source0.25 curve, including outside-source unity. Both changed integration targets passed26 tests with one existing ignored owner-preparation test; manual-linear sampled gains, half-open return to automatic gain, source boundaries and existing preview/export behavior passed. Formatting, diff/PowerShell parsing and strict Clippy (`--lib --test editor_workflow --test effective_envelopes --example pan-drawing-probe`) passed. The diagnostic unit-return timer stub was repaired locally for strict Clippy; production timers were unchanged.

Two same-process alternating-order2176-column comparisons observed small complete median savings (13.831vs14.093ms; repeat9.535vs9.665ms) and gain median savings (5.111vs5.353ms; repeat3.491vs3.592ms). Repeated complete p95 was slightly worse; no broad tail/native CPU/GPU/presentation improvement is claimed. All six240-sample arrays from the repeat are retained externally as `perf07-interleaved-2176-raw.txt`, SHA-256 `79bd385bec36fc0972b78381800638f398e3d8ec695849e3378c1441bf44bbb9`. The4096-column control still exceeds the inherited16.7ms diagnostic budget; wider follow costs remain explicit rather than hidden by the modest fix.

Exclusive build/measurement lease released to root after these checks. Status remains claimed for root's combined application/canvas and native clipping/edit/zoom/fit/resize/theme/scale/recording-bound/middle-pan-release acceptance; those are not all claimed from numeric curve checks. Existing invalidation and viewport behavior are unchanged. Ticket42, playback, selection and envelope/audio contracts remain untouched. No physical gesture, listening or presented-frame measurement is claimed.

