# Build and develop CleanSlate

This guide covers source builds and development checks. For the editor workflow, see [the usage guide](editor-workflow.md). For detection, see [ownership and bleed](ownership-explained.md).

New contributors should start with [contributing](../CONTRIBUTING.md) for proposing work, claiming tickets and opening a pull request. The [public roadmap](../ROADMAP.md) explains current priorities and future possibilities.

## Build from source

Start feature work from `development`. Read [repository instructions](../AGENTS.md), the [branch and release workflow](branch-workflow.md), and the [active implementation tracker](../.scratch/offline-supervised-migration/implementation.md) before choosing work. Use `main` for the current release-ready source; release tags identify published packages.

The Rust package uses edition 2024 and pins Rust 1.99.0 in `rust-toolchain.toml`. Install the Windows MSVC build prerequisites. For ordinary local iteration, use `pwsh -NoProfile -File scripts/gui-dev.ps1 -BuildOnly` and `pwsh -NoProfile -File scripts/check.ps1 -Local`; these share one application-data build cache and profile across worktrees. For an intentional release-profile source build, run:

```powershell
cargo build --release --locked
./target/release/rust-app.exe
```

Analysis is acoustic-only; Nemotron and its native runtime have been removed. For repeated GUI changes, use the dedicated [GUI development guide](gui-development.md) and its build-and-launch script. That workflow preserves the saved source project by opening a separate development copy. Its default test recording is local development material and is not supplied by these instructions.

## Checks

Pull requests run the same checks as release tags on Windows x64, Linux x64,
macOS Apple Silicon and macOS Intel. Run `./scripts/check.ps1` with PowerShell 7
for hosted checks, or `./scripts/check.ps1 -Local` for local iteration with the shared cache/profile. It also checks packaging scripts and
workflow invariants. PR jobs have read-only permissions and do not build release
packages or save caches. Configure the four `Check <platform>` statuses as required
branch checks if merges must wait for them.

For compact release-run diagnostics and artifact readiness, see
[Release status](release-status.md).

```powershell
cargo test --all-targets --locked
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
```

Automated checks do not establish independent recording quality or owner listening approval. See [release evaluation](release-evaluation.md) and the tracker for outstanding qualification. Generated builds and local recording material should remain outside published artifacts.

## Packaging

Future version tags build a Windows 11 x64 portable ZIP, separate macOS Apple Silicon and Intel DMGs, and a Linux x64 AppImage. Each package includes the application, required runtime files, MIT application license, dependency notices, quick-start instructions and build checksums. Packages exclude local recordings, projects, caches and research assets.

See [the release guide](releasing.md) for native package commands, platform checks and version tags that prepare GitHub draft prereleases for manual publication. Package checks do not establish fresh physical listening or long-session qualification. Current sound/performance is owner-accepted; unperformed measurements remain unclaimed, and release checks follow the [per-release checklist](branch-workflow.md#release-checklist).

The removed Nemotron implementation remains a [research idea](research/nemotron.md) and is recoverable through Git history.

## Architecture references

- [Ownership explanation](ownership-explained.md)
- [Acoustic evidence](acoustic-evidence.md)
- [Supervised ownership model recipe](supervised-ownership-models.md)
- [Effective envelopes](effective-envelopes.md)
- [Accepted architecture](../.scratch/offline-supervised-migration/spec.md)
- [Domain glossary](../CONTEXT.md)
