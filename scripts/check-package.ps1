param([string]$Package = (Join-Path $PSScriptRoot '../dist/CleanSlate-0.1.0-alpha.1-windows-x64'))
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$Package = (Resolve-Path -LiteralPath $Package).Path
$manifest = Get-Content -LiteralPath (Join-Path $Package 'package-manifest.json') -Raw | ConvertFrom-Json
if ($manifest.platform -ne 'windows-x64' -or $manifest.version -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z]+(?:\.[0-9A-Za-z]+)*)?$') { throw 'Invalid package identity' }
$required = @('CleanSlate.exe', 'LICENSE', 'README.txt', 'THIRD-PARTY-NOTICES.txt', 'concrt140.dll', 'msvcp140.dll', 'msvcp140_1.dll', 'msvcp140_2.dll', 'msvcp140_atomic_wait.dll', 'msvcp140_codecvt_ids.dll', 'vcruntime140.dll', 'vcruntime140_1.dll')
$expected = @($manifest.files.path)
if (@($expected | Sort-Object -Unique).Count -ne $expected.Count) { throw 'Duplicate manifest paths' }
foreach ($file in $required) { if ($file -notin $expected) { throw "Package missing $file" } }
if (-not @($expected | Where-Object { $_ -like 'licenses/*' }).Count) { throw 'Dependency license texts missing' }
foreach ($file in $manifest.files) {
    if ($file.path -notin $required -and $file.path -notmatch '^licenses/[A-Za-z0-9._+/-]+$') { throw "Unexpected package file: $($file.path)" }
    if ($file.path -match '(^|/)\.\.?(/|$)' -or $file.sha256 -notmatch '^[0-9a-f]{64}$') { throw 'Invalid manifest path or checksum' }
    $path = [IO.Path]::GetFullPath((Join-Path $Package $file.path))
    if (-not $path.StartsWith($Package + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Manifest path escaped package' }
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $file.sha256) { throw "Integrity failure: $($file.path)" }
}
$actual = @(Get-ChildItem -LiteralPath $Package -File -Recurse | ForEach-Object { [IO.Path]::GetRelativePath($Package, $_.FullName).Replace('\', '/') })
if (Compare-Object ($expected + 'package-manifest.json' | Sort-Object) ($actual | Sort-Object)) { throw 'Package contains unexpected or missing files' }
$check = Start-Process -FilePath (Join-Path $Package 'CleanSlate.exe') -ArgumentList '--package-self-check' -Wait -PassThru -WindowStyle Hidden
if ($check.ExitCode -ne 0) { throw "Packaged offline acoustic inference failed: $($check.ExitCode)" }
Write-Output 'Verified offline acoustic package: pass'
