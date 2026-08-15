$ErrorActionPreference = 'Stop'
$package = Join-Path $PSScriptRoot '..\dist\CleanSlate-windows-x64'
$files = Get-ChildItem -LiteralPath $package -File
if (($files.Name | Sort-Object) -join ',' -ne 'CleanSlate.exe') {
    throw "package contains unexpected or missing files: $($files.Name -join ', ')"
}
if ($files.Name -match 'onnxruntime.*\.dll') {
    throw 'ONNX Runtime must be linked into the executable'
}
Write-Output 'self-contained package: pass'
