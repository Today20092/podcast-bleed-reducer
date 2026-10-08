# Introduce a source-aware timebase and versioned project contract

Status: done
Assignee: timebase_implementation
Blocked by: 01
Spec: ../spec.md

## Scope

Make source sample rate/frame count explicit and use a checked common integer time coordinate for labels, regions, protections and editable envelopes. Define stable track/speaker IDs and one dedicated track per speaker, with up to eight speakers. Convert native sample indices through their own rates; remove implicit 48 kHz coordinate contracts. Preserve half-open intervals and rounding rules that cannot shorten preserved speech.

Create the fresh versioned project contract with source bindings, speaker map, active-speaker-set reference labels, settings, manual overrides/protections, review/result/render identity and recipe provenance. No old-project conversion is required. Validate on load/save, save atomically, and distinguish a moved identical source from changed audio. Bound labels/edits on changed sources remain recorded but cannot silently apply until explicitly reviewed/rebound. Cache loss must not make a project uneditable.

## Reach

src/audio_io.rs Stem metadata, src/calibration.rs CalibrationExample/OwnershipSettings, src/project.rs Project/open/save/source invalidation, src/engine.rs Region/Protection/GainEnvelope, src/timeline.rs and app/waveform/quality coordinate consumers. Update existing src/bin/{ownership_diagnose,ownership_evaluate,preview_benchmark,release_gate}.rs callers, exports and fixtures using fixed 48,000 conversions or old sample-index assumptions.

## Acceptance

- Equal physical timestamps at 44.1/48/96 kHz map correctly, including first/last samples and large two-hour indices.
- Malformed versions/IDs, contradictory labels, invalid bounds/nonfinite values and checked-arithmetic overflow fail safely.
- Track reordering and relinking cannot transfer a speaker, label or override to different audio.
- Atomic save failure retains prior project data; save/reload retains confirmed human decisions.
- Current callers compile against one consistent contract rather than parallel incompatible timebases.

## Implementation evidence

The project clock uses 28,224,000 checked integer ticks per second, exact for 44.1/48/96 kHz. Native frame grids stay explicit; half-open preserved edges round outward. Existing timeline, reference, review, evaluation, waveform, audition and render consumers now use physical time or a declared source/detector grid.

Fresh schema version 1 persists stable track/speaker IDs, SHA-256 source bindings, active-speaker-set references, manual curves/protections, result/render identities, recipe provenance and review states. Atomic replacement preserves prior disk and in-memory state on failure. Identical moved sources relink; changed sources retain dormant human decisions until explicit source review and rebinding. Reordering reconstructs index-based detector adapters from stable IDs.

Validation on Windows, 2026-10-08:

- `cargo fmt --all -- --check` and `git diff --check` passed.
- `CARGO_INCREMENTAL=0 cargo test --all-targets --locked -j 1 --target-dir target/finish-timebase --config 'profile.test.package.rust-app.debug=0'`: 92 passed, 0 failed, 6 ignored. Includes three timebase tests, nine project persistence/relinking tests, app source-change/job rejection checks and existing playback/render/export tests.
- Four old app fixtures assumed deleted or replaced sources could still receive decisions or reanalysis. The tests now assert blocking before explicit relink/review, preserved completed-preview identity, and rejection of obsolete jobs; protection actions run against available bound audio.
- A debug-symbol build encountered `rustc-LLVM ERROR: out of memory`. The successful serial check disables debug symbols only for this package; dependencies and runtime assertions are unchanged. No toolchain or system settings changed.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets --all-features --locked -j 1 --target-dir target/finish-timebase --config 'profile.dev.package.rust-app.debug=0' --config 'profile.test.package.rust-app.debug=0' -- -D warnings` passed. Removed unnecessary casts and identity operations in migrated test fixtures without changing expected values.

General decoding, 16 kHz analysis copies and mixed-rate device preparation remain tickets 03/04. This ticket defines and adapts the time/project contract; it does not claim those later capabilities or release listening acceptance.
