#requires -Version 7.0
param(
    [Parameter(Mandatory)][string]$PreviewDirectory,
    [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedPluginSha256,
    [Parameter(Mandatory)][string]$ReceiptPath,
    [string]$PluginRoot = [Environment]::GetFolderPath('CommonProgramFiles')
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$PreviewDirectory = (Resolve-Path -LiteralPath $PreviewDirectory).Path
$PluginRoot = [IO.Path]::GetFullPath($PluginRoot)
$ReceiptPath = [IO.Path]::GetFullPath($ReceiptPath)
if (Test-Path -LiteralPath $ReceiptPath) { throw 'Choose a new delivery receipt; existing evidence is never replaced.' }
$manifest = Get-Content -LiteralPath (Join-Path $PreviewDirectory 'gui-preview-manifest.json') -Raw | ConvertFrom-Json
if ($manifest.plugin_sha256 -cne $ExpectedPluginSha256 -or $manifest.default_bypass -ne $true -or $manifest.default_armed -ne $false) {
    throw 'Preview hash/defaults differ from the reviewed bypassed, unarmed candidate.'
}
foreach ($file in $manifest.files) {
    $path = [IO.Path]::GetFullPath((Join-Path $PreviewDirectory $file.path))
    if (-not $path.StartsWith($PreviewDirectory.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Preview inventory escapes its directory.' }
    if ((Get-Item -LiteralPath $path).Length -ne $file.bytes -or
        (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $file.sha256) { throw "Preview bytes changed: $($file.path)" }
}
$links = @(
    @{Path=(Join-Path $PluginRoot 'CLAP/CleanSlate Smart Gate');Target=$PreviewDirectory;Binary='CleanSlate Smart Gate.clap'},
    @{Path=(Join-Path $PluginRoot 'VST3/CleanSlate Smart Gate.vst3');Target=(Join-Path $PreviewDirectory 'CleanSlate Smart Gate.vst3');Binary='Contents/x86_64-win/CleanSlate Smart Gate.vst3'}
)
# Validate every destination and its parents before creating either installed link.
foreach ($link in $links) {
    if ((Get-FileHash -LiteralPath (Join-Path $link.Target $link.Binary)).Hash.ToLowerInvariant() -cne $ExpectedPluginSha256) { throw 'Format binary differs from reviewed plugin.' }
    $parent = Split-Path -Parent $link.Path
    while ($parent) {
        $item = Get-Item -LiteralPath $parent -Force -ErrorAction SilentlyContinue
        if ($item -and ((-not $item.PSIsContainer) -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint))) { throw "Plugin parent must be an ordinary directory: $parent" }
        $parent = [IO.Path]::GetDirectoryName($parent)
    }
    $existing = Get-Item -LiteralPath $link.Path -Force -ErrorAction SilentlyContinue
    if ($existing -and ($existing.LinkType -ne 'Junction' -or
        [IO.Path]::GetFullPath([string]$existing.Target) -ne [IO.Path]::GetFullPath($link.Target))) {
        throw "Existing Gate destination is not this development link: $($link.Path)"
    }
}
$created = [Collections.Generic.List[string]]::new()
try {
    foreach ($link in $links) {
        if (-not (Get-Item -LiteralPath $link.Path -Force -ErrorAction SilentlyContinue)) {
            New-Item -ItemType Directory -Path (Split-Path -Parent $link.Path) -Force | Out-Null
            New-Item -ItemType Junction -Path $link.Path -Target $link.Target | Out-Null
            $created.Add($link.Path)
        }
        if ((Get-FileHash -LiteralPath (Join-Path $link.Path $link.Binary)).Hash.ToLowerInvariant() -cne $ExpectedPluginSha256) { throw 'Linked Gate bytes differ from reviewed plugin.' }
    }
    $record = [ordered]@{purpose='Local DEVELOPMENT install';source_commit=(& git -C (Split-Path -Parent $PSScriptRoot) rev-parse HEAD);plugin_source_commit=$manifest.plugin_source_commit;plugin_sha256=$ExpectedPluginSha256;default_bypass=$true;default_armed=$false;links=$links;created_links=$created.ToArray();preview=$PreviewDirectory;qualification='Linked verified bytes only; DAW scan, native GUI, armed audio and listening remain separate.'}
    New-Item -ItemType Directory -Path (Split-Path -Parent $ReceiptPath) -Force | Out-Null
    $record | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $ReceiptPath
} catch {
    # Roll back only junctions created by this invocation, never their targets.
    foreach ($path in $created) {
        $link = $links | Where-Object Path -eq $path
        $item = Get-Item -LiteralPath $path -Force
        if ($item.LinkType -ne 'Junction' -or [IO.Path]::GetFullPath([string]$item.Target) -ne [IO.Path]::GetFullPath($link.Target)) { throw "Refusing changed rollback link: $path" }
        Remove-Item -LiteralPath $path -Force
    }
    throw
}
$links | ForEach-Object { Write-Output "Verified development junction: $($_.Path) -> $($_.Target)" }
Write-Output "Delivery receipt: $ReceiptPath"
