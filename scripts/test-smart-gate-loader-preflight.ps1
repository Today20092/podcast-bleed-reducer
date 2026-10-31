#requires -Version 7.0
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'smart-gate-loader-results.ps1')
foreach ($case in @(
    @{ Code = 101; Out = ''; Err = 'generic panic' },
    @{ Code = 42; Out = ''; Err = 'generic assertion failure' },
    @{ Code = 101; Out = ''; Err = 'LOADER_RUNTIME_PATH_REJECTED: fallback' },
    @{ Code = 43; Out = ''; Err = 'LOADER_DEPENDENCY_MISSING: Windows error 127' },
    @{ Code = 0; Out = 'PASS: unexpected'; Err = 'LOADER_RUNTIME_PATH_REJECTED: fallback' }
)) {
    $rejected = $false
    try { Assert-GateLoaderNegativeResult $case.Code $case.Out $case.Err } catch { $rejected = $true }
    if (-not $rejected) { throw "Unclassified exit $($case.Code) was accepted as loader proof." }
}
Assert-GateLoaderNegativeResult 42 '' 'LOADER_RUNTIME_PATH_REJECTED: fallback'
Assert-GateLoaderNegativeResult 43 '' 'LOADER_DEPENDENCY_MISSING: Windows error 126'
Write-Output 'PASS: negative classification rejects generic panics and requires a defined loader failure.'
$fixture = Join-Path ([IO.Path]::GetTempPath()) "gate-loader-tamper-$([Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $fixture | Out-Null
try {
    # Same length as the reviewed runtime: rejection must check bytes, not just size.
    $reviewed = Get-Content -LiteralPath (Join-Path $PSScriptRoot '../docs/licenses/directml-1.15.4-runtime.json') -Raw | ConvertFrom-Json
    $runtime = $reviewed.files | Where-Object { $_.path -ceq 'DirectML.dll' }
    $stream = [IO.File]::Create((Join-Path $fixture 'DirectML.dll'))
    try { $stream.SetLength($runtime.bytes) } finally { $stream.Dispose() }
    $rejected = $false
    try {
        & (Join-Path $PSScriptRoot 'test-smart-gate-loader.ps1') -PluginDirectory $fixture -HostDirectory $fixture
    } catch {
        if ($_.Exception.Message -notlike 'Candidate DirectML differs from reviewed inventory:*') { throw }
        $rejected = $true
    }
    if (-not $rejected) { throw 'Tampered DirectML passed preflight.' }
    Write-Output 'PASS: same-length tampered DirectML rejected before host execution.'
} finally {
    # Only this script-created immediate temp child is eligible for recursive cleanup.
    if ([IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($fixture)) -ne [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\')) { throw 'Unsafe preflight fixture cleanup target.' }
    Remove-Item -LiteralPath $fixture -Recurse -Force
}
