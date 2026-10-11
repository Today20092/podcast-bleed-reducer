# Build and develop CleanSlate

This guide covers source builds and development checks. For the editor workflow, see [the usage guide](editor-workflow.md). For detection, see [ownership and bleed](ownership-explained.md).

## Build from source

Start from `main`. Read [repository instructions](../AGENTS.md) and the [active implementation tracker](../.scratch/offline-supervised-migration/implementation.md) before choosing work.

The Rust package uses edition 2024. Install a compatible Rust toolchain and the Windows MSVC build prerequisites, then run these commands from the repository root:

```powershell
cargo build --release --locked
./target/release/rust-app.exe
```

Analysis is acoustic-only; Nemotron and its native runtime have been removed. For repeated GUI changes, use the dedicated [GUI development guide](gui-development.md) and its build-and-launch script. That workflow preserves the saved source project by opening a separate development copy. Its default test recording is local development material and is not supplied by these instructions.

## Checks

```powershell
cargo test --all-targets --locked
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
```

Automated checks do not establish independent recording quality or owner listening approval. See [release evaluation](release-evaluation.md) and the tracker for outstanding qualification. Generated builds and local recording material should remain outside published artifacts.

## Packaging

Future version tags build a Windows 11 x64 portable ZIP, separate macOS Apple Silicon and Intel DMGs, and a Linux x64 AppImage. Each package includes the application, required runtime files, MIT application license, dependency notices, quick-start instructions and build checksums. Packages exclude local recordings, projects, caches and research assets.

See [the release guide](releasing.md) for native package commands, platform checks and future version tags that prepare GitHub draft prereleases for manual publication. Package checks do not establish listening approval or long-session qualification; those remain pending in the release evaluation tracker.

The removed Nemotron implementation remains a [research idea](research/nemotron.md) and is recoverable through Git history.

## Architecture references

- [Ownership explanation](ownership-explained.md)
- [Acoustic evidence](acoustic-evidence.md)
- [Supervised ownership model recipe](supervised-ownership-models.md)
- [Effective envelopes](effective-envelopes.md)
- [Accepted architecture](../.scratch/offline-supervised-migration/spec.md)
- [Domain glossary](../CONTEXT.md)
