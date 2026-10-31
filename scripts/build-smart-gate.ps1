#requires -Version 7.0
param([string]$TargetDir, [string]$ProbeBinary, [string]$RuntimeDirectory, [string]$RuntimeReceipt, [string]$MsvcRuntimeDirectory)
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
if (-not $RuntimeDirectory -or -not $RuntimeReceipt -or -not $MsvcRuntimeDirectory) {
    throw 'Explicit verified -RuntimeDirectory, -RuntimeReceipt and -MsvcRuntimeDirectory are required; the build entrypoint does not download runtimes.'
}
$manifest = Join-Path $root 'plugins/smart-gate/Cargo.toml'
$settings = & (Join-Path $PSScriptRoot 'local-build-settings.ps1') -TargetDir $TargetDir
$pluginSourceCommit = (& git -C $root rev-parse HEAD)
function Invoke-GateCargo([string[]]$Arguments) {
    & (Join-Path $PSScriptRoot 'resource-guard.ps1') -FilePath cargo -ArgumentList $Arguments -WorkingDirectory $root
}
# Cargo steps keep shared timeout/ownership; memory accounting is informational.
Invoke-GateCargo (@('fmt', '--manifest-path', $manifest, '--all', '--', '--check'))
Invoke-GateCargo (@('test', '--manifest-path', $manifest, '--all-targets') + $settings.CargoArguments)
Invoke-GateCargo (@('clippy', '--manifest-path', $manifest, '--all-targets') + $settings.CargoArguments + @('--', '-D', 'warnings'))
Invoke-GateCargo (@('build', '--manifest-path', $manifest, '--lib', '--examples') + $settings.CargoArguments)
Assert-GateOutputParents
$dll = Join-Path $settings.TargetDir 'debug/cleanslate_smart_gate.dll'
$pluginHash = (Get-FileHash -LiteralPath $dll -Algorithm SHA256).Hash.ToLowerInvariant()
$candidate = & (Join-Path $PSScriptRoot 'package-smart-gate.ps1') -PluginBinary $dll -PluginSourceCommit $pluginSourceCommit -ExpectedPluginSha256 $pluginHash -RuntimeDirectory $RuntimeDirectory -RuntimeReceipt $RuntimeReceipt -MsvcRuntimeDirectory $MsvcRuntimeDirectory -OutputParent $parent
$bundle = Join-Path $candidate 'CleanSlate Smart Gate.vst3/Contents/x86_64-win'
$artifacts = @((Join-Path $candidate 'CleanSlate Smart Gate.clap'), (Join-Path $bundle 'CleanSlate Smart Gate.vst3'))
& (Join-Path $PSScriptRoot 'test-smart-gate-hosts.ps1') -PluginDirectory $candidate -TargetDir $settings.TargetDir -ProbeBinary $ProbeBinary
$receipt = Get-Content -LiteralPath (Join-Path $candidate 'build-receipt.json') -Raw | ConvertFrom-Json -AsHashtable
$receipt.source_dirty = [bool](& git -C $root status --porcelain)
$receipt.candidate = $candidate
$receipt.artifact_hashes = @($artifacts | Get-FileHash -Algorithm SHA256 | Select-Object Path, Hash)
$receipt.qualification = 'Uninstalled engineering candidate; reviewed DirectML inventory and unarmed ABI host checks passed. Independent loader, armed gating, native GUI and release acceptance remain unqualified.'
$receipt | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $candidate 'build-receipt.json') -Encoding utf8
Write-Output "Checked candidate preserved: $candidate"
# Installation links have distinct reserved names documented in the product README; this script never creates them.
