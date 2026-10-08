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

Run `target/release/rust-app.exe`. `scripts/package.ps1` packages the release app as `dist/CleanSlate-windows-x64/CleanSlate.exe`; `scripts/check-package.ps1` checks that package's contents.

## Audio behavior

Source audio stays unchanged. Analysis preview and live audition are separate from final WAV rendering. The migration retains conservative preservation, source identity checks and safe output transactions. See the accepted specification for new label, edit and stale-result behavior.

## Development records

- [Domain glossary](CONTEXT.md)
- [Accepted architecture decision](docs/adr/0006-offline-supervised-editable-envelopes.md)
- [Implementation sequence](.scratch/offline-supervised-migration/implementation.md)

Generated builds are ignored by Git. Listening and measured model/quality acceptance remain required before release.
