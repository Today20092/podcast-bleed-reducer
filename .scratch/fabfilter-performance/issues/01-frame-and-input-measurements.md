# 01: Measure application frame pacing and input feedback

Status: claimed
Assignee: perf01_implementer
Type: task
Blocked by: None (can start immediately)
Source proposal: P2

**What to build:** Provide an opt-in, bounded performance capture that exercises the real editor and reports reproducible frame/interaction observations alongside existing processing and playback measurements.

## Acceptance criteria

- [x] Reuse the current preview/editor/pan/device probes and add only missing application-level observations; normal operation has no unbounded event history or per-frame file I/O.
- [ ] Capture idle, playback, locked-follow, pan, zoom and sound edits with/without background work using reproducible commands and scenario metadata.
- [x] Report sample counts and median/p95/p99 where meaningful, input-to-visible/update timing, rebuild counts and resource observations; name the exact observed boundary. CPU geometry time and redraw requests must never be labeled presented GPU frames or physical display latency.
- [ ] Record source/build identity and available display/device information; unavailable hardware metrics are explicit rather than fabricated.
- [ ] Retain a baseline for dependent tickets and add public-report or application-boundary checks through the confirmed test seams; preserve source files and existing projects.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. The owner confirmed the existing application, playback-output, shared preview/export envelope and benchmark/report test boundaries on 2026-10-21. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

### Scoped implementation and validation — 2026-10-21

Implemented a bounded optional recorder selected by CLEANSLATE_PERF_REPORT and CLEANSLATE_PERF_SCENARIO. It observes actual application-update spans, worker polls, native redraw requests, waveform/gain draw and cache-build closures, input counts/latest-input-to-first-canvas-completion and window size/scale events. Its passive raw-event filter returns no application messages, and requests no timer, layout or redraw. Capture disabled adds no observation clock, lock, history or file write. Enabled capture retains at most4096 samples per each of8 fixed metrics, records overwritten counts and writes one new JSON file on graceful application exit; existing files are refused. Labels are bounded and control characters removed. No keys, text, project paths or audio contents are recorded.

The profile-gui launcher uses the existing separate GUI-development project flow, shared build settings, source commit/dirty marker and built executable hash. Collection overhead, launch-provided identity, retained-window nearest-rank statistics, cache-closure versus whole-canvas spans and unavailable hardware/process measurements are explicit in the GUI guide. Root will collect process CPU/working-set data as a separate labeled companion; no fake CPU/GPU/presentation metrics are generated.

TDD RED: the new public report test failed with unresolved rust_app::performance before implementation. GREEN: all3 performance_capture integration tests passed, covering empirical percentiles/labels, bounded rolling history and existing-file preservation. Scoped strict Clippy (--lib --test performance_capture), cargo fmt check, diff check and PowerShell parsing passed. All commands used local-build-settings and the pinned/shared single-job configuration. A queued worktree build selected an older library; verified perf01 metadata/root, re-saved the identical lib module line to update its local mtime, then explicit --lib verbose build showed Dirty src/lib.rs and correct perf01 CARGO_MANIFEST_DIR. The exclusive fresh library and test runs then passed, without cache/profile/dependency changes.

The old pan command failed against retired target/owner-ui paths. After reusing shared target settings, the standalone probe also exposed obsolete private source/timeline/appearance interfaces. Its guarded unused-loading stubs and production module aliases were updated; it now compiles current waveform/gain/timeline/color code without source/cache writes. The read-only three-track/1900-column/40-pan software-geometry scenario passed at median5.72ms/max9.62ms on the saved quick-test project. This is a fresh synthetic CPU/software geometry observation, not native displayed FPS or GPU timing, and is not a comparison against the historical probe's different source/build context.

Status remains claimed for root's native scenario baseline/receipt work after merging this implementation, before ticket02's optimization. Exact idle/playback/follow/pan/zoom/edit scenarios with/without jobs, current machine metadata and complete native observations have not been claimed passed. Root will perform combined integration checks and GUI preservation. Forced termination, listening/physical latency and GPU presentation are outside this recorder's evidence.

