param([switch]$Local)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
function Invoke-CheckCargo([string[]]$Arguments) {
    if ($Local) {
        & "$PSScriptRoot/resource-guard.ps1" -FilePath cargo -ArgumentList $Arguments
    } else {
        & cargo @Arguments
        if ($LASTEXITCODE -ne 0) { throw "Cargo check failed: $($Arguments[0])" }
    }
}
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
    & "$PSScriptRoot/test-action-contract.ps1"
    Invoke-CheckCargo (@('fmt', '--all', '--', '--check'))
    Invoke-CheckCargo (@('clippy', '--all-targets', '--all-features') + $cargoArguments + @('--', '-D', 'warnings'))
    Invoke-CheckCargo (@('test', '--all-targets') + $cargoArguments + @('--', '--nocapture'))
    Invoke-CheckCargo (@('test', '-p', 'cleanslate-ownership-model') + $cargoArguments)
} finally {
    Pop-Location
}
