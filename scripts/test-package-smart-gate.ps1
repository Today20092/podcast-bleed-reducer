param([Parameter(Mandatory)][string]$VerifiedRuntimeDirectory, [Parameter(Mandatory)][string]$VerifiedMsvcRuntimeDirectory)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$packager = Join-Path $PSScriptRoot 'package-smart-gate.ps1'
$fixture = Join-Path ([IO.Path]::GetTempPath()) ('cleanslate-gate-package-' + [Guid]::NewGuid())
$runtime = Join-Path $fixture 'runtime'
$msvcRuntime = Join-Path $fixture 'msvc-runtime'
$output = Join-Path $fixture 'candidates'
New-Item -ItemType Directory -Path $runtime, $msvcRuntime, $output | Out-Null
$plugin = Join-Path $fixture 'fixture.dll'
$receipt = Join-Path $fixture 'runtime-receipt.json'
# Only the plugin is synthetic; runtime bytes must match the pinned official inventory.
# Neither plugin nor runtime is loaded by this filesystem fixture.
[IO.File]::WriteAllText($plugin, 'synthetic plugin fixture')
$pluginProvenance = @{PluginSourceCommit=(& git -C (Split-Path -Parent $PSScriptRoot) rev-parse HEAD);ExpectedPluginSha256=(Get-FileHash -LiteralPath $plugin).Hash.ToLowerInvariant()}
$inventory = Get-Content -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'docs/licenses/directml-1.15.4-runtime.json') -Raw | ConvertFrom-Json
foreach ($file in $inventory.files) {
    Copy-Item -LiteralPath (Join-Path $VerifiedRuntimeDirectory $file.path) -Destination $runtime
}
$msvc = Get-Content -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'docs/licenses/msvc-14.44.35211-runtime.json') -Raw | ConvertFrom-Json
foreach ($file in $msvc.files) { Copy-Item -LiteralPath (Join-Path $VerifiedMsvcRuntimeDirectory $file.path) -Destination $msvcRuntime }
$inventory | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $receipt -Encoding utf8
$candidate = & $packager @pluginProvenance -PluginBinary $plugin -RuntimeDirectory $runtime -RuntimeReceipt $receipt -MsvcRuntimeDirectory $msvcRuntime -OutputParent $output
$buildRecord = Get-Content -LiteralPath (Join-Path $candidate 'build-receipt.json') -Raw | ConvertFrom-Json
if ($buildRecord.plugin_source_commit -cne $pluginProvenance.PluginSourceCommit -or $buildRecord.plugin_sha256 -cne $pluginProvenance.ExpectedPluginSha256) { throw 'Plugin artifact provenance was not preserved.' }
if (-not (Test-Path -LiteralPath $candidate -PathType Container)) { throw 'A complete candidate was not published.' }
foreach ($relative in @('CleanSlate Smart Gate.clap', 'CleanSlate Smart Gate.vst3/Contents/x86_64-win/CleanSlate Smart Gate.vst3')) {
    if ((Get-FileHash -LiteralPath (Join-Path $candidate $relative)).Hash -ne (Get-FileHash -LiteralPath $plugin).Hash) {
        throw "Plugin bytes changed while staging $relative"
    }
}
foreach ($relative in @($inventory.files.path)) {
    if ((Get-FileHash -LiteralPath (Join-Path $candidate $relative)).Hash -ne (Get-FileHash -LiteralPath (Join-Path $runtime $relative)).Hash) {
        throw "Runtime inventory was not staged exactly: $relative"
    }
    if ((Get-FileHash -LiteralPath (Join-Path $candidate "CleanSlate Smart Gate.vst3/Contents/x86_64-win/$relative")).Hash -ne
        (Get-FileHash -LiteralPath (Join-Path $runtime $relative)).Hash) {
        throw "VST3 bundle runtime/notice inventory was not preserved: $relative"
    }
}
foreach ($file in $msvc.files) {
    foreach ($relative in @($file.path, "CleanSlate Smart Gate.vst3/Contents/x86_64-win/$($file.path)")) {
        if ((Get-FileHash -LiteralPath (Join-Path $candidate $relative)).Hash.ToLowerInvariant() -cne $file.sha256) { throw "CRT bytes missing or changed: $relative" }
    }
}
if ((Get-FileHash -LiteralPath (Join-Path $candidate 'CleanSlate Smart Gate.vst3/Contents/x86_64-win/DirectML.dll')).Hash -ne
    (Get-FileHash -LiteralPath (Join-Path $runtime 'DirectML.dll')).Hash) { throw 'VST3 runtime sidecar is missing or changed.' }
foreach ($notice in @('embedded-model-runtime.txt', 'bundled-dependencies.txt', 'msvc-runtime.txt')) {
    if ((Get-FileHash -LiteralPath (Join-Path $candidate "licenses/$notice")).Hash -ne
        (Get-FileHash -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) "docs/licenses/$notice")).Hash) {
        throw "Required bundled notice was not preserved: $notice"
    }
    if ((Get-FileHash -LiteralPath (Join-Path $candidate "CleanSlate Smart Gate.vst3/Contents/Resources/licenses/$notice")).Hash -ne
        (Get-FileHash -LiteralPath (Join-Path $candidate "licenses/$notice")).Hash) {
        throw "VST3 bundle lost its repository notice: $notice"
    }
}
if ((Get-FileHash -LiteralPath (Join-Path $candidate 'CleanSlate Smart Gate.vst3/Contents/Resources/LICENSE')).Hash -ne
    (Get-FileHash -LiteralPath (Join-Path $candidate 'LICENSE')).Hash) { throw 'VST3 bundle lost the project license.' }
