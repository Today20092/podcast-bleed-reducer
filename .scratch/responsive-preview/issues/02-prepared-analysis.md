# 02: Reuse source analysis for warm previews

**Working branch:** codex/responsive-analysis
**Blocked by:** None
**Status:** ready-for-agent
**Completion:** implemented, tested, reviewed, and integrated
**What to build:** Retained detector facts, validated source identity and cache-aware preview derivation wired into the editor.

- [x] Cache probabilities, energies and noise floors before thresholding; sensitivity/bleed/padding use those facts.
- [x] Attenuation/Open/Close/protections reuse ownership when appropriate and rebuild only curves.
- [x] Invalidate changed/relinked/ordered sources and preprocessing identity; preserve cold APIs and render checks.
- [x] Avoid retained duplicate full PCM for analysis; demonstrate cold/warm equivalence and inference reuse.
- [x] Stage reporting is meaningful; preserve existing async behavior for later scheduling ticket.
- [x] Commit tests and real-session-capable benchmark entry point.

## Answer

`PreviewCache::analyze(paths, protections, parameters, progress, cancelled)` retains immutable detector facts and ownership through shared `Arc`s. Editor workers clone the cache and return `(AnalysisPreview, PreviewCache)` together; the existing receiver still controls installation. Waveform/source installation clears the editor cache. Removed the same-path/parameter early return so changed files are rechecked.

`PreviewCache::stats()` reports successful preparations, inferred frames and ownership derivations. `preview_stage(u8)` labels source checks, loading/validation, detector preparation, ownership and gain construction. `analyze_preview` remains a cold compatibility entry point. Final rendering still loads and validates sources independently.

Source identity reuses ordered path/byte-length/mtime stamps, checked before reuse and after preparation/derivation. `vad::PREPARATION_ID` identifies model/preprocessing/energy/frame policy and must change with that policy. No PCM is retained in the cache. Reference-session fact payload is 3,744,708 bytes, excluding ownership and allocation overhead.

### Validation

- Red/green: public cache equivalence test initially failed on the missing `PreviewCache`; editor reuse test initially failed on the missing cache integration. Both pass after implementation.
- `cargo test --test prepared_analysis`: 3 passed, covering exact cold/warm previews, detector and ownership reuse, reordered/relinked/same-size-modified sources, invalid replacement, cancellation and invalid protections.
- `cargo test --lib editor_retains_preparation`: passed, including unchanged parameters after same-path replacement.
- `cargo test --all-targets`: 40 passed, 2 existing local-evidence tests ignored.
- `cargo clippy --all-targets -- -D warnings`, formatting and `git diff --check`: passed.
- `cargo build --release --bin preview_benchmark`: passed using this worktree's own `target` directory.

### Benchmark

Run `cargo run --release --bin preview_benchmark -- 3 <stem.wav> [stem.wav ...]` with the three source paths in the spec. It reports cold milliseconds, gain-only and ownership median/p95, successful preparation/inference counts, and asserts exact restoration of the original preview. It does not write audio.

The read-only three-reference-source run was intentionally stopped at the integration task's request before it emitted timing results. It was still exercising the old gain constructor, which ticket 01 replaces. No cold/warm latency claim is made from this incomplete run. Observed process working set fell from about 2.9 GB during preparation to 39 MB afterward. Integration will record complete timings with the optimized envelope implementation. No reference WAVs were copied, retained as evidence or changed; no human listening is claimed.

### Standards review

No documented-standard breaches or actionable baseline smells found against base `528c65de57e5b78759f763934194a000a51132bf`.

### Spec review

No actionable ticket-02 findings. Source stamps cannot detect same-size replacements that preserve modification time. Scheduling and gain optimization remain assigned to their own tickets. Cancellation checks occur at detector frames and between stages/tracks; existing whole-file reads and the unchanged gain constructor cannot be interrupted internally.
