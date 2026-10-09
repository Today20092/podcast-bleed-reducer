# Build and develop CleanSlate

This guide covers source builds and development checks. For the editor workflow, see [the usage guide](editor-workflow.md). For detection, see [ownership and bleed](ownership-explained.md).

## Build from source

Use the `codex/offline-supervised` branch. Read [repository instructions](../AGENTS.md) and the [active implementation tracker](../.scratch/offline-supervised-migration/implementation.md) before choosing work.

The Rust package uses edition 2024. Install a compatible Rust toolchain and the Windows MSVC build prerequisites, then run these commands from the repository root:

```powershell
cargo build --release --locked
./target/release/rust-app.exe
```

Acoustic-first analysis requires no Nemotron download. For repeated GUI changes, use the dedicated [GUI development guide](gui-development.md) and its build-and-launch script. That workflow preserves the saved source project by opening a separate development copy. Its default test recording is local development material and is not supplied by these instructions.

## Checks

```powershell
cargo test --all-targets --locked
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
```

Automated checks do not establish independent recording quality or owner listening approval. See [release evaluation](release-evaluation.md) and the tracker for outstanding qualification. Generated builds and local recording material should remain outside published artifacts.

## Experimental Nemotron runtime

Nemotron-assisted analysis is work in progress, disabled by default. Its benefit over acoustic-only analysis has not been established. This setup is for development and comparative evaluation, not a requirement for the ordinary acoustic workflow.

The experimental path requires its verified model and native CPU runtime. The editor's Download / verify action explicitly acquires the model; verification of installed assets remains offline. The editor displays model identity, size and the distinct weights/runtime licenses before acquisition.

For native Windows runtime development, install Visual Studio 2022 C++ Build Tools with Windows SDK, CMake >=3.26 and Ninja, then run:

```powershell
pwsh -File scripts/build-native-runtime.ps1 -Jobs 2
```

The recipe freezes NeMo-Speech.cpp, ggml and vcpkg revisions, applies the recorded PATH deduplication fix, and builds the CPU diarization SDK with AVX2/FMA/F16C. AVX-512, OpenMP and GPU backends are disabled. Outputs stay in ignored `native-runtime/`; native-source caches stay in ignored `.native-build/`. Packaged users do not need the runtime build tools, Python, keys or a GPU.

```powershell
# Explicitly download/verify the model and exercise native CPU inference.
cargo run --bin model-check -- --download

# Use installed local bytes for subsequent checks.
cargo run --bin model-check
```

See [runtime and model notices](runtime-notices.md) for exact identities, licenses and redistribution requirements.

## Packaging

[scripts/package.ps1](../scripts/package.ps1) requires the built native package and includes its DLL dependency closure, binary manifest, licenses and headless checker beside the app. [scripts/check-package.ps1](../scripts/check-package.ps1) verifies package identity and offline native inference. Packaging checks do not replace the Windows release qualification ticket.

## Architecture references

- [Ownership explanation](ownership-explained.md)
- [Acoustic evidence](acoustic-evidence.md)
- [Supervised ownership model recipe](supervised-ownership-models.md)
- [Effective envelopes](effective-envelopes.md)
- [Accepted architecture](../.scratch/offline-supervised-migration/spec.md)
- [Domain glossary](../CONTEXT.md)
