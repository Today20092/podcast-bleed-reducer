# GUI development

From the repository root, run:

```powershell
pwsh -NoProfile -File scripts/gui-dev.ps1
```

Save and close the development window before running it again. The script refuses to rebuild over a running development executable. It builds only `rust-app`, uses one Cargo job, locked/offline dependencies and incremental local compilation, then opens a separate saved-project copy. The window title is `CleanSlate - GUI development`.

The default input is `test-audio/quick-test-10min/261005-quick-test.cleanslate`. On first launch, the script copies it and available analysis/waveform sidecars to `261005-quick-test.gui-dev.cleanslate` beside it. Later launches keep that development copy and its edits. The original saved project and source audio are untouched. Ordinary source validation, cache compatibility checks and playback preparation still run; automatic opening does not bypass them. Analysis is not forced. If the saved result is stale, use Analyze explicitly.

```powershell
# Build without launching or copying a project.
pwsh -NoProfile -File scripts/gui-dev.ps1 -BuildOnly

# Use another saved project; a separate sibling copy is created.
pwsh -NoProfile -File scripts/gui-dev.ps1 -Project 'C:/path/session.cleanslate'
```

The script reuses the retained `~/.codex/worktrees/offline-timebase/podcast-automix-rust-gui/target/owner-ui` cache when present. Otherwise it uses `target/owner-ui`. `-TargetDir` overrides that location. Keep the same cache and `scripts/gui-dev.toml` settings between iterations. Switching profiles, target directories or compiler versions can rebuild dependencies. A fresh cache needs dependencies downloaded before offline builds work. Optional Nemotron still requires its normal verified runtime installation.

## Measurements on 2026-10-09

The GUI configuration now uses optimization level 1 for local app/test code, retaining incremental compilation and the existing dependency settings. The unoptimized candidate spent a median 31.67 ms preparing drawing data for a synthetic three-track, ten-minute, 1900-column zoom workload. The identical workload took 9.22 ms with light optimization, with a 15.75 ms maximum across 24 zoom changes. These are production gain-query/curve kernels, not full GPU frames or a measured display FPS. The first test-profile transition compiled local code in 98 seconds; dependency compilation was not repeated. Subsequent GUI iteration timings below describe the earlier level-0 configuration and should not be assumed for level 1.

| Build | Measured time |
|---|---:|
| Initial main-checkout build into retained cache, nonincremental, including several dependency rebuilds | 72 seconds reported by Cargo |
| First incremental local build | 31.59 seconds |
| Subsequent small GUI title change | 9.82 seconds |
| No-change GUI build | 0.80 seconds |

These are observed build times on this machine, not a controlled clean-build speed comparison or a guarantee for every change. They exclude project opening and audio preparation. Cargo timing reports are saved under the selected target's `cargo-timings` directory.

Validation: 23 applicable app tests passed, one opt-in ignored; strict all-target/all-feature Clippy, formatting, diff and PowerShell parser checks passed. Launch produced a responding native development window. The saved owner project remained byte-identical, the separate development project initially matched it, and rebuilding while that window was open was correctly rejected. Physical playback/opening latency and listening were not measured.

## Hot reload assessment

Iced 0.14 supports experimental hot reload through `iced/hot` and `cargo-hot`. Its documented limitation is patching the root crate. Our executable delegates to `rust_app::app::run()` and the GUI lives in the library target. The current cargo-hot source selects the binary/example target and reruns its rustc invocation for patches. That makes ordinary library GUI edits a poor fit for the documented path. No runtime hot-reload trial or new dependency was added; a separate executable UI boundary would need investigation before adoption.

Sources: [Iced hot-reload documentation](https://github.com/iced-rs/iced/pull/3000), [cargo-hot implementation](https://github.com/hecrj/cargo-hot/blob/master/src/main.rs), [Cargo build performance](https://doc.rust-lang.org/cargo/guide/build-performance.html).
