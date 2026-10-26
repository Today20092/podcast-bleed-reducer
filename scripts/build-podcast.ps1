#requires -Version 7.0
param()
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $root 'plugins/podcast/Cargo.toml'
$settings = & (Join-Path $PSScriptRoot 'local-build-settings.ps1')
function Invoke-PodcastCargo([string[]]$Arguments) {
    & (Join-Path $PSScriptRoot 'resource-guard.ps1') -FilePath (Get-Command cargo -CommandType Application | Select-Object -First 1).Source -ArgumentList $Arguments -WorkingDirectory $root
}
$cargoArgs = $settings.CargoArguments
Invoke-PodcastCargo (@('fmt', '--manifest-path', $manifest, '--', '--check'))
Invoke-PodcastCargo (@('test', '--manifest-path', $manifest) + $cargoArgs)
Invoke-PodcastCargo (@('clippy', '--manifest-path', $manifest, '--all-targets') + $cargoArgs + @('--', '-D', 'warnings'))
Invoke-PodcastCargo (@('build', '--manifest-path', $manifest) + $cargoArgs)
Invoke-PodcastCargo (@('build', '--manifest-path', $manifest, '--examples') + $cargoArgs)
# Publish a new candidate only: installation/link activation is a separate owner action.
foreach ($directory in @((Join-Path $root 'dist'), (Join-Path $root 'dist/podcast-candidate'))) {
    if ((Test-Path -LiteralPath $directory) -and ((Get-Item -LiteralPath $directory).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "Podcast output must not follow a link: $directory"
    }
}
$output = Join-Path $root ('dist/podcast-candidate/' + [Guid]::NewGuid().ToString('N'))
if (Test-Path -LiteralPath $output) { throw 'Podcast candidate already exists; refusing replacement.' }
$bundle = Join-Path $output 'CleanSlate Podcast.vst3/Contents/x86_64-win'
New-Item -ItemType Directory -Path $bundle | Out-Null
$dll = Join-Path $settings.TargetDir 'debug/cleanslate_podcast.dll'
Copy-Item -LiteralPath $dll -Destination (Join-Path $output 'CleanSlate Podcast.clap')
Copy-Item -LiteralPath $dll -Destination (Join-Path $bundle 'CleanSlate Podcast.vst3')
Copy-Item -LiteralPath (Join-Path $root 'plugins/podcast/README.md') -Destination $output
Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $output
Copy-Item -LiteralPath (Join-Path $root 'plugins/fl-probe/NICE-PLUG-LICENSE.txt') -Destination $output
foreach ($notice in @('embedded-model-runtime.txt', 'bundled-dependencies.txt', 'msvc-runtime.txt')) {
    Copy-Item -LiteralPath (Join-Path $root "docs/licenses/$notice") -Destination $output
}
& (Join-Path $PSScriptRoot 'test-plugin-hosts.ps1') -Product Podcast -PluginDirectory $output -TargetDir $settings.TargetDir
$expectedHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $dll).Hash
$artifacts = Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $output 'CleanSlate Podcast.clap'), (Join-Path $bundle 'CleanSlate Podcast.vst3')
if (@($artifacts | Where-Object Hash -ne $expectedHash).Count -ne 0) {
    throw 'Podcast bundle differs from its source DLL.'
}
$artifacts
Write-Output "Checked Podcast candidate: $output"
