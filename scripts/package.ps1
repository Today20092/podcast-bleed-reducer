param([string]$Version = '0.1.0-alpha.1', [string]$TargetDir = '')
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if ($Version -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z]+(?:\.[0-9A-Za-z]+)*)?$') { throw 'Invalid release version' }
if (-not $IsWindows -or $env:PROCESSOR_ARCHITECTURE -ne 'AMD64') { throw 'Packaging requires Windows x64' }
if (-not $TargetDir) { $TargetDir = Join-Path $root 'target/release-build' }
$TargetDir = [IO.Path]::GetFullPath($TargetDir)
$dist = Join-Path $root 'dist'
$name = "CleanSlate-$Version-windows-x64"
$package = Join-Path $dist $name
$stageRoot = Join-Path $dist ('.staging-' + [guid]::NewGuid().ToString('N'))
$stage = Join-Path $stageRoot $name
New-Item -ItemType Directory -Path $stage -Force | Out-Null
try {
    cargo build --manifest-path (Join-Path $root 'Cargo.toml') --release --locked --bin rust-app --target x86_64-pc-windows-msvc --config (Join-Path $PSScriptRoot 'release.toml') --target-dir $TargetDir
    if ($LASTEXITCODE -ne 0) { throw 'Release build failed; existing package retained' }
    Copy-Item -LiteralPath (Join-Path $TargetDir 'x86_64-pc-windows-msvc/release/rust-app.exe') -Destination (Join-Path $stage 'CleanSlate.exe')
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    $installation = & $vswhere -latest -version '[17.0,18.0)' -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($LASTEXITCODE -ne 0 -or -not $installation) { throw 'Cannot locate licensed MSVC redistributable files' }
    $redistRoot = Join-Path $installation 'VC/Redist/MSVC'
    $redist = Get-ChildItem -LiteralPath $redistRoot -Directory | Where-Object { $_.Name -match '^\d+\.\d+\.\d+$' } | Sort-Object { [version]$_.Name } -Descending | Select-Object -First 1
    if (-not $redist) { throw 'MSVC redistributable version missing' }
    $crt = Join-Path $redist.FullName 'x64/Microsoft.VC143.CRT'
    foreach ($dll in @('concrt140.dll', 'msvcp140.dll', 'msvcp140_1.dll', 'msvcp140_2.dll', 'msvcp140_atomic_wait.dll', 'msvcp140_codecvt_ids.dll', 'vcruntime140.dll', 'vcruntime140_1.dll')) {
        Copy-Item -LiteralPath (Join-Path $crt $dll) -Destination $stage
    }
    Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $root 'docs/portable-readme.txt') -Destination (Join-Path $stage 'README.txt')
    Copy-Item -LiteralPath (Join-Path $root 'docs/licenses') -Destination (Join-Path $stage 'licenses') -Recurse
    $metadataText = & cargo metadata --manifest-path (Join-Path $root 'Cargo.toml') --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'Cannot resolve dependency license metadata' }
    $metadata = ($metadataText -join "`n") | ConvertFrom-Json
    $application = $metadata.packages | Where-Object { $_.manifest_path -eq (Join-Path $root 'Cargo.toml') -or $_.id -eq $metadata.resolve.root }
    if ($application.version -ne $Version) { throw "Cargo version $($application.version) does not match release $Version" }
    $resolved = @($metadata.resolve.nodes.id)
    $notices = [Collections.Generic.List[string]]::new()
    $notices.Add('CleanSlate third-party dependency notices. See licenses/ for license texts and embedded model/runtime notices.')
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
        $destination = Join-Path $stage "licenses/$($dependency.name)-$($dependency.version)"
        New-Item -ItemType Directory -Path $destination -Force | Out-Null
        foreach ($license in ($licenseFiles | Sort-Object FullName -Unique)) { Copy-Item -LiteralPath $license.FullName -Destination $destination }
    }
    $notices | Set-Content -LiteralPath (Join-Path $stage 'THIRD-PARTY-NOTICES.txt') -Encoding utf8
    $commit = & git -C $root rev-parse HEAD
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read build commit' }
    $rustc = & rustc --version
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read compiler version' }
    $dirty = & git -C $root status --porcelain -- Cargo.toml Cargo.lock src scripts docs .github LICENSE README.md
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read working-tree state' }
    $inventory = @(Get-ChildItem -LiteralPath $stage -File -Recurse | Sort-Object FullName | ForEach-Object {
        @{ path = [IO.Path]::GetRelativePath($stage, $_.FullName).Replace('\', '/'); sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant() }
    })
    @{ version = $Version; platform = 'windows-x64'; commit = "$commit"; dirty = [bool]$dirty; rustc = "$rustc"; cargo_lock_sha256 = (Get-FileHash -LiteralPath (Join-Path $root 'Cargo.lock') -Algorithm SHA256).Hash.ToLowerInvariant(); files = $inventory } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $stage 'package-manifest.json') -Encoding utf8
    & (Join-Path $PSScriptRoot 'check-package.ps1') -Package $stage
    $stagedZip = Join-Path $stageRoot "$name.zip"
    Get-ChildItem -LiteralPath $stage -Recurse | ForEach-Object { $_.LastWriteTime = [datetime]'2000-01-01T00:00:00' }
    Compress-Archive -LiteralPath $stage -DestinationPath $stagedZip
    $zipHash = (Get-FileHash -LiteralPath $stagedZip -Algorithm SHA256).Hash.ToLowerInvariant()
    # Replace the old package only after build, validation and archive creation succeed.
    if (Test-Path -LiteralPath $package) {
        if (-not ([IO.Path]::GetFullPath($package)).StartsWith($dist + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Package path escaped dist' }
        Remove-Item -LiteralPath $package -Recurse -Force
    }
    Move-Item -LiteralPath $stage -Destination $package
    Move-Item -LiteralPath $stagedZip -Destination "$package.zip" -Force
    "$zipHash  $name.zip" | Set-Content -LiteralPath "$package.zip.sha256" -Encoding ascii
    Write-Output "$package.zip"
} finally {
    if (-not ([IO.Path]::GetFullPath($stageRoot)).StartsWith($dist + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Staging path escaped dist' }
    if (Test-Path -LiteralPath $stageRoot) { Remove-Item -LiteralPath $stageRoot -Recurse -Force }
}
