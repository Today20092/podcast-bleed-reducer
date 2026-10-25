# Run once from an administrator PowerShell. Rebuild with build-fl-probe.ps1 as usual.
param()
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path -Parent $PSScriptRoot
$output = Join-Path $root 'dist/fl-probe'
$commonFiles = [Environment]::GetFolderPath('CommonProgramFiles')
$links = @(
    @{ Kind = 'Junction'; Path = (Join-Path $commonFiles 'VST3/CleanSlate FL Probe.vst3'); Target = (Join-Path $output 'CleanSlate FL Probe.vst3'); Binary = 'Contents/x86_64-win/CleanSlate FL Probe.vst3' },
    @{ Kind = 'SymbolicLink'; Path = (Join-Path $commonFiles 'CLAP/CleanSlate FL Probe.clap'); Target = (Join-Path $output 'CleanSlate FL Probe.clap'); Binary = '' }
)

# Validate both destinations before creating anything; never replace an installed plugin.
foreach ($link in $links) {
    if (-not (Test-Path -LiteralPath $link.Target)) { throw "Build the probe first: missing $($link.Target)" }
    $existing = Get-Item -LiteralPath $link.Path -Force -ErrorAction SilentlyContinue
    if ($existing) {
        if ($existing.LinkType -ne $link.Kind -or [IO.Path]::GetFullPath([string]$existing.Target) -ne [IO.Path]::GetFullPath($link.Target)) {
            throw "Destination already exists and is not the expected development link: $($link.Path). Preserve or move that existing installation before retrying."
        }
    }
}
foreach ($link in $links) {
    if (-not (Get-Item -LiteralPath $link.Path -Force -ErrorAction SilentlyContinue)) {
        New-Item -ItemType Directory -Path (Split-Path -Parent $link.Path) -Force | Out-Null
        New-Item -ItemType $link.Kind -Path $link.Path -Target $link.Target | Out-Null
    }
    $sourceBinary = if ($link.Binary) { Join-Path $link.Target $link.Binary } else { $link.Target }
    $linkedBinary = if ($link.Binary) { Join-Path $link.Path $link.Binary } else { $link.Path }
    if ((Get-FileHash -LiteralPath $sourceBinary).Hash -ne (Get-FileHash -LiteralPath $linkedBinary).Hash) {
        throw "Linked plugin verification failed: $linkedBinary"
    }
    Write-Output "$($link.Kind): $($link.Path) -> $($link.Target)"
}
Write-Output 'Verified development links. Rescan in FL Studio with Verify plugins enabled. Close FL Studio before rebuilding; it may lock loaded binaries.'
