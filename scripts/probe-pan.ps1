param([string]$Project)
$ErrorActionPreference = 'Stop'
$panRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if (-not $Project) { $Project = Join-Path $panRoot 'test-audio/quick-test-10min/261005-quick-test.gui-dev.cleanslate' }
$Project = (Resolve-Path -LiteralPath $Project).Path
$panTarget = Join-Path $env:USERPROFILE '.codex/worktrees/offline-timebase/podcast-automix-rust-gui/target/owner-ui'
if (-not (Test-Path -LiteralPath $panTarget)) { $panTarget = Join-Path $panRoot 'target/owner-ui' }
$panDeps = Join-Path $panTarget 'debug/deps'
$panApp = (Get-ChildItem -LiteralPath $panDeps -Filter 'librust_app-*.rlib' | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
$panIced = (Get-ChildItem -LiteralPath $panDeps -Filter 'libiced-*.rlib' | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
$panSkia = (Get-ChildItem -LiteralPath $panDeps -Filter 'libiced_tiny_skia-*.rlib' | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
$panSerde = (Get-ChildItem -LiteralPath $panDeps -Filter 'libserde-*.rlib' | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
$panJson = (Get-ChildItem -LiteralPath $panDeps -Filter 'libserde_json-*.rlib' | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
# shortcut: Windows import-library version follows this lockfile; update after native dependency upgrades.
$panNative = (Get-ChildItem (Join-Path $env:USERPROFILE '.cargo/registry/src/*/windows_x86_64_msvc-0.52.6/lib/windows.0.52.0.lib') | Select-Object -First 1).DirectoryName
if (-not $panApp -or -not $panIced -or -not $panSkia -or -not $panSerde -or -not $panJson -or -not $panNative) { throw 'Build the GUI with gui-dev.ps1 first to populate the retained dependencies.' }
$panSource = Join-Path $panRoot '.scratch/offline-supervised-editor/pan-drawing-probe.rs'
$panExe = [IO.Path]::ChangeExtension($panSource, '.exe')
# Compile the current production canvas directly; no project/source/cache writes.
& rustc --edition 2024 $panSource --extern "rust_app=$panApp" --extern "iced=$panIced" --extern "iced_tiny_skia=$panSkia" --extern "serde=$panSerde" --extern "serde_json=$panJson" -L "dependency=$panDeps" -L "native=$panNative" -C opt-level=1 -C debuginfo=0 -A warnings -o $panExe
if ($LASTEXITCODE -ne 0) { throw 'Pan probe compilation failed.' }
& $panExe $Project
if ($LASTEXITCODE -ne 0) { throw 'Pan drawing exceeds the probe budget.' }
