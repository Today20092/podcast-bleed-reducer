# CleanSlate

CleanSlate is a standalone Windows podcast automixer written in Rust with Iced. Import synchronized 48 kHz mono WAV stems, inspect ownership and gain curves, audition Original/Processed/Removed signal, protect regions, and export validated processed WAVs.

The `codex/cleanslate-responsive` branch integrates the responsive-preview work. Its implementation and acceptance record are in [.scratch/responsive-preview](.scratch/responsive-preview/spec.md). Completion evidence is recorded per ticket; the integration ticket tracks the final build.

## Build and test

```powershell
cargo test --all-targets --locked
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo build --release --locked
```

Run `target/release/rust-app.exe`. `scripts/package.ps1` packages the release app as `dist/CleanSlate-windows-x64/CleanSlate.exe`; `scripts/check-package.ps1` checks that package's contents.

## Audio behavior

Source audio stays unchanged. An analysis preview and live audition are separate from a final WAV render. A changed parameter can make an existing render stale without disabling audition. Protections and uncertain evidence favor preservation.

Saved sensitivity values retain the legacy speech-threshold meaning. The responsive editor presents higher sensitivity as admitting weaker evidence. Gain-aware waveform summaries are conservative display approximations; audition and render use the shared actual envelope.

## Development records

- [Domain glossary](CONTEXT.md)
- [Architectural decisions](docs/adr)
- [Responsive preview specification](.scratch/responsive-preview/spec.md)
- [Delegated design decisions](.scratch/responsive-preview/decisions.md)
- [Integration acceptance](.scratch/responsive-preview/issues/07-integration.md)

Benchmarks and automated checks do not replace listening review of a newly built application.
