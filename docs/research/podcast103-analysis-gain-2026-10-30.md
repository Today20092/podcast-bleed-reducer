# Podcast bounded Analysis gain checkpoint

Ticket103 remains claimed. This slice adds a cached-source **Analysis gain**
display, not a statement of the gain currently rendered by the host callback.
Sound changes still invalidate analysis and require Analyze. Native80 and its
blocked dependants, installed links and publication are unchanged.

## Source and ownership

Baseline `c5fedc3` selectively imports only the aggregate `e29cb58..942420f`
changes to `src/engine.rs`, `src/effective_envelope.rs` and
`tests/effective_envelopes.rs`: shared110's exact bounded gain query. No Desktop
branch merge or whole-file replacement occurred. Core projection source is
`98813690c9a7fdc78a47ed87c32ca3e881837aaa`; the drawing source is committed with
this report.

`SessionCapture::analysis_view` snapshots an immutable `Arc<Published>` and
bounded per-column verified source intervals. It releases both capture state and
analysis locks before exact extrema evaluation, then uses a final nonblocking
state lock to recheck those columns plus revision/overrun. Projection queries
clamp to 2,048 columns and eight indexed edges per column, only for requested
visible members. Empty/unverified intervals have no gain detail. TooDense has an
explicit Zoom in hint, with no fabricated unity or sampled curve. Sparse member
mapping, rate/coverage bounds and half-open native-frame clipping are retained.

All production overrun setters invalidate the group revision before setting the
flag. Worker trust withdrawal therefore cannot silently retain the old published
revision. Audio-stream reset only clears its private replay-verification ring;
that ring does not define cached Analysis gain validity. No new generation is
needed. A private Weak capture identity additionally rejects cross-capture or
reopened-capture cached views. It keeps the original allocation identity from
being reused while a snapshot exists.

The editor derives the member mask from actual lane rectangles intersecting the
clip, caches by revision/range/column count/member mask, and checks atomic
validity again before drawing a cached projection. Existing waveform retention
during capture contention cannot retain invalidated gain. Gain/attenuation quads
are batched with existing egui meshes; source peaks, human labels, selection,
playhead and input gestures retain their roles. The gutter legend avoids the
human-label strip and adds no layout height.

The Arc snapshot makes the read-lock interval small and avoids full envelope or
range-index cloning. If a publisher replaces the snapshot during a query, the
last Arc may be destroyed on the query thread. Exact query work and geometry are
bounded; total UI latency including destruction is not established by that
budget. Comparable performance qualification remains open; no retirement queue
was added without a measured need.

## Semantic evidence

Actual product manifest: `plugins/podcast/Cargo.toml`. Pinned actual Cargo/Rustc,
shared offline single-job settings and selected LLD were used. The wrapper has
timeout/serialization policy only; no RAM preflight, reserve or cap. Every final
receipt records root0, no remaining job PIDs and cleanup verified. Ticket109's
broader descendant-accounting caveat remains.

- Projection RED: `20261030-223156-3d4ab3d9f56e49bc9a627e86321653b4`.
- First core GREEN: `20261030-223442-4b1249afc2b3493cae9e8f8c70036d81`.
- Three public core regressions GREEN:
  `20261030-223818-31a2799e28e443a79eec71533dc82e10` (0.04 seconds).
- Actual editor missing gain legend RED:
  `20261030-223820-2f0857ef2cec4d358b523c1e58a8f88a`.
- Compact readiness overflow RED:
  `20261030-224546-6c4eb497dd4a44efb8c1ec180309a4d5`.
- Human-label overlap assertion RED:
  `20261030-225019-4e63a7511ed04e32a33c6855cc75ee11`. The assertion was followed
  by egui's unapplied-texture Drop panic; this is an aborted test process, not a
  completed single-failure suite.
- Final compile: `20261030-225035-c853967e862d49e8884c80947ee0a6e1`.
- Full actual product suite: **86 passed**, 1.43 seconds,
  `20261030-225131-c563bed9548e49998d3a4c1a63279b5b`.
- Strict library/test Clippy: pass, 12.32 seconds,
  `20261030-225134-db14817b4b164570a515b4f14461bcf5`.
