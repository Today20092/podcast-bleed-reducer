param([Parameter(Mandatory)][string]$AppDir, [Parameter(Mandatory)][string]$Resources, [Parameter(Mandatory)][string]$StageRoot, [Parameter(Mandatory)][string]$Version)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if (-not $IsLinux) { throw 'AppImage bundling requires Linux' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$tool = Join-Path $StageRoot 'linuxdeploy.AppImage'
Invoke-WebRequest 'https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage' -OutFile $tool
if ((Get-FileHash -LiteralPath $tool -Algorithm SHA256).Hash.ToLowerInvariant() -ne 'c20cd71e3a4e3b80c3483cef793cda3f4e990aca14014d23c544ca3ce1270b4d') { throw 'linuxdeploy checksum mismatch' }
& chmod +x $tool (Join-Path $AppDir 'usr/bin/CleanSlate')
if ($LASTEXITCODE -ne 0) { throw 'Cannot set Linux executable permissions' }
$desktop = Join-Path $StageRoot 'cleanslate.desktop'
@"
[Desktop Entry]
Type=Application
Name=CleanSlate
Comment=Offline microphone bleed reduction for podcasts
Exec=CleanSlate
Icon=cleanslate
Categories=AudioVideo;Audio;
Terminal=false
X-AppImage-Version=$Version
"@ | Set-Content -LiteralPath $desktop -Encoding utf8
$env:APPIMAGE_EXTRACT_AND_RUN = '1'
& $tool --appdir $AppDir --executable (Join-Path $AppDir 'usr/bin/CleanSlate') --desktop-file $desktop --icon-file (Join-Path $root 'assets/cleanslate.svg')
if ($LASTEXITCODE -ne 0) { throw 'Linux dependency bundling failed' }
foreach ($link in @(@('cleanslate.desktop', 'usr/share/applications/cleanslate.desktop'), @('cleanslate.svg', 'usr/share/icons/hicolor/scalable/apps/cleanslate.svg'), @('.DirIcon', 'cleanslate.svg'))) {
    if (-not (Test-Path -LiteralPath (Join-Path $AppDir $link[0]))) {
        & ln -s $link[1] (Join-Path $AppDir $link[0])
        if ($LASTEXITCODE -ne 0) { throw 'Cannot create AppImage desktop metadata' }
    }
}
$appRun = Join-Path $AppDir 'AppRun'
$binary = Join-Path $AppDir 'usr/bin/CleanSlate'
$binaryHash = (Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash
# linuxdeploy creates AppRun as a symlink; remove it before writing the launcher.
Remove-Item -LiteralPath $appRun -Force
@'
#!/bin/sh
set -eu
appdir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
export LD_LIBRARY_PATH="$appdir/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$appdir/usr/bin/CleanSlate" "$@"
'@ | Set-Content -LiteralPath $appRun -Encoding utf8
if ((Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash -ne $binaryHash) { throw 'AppRun creation modified the application executable' }
& chmod +x (Join-Path $AppDir 'AppRun')
if ($LASTEXITCODE -ne 0) { throw 'Cannot set AppRun permissions' }

# Package the distribution's exact library copyright records and their referenced license texts.
$linuxLicenses = Join-Path $Resources 'licenses/linux-system'
New-Item -ItemType Directory -Path $linuxLicenses -Force | Out-Null
$inventory = [Collections.Generic.List[string]]::new()
$systemLibraries = & ldconfig -p
if ($LASTEXITCODE -ne 0) { throw 'Cannot locate system library origins' }
foreach ($library in (Get-ChildItem -LiteralPath (Join-Path $AppDir 'usr/lib') -File)) {
    $original = @($systemLibraries | Where-Object { $_ -match ('^\s*' + [regex]::Escape($library.Name) + '\s.*=>\s+') } | ForEach-Object { ($_ -split '=>\s+')[1] })
    if (-not $original.Count) { throw "Missing system origin for $($library.Name)" }
    $libraryPath = & realpath $original[0]
    if ($LASTEXITCODE -ne 0) { throw 'Cannot resolve system library origin' }
    $owner = @(& dpkg-query -S $libraryPath 2>$null)
    if ($LASTEXITCODE -ne 0) { $owner = @(& dpkg-query -S $original[0] 2>$null) }
    if ($LASTEXITCODE -ne 0) { throw "No license package for $($library.Name)" }
    $packageName = ($owner[0] -split ': ')[0]
    $baseName = ($packageName -split ':')[0]
    $copyright = "/usr/share/doc/$baseName/copyright"
    if (-not (Test-Path -LiteralPath $copyright)) { throw "Missing copyright for $packageName" }
    Copy-Item -LiteralPath $copyright -Destination (Join-Path $linuxLicenses "$baseName.txt")
    $packageVersion = & dpkg-query -W '-f=${Version}' $packageName
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read system library version' }
    $inventory.Add("$($library.Name): $packageName $packageVersion. Copyright: licenses/linux-system/$baseName.txt")
}
Copy-Item -LiteralPath '/usr/share/common-licenses' -Destination (Join-Path $linuxLicenses 'common-licenses') -Recurse
$inventory | Set-Content -LiteralPath (Join-Path $linuxLicenses 'inventory.txt') -Encoding utf8
Copy-Item -LiteralPath (Join-Path $root 'docs/licenses/appimage-runtime.txt') -Destination (Join-Path $Resources 'licenses/appimage-runtime.txt')
