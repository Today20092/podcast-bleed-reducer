$ErrorActionPreference = 'Stop'

$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$native = Join-Path $root 'native-runtime'
if (-not (Test-Path -LiteralPath (Join-Path $native 'bin\runtime-manifest.json'))) {
    throw 'Build the pinned CPU runtime first: pwsh -File scripts/build-native-runtime.ps1'
}

cargo build --manifest-path (Join-Path $root 'Cargo.toml') --release --locked
if ($LASTEXITCODE -ne 0) { throw 'Release build failed; existing package retained' }
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$package = Join-Path $root 'dist\CleanSlate-windows-x64'
if (-not $package.StartsWith($root, [StringComparison]::OrdinalIgnoreCase)) {
    throw "package path escaped repository: $package"
}
if (Test-Path -LiteralPath $package) {
    Remove-Item -LiteralPath $package -Recurse -Force
}
New-Item -ItemType Directory -Path $package | Out-Null
Copy-Item -LiteralPath (Join-Path $PSScriptRoot '..\target\release\rust-app.exe') -Destination (Join-Path $package 'CleanSlate.exe')
Copy-Item -LiteralPath (Join-Path $PSScriptRoot '..\target\release\model-check.exe') -Destination (Join-Path $package 'model-check.exe')
Copy-Item -LiteralPath (Join-Path $native 'bin') -Destination (Join-Path $package 'native-runtime') -Recurse
Copy-Item -LiteralPath (Join-Path $native 'share\licenses') -Destination (Join-Path $package 'licenses') -Recurse
Copy-Item -LiteralPath (Join-Path $root 'docs\licenses\OpenMDW-1.1.txt') -Destination (Join-Path $package 'licenses\OpenMDW-1.1.txt')

$archive = "$package.zip"
if (Test-Path -LiteralPath $archive) {
    Remove-Item -LiteralPath $archive -Force
}
Compress-Archive -LiteralPath $package -DestinationPath $archive
Write-Output $archive
