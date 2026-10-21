param(
    [string]$Project,
    [Parameter(Mandatory)][string]$Report,
    [Parameter(Mandatory)][string]$Scenario
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if (-not [IO.Path]::IsPathRooted($Report)) { throw 'Use an absolute report path.' }
$Report = [IO.Path]::GetFullPath($Report)
if ((Test-Path -LiteralPath $Report) -or -not (Test-Path -LiteralPath (Split-Path -Parent $Report) -PathType Container)) {
    throw 'Use a new report path in an existing directory; existing files are never replaced.'
}
$settings = & (Join-Path $PSScriptRoot 'local-build-settings.ps1')
$exe = Join-Path $settings.TargetDir 'debug/rust-app.exe'
& (Join-Path $PSScriptRoot 'gui-dev.ps1') -BuildOnly
$source = & git -C $root rev-parse HEAD
if ($LASTEXITCODE -ne 0) { throw 'Could not record source commit.' }
if (& git -C $root status --porcelain) { $source += '-dirty' }
$variables = @{
    CLEANSLATE_PERF_REPORT = $Report
    CLEANSLATE_PERF_SCENARIO = $Scenario
    CLEANSLATE_PERF_SOURCE_COMMIT = $source
    CLEANSLATE_PERF_EXECUTABLE_SHA256 = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()
}
$previous = @{}
try {
    foreach ($name in $variables.Keys) {
        $previous[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
        [Environment]::SetEnvironmentVariable($name, $variables[$name], 'Process')
    }
    & (Join-Path $PSScriptRoot 'gui-dev.ps1') -Project $Project
} finally {
    foreach ($name in $previous.Keys) {
        [Environment]::SetEnvironmentVariable($name, $previous[$name], 'Process')
    }
}
Write-Output "Capture scenario: $Scenario. Close this development window normally to write $Report"
Write-Output 'Observations are application/update/canvas elapsed time and redraw requests, not GPU presentation or physical latency. Collect process CPU/working set separately.'
