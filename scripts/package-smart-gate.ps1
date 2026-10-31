#requires -Version 7.0
param(
    [Parameter(Mandatory)][string]$PluginBinary,
    [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{40}$')][string]$PluginSourceCommit,
    [Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedPluginSha256,
    [Parameter(Mandatory)][string]$RuntimeDirectory,
    [Parameter(Mandatory)][string]$RuntimeReceipt,
    [Parameter(Mandatory)][string]$MsvcRuntimeDirectory,
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
foreach ($path in @($PluginBinary, $RuntimeDirectory, $RuntimeReceipt, $MsvcRuntimeDirectory, $OutputParent)) { Assert-PlainPath $path }
$PluginBinary = (Resolve-Path -LiteralPath $PluginBinary).Path
$RuntimeDirectory = (Resolve-Path -LiteralPath $RuntimeDirectory).Path
$MsvcRuntimeDirectory = (Resolve-Path -LiteralPath $MsvcRuntimeDirectory).Path
$OutputParent = [IO.Path]::GetFullPath($OutputParent)
$inventory = Get-Content -LiteralPath $RuntimeReceipt -Raw | ConvertFrom-Json
$reviewed = Get-Content -LiteralPath (Join-Path $root 'docs/licenses/directml-1.15.4-runtime.json') -Raw | ConvertFrom-Json
if ($inventory.schema_version -ne 1 -or -not $inventory.files.Count) { throw 'A versioned runtime receipt is required.' }
foreach ($field in @('package', 'version', 'url', 'license_url', 'package_bytes', 'package_sha256')) {
    if ($inventory.source.$field -cne $reviewed.source.$field) { throw 'Runtime provenance does not identify the reviewed DirectML package.' }
}
if ($inventory.files.Count -ne $reviewed.files.Count) { throw 'The complete reviewed runtime/notice inventory is required.' }
$filesToStage = @()
foreach ($file in $inventory.files) {
    $expected = @($reviewed.files | Where-Object { $_.path -ceq $file.path })
    if ($expected.Count -ne 1 -or $file.sha256 -cne $expected[0].sha256 -or
        $file.archive_entry -cne $expected[0].archive_entry -or $file.bytes -ne $expected[0].bytes) {
        throw 'Runtime receipt differs from the reviewed archive inventory.'
    }
    $filesToStage += @{File=$file; Source=(Join-Path $RuntimeDirectory $file.path)}
}
$msvc = Get-Content -LiteralPath (Join-Path $root 'docs/licenses/msvc-14.44.35211-runtime.json') -Raw | ConvertFrom-Json
foreach ($file in $msvc.files) { $filesToStage += @{File=$file; Source=(Join-Path $MsvcRuntimeDirectory $file.path)} }
$seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
foreach ($entry in $filesToStage) {
    $file = $entry.File
    if ($file.path -notmatch '^[A-Za-z0-9_.-]+\.(dll|txt)$' -or -not $seen.Add($file.path) -or
        $file.sha256 -cnotmatch '^[0-9a-f]{64}$') { throw 'Runtime inventory contains an invalid or duplicate file.' }
    Assert-PlainPath $entry.Source
    if ((Get-FileHash -LiteralPath $entry.Source -Algorithm SHA256).Hash.ToLowerInvariant() -cne $file.sha256) { throw "Runtime bytes changed: $($file.path)" }
}
$pluginHash = (Get-FileHash -LiteralPath $PluginBinary -Algorithm SHA256).Hash
if ($pluginHash.ToLowerInvariant() -cne $ExpectedPluginSha256) { throw 'Plugin bytes differ from the build artifact receipt.' }
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
foreach ($entry in $filesToStage) {
    $file = $entry.File
    # Keep the verified notices with the redistributable if the VST3 bundle is copied alone.
    $destinations = @((Join-Path $stage $file.path), (Join-Path $bundle $file.path))
    foreach ($destination in $destinations) {
        Copy-Item -LiteralPath $entry.Source -Destination $destination
        if ((Get-FileHash -LiteralPath $destination).Hash.ToLowerInvariant() -ne $file.sha256) { throw 'Runtime bytes changed during staging.' }
    }
}
Copy-Item -LiteralPath (Join-Path $root 'plugins/smart-gate/README.md') -Destination $stage
Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $stage
$licenses = Join-Path $stage 'licenses'
$bundleResources = Join-Path $stage 'CleanSlate Smart Gate.vst3/Contents/Resources'
$bundleLicenses = Join-Path $bundleResources 'licenses'
New-Item -ItemType Directory -Path $licenses, $bundleLicenses | Out-Null
Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $bundleResources
foreach ($notice in @('embedded-model-runtime.txt', 'bundled-dependencies.txt', 'msvc-runtime.txt')) {
    Copy-Item -LiteralPath (Join-Path $root "docs/licenses/$notice") -Destination $licenses
    Copy-Item -LiteralPath (Join-Path $root "docs/licenses/$notice") -Destination $bundleLicenses
}
$inventory | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $stage 'runtime-receipt.json') -Encoding utf8
$msvc | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $stage 'msvc-runtime-receipt.json') -Encoding utf8
$record = [ordered]@{
    plugin_source_commit = $PluginSourceCommit
    packaging_source_commit = (& git -C $root rev-parse HEAD)
    plugin_sha256 = $pluginHash.ToLowerInvariant()
    runtime = $inventory
    msvc_runtime = $msvc
    installed = $false
    qualification = 'Uninstalled engineering candidate; runtime inventory hashes checked. Host, gating, native GUI and release qualification are separate.'
}
$record | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $stage 'build-receipt.json') -Encoding utf8
Assert-PlainPath $OutputParent
[IO.Directory]::Move($stage, $candidate)
Write-Output $candidate
