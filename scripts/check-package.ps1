$ErrorActionPreference = 'Stop'
$package = Join-Path $PSScriptRoot '..\dist\CleanSlate-windows-x64'
$files = Get-ChildItem -LiteralPath $package -File
if (($files.Name | Sort-Object) -join ',' -ne 'CleanSlate.exe,release_gate.exe') {
    throw "package contains unexpected or missing files: $($files.Name -join ', ')"
}
if ($files.Name -match 'onnxruntime.*\.dll') {
    throw 'ONNX Runtime must be linked into the executable'
}
$ErrorActionPreference = 'Continue'
$output = & (Join-Path $package 'release_gate.exe') 2>&1
$ErrorActionPreference = 'Stop'
if ($output -notmatch '^Error: "usage: release_gate') {
    throw "packaged release gate did not start: $output"
}
Write-Output 'self-contained package: pass'