- CPU gallery: 24 states/sizes/scales pass in 7.57 seconds,
  `20261030-225236-56c609214a594922978d563a37767300`.
- All changed Rust files pass rustfmt; `git diff --check` passes.

Core regressions cover exact manual-ramp extrema, dense refusal/zoom recovery,
partial analysis clipping, more columns than native frames, sparse/visible-member
mapping, source gaps, state-lock contention, settings invalidation and
cross-capture/reopen identity. Actual editor regressions observe bounded painted
gain content and zero gain geometry after invalidation despite a retained
waveform. Scrolled-lane tests reject offscreen queries/geometry.

The new compact gallery exposed bottom571 at height560. The cause was the latent
fixed footer reserve285 plus waveform minimum140 when analyzed readiness wraps.
Reserve300 (340 with replacement) and minimum120 let the existing available-height
budget work. No action/information was hidden and the bounds assertion was not
relaxed. The new legend was not the height cause. A later real-label fixture
exposed header overlap; moving the legend into the existing gutter fixes it.

## Preserved artifact and limits

Exact Cargo compiler-artifact executable:
`%LOCALAPPDATA%/CleanSlate/BuildCache/debug/deps/cleanslate_podcast-a312d5019362a7be.exe`.
Preserved executable, receipt sets and 24 PPM/PNG pairs:
`%LOCALAPPDATA%/CleanSlate/Podcast/Workflow103Checkpoints/20261030-analysis-gain-7cbe77bfd9084dbf9e011c193cf2bf64/`.
SHA256: `e00bf099c65c8d408363156bca72947cc57d920f6e8607845dd6e05b9de8b2a4`.
GREEN receipts identify core HEAD9881369 plus the drawing/test changes committed
with this report.

The gallery covers ready/unverified/dense/stale, compact720×560 and full1000×680,
100/150/200% CPU rasterization. Compact ready/dense, full unverified100 and compact
ready200 were visually inspected: footer actions fit, gain legend and labels do
not overlap, dense detail is suppressed and source/gain gaps remain visible.
These are CPU fixture captures, not native DPI/window/presentation evidence.

This first gallery's synthetic publication helper retained the prior
`state.message`, so ready images contain an old Analyze-again message beside
ready. Production publication sets its ready message and does not have that
fixture omission. Keep these images as intermediate evidence; a separate
test-fixture correction and gallery rerun is required before representative
gallery acceptance. No broader performance, native host, listening/PDC/export,
stable release, whole103 completion or independent final visual review is claimed.

## Corrected representative gallery

Production source remains `d3ebcbe0e50dd8236813bf3d452b20d54e420bb5`, qualified by
the 86-test/full strict lint results above. A separate test-only correction sets
the synthetic publisher's message to the exact production ready message and uses
the real settings setter for the stale case. Its displayed Analyze-again reason
is asserted. No production behavior changed, and the full suite was not repeated
for fixture copy alone.

Actual product harness compile
`20261030-225700-19d720281ef54c6cad53c311b8d70efa` passes (34.08 compiler seconds).
The corrected 24-case gallery/bounds rerun
`20261030-225747-2b7710007e7a43398cb612f106abec0f` passes in 6.94 seconds.
Both receipts finalize root0, no remaining job PIDs and cleanup verified, without
memory gates. Formatting and diff checks pass. Impeccable's mechanical detector
returned no findings for the Rust editor; that is not native presentation proof.

Corrected executable, both receipt sets and 24 PPM/PNG pairs:
`%LOCALAPPDATA%/CleanSlate/Podcast/Workflow103Checkpoints/20261030-analysis-gain-fixture-7bbbffaf02184307b5ae27552cfbd909/`.
Executable SHA256:
`fa99ac03b4c0dc2d57429f79b8d27943cfa6771d39a4ddc85a3f00143d059093`.
The earlier executable/gallery remain preserved separately. Corrected compact
ready/stale100 were visually inspected; ready copy agrees with readiness and
stale copy explains Analyze while gain/attenuation are absent. The independent
root reviewer also inspected the earlier compact ready/dense100 and full
unverified200, finding readable hints, label separation, footer reachability and
gain suppression across the unverified span. Native/performance limits remain.
