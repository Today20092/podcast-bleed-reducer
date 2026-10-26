#requires -Version 7.0
param([string]$TargetDir, [string]$ProbeBinary)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ([Environment]::OSVersion.Platform -ne 'Win32NT') { throw 'This candidate script currently targets Windows x64 only.' }
$root = Split-Path -Parent $PSScriptRoot
$parent = Join-Path $root 'dist/smart-gate-candidates'
function Assert-GateOutputParents {
    foreach ($directory in @($root, (Join-Path $root 'dist'), $parent)) {
        try { $item = Get-Item -LiteralPath $directory -Force -ErrorAction Stop }
        catch [System.Management.Automation.ItemNotFoundException] { continue }
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) {
            throw "Smart Gate output must not follow a link: $directory"
        }
        if (-not $item.PSIsContainer) {
            throw "Smart Gate output parent must be a directory: $directory"
        }
    }
}
Assert-GateOutputParents
$manifest = Join-Path $root 'plugins/smart-gate/Cargo.toml'
$settings = & (Join-Path $PSScriptRoot 'local-build-settings.ps1') -TargetDir $TargetDir
function Invoke-GateCargo([string[]]$Arguments) {
    & (Join-Path $PSScriptRoot 'resource-guard.ps1') -FilePath cargo -ArgumentList $Arguments -WorkingDirectory $root
}
# All Cargo steps keep the heavy policy; callers cannot relabel compilation as lightweight.
Invoke-GateCargo (@('fmt', '--manifest-path', $manifest, '--all', '--', '--check'))
Invoke-GateCargo (@('test', '--manifest-path', $manifest, '--all-targets') + $settings.CargoArguments)
Invoke-GateCargo (@('clippy', '--manifest-path', $manifest, '--all-targets') + $settings.CargoArguments + @('--', '-D', 'warnings'))
Invoke-GateCargo (@('build', '--manifest-path', $manifest, '--lib', '--examples') + $settings.CargoArguments)
Assert-GateOutputParents
New-Item -ItemType Directory -Path $parent -Force | Out-Null
$candidate = Join-Path $parent ([Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $candidate | Out-Null
$bundle = Join-Path $candidate 'CleanSlate Smart Gate.vst3/Contents/x86_64-win'
New-Item -ItemType Directory -Path $bundle -Force | Out-Null
$dll = Join-Path $settings.TargetDir 'debug/cleanslate_smart_gate.dll'
$artifacts = @((Join-Path $candidate 'CleanSlate Smart Gate.clap'), (Join-Path $bundle 'CleanSlate Smart Gate.vst3'))
foreach ($path in $artifacts) { Copy-Item -LiteralPath $dll -Destination $path }
Copy-Item -LiteralPath (Join-Path $root 'plugins/smart-gate/README.md') -Destination $candidate
Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $candidate
& (Join-Path $PSScriptRoot 'test-smart-gate-hosts.ps1') -PluginDirectory $candidate -TargetDir $settings.TargetDir -ProbeBinary $ProbeBinary
$receipt = [ordered]@{
    source_commit = (& git -C $root rev-parse HEAD)
    source_dirty = [bool](& git -C $root status --porcelain)
    candidate = $candidate
    artifact_hashes = @($artifacts | Get-FileHash -Algorithm SHA256 | Select-Object Path, Hash)
    installed = $false
    qualification = 'Pass-through prototype; no detector, gating, native GUI or release acceptance.'
}
$receipt | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $candidate 'build-receipt.json') -Encoding utf8
Write-Output "Checked candidate preserved: $candidate"
# Installation links have distinct reserved names documented in the product README; this script never creates them.
