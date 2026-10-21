# 02: Retain unchanged waveform geometry during worker polling

Status: done
Assignee: perf02_implementer
Type: task
Blocked by: None (can start immediately)
Source proposal: P1

**What to build:** Keep the existing waveform/gain caches valid when a background poll or unrelated interface action cannot change their contents, while every visible data, style, size and viewport change still redraws correctly.

## Acceptance criteria

- [x] An unchanged background poll does not rebuild static waveform/gain geometry; record baseline and changed rebuild/timing evidence.
- [x] Source/analysis/preview completion, selection and manual edits, theme/resize, zoom and viewport changes remain visible without stale geometry.
- [x] Playhead and transient overlays continue to move without forcing unrelated static geometry work; keep lock, fit and middle-pan semantics.
- [x] Use observable application/drawing behavior at confirmed seams to demonstrate the regression before the fix and verify it afterwards.

## Context

Follow the sibling accepted specification and the owner's FabFilter performance investigation. The owner confirmed the existing application, playback-output, shared preview/export envelope and benchmark/report test boundaries on 2026-10-21. Claim this ticket and record an assignee before implementation. Measurement-dependent changes retain their condition; unsupported or inconclusive checks must be reported rather than marked passed.

## Comments

2026-10-21: Owner authorized all eight report proposals as separate tickets and requested implement-spec execution on one integration branch. No production work or acceptance check is claimed by creating this record.

2026-10-21 — perf02_implementer claimed the independent ticket on `codex/perf-02-invalidation`, verified based on integration `492d2ae`, and read the accepted test-boundary confirmation. The test-only renderer exposure uses the already locked/transitively installed `iced_tiny_skia`0.14.0; no dependency version or additional package is introduced.

First TDD slice uses application Tick dispatch and the production waveform canvas's actual cached software primitive storage. A live empty source-inspection channel keeps the worker-poll scenario active. RED:100 empty polls caused100 cache rebuilds and the zero-rebuild assertion failed. GREEN after suppressing generic Tick invalidation:100 empty polls caused0 rebuilds. Single-run loop observations were4,925us before and3,009us after, including uncached overlay preparation; these are synthetic software observations, not native/GPU frame or physical latency measurements. Rebuild counts establish the targeted regression; the small timing sample is not a general speedup claim.

Next slice checks progressive source waveform installation, status-only progress reuse and failure restoring the previous drawing. Cargo starts are coordinated exclusively by the integration owner because target-lock waits alone allowed cross-worktree artifact reuse; this does not change the pinned build profile or target.

Second TDD slice: with the Tick-only optimization, a genuinely different progressive waveform reused the old cached primitive storage, so the visible-data test failed. The fix invalidates on real source/import data and completion, source-inspection acceptance, accepted analysis installation and warm effective-envelope installation. Empty/status-only polls retain the cache. The regression now compares changed drawing against a fresh canvas, verifies status-only reuse, and verifies failure restores the original waveform drawing. The full affected application group passed:68 tests,0 failures,4 existing opt-in checks ignored. Existing checks cover accepted imports, cancellation/stale jobs, manual edits and appearance, zoom/fit/lock/middle-pan, playback frames and export safeguards.

## Completion evidence

Implementation commit: `a33ebb1`; merged current integration tip `e520273` into this worktree at `2580e2c` before final verification. No merge conflicts or new dependency packages/versions. The shared revision is now retained for unchanged polls; both production waveform/gain caches use it. Canvas size/theme invalidation and uncached moving overlays remain unchanged. Other controller messages retain conservative invalidation rather than a fragile broad whitelist.

At the merged source, shared local-build settings (locked/offline, single job, pinned profile/target) passed:

- `cargo test --lib app::tests::`: 68 passed, 0 failed, 4 existing opt-in ignored; both new cache regressions passed.
- `cargo test --lib waveform::`: 16 passed, 0 failed, 1 existing native-reference visual check ignored.
- `cargo test --lib gain_editor::` matched 0 cases (not counted as coverage); existing application gain/edit tests above cover that interaction path.
- `cargo clippy --lib --tests -- -D warnings`, all-target formatting check and diff checks passed.

The cache tests observe real production waveform geometry and fresh-drawing equivalence rather than asserting only the private revision. Code inspection verifies that source/import, accepted analysis and effective-envelope data writers dirty the common key, while the existing app/waveform tests retain manual edit, cancellation, appearance, zoom/lock/pan and overlay behavior. No new waveform/gain algorithms, audio math, cache format, source-validation or export paths were introduced.

Native GUI presentation/FPS, physical input/listening and the final combined executable were not checked in this isolated ticket; those remain integration-level observations. The two single-run software loop timings above are not a reproducible hardware speed claim. The ticket's scoped behavioral/cache checks are complete.

