#requires -Version 7.0
param(
    [Parameter(Mandatory)][string]$PluginBinary,
    [Parameter(Mandatory)][string]$RuntimeDirectory,
    [Parameter(Mandatory)][string]$RuntimeReceipt,
    [Parameter(Mandatory)][string]$OutputParent
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path -Parent $PSScriptRoot
function Assert-PlainPath([string]$Path) {
    $current = [IO.Path]::GetFullPath($Path)
    while ($current) {
        if (Test-Path -LiteralPath $current) {
            $item = Get-Item -LiteralPath $current -Force
            if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Linked package input/output is not accepted: $current" }
        }
        $current = [IO.Path]::GetDirectoryName($current)
    }
}
foreach ($path in @($PluginBinary, $RuntimeDirectory, $RuntimeReceipt, $OutputParent)) { Assert-PlainPath $path }
$PluginBinary = (Resolve-Path -LiteralPath $PluginBinary).Path
$RuntimeDirectory = (Resolve-Path -LiteralPath $RuntimeDirectory).Path
$OutputParent = [IO.Path]::GetFullPath($OutputParent)
$inventory = Get-Content -LiteralPath $RuntimeReceipt -Raw | ConvertFrom-Json
$reviewed = Get-Content -LiteralPath (Join-Path $root 'docs/licenses/directml-1.15.4-runtime.json') -Raw | ConvertFrom-Json
if ($inventory.schema_version -ne 1 -or -not $inventory.files.Count) { throw 'A versioned runtime receipt is required.' }
foreach ($field in @('package', 'version', 'url', 'license_url', 'package_bytes', 'package_sha256')) {
    if ($inventory.source.$field -cne $reviewed.source.$field) { throw 'Runtime provenance does not identify the reviewed DirectML package.' }
}
if ($inventory.files.Count -ne $reviewed.files.Count) { throw 'The complete reviewed runtime/notice inventory is required.' }
$seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
foreach ($file in $inventory.files) {
    if ($file.path -notmatch '^[A-Za-z0-9_.-]+\.(dll|txt)$' -or -not $seen.Add($file.path) -or
        $file.sha256 -cnotmatch '^[0-9a-f]{64}$') { throw 'Runtime inventory contains an invalid or duplicate file.' }
    $expected = @($reviewed.files | Where-Object { $_.path -ceq $file.path })
    if ($expected.Count -ne 1 -or $file.sha256 -cne $expected[0].sha256 -or
        $file.archive_entry -cne $expected[0].archive_entry -or $file.bytes -ne $expected[0].bytes) {
        throw 'Runtime receipt differs from the reviewed archive inventory.'
    }
    $source = Join-Path $RuntimeDirectory $file.path
    Assert-PlainPath $source
    if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -ne $file.sha256) { throw "Runtime bytes changed: $($file.path)" }
}
if (-not $seen.Contains('DirectML.dll') -or -not @($inventory.files | Where-Object { $_.path -like '*.txt' }).Count) { throw 'The runtime and accompanying notices are required.' }
$pluginHash = (Get-FileHash -LiteralPath $PluginBinary -Algorithm SHA256).Hash
New-Item -ItemType Directory -Path $OutputParent -Force | Out-Null
$id = [Guid]::NewGuid().ToString('N')
$stage = Join-Path $OutputParent ('.staging-' + $id)
$candidate = Join-Path $OutputParent $id
$bundle = Join-Path $stage 'CleanSlate Smart Gate.vst3/Contents/x86_64-win'
New-Item -ItemType Directory -Path $bundle | Out-Null
foreach ($destination in @((Join-Path $stage 'CleanSlate Smart Gate.clap'), (Join-Path $bundle 'CleanSlate Smart Gate.vst3'))) {
    Copy-Item -LiteralPath $PluginBinary -Destination $destination
    if ((Get-FileHash -LiteralPath $destination).Hash -ne $pluginHash) { throw 'Plugin bytes changed during staging.' }
}
foreach ($file in $inventory.files) {
    $destinations = @((Join-Path $stage $file.path))
    if ($file.path -like '*.dll') { $destinations += Join-Path $bundle $file.path }
    foreach ($destination in $destinations) {
        Copy-Item -LiteralPath (Join-Path $RuntimeDirectory $file.path) -Destination $destination
        if ((Get-FileHash -LiteralPath $destination).Hash.ToLowerInvariant() -ne $file.sha256) { throw 'Runtime bytes changed during staging.' }
    }
}
Copy-Item -LiteralPath (Join-Path $root 'plugins/smart-gate/README.md') -Destination $stage
Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $stage
$licenses = Join-Path $stage 'licenses'
New-Item -ItemType Directory -Path $licenses | Out-Null
foreach ($notice in @('embedded-model-runtime.txt', 'bundled-dependencies.txt', 'msvc-runtime.txt')) {
    Copy-Item -LiteralPath (Join-Path $root "docs/licenses/$notice") -Destination $licenses
}
$inventory | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $stage 'runtime-receipt.json') -Encoding utf8
$record = [ordered]@{
    source_commit = (& git -C $root rev-parse HEAD)
    plugin_sha256 = $pluginHash.ToLowerInvariant()
    runtime = $inventory
    installed = $false
    qualification = 'Uninstalled engineering candidate; runtime inventory hashes checked. Host, gating, native GUI and release qualification are separate.'
}
$record | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $stage 'build-receipt.json') -Encoding utf8
Assert-PlainPath $OutputParent
[IO.Directory]::Move($stage, $candidate)
Write-Output $candidate
