param([Parameter(Mandatory)][string]$Package, [Parameter(Mandatory)][string]$Archive,
    [string]$Minisign, [string]$SecretKey)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$manifestPath = Join-Path $Package 'package-manifest.json'
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ($manifest.platform -ne 'windows-x64' -or $manifest.schema -ne 1 -or $manifest.product -ne 'CleanSlate') { throw 'Unsupported update package' }
& (Join-Path $PSScriptRoot 'check-package.ps1') -Package $Package -IntegrityOnly
$expectedName = "CleanSlate-$($manifest.version)-windows-x64.zip"
if ([IO.Path]::GetFileName($Archive) -cne $expectedName) { throw 'Archive name/version mismatch' }
$descriptor = [ordered]@{
    schema = 1; product = 'CleanSlate'; repository = 'Today20092/podcast-bleed-reducer'
    version = $manifest.version; tag = "v$($manifest.version)"; platform = 'windows-x64'
    archive = $expectedName; sha256 = (Get-FileHash -LiteralPath $Archive -Algorithm SHA256).Hash.ToLowerInvariant()
    bytes = (Get-Item -LiteralPath $Archive).Length
    expanded_bytes = [long](($manifest.files | Measure-Object -Property bytes -Sum).Sum + (Get-Item -LiteralPath $manifestPath).Length)
    entries = [long]20000
    manifest_sha256 = (Get-FileHash -LiteralPath $manifestPath -Algorithm SHA256).Hash.ToLowerInvariant()
}
$destination = "$Archive.update.json"
[IO.File]::WriteAllText($destination, ($descriptor | ConvertTo-Json), [Text.UTF8Encoding]::new($false))
if ($SecretKey) {
    if (-not $Minisign) { throw 'Provide a trusted local Minisign executable; private keys never enter CI.' }
    & $Minisign -S -m $destination -s $SecretKey
    if ($LASTEXITCODE -ne 0) { throw 'Signing failed' }
    $publicKey = (Get-Content -LiteralPath (Join-Path $PSScriptRoot '../assets/update-public-key.txt') -Raw).Trim()
    if (-not $publicKey) { throw 'No pinned bootstrap public key' }
    & $Minisign -V -m $destination -P $publicKey
    if ($LASTEXITCODE -ne 0) { throw 'Signature does not match the pinned bootstrap key' }
}
Write-Output "Descriptor: $destination. Signing is an explicit owner step; an unsigned descriptor cannot enable installation."
