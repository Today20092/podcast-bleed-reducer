#requires -Version 7.0
$ErrorActionPreference = 'Stop'
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
