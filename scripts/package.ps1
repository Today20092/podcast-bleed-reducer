$ErrorActionPreference = 'Stop'

cargo build --release --locked
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

$archive = "$package.zip"
if (Test-Path -LiteralPath $archive) {
    Remove-Item -LiteralPath $archive -Force
}
Compress-Archive -LiteralPath $package -DestinationPath $archive
Write-Output $archive
