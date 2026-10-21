# 01: Measure application frame pacing and input feedback

Status: done
Assignee: perf01_implementer
Type: task
Blocked by: None (can start immediately)
Source proposal: P2

**What to build:** Provide an opt-in, bounded performance capture that exercises the real editor and reports reproducible frame/interaction observations alongside existing processing and playback measurements.

## Acceptance criteria

- [x] Reuse the current preview/editor/pan/device probes and add only missing application-level observations; normal operation has no unbounded event history or per-frame file I/O.
- [x] Capture idle, playback, locked-follow, pan, zoom and sound edits with/without background work using reproducible commands and scenario metadata. Representative native mixed-run and software-pan coverage are recorded below; isolated action/background combinations and physical middle-drag are not claimed.
- [x] Report sample counts and median/p95/p99 where meaningful, input-to-visible/update timing, rebuild counts and resource observations; name the exact observed boundary. CPU geometry time and redraw requests must never be labeled presented GPU frames or physical display latency.
- [x] Record source/build identity and available display/device information; unavailable hardware metrics are explicit rather than fabricated.
- [x] Retain a baseline for dependent tickets and add public-report or application-boundary checks through the confirmed test seams; preserve source files and existing projects.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. The owner confirmed the existing application, playback-output, shared preview/export envelope and benchmark/report test boundaries on 2026-10-21. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

### Scoped implementation and validation — 2026-10-21

Implemented a bounded optional recorder selected by CLEANSLATE_PERF_REPORT and CLEANSLATE_PERF_SCENARIO. It observes actual application-update spans, worker polls, native redraw requests, waveform/gain draw and cache-build closures, input counts/latest-input-to-first-canvas-completion and window size/scale events. Its passive raw-event filter returns no application messages, and requests no timer, layout or redraw. Capture disabled adds no observation clock, lock, history or file write. Enabled capture retains at most 4096 samples per each of8 fixed metrics, records overwritten counts and writes one new JSON file on graceful application exit; existing files are refused. Labels are bounded and control characters removed. No keys, text, project paths or audio contents are recorded.

The profile-gui launcher uses the existing separate GUI-development project flow, shared build settings, source commit/dirty marker and built executable hash. Collection overhead, launch-provided identity, retained-window nearest-rank statistics, cache-closure versus whole-canvas spans and unavailable hardware/process measurements are explicit in the GUI guide. Root will collect process CPU/working-set data as a separate labeled companion; no fake CPU/GPU/presentation metrics are generated.

TDD RED: the new public report test failed with unresolved rust_app::performance before implementation. GREEN: all 3 performance_capture integration tests passed, covering empirical percentiles/labels, bounded rolling history and existing-file preservation. Scoped strict Clippy (--lib --test performance_capture), cargo fmt check, diff check and PowerShell parsing passed. All commands used local-build-settings and the pinned/shared single-job configuration. A queued worktree build selected an older library; verified perf01 metadata/root, re-saved the identical lib module line to update its local mtime, then explicit --lib verbose build showed Dirty src/lib.rs and correct perf01 CARGO_MANIFEST_DIR. The exclusive fresh library and test runs then passed, without cache/profile/dependency changes.

The old pan command failed against retired target/owner-ui paths. After reusing shared target settings, the standalone probe also exposed obsolete private source/timeline/appearance interfaces. Its guarded unused-loading stubs and production module aliases were updated; it now compiles current waveform/gain/timeline/color code without source/cache writes. The read-only three-track/1900-column/40-pan software-geometry scenario passed at median 5.72ms/max 9.62ms on the saved quick-test project. This is a fresh synthetic CPU/software geometry observation, not native displayed FPS or GPU timing, and is not a comparison against the historical probe's different source/build context.

This implementation was initially claimed pending root's native baseline. The receipt below completes representative instrumentation/baseline acceptance; it does not claim a full isolated scenario matrix or physical middle-drag. Root retains combined integration checks and GUI preservation. Forced termination, listening/physical latency and GPU presentation are outside this recorder's evidence.

### Native baseline and completion — 2026-10-21

Root captured the unoptimized native baseline at source e520273aaa4f365fd05d308cac2d9504a2ae97af before ticket02 integration. Executable SHA-256: 7efdaba74806ef0d516377012ce7d0e4b3efcff101f8b6e767583d42ca0283d2. The valid report and OS companion are retained outside the repo as baseline-native.json and baseline-os.json; [the portable baseline summary](../../../docs/research/fabfilter-performance-baseline-2026-10-21.md) contains artifact/source hashes, workload, native actions, per-metric counts/retained-window percentiles, separate OS samples and limitations.

The native run used a private working copy with three 600-second 48 kHz tracks and 18 references, maximized 2560×1392, final logical client 2560×1369. It covered stopped idle, Analyze concurrent with Original playback (one approximately 22.5-second job), zoom600→480 seconds, gain-editor display, warm attenuation−11→−12 dB, locked follow, minimize, restore with playback continuing and graceful Alt+F4. No compilation ran during the three separate five-second OS samples. Original project SHA-256 remained 20bb912825964a2ba9b717525477f6af365abb3e434084cf7836e6b5725cbd98; the separate post-capture working-copy hash is recorded in the summary. No recording contents or source/project paths are included in the report.

This single 372.578-second mixed capture retains only the latest4096 observations per metric. It is not an isolated per-action timing study: native redraw interval median 9.773ms/p95 30.022ms; individual waveform draw median 3.700ms/p95 4.214ms; gain draw median 2.232ms/p95 2.568ms;15 mixed latest-input-to-first-canvas measurements median 5.079ms/p95=max 155.662ms. Each retained window covers a different interval; cache-build counts are canvas closure counts, not frames. They cannot be aligned retrospectively to specific actions or the OS windows, and do not establish presented FPS, GPU work, physical feedback or sound latency.

Separate process samples observed 0.30% one-core CPU stopped,97.29% visible locked follow with gain editor,105.47% minimized; corresponding maximum sampled working sets were 189.76/233.73/233.79MiB. These are one run with ten samples per phase, not statistically established changes or peak memory; one-core units aggregate process threads and are not total-machine CPU. The minimized finding informs ticket06 without proving the cause. Native middle-button drag was unavailable through the automation API; the software pan probe is the available geometry coverage, and physical gesture acceptance remains separate. Initial scale, audio-device identity/rate, GPU presentation and listening/onset were not measured here.

