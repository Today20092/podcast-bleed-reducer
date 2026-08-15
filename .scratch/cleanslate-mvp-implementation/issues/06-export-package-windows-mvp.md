# 06 — Export and package the Windows MVP

**What to build:** A user with a current reviewed render can safely export the complete result and run CleanSlate as a self-contained Windows desktop application that satisfies the locked release contract.

**Blocked by:** 05 — Review, protect, and rerender decisions.

**Working branch:** `feature/cleanslate-mvp`

**Status:** done

- [x] Allow export only from a current render into a user-selected new or empty folder.
- [x] Write one `<source>-clean.wav` per input stem plus one JSON render report without overwriting sources.
- [x] Block existing target collisions and require another destination.
- [x] Commit the complete export set atomically so failure or cancellation leaves no partial export.
- [x] Package the application for Windows with the pinned embedded model and CPU runtime, requiring no adjacent ONNX Runtime DLL.
- [x] Run the deterministic, labeled-reference, endurance, safe-cancellation, and listening release checks against the packaged application.
- [x] Defer adjustable attenuation, installer work, GPU execution, resampling, and macOS/Linux packaging.

## Verification

- `cargo test --all-targets`, `cargo fmt --all -- --check`, strict all-target/all-feature Clippy, and `cargo build --release --locked`: passed.
- `scripts/package.ps1` and `scripts/check-package.ps1`: passed; the package contains only `CleanSlate.exe` and `release_gate.exe`, with the pinned model embedded and no adjacent ONNX Runtime DLL.
- Packaged labeled gate: `../evidence/packaged-labeled-reference-2/release-gate.md` passed with 2,565,744 preserved samples, 131,376 preserved overlap samples, and reduced bleed RMS.
- Packaged endurance gate: `../evidence/packaged-full-session/release-gate.md` completed all three 239,660,221-sample stems atomically in 664.063 seconds with independent integrity verification.
- Packaged cancellation: `../evidence/packaged-cancellation.md` passed through `release_gate.exe --cancel` and left no destination. Deterministic export coverage also verifies delivery names, untouched sources, empty-folder export, collision blocking, stale-render blocking, and cancellation cleanup.
- Existing ticket 03 human listening sign-off is recorded in the packaged labeled and endurance evidence because packaging uses the same pinned engine and embedded model.
- Standards/spec review completed against `dc44121`; the packaged cancellation and listening-evidence findings were addressed. The reported empty-folder concern was disproved by the passing Windows integration test.
