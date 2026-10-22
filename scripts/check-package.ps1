param([string]$Package = (Join-Path $PSScriptRoot '../dist/CleanSlate-0.1.0-alpha.7-windows-x64'), [switch]$IntegrityOnly)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$Package = (Resolve-Path -LiteralPath $Package).Path
$manifest = Get-Content -LiteralPath (Join-Path $Package 'package-manifest.json') -Raw | ConvertFrom-Json
if ($manifest.platform -notin @('windows-x64', 'macos-arm64', 'macos-x64', 'linux-x64') -or $manifest.version -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z]+(?:\.[0-9A-Za-z]+)*)?$') { throw 'Invalid package identity' }
switch ($manifest.platform) {
    'windows-x64' {
        $executable = 'CleanSlate.exe'; $resources = ''
        $required = @('CleanSlate.exe', 'concrt140.dll', 'msvcp140.dll', 'msvcp140_1.dll', 'msvcp140_2.dll', 'msvcp140_atomic_wait.dll', 'msvcp140_codecvt_ids.dll', 'vcruntime140.dll', 'vcruntime140_1.dll')
        $allowed = '^licenses/[A-Za-z0-9._+/-]+$'
    }
    { $_ -like 'macos-*' } {
        $executable = 'CleanSlate.app/Contents/MacOS/CleanSlate'; $resources = 'CleanSlate.app/Contents/Resources/'
        $required = @($executable, 'CleanSlate.app/Contents/Info.plist', 'CleanSlate.app/Contents/_CodeSignature/CodeResources', "${resources}cleanslate.icns")
        $allowed = '^CleanSlate\.app/Contents/Resources/licenses/[A-Za-z0-9._+/-]+$'
    }
    'linux-x64' {
        $executable = 'AppRun'; $resources = 'usr/share/cleanslate/'
        $required = @('AppRun', 'usr/bin/CleanSlate', 'cleanslate.desktop', 'cleanslate.svg', '.DirIcon')
        $allowed = '^(usr/lib/[A-Za-z0-9._+/-]+\.so(?:\.[0-9]+)*|usr/share/(cleanslate/licenses/[A-Za-z0-9._+/-]+|applications/cleanslate\.desktop|icons/hicolor/scalable/apps/cleanslate\.svg))$'
    }
}
$required += @("${resources}LICENSE", "${resources}README.txt", "${resources}THIRD-PARTY-NOTICES.txt")
$expected = @($manifest.files.path)
if (@($expected | Sort-Object -Unique).Count -ne $expected.Count) { throw 'Duplicate manifest paths' }
foreach ($file in $required) { if ($file -notin $expected) { throw "Package missing $file" } }
if (-not @($expected | Where-Object { $_ -like "${resources}licenses/*" }).Count) { throw 'Dependency license texts missing' }
foreach ($file in $manifest.files) {
    if ($file.path -notin $required -and $file.path -notmatch $allowed) { throw "Unexpected package file: $($file.path)" }
    if ($file.path -match '(^|/)\.\.?(/|$)' -or $file.sha256 -notmatch '^[0-9a-f]{64}$') { throw 'Invalid manifest path or checksum' }
    $path = [IO.Path]::GetFullPath((Join-Path $Package $file.path))
    if (-not $path.StartsWith($Package + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Manifest path escaped package' }
    $item = Get-Item -LiteralPath $path -Force
    if ($item.LinkTarget) {
        $resolved = $item.ResolveLinkTarget($true)
        if (-not $resolved -or -not $resolved.FullName.StartsWith($Package + [IO.Path]::DirectorySeparatorChar, [StringComparison]::Ordinal)) { throw 'Package symlink escaped package' }
    }
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $file.sha256) { throw "Integrity failure: $($file.path)" }
}
$actual = @(Get-ChildItem -LiteralPath $Package -File -Recurse -Force | ForEach-Object { [IO.Path]::GetRelativePath($Package, $_.FullName).Replace('\', '/') })
if (Compare-Object ($expected + 'package-manifest.json' | Sort-Object) ($actual | Sort-Object)) { throw 'Package contains unexpected or missing files' }
if ($IntegrityOnly) { Write-Output 'Verified package integrity: pass'; return }
if ($IsWindows -and $manifest.platform -eq 'windows-x64') {
    $check = Start-Process -FilePath (Join-Path $Package $executable) -ArgumentList '--package-self-check' -Wait -PassThru -WindowStyle Hidden
    $exitCode = $check.ExitCode
} elseif (($IsMacOS -and $manifest.platform -like 'macos-*') -or ($IsLinux -and $manifest.platform -eq 'linux-x64')) {
    if ($IsMacOS) {
        & codesign --verify --strict (Join-Path $Package 'CleanSlate.app')
        if ($LASTEXITCODE -ne 0) { throw 'macOS bundle signature verification failed' }
    }
    & (Join-Path $Package $executable) --package-self-check
    $exitCode = $LASTEXITCODE
} else { throw 'Run this package on its native operating system, or use -IntegrityOnly' }
if ($exitCode -ne 0) { throw "Packaged offline acoustic inference failed: $exitCode" }
Write-Output 'Verified offline acoustic package: pass'
