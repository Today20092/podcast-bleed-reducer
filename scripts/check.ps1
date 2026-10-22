param([switch]$Local)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    $cargoArguments = @('--locked')
    if ($Local) {
        & "$PSScriptRoot/test-local-build.ps1"
        $settings = & "$PSScriptRoot/local-build-settings.ps1"
        $cargoArguments = $settings.CargoArguments
    }
    & "$PSScriptRoot/test-ci.ps1"
    & "$PSScriptRoot/test-packaging.ps1"
    & "$PSScriptRoot/test-release-status.ps1"
    cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting failed' }
    cargo clippy --all-targets --all-features @cargoArguments -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Clippy failed' }
    cargo test --all-targets @cargoArguments -- --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
} finally {
    Pop-Location
}
