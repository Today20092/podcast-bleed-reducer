param([switch]$Fetch)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $root 'plugins/fl-probe/Cargo.toml'
$settings = & (Join-Path $PSScriptRoot 'local-build-settings.ps1')
function Invoke-ProbeCargo([string[]]$Arguments) {
    & (Join-Path $PSScriptRoot 'resource-guard.ps1') -FilePath (Get-Command cargo -CommandType Application | Select-Object -First 1).Source -ArgumentList $Arguments -WorkingDirectory $root
}
if ($Fetch) {
    Invoke-ProbeCargo (@('fetch', '--manifest-path', $manifest, '--locked'))
}
$cargoArgs = $settings.CargoArguments
Invoke-ProbeCargo (@('fmt', '--manifest-path', $manifest, '--', '--check'))
Invoke-ProbeCargo (@('test', '--manifest-path', $manifest) + $cargoArgs)
Invoke-ProbeCargo (@('clippy', '--manifest-path', $manifest, '--all-targets') + $cargoArgs + @('--', '-D', 'warnings'))
Invoke-ProbeCargo (@('build', '--manifest-path', $manifest) + $cargoArgs)
Invoke-ProbeCargo (@('build', '--manifest-path', $manifest, '--examples') + $cargoArgs)
$output = Join-Path $root 'dist/fl-probe-candidate'
$bundle = Join-Path $output 'CleanSlate FL Probe.vst3/Contents/x86_64-win'
New-Item -ItemType Directory -Path $bundle -Force | Out-Null
$dll = Join-Path $settings.TargetDir 'debug/cleanslate_fl_probe.dll'
Copy-Item -LiteralPath $dll -Destination (Join-Path $output 'CleanSlate FL Probe.clap')
Copy-Item -LiteralPath $dll -Destination (Join-Path $bundle 'CleanSlate FL Probe.vst3')
Copy-Item -LiteralPath (Join-Path $root 'plugins/fl-probe/README.md') -Destination $output
Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $output
Copy-Item -LiteralPath (Join-Path $root 'plugins/fl-probe/NICE-PLUG-LICENSE.txt') -Destination $output
foreach ($notice in @('embedded-model-runtime.txt', 'bundled-dependencies.txt', 'msvc-runtime.txt')) {
    Copy-Item -LiteralPath (Join-Path $root "docs/licenses/$notice") -Destination $output
}
& (Join-Path $PSScriptRoot 'test-plugin-hosts.ps1') -PluginDirectory $output -TargetDir $settings.TargetDir
$artifacts = @((Join-Path $output 'CleanSlate FL Probe.clap'), (Join-Path $bundle 'CleanSlate FL Probe.vst3'))
$artifacts | Get-FileHash -Algorithm SHA256 | Format-Table Hash, Path -AutoSize
$installed = Join-Path $root 'dist/fl-probe'
foreach ($relative in @('CleanSlate FL Probe.clap', 'CleanSlate FL Probe.vst3/Contents/x86_64-win/CleanSlate FL Probe.vst3')) {
    $destination = Join-Path $installed $relative
    if (Test-Path -LiteralPath $destination) {
        try {
            $exclusive = [IO.File]::Open($destination, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
            $exclusive.Dispose()
        } catch {
            throw "Close the host using $destination before updating development links. The checked candidate remains in $output. $($_.Exception.Message)"
        }
    }
}
$existingClap = Join-Path $installed 'CleanSlate FL Probe.clap'
if (Test-Path -LiteralPath $existingClap) {
    $identity = (Get-FileHash -LiteralPath $existingClap -Algorithm SHA256).Hash.ToLowerInvariant()
    $preserved = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) "CleanSlate/VerifiedBuilds/fl-probe/$identity"
    if (-not (Test-Path -LiteralPath $preserved)) {
        New-Item -ItemType Directory -Path $preserved -Force | Out-Null
        Get-ChildItem -LiteralPath $installed | Copy-Item -Destination $preserved -Recurse
        if ((Get-FileHash -LiteralPath (Join-Path $preserved 'CleanSlate FL Probe.clap')).Hash.ToLowerInvariant() -ne $identity) {
            throw 'Previous probe preservation failed.'
        }
    }
    Write-Output "Previous probe preserved: $preserved"
}
New-Item -ItemType Directory -Path $installed -Force | Out-Null
Get-ChildItem -LiteralPath $output | Copy-Item -Destination $installed -Recurse -Force
if ((Get-FileHash -LiteralPath (Join-Path $installed 'CleanSlate FL Probe.clap')).Hash -ne (Get-FileHash -LiteralPath $artifacts[0]).Hash) {
    throw 'Published CLAP differs from the tested candidate.'
}
$installedVst = Join-Path $installed 'CleanSlate FL Probe.vst3/Contents/x86_64-win/CleanSlate FL Probe.vst3'
if ((Get-FileHash -LiteralPath $installedVst).Hash -ne (Get-FileHash -LiteralPath $artifacts[1]).Hash) {
    throw 'Published VST3 differs from the tested candidate.'
}
Write-Output "Tested plugin bundles published to development links: $installed"
