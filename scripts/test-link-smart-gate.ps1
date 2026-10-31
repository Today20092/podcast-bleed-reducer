#requires -Version 7.0
param([Parameter(Mandatory)][string]$PreviewDirectory, [Parameter(Mandatory)][string]$ExpectedPluginSha256)
$ErrorActionPreference = 'Stop'
$fixture = Join-Path ([IO.Path]::GetTempPath()) "gate-delivery-$([Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $fixture | Out-Null
$helper = Join-Path $PSScriptRoot 'link-smart-gate.ps1'
function Assert-Refused([scriptblock]$Action, [string]$ExpectedMessage) {
    $refused = $false
    try { & $Action } catch { if ($_.Exception.Message -notlike $ExpectedMessage) { throw }; $refused = $true }
    if (-not $refused) { throw 'Unsafe delivery was accepted.' }
}
$wrongRoot = Join-Path $fixture 'wrong-hash'
Assert-Refused { & $helper -PreviewDirectory $PreviewDirectory -ExpectedPluginSha256 ('0' * 64) -PluginRoot $wrongRoot -ReceiptPath (Join-Path $fixture 'wrong.json') } 'Preview hash/defaults differ*'
if (Test-Path -LiteralPath $wrongRoot) { throw 'Hash rejection changed the plugin root.' }
$conflict = Join-Path $fixture 'conflict'
$sentinel = Join-Path $conflict 'VST3/CleanSlate Smart Gate.vst3/sentinel.txt'
New-Item -ItemType Directory -Path (Split-Path -Parent $sentinel) -Force | Out-Null
'existing owner data' | Set-Content -LiteralPath $sentinel
$sentinelHash = (Get-FileHash -LiteralPath $sentinel).Hash
Assert-Refused { & $helper -PreviewDirectory $PreviewDirectory -ExpectedPluginSha256 $ExpectedPluginSha256 -PluginRoot $conflict -ReceiptPath (Join-Path $fixture 'conflict.json') } 'Existing Gate destination*'
if (Test-Path -LiteralPath (Join-Path $conflict 'CLAP')) { throw 'Conflict was detected after creating another installed link.' }
if ((Get-FileHash -LiteralPath $sentinel).Hash -ne $sentinelHash) { throw 'Existing item changed.' }
$valid = Join-Path $fixture 'valid'
$receipt = Join-Path $fixture 'valid.json'
& $helper -PreviewDirectory $PreviewDirectory -ExpectedPluginSha256 $ExpectedPluginSha256 -PluginRoot $valid -ReceiptPath $receipt
$record = Get-Content -LiteralPath $receipt -Raw | ConvertFrom-Json
if ($record.created_links.Count -ne 2) { throw 'Both links were not recorded.' }
& $helper -PreviewDirectory $PreviewDirectory -ExpectedPluginSha256 $ExpectedPluginSha256 -PluginRoot $valid -ReceiptPath (Join-Path $fixture 'repeat.json')
$repeat = Get-Content -LiteralPath (Join-Path $fixture 'repeat.json') -Raw | ConvertFrom-Json
if ($repeat.created_links.Count -ne 0) { throw 'Idempotent delivery recreated links.' }
# Exercise rollback without target recursion: this is the exact two-link manifest.
foreach ($link in $record.links) {
    $item = Get-Item -LiteralPath $link.Path -Force
    if ($item.LinkType -ne 'Junction' -or [IO.Path]::GetFullPath([string]$item.Target) -ne [IO.Path]::GetFullPath($link.Target)) { throw 'Unexpected fixture junction.' }
    Remove-Item -LiteralPath $link.Path -Force
}
if ((Get-FileHash -LiteralPath (Join-Path $PreviewDirectory 'CleanSlate Smart Gate.clap')).Hash.ToLowerInvariant() -ne $ExpectedPluginSha256) { throw 'Rollback modified preview target.' }
$blockedReceiptParent = Join-Path $fixture 'blocked-receipt-parent'
'retain this file' | Set-Content -LiteralPath $blockedReceiptParent
$transactionRoot = Join-Path $fixture 'transaction-rollback'
Assert-Refused { & $helper -PreviewDirectory $PreviewDirectory -ExpectedPluginSha256 $ExpectedPluginSha256 -PluginRoot $transactionRoot -ReceiptPath (Join-Path $blockedReceiptParent 'receipt.json') } '*'
foreach ($relative in @('CLAP/CleanSlate Smart Gate','VST3/CleanSlate Smart Gate.vst3')) {
    if (Get-Item -LiteralPath (Join-Path $transactionRoot $relative) -Force -ErrorAction SilentlyContinue) { throw 'Failed delivery retained a newly created link.' }
}
if (-not (Test-Path -LiteralPath (Join-Path $PreviewDirectory 'CleanSlate Smart Gate.clap'))) { throw 'Failed-delivery rollback removed preview target.' }
$mixedRoot = Join-Path $fixture 'mixed-permissions'
$deniedClap = Join-Path $mixedRoot 'CLAP'
New-Item -ItemType Directory -Path $deniedClap -Force | Out-Null
$acl = Get-Acl -LiteralPath $deniedClap
$originalSddl = $acl.GetSecurityDescriptorSddlForm([Security.AccessControl.AccessControlSections]::All)
$deny = [Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.WindowsIdentity]::GetCurrent().User, [Security.AccessControl.FileSystemRights]::Write, [Security.AccessControl.AccessControlType]::Deny)
try {
    $acl.AddAccessRule($deny)
    Set-Acl -LiteralPath $deniedClap -AclObject $acl
    Assert-Refused { & $helper -PreviewDirectory $PreviewDirectory -ExpectedPluginSha256 $ExpectedPluginSha256 -PluginRoot $mixedRoot -ReceiptPath (Join-Path $fixture 'mixed-both.json') } '*denied*'
    $mixedReceipt = Join-Path $fixture 'mixed-vst3.json'
    & $helper -PreviewDirectory $PreviewDirectory -ExpectedPluginSha256 $ExpectedPluginSha256 -PluginRoot $mixedRoot -Format VST3 -ReceiptPath $mixedReceipt
    $mixed = Get-Content -LiteralPath $mixedReceipt -Raw | ConvertFrom-Json
    if ($mixed.links.Count -ne 1 -or $mixed.links[0].Format -ne 'VST3' -or $mixed.created_links.Count -ne 1) { throw 'Per-format fallback did not isolate VST3.' }
    $link = $mixed.links[0]
    $item = Get-Item -LiteralPath $link.Path -Force
    if ($item.LinkType -ne 'Junction' -or [IO.Path]::GetFullPath([string]$item.Target) -ne [IO.Path]::GetFullPath($link.Target)) { throw 'Unexpected mixed-permission fixture link.' }
    Remove-Item -LiteralPath $link.Path -Force
} finally {
    $restore = [Security.AccessControl.DirectorySecurity]::new()
    $restore.SetSecurityDescriptorSddlForm($originalSddl)
    Set-Acl -LiteralPath $deniedClap -AclObject $restore
}
Write-Output "PASS: reviewed-hash refusal, conflict preservation, verified links, idempotence, failed-delivery rollback and mixed-permission per-format delivery. Retained fixture: $fixture"
