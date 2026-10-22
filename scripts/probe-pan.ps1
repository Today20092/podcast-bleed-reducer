param([string]$Project, [switch]$Follow, [string]$AnalysisCache, [int]$Width = 1900, [switch]$Cached, [switch]$Compare)
$ErrorActionPreference = 'Stop'
$panRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if (-not $Project) { $Project = Join-Path $panRoot 'test-audio/quick-test-10min/261005-quick-test.cleanslate' }
$Project = (Resolve-Path -LiteralPath $Project).Path
$panSettings = & (Join-Path $PSScriptRoot 'local-build-settings.ps1')
$panProbeArguments = @($Project)
if ($Follow) { $panProbeArguments += '--follow' }
if ($AnalysisCache) {
    if (-not $Follow) { throw 'Actual-envelope analysis measurements require -Follow.' }
    $panProbeArguments += @([IO.Path]::GetFullPath($AnalysisCache), $Width)
}
if ($Cached) {
    if (-not $AnalysisCache) { throw '-Cached requires -AnalysisCache.' }
    $panProbeArguments += '--cached'
}
if ($Compare) {
    if (-not $AnalysisCache -or $Cached) { throw '-Compare requires -AnalysisCache and excludes -Cached.' }
    $panProbeArguments += '--compare'
}
# Cargo selects matching renderer artifacts; newest retained rlibs can belong to different feature sets.
& cargo run --manifest-path (Join-Path $panRoot 'Cargo.toml') --example pan-drawing-probe @($panSettings.CargoArguments) -- @panProbeArguments
if ($LASTEXITCODE -ne 0) { throw 'Pan drawing probe failed.' }
