# Builds the shipping Smart Gate folder. Does not sign the binary and does not copy it into Program Files.
$ErrorActionPreference = "Stop"
$plugins = Split-Path -Parent $PSScriptRoot
$repo = Split-Path -Parent $plugins
$gate = Join-Path $PSScriptRoot "build\CleanSlateSmartGate_artefacts\Release\VST3\CleanSlate Smart Gate.vst3"
$dll = Join-Path $plugins "smart-gate\target\release\cleanslate_smart_gate.dll"
cargo build --release --manifest-path (Join-Path $plugins "smart-gate\Cargo.toml")
cmake --build (Join-Path $PSScriptRoot "build") --config Release --target CleanSlateSmartGate_VST3 -- /m /nologo
$destDll = Join-Path $gate "Contents\x86_64-win\cleanslate_smart_gate.dll"
Copy-Item -LiteralPath $dll -Destination $destDll -Force
$win = Join-Path $gate "Contents\x86_64-win"
Copy-Item -LiteralPath (Join-Path $PSScriptRoot "THIRD-PARTY-NOTICES.txt") -Destination (Join-Path $win "THIRD-PARTY-NOTICES.txt") -Force
Copy-Item -LiteralPath (Join-Path $PSScriptRoot "LICENSES\AGPL-3.0.txt") -Destination (Join-Path $win "AGPL-3.0.txt") -Force
Copy-Item -LiteralPath (Join-Path $repo "docs\licenses\embedded-model-runtime.txt") -Destination (Join-Path $win "embedded-model-runtime.txt") -Force
Write-Output "Packaged $gate"
