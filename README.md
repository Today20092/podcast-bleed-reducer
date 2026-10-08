# CleanSlate

CleanSlate is a Rust/Iced podcast editor being migrated to offline, human-supervised multitrack ownership analysis and protected editable gain envelopes.

The [decision map](.scratch/offline-supervised-migration/map.md) is the sole active roadmap. All six planning decisions are resolved; the [accepted architecture](.scratch/offline-supervised-migration/spec.md) and [15 implementation tickets](.scratch/offline-supervised-migration/implementation.md) define the work ahead.

## Current implementation and target

The retained implementation imports synchronized 48 kHz mono WAVs, provides waveform review and Original/Processed/Removed audition, supports unity protections, and renders validated individual WAVs. Its detector has not passed the new independent quality gate.

The accepted target adds common mono formats and mixed native rates, 16 kHz analysis copies, required local Nemotron diarization, human reference labels, session ownership learning, and protected editable envelopes. Windows is the first release, tested for eight speakers and two-hour recordings. These are planned capabilities until their implementation tickets pass.

## Build and test

Use `codex/offline-supervised`. `main` contains the same cleaned starting point. Read [AGENTS.md](AGENTS.md) before choosing a ticket.

```powershell
cargo test --all-targets --locked
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo build --release --locked
```

Run `target/release/rust-app.exe`. The required Nemotron model is acquired explicitly through the editor's Download / verify action; installed-model verification remains offline. Identity, size and distinct weights/runtime licenses are displayed before acquisition. This adds native model acquisition/loading; integration into ownership analysis follows ticket 06.

For native Windows development, install Visual Studio 2022 C++ Build Tools with Windows SDK, CMake >=3.26 and Ninja, then run `pwsh -File scripts/build-native-runtime.ps1 -Jobs 2`. The recipe freezes NeMo-Speech.cpp, ggml and vcpkg revisions, applies the recorded PATH deduplication fix and builds the CPU diarization SDK with AVX2/FMA/F16C (no AVX-512, OpenMP or GPU). Outputs stay in ignored `native-runtime/`; native-source caches stay in ignored `.native-build/`. End users need none of these build tools, Python, keys or a GPU.

`cargo run --bin model-check -- --download` explicitly acquires/verifies the model and exercises native CPU inference. Subsequent `cargo run --bin model-check` invocations use only local bytes. `scripts/package.ps1` requires the built native package and includes its DLL dependency closure, binary manifest, licenses and headless checker beside the app. `scripts/check-package.ps1` verifies package identity and offline native inference. See [runtime/model notices](docs/runtime-notices.md).

## Audio behavior

Source audio stays unchanged. Analysis preview and live audition are separate from final WAV rendering. The migration retains conservative preservation, source identity checks and safe output transactions. See the accepted specification for new label, edit and stale-result behavior.

## Development records

- [Domain glossary](CONTEXT.md)
- [Accepted architecture decision](docs/adr/0006-offline-supervised-editable-envelopes.md)
- [Implementation sequence](.scratch/offline-supervised-migration/implementation.md)

Generated builds are ignored by Git. Listening and measured model/quality acceptance remain required before release.