$second = & $packager @pluginProvenance -PluginBinary $plugin -RuntimeDirectory $runtime -RuntimeReceipt $receipt -MsvcRuntimeDirectory $msvcRuntime -OutputParent $output
if ($second -eq $candidate -or -not (Test-Path -LiteralPath $candidate -PathType Container)) { throw 'A new candidate must not overwrite an existing candidate.' }
$before = @(Get-ChildItem -LiteralPath $output -Directory).Count
[IO.File]::WriteAllText((Join-Path $runtime 'DirectML.dll'), 'changed after receipt')
$rejected = $false
try { & $packager @pluginProvenance -PluginBinary $plugin -RuntimeDirectory $runtime -RuntimeReceipt $receipt -MsvcRuntimeDirectory $msvcRuntime -OutputParent $output | Out-Null }
catch { $rejected = $true }
if (-not $rejected -or @(Get-ChildItem -LiteralPath $output -Directory).Count -ne $before) {
    throw 'Changed runtime bytes were published or previous candidates changed.'
}
Copy-Item -LiteralPath (Join-Path $VerifiedRuntimeDirectory 'DirectML.dll') -Destination $runtime
function Assert-RejectedPackage([string]$Case, [string]$InputDirectory = $runtime, [string]$Destination = $output) {
    $message = $null
    try { & $packager @pluginProvenance -PluginBinary $plugin -RuntimeDirectory $InputDirectory -RuntimeReceipt $receipt -MsvcRuntimeDirectory $msvcRuntime -OutputParent $Destination | Out-Null }
    catch { $message = $_.Exception.Message }
    if (-not $message -or @(Get-ChildItem -LiteralPath $output -Directory).Count -ne $before) {
        throw "$Case was accepted or changed published candidates."
    }
}
Move-Item -LiteralPath (Join-Path $msvcRuntime 'msvcp140.dll') -Destination (Join-Path $fixture 'saved-crt.dll')
Assert-RejectedPackage 'missing CRT'
Move-Item -LiteralPath (Join-Path $fixture 'saved-crt.dll') -Destination (Join-Path $msvcRuntime 'msvcp140.dll')
[IO.File]::WriteAllText((Join-Path $msvcRuntime 'msvcp140.dll'), 'tampered CRT')
Assert-RejectedPackage 'tampered CRT'
Copy-Item -LiteralPath (Join-Path $VerifiedMsvcRuntimeDirectory 'msvcp140.dll') -Destination $msvcRuntime
[IO.File]::WriteAllText($plugin, 'changed after artifact receipt')
Assert-RejectedPackage 'changed plugin artifact'
[IO.File]::WriteAllText($plugin, 'synthetic plugin fixture')
# Missing input and a different provenance identity must fail before publication.
Move-Item -LiteralPath (Join-Path $runtime 'LICENSE.txt') -Destination (Join-Path $fixture 'saved-notice.txt')
Assert-RejectedPackage 'missing notice'
Move-Item -LiteralPath (Join-Path $fixture 'saved-notice.txt') -Destination (Join-Path $runtime 'LICENSE.txt')
$inventory.source.version = '0.0.0'
$inventory | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $receipt -Encoding utf8
Assert-RejectedPackage 'unreviewed runtime version'
$inventory.source.version = '1.15.4'
$inventory | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $receipt -Encoding utf8
# A forged receipt must not turn arbitrary bytes into an approved runtime.
[IO.File]::WriteAllText((Join-Path $runtime 'DirectML.dll'), 'forged bytes with matching receipt')
$inventory.files[0].sha256 = (Get-FileHash -LiteralPath (Join-Path $runtime 'DirectML.dll')).Hash.ToLowerInvariant()
$inventory | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $receipt -Encoding utf8
foreach ($invalidPath in @('../escaped.dll', 'LICENSE.txt')) {
    $inventory.files[0].path = $invalidPath
    $inventory | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $receipt -Encoding utf8
    Assert-RejectedPackage "invalid/duplicate inventory path $invalidPath"
}
$inventory.files[0].path = 'DirectML.dll'
$inventory | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $receipt -Encoding utf8
Assert-RejectedPackage 'forged runtime hash'
Copy-Item -LiteralPath (Join-Path $VerifiedRuntimeDirectory 'DirectML.dll') -Destination $runtime
$inventory.files[0].sha256 = (Get-FileHash -LiteralPath (Join-Path $runtime 'DirectML.dll')).Hash.ToLowerInvariant()
$inventory | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $receipt -Encoding utf8
if ([Environment]::OSVersion.Platform -eq 'Win32NT') {
    $linkedRuntime = Join-Path $fixture 'linked-runtime'
    New-Item -ItemType Junction -Path $linkedRuntime -Target $runtime | Out-Null
    Assert-RejectedPackage 'linked runtime' $linkedRuntime
    $linkedOutput = Join-Path $fixture 'linked-output'
    New-Item -ItemType Junction -Path $linkedOutput -Target $output | Out-Null
    Assert-RejectedPackage 'linked output' $runtime (Join-Path $linkedOutput 'nested')
    if (Test-Path -LiteralPath (Join-Path $output 'nested')) { throw 'Linked output target was modified.' }
}
if ((Get-FileHash -LiteralPath (Join-Path $candidate 'CleanSlate Smart Gate.clap')).Hash -ne (Get-FileHash -LiteralPath $plugin).Hash) {
    throw 'A rejected staging request changed an existing candidate.'
}
if (Test-Path -LiteralPath (Join-Path $fixture 'escaped.dll')) { throw 'An inventory path escaped staging.' }
Write-Output "Gate staging checks passed (synthetic plugin, verified runtime, no loading); evidence: $fixture"
