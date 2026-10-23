param([string]$Version = '1.1.0', [string]$TargetDir = '')
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if ($Version -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z]+(?:\.[0-9A-Za-z]+)*)?$') { throw 'Invalid release version' }
$architecture = [Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
if ($IsWindows -and $architecture -eq 'X64') {
    $platform = 'windows-x64'; $target = 'x86_64-pc-windows-msvc'; $executable = 'CleanSlate.exe'; $extension = 'zip'
} elseif ($IsMacOS -and $architecture -in @('Arm64', 'X64')) {
    $platform = if ($architecture -eq 'Arm64') { 'macos-arm64' } else { 'macos-x64' }
    $target = if ($architecture -eq 'Arm64') { 'aarch64-apple-darwin' } else { 'x86_64-apple-darwin' }
    $executable = 'CleanSlate.app/Contents/MacOS/CleanSlate'; $extension = 'dmg'
} elseif ($IsLinux -and $architecture -eq 'X64') {
    $platform = 'linux-x64'; $target = 'x86_64-unknown-linux-gnu'; $executable = 'usr/bin/CleanSlate'; $extension = 'AppImage'
} else { throw 'Packaging supports Windows x64, macOS arm64/x64 and Linux x64' }
if (-not $TargetDir) { $TargetDir = Join-Path $root 'target/release-build' }
$TargetDir = [IO.Path]::GetFullPath($TargetDir)
$dist = Join-Path $root 'dist'
$name = "CleanSlate-$Version-$platform"
$package = Join-Path $dist $name
$stageRoot = Join-Path $dist ('.staging-' + [guid]::NewGuid().ToString('N'))
$stage = Join-Path $stageRoot $name
New-Item -ItemType Directory -Path $stage -Force | Out-Null
try {
    cargo build --manifest-path (Join-Path $root 'Cargo.toml') --release --locked --bin rust-app --target $target --config (Join-Path $PSScriptRoot 'release.toml') --target-dir $TargetDir
    if ($LASTEXITCODE -ne 0) { throw 'Release build failed; existing package retained' }
    $binary = if ($IsWindows) { 'rust-app.exe' } else { 'rust-app' }
    $destination = Join-Path $stage $executable
    New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $TargetDir "$target/release/$binary") -Destination $destination
    if ($IsWindows) {
        $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
        $installation = & $vswhere -latest -version '[17.0,19.0)' -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if ($LASTEXITCODE -ne 0 -or -not $installation) { throw 'Cannot locate licensed MSVC redistributable files' }
        $redistRoot = Join-Path $installation 'VC/Redist/MSVC'
        $redist = Get-ChildItem -LiteralPath $redistRoot -Directory | Where-Object { $_.Name -match '^\d+\.\d+\.\d+$' } | Sort-Object { [version]$_.Name } -Descending | Select-Object -First 1
        if (-not $redist) { throw 'MSVC redistributable version missing' }
        $crtFolders = @(Get-ChildItem -LiteralPath (Join-Path $redist.FullName 'x64') -Directory | Where-Object { $_.Name -match '^Microsoft\.VC14\d\.CRT$' })
        if ($crtFolders.Count -ne 1) { throw 'Expected one app-local MSVC CRT directory' }
        $crt = $crtFolders[0].FullName
        foreach ($dll in @('concrt140.dll', 'msvcp140.dll', 'msvcp140_1.dll', 'msvcp140_2.dll', 'msvcp140_atomic_wait.dll', 'msvcp140_codecvt_ids.dll', 'vcruntime140.dll', 'vcruntime140_1.dll')) {
            Copy-Item -LiteralPath (Join-Path $crt $dll) -Destination $stage
        }
    }
    $resources = if ($IsMacOS) { Join-Path $stage 'CleanSlate.app/Contents/Resources' } elseif ($IsLinux) { Join-Path $stage 'usr/share/cleanslate' } else { $stage }
    New-Item -ItemType Directory -Path $resources -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $resources
    (Get-Content -LiteralPath (Join-Path $root 'docs/portable-readme.txt') -Raw).Replace('@VERSION@', $Version).Replace('@PLATFORM@', $platform) | Set-Content -LiteralPath (Join-Path $resources 'README.txt') -Encoding utf8
    Copy-Item -LiteralPath (Join-Path $root 'docs/licenses') -Destination (Join-Path $resources 'licenses') -Recurse
    $metadataText = & cargo metadata --manifest-path (Join-Path $root 'Cargo.toml') --locked --format-version 1 --filter-platform $target
    if ($LASTEXITCODE -ne 0) { throw 'Cannot resolve dependency license metadata' }
    $metadata = ($metadataText -join "`n") | ConvertFrom-Json
    $application = $metadata.packages | Where-Object { $_.manifest_path -eq (Join-Path $root 'Cargo.toml') -or $_.id -eq $metadata.resolve.root }
    if ($application.version -ne $Version) { throw "Cargo version $($application.version) does not match release $Version" }
    $resolved = @($metadata.resolve.nodes.id)
    $notices = [Collections.Generic.List[string]]::new()
    $notices.Add('CleanSlate third-party dependency notices. See licenses/ for license texts and embedded model/runtime notices.')
    Copy-Item -LiteralPath (Join-Path $root 'assets/lucide/LICENSE') -Destination (Join-Path $resources 'licenses/lucide.txt')
    $notices.Add('Lucide outline icons: ISC; Feather-derived icons also carry MIT. License text: licenses/lucide.txt. Upstream revision: 20aef31db93fe4d9f398dbb15b120d99c48eef95.')
    $bundled = Join-Path $root 'docs/licenses/bundled-dependencies.txt'
    $covered = @(Get-Content -LiteralPath $bundled | Where-Object { $_ -like 'Applies to:*' } | ForEach-Object { $_.Substring(11).Split(',').Trim() })
    foreach ($dependency in ($metadata.packages | Where-Object { $_.id -in $resolved -and $_.source } | Sort-Object name, version)) {
        $notices.Add("$($dependency.name) $($dependency.version): $($dependency.license)")
        $sourceDir = Split-Path -Parent $dependency.manifest_path
        $licenseFiles = @(Get-ChildItem -LiteralPath $sourceDir -File | Where-Object { $_.Name -match '^(LICENSE|LICENCE|COPYING|NOTICE)([._-]|$)' })
        if ($dependency.license_file) { $licenseFiles += Get-Item -LiteralPath (Join-Path $sourceDir $dependency.license_file) }
        $fallback = Join-Path $root "docs/licenses/$($dependency.name).txt"
        if (-not $licenseFiles.Count -and (Test-Path -LiteralPath $fallback)) { $licenseFiles += Get-Item -LiteralPath $fallback }
        if (-not $licenseFiles.Count -and $dependency.name -in $covered) {
            $notices.Add('License text: licenses/bundled-dependencies.txt')
            continue
        }
        if (-not $licenseFiles.Count) { throw "Missing license text for $($dependency.name) $($dependency.version)" }
        $licenseDestination = Join-Path $resources "licenses/$($dependency.name)-$($dependency.version)"
        New-Item -ItemType Directory -Path $licenseDestination -Force | Out-Null
        foreach ($license in ($licenseFiles | Sort-Object FullName -Unique)) { Copy-Item -LiteralPath $license.FullName -Destination $licenseDestination }
    }
    if ($IsMacOS) {
        Copy-Item -LiteralPath (Join-Path $root 'assets/cleanslate.icns') -Destination (Join-Path $resources 'cleanslate.icns')
        $shortVersion = $Version.Split('-')[0]
        @"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>CleanSlate</string>
<key>CFBundleDisplayName</key><string>CleanSlate</string>
<key>CFBundleIdentifier</key><string>io.github.Today20092.CleanSlate</string>
<key>CFBundleExecutable</key><string>CleanSlate</string>
<key>CFBundleIconFile</key><string>cleanslate.icns</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>$shortVersion</string>
<key>CFBundleVersion</key><string>$shortVersion</string>
<key>LSMinimumSystemVersion</key><string>13.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSPrincipalClass</key><string>NSApplication</string>
</dict></plist>
"@ | Set-Content -LiteralPath (Join-Path $stage 'CleanSlate.app/Contents/Info.plist') -Encoding utf8
        $linkedLibraries = & otool -L (Join-Path $stage $executable)
        if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect macOS runtime dependencies' }
        foreach ($line in ($linkedLibraries | Select-Object -Skip 1)) {
            if ($line.Trim() -notmatch '^(/System/Library/|/usr/lib/)') { throw "Unbundled macOS library: $line" }
        }
    } elseif ($IsLinux) {
        & (Join-Path $PSScriptRoot 'package-linux.ps1') -AppDir $stage -Resources $resources -StageRoot $stageRoot -Version $Version
    }
    $notices | Set-Content -LiteralPath (Join-Path $resources 'THIRD-PARTY-NOTICES.txt') -Encoding utf8
    if ($IsMacOS) {
        # Ad-hoc signing permits Apple Silicon execution; this is not Developer ID signing or notarization.
        & codesign --force --sign - (Join-Path $stage 'CleanSlate.app')
        if ($LASTEXITCODE -ne 0) { throw 'macOS ad-hoc signing failed' }
    }
    $commit = & git -C $root rev-parse HEAD
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read build commit' }
    $rustc = & rustc --version
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read compiler version' }
    $dirty = & git -C $root status --porcelain -- Cargo.toml Cargo.lock src assets scripts tests docs .github LICENSE README.md
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read working-tree state' }
    $inventory = @(Get-ChildItem -LiteralPath $stage -File -Recurse -Force | Sort-Object FullName | ForEach-Object {
        @{ path = [IO.Path]::GetRelativePath($stage, $_.FullName).Replace('\', '/'); sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant(); bytes = $_.Length }
    })
    @{ schema = 1; product = 'CleanSlate'; package_kind = $(if ($IsWindows) { 'portable-zip' } else { $extension }); entrypoint = $executable; version = $Version; platform = $platform; commit = "$commit"; dirty = [bool]$dirty; rustc = "$rustc"; cargo_lock_sha256 = (Get-FileHash -LiteralPath (Join-Path $root 'Cargo.lock') -Algorithm SHA256).Hash.ToLowerInvariant(); files = $inventory } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $stage 'package-manifest.json') -Encoding utf8
    & (Join-Path $PSScriptRoot 'check-package.ps1') -Package $stage
    $archive = Join-Path $stageRoot "$name.$extension"
    if ($IsWindows) {
        Get-ChildItem -LiteralPath $stage -Recurse | ForEach-Object { $_.LastWriteTime = [datetime]'2000-01-01T00:00:00' }
        Compress-Archive -LiteralPath $stage -DestinationPath $archive
        $extracted = Join-Path $stageRoot 'extracted'
        Expand-Archive -LiteralPath $archive -DestinationPath $extracted
        & (Join-Path $PSScriptRoot 'check-package.ps1') -Package (Join-Path $extracted $name)
    } elseif ($IsMacOS) {
        & hdiutil create -volname CleanSlate -srcfolder $stage -format UDZO -ov $archive
        if ($LASTEXITCODE -ne 0) { throw 'DMG creation failed' }
        $mounted = Join-Path $stageRoot 'mounted'
        & hdiutil attach $archive -readonly -nobrowse -mountpoint $mounted
        if ($LASTEXITCODE -ne 0) { throw 'DMG mount failed' }
        try { & (Join-Path $PSScriptRoot 'check-package.ps1') -Package $mounted } finally {
            & hdiutil detach $mounted
            if ($LASTEXITCODE -ne 0) { throw 'DMG detach failed' }
        }
    } else {
        $env:LDAI_OUTPUT = $archive
        $env:OUTPUT = $archive
        $env:APPIMAGE_EXTRACT_AND_RUN = '1'
        & (Join-Path $stageRoot 'linuxdeploy.AppImage') --appdir $stage --output appimage
        if ($LASTEXITCODE -ne 0) { throw 'AppImage creation failed' }
        Push-Location $stageRoot
        try {
            & $archive --appimage-extract | Out-Null
            if ($LASTEXITCODE -ne 0) { throw 'AppImage extraction failed' }
            & (Join-Path $PSScriptRoot 'check-package.ps1') -Package (Join-Path $stageRoot 'squashfs-root')
            & $archive --package-self-check
            if ($LASTEXITCODE -ne 0) { throw 'AppImage offline self-check failed' }
        } finally { Pop-Location }
    }
    $archiveHash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    # Replace the old package only after build, validation and archive creation succeed.
    if (Test-Path -LiteralPath $package) {
        if (-not ([IO.Path]::GetFullPath($package)).StartsWith($dist + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Package path escaped dist' }
        $backup = Join-Path $stageRoot 'previous-package'
        if (-not ([IO.Path]::GetFullPath($backup)).StartsWith($dist + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Backup path escaped dist' }
        Move-Item -LiteralPath $package -Destination $backup
    }
    try {
        Move-Item -LiteralPath $stage -Destination $package
    } catch {
        if (Test-Path -LiteralPath (Join-Path $stageRoot 'previous-package')) { Move-Item -LiteralPath (Join-Path $stageRoot 'previous-package') -Destination $package }
        throw
    }
    Move-Item -LiteralPath $archive -Destination "$package.$extension" -Force
    "$archiveHash  $name.$extension" | Set-Content -LiteralPath "$package.$extension.sha256" -Encoding ascii
    if ($IsWindows) {
        & (Join-Path $PSScriptRoot 'update-metadata.ps1') -Package $package -Archive "$package.$extension"
    }
    Write-Output "$package.$extension"
} finally {
    if (-not ([IO.Path]::GetFullPath($stageRoot)).StartsWith($dist + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Staging path escaped dist' }
    if (Test-Path -LiteralPath $stageRoot) { Remove-Item -LiteralPath $stageRoot -Recurse -Force }
}
