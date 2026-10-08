$ErrorActionPreference = 'Stop'
$package = Join-Path $PSScriptRoot '..\dist\CleanSlate-windows-x64'
$files = Get-ChildItem -LiteralPath $package -File
if (($files.Name | Sort-Object) -join ',' -ne 'CleanSlate.exe,model-check.exe') {
    throw "package contains unexpected or missing files: $($files.Name -join ', ')"
}
if ($files.Name -match 'onnxruntime.*\.dll') {
    throw 'ONNX Runtime must be linked into the executable'
}
$native = Join-Path $package 'native-runtime'
$manifest = Get-Content -LiteralPath (Join-Path $native 'runtime-manifest.json') -Raw | ConvertFrom-Json
if ($manifest.revision -ne '8642eaa5cc51efbc17ad0f3e433944ba858a873f') { throw 'Unexpected native runtime revision' }
foreach ($binary in $manifest.binaries.PSObject.Properties) {
    if ((Get-FileHash -LiteralPath (Join-Path $native $binary.Name) -Algorithm SHA256).Hash.ToLowerInvariant() -ne $binary.Value) {
        throw "Packaged native dependency integrity failure: $($binary.Name)"
    }
}
if (-not (Test-Path -LiteralPath (Join-Path $package 'licenses\OpenMDW-1.1.txt'))) { throw 'Model agreement missing' }
& (Join-Path $package 'model-check.exe') --runtime $native
if ($LASTEXITCODE -ne 0) { throw 'Packaged offline native/model inference failed; explicitly acquire the model first' }
Write-Output 'verified offline CPU package: pass'
