#requires -Version 7.0
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path -Parent $PSScriptRoot
$channel = [regex]::Match((Get-Content -LiteralPath (Join-Path $root 'rust-toolchain.toml') -Raw), '(?m)^\s*channel\s*=\s*"([^"]+)"').Groups[1].Value
if (-not $channel) { throw 'The repository must pin its Rust toolchain.' }
# Resolve paths only; heavy work must launch the actual executables, not rustup proxies.
$pinnedCargo = & rustup which --toolchain $channel cargo
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $pinnedCargo -PathType Leaf)) { throw 'Pinned Cargo is unavailable.' }
$pinnedRustc = & rustup which --toolchain $channel rustc
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $pinnedRustc -PathType Leaf)) { throw 'Pinned Rustc is unavailable.' }
$originalPath = $env:PATH
$originalRustc = $env:RUSTC
try {
    $env:PATH = (Split-Path -Parent $pinnedCargo) + [IO.Path]::PathSeparator + $originalPath
    $env:RUSTC = $pinnedRustc
    if (-not ('CleanSlate.Build.OwnedTask' -as [type])) { Add-Type -Path (Join-Path $PSScriptRoot 'resource-guard.cs') }
    if (-not [CleanSlate.Build.OwnedTask]::IsContained($env:CLEANSLATE_RESOURCE_JOB)) {
        & (Join-Path $PSScriptRoot 'resource-guard.ps1') -FilePath (Get-Process -Id $PID).Path -ArgumentList @('-NoProfile', '-File', $PSCommandPath) -WorkingDirectory $root
        return
    }
    # Normal candidate is copied and checks driver absence before enabling the debug-only seam.
    & (Join-Path $PSScriptRoot 'build-podcast.ps1')
    $settings = & (Join-Path $PSScriptRoot 'local-build-settings.ps1')
    $manifest = Join-Path $root 'plugins/podcast/Cargo.toml'
    function Invoke-AcceptanceCargo([string[]]$Arguments) {
        & $pinnedCargo @Arguments
        if ($LASTEXITCODE -ne 0) { throw 'Podcast acceptance build failed.' }
    }
    $parent = Join-Path $root 'dist/podcast-acceptance-driver'
    foreach ($path in @((Join-Path $root 'dist'), $parent)) {
        if ((Test-Path -LiteralPath $path) -and ((Get-Item -LiteralPath $path).Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Acceptance output must not follow a link.' }
    }
    $output = Join-Path $parent ([Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $output | Out-Null
    $normalReference = Join-Path $output 'Podcast-normal-reference.dll'
    Copy-Item -LiteralPath (Join-Path $settings.TargetDir 'debug/cleanslate_podcast.dll') -Destination $normalReference
    try {
        Invoke-AcceptanceCargo (@('clippy', '--manifest-path', $manifest, '--features', 'acceptance-driver', '--all-targets') + $settings.CargoArguments + @('--', '-D', 'warnings'))
        Invoke-AcceptanceCargo (@('build', '--manifest-path', $manifest, '--features', 'acceptance-driver') + $settings.CargoArguments)
        Invoke-AcceptanceCargo (@('build', '--manifest-path', $manifest, '--features', 'acceptance-driver', '--examples') + $settings.CargoArguments)
        $dll = Join-Path $output 'Podcast-acceptance-only.dll'
        Copy-Item -LiteralPath (Join-Path $settings.TargetDir 'debug/cleanslate_podcast.dll') -Destination $dll
        'TEST ARTIFACT ONLY. Contains an explicit debug acceptance driver. Never install or distribute. Normal candidates are preserved separately.' | Set-Content -LiteralPath (Join-Path $output 'DO-NOT-INSTALL.txt')
        # Exercise the exact build guard without compiling a second release dependency graph.
        $guardCheck = Join-Path $output 'release-guard-check.exe'
        & $pinnedRustc (Join-Path $root 'plugins/fl-probe/product-build.rs') '--edition=2024' '-o' $guardCheck
        if ($LASTEXITCODE -ne 0) { throw 'Could not compile the release guard check.' }
        $guardInfo = [Diagnostics.ProcessStartInfo]::new($guardCheck)
        $guardInfo.UseShellExecute = $false
        $guardInfo.CreateNoWindow = $true
        $guardInfo.RedirectStandardOutput = $true
        $guardInfo.RedirectStandardError = $true
        $guardInfo.Environment['CARGO_PKG_NAME'] = 'cleanslate-podcast'
        $guardInfo.Environment['CARGO_FEATURE_ACCEPTANCE_DRIVER'] = '1'
        $guardInfo.Environment['PROFILE'] = 'release'
        $guardProcess = [Diagnostics.Process]::Start($guardInfo)
        try {
            $guardOutput = $guardProcess.StandardOutput.ReadToEnd()
            $guardError = $guardProcess.StandardError.ReadToEnd()
            $guardProcess.WaitForExit()
            ($guardOutput + $guardError) | Set-Content -LiteralPath (Join-Path $output 'release-guard-check.log')
            if ($guardProcess.ExitCode -eq 0 -or $guardError -notmatch 'acceptance driver must never ship') { throw 'Release driver guard did not reject the artifact.' }
        } finally { $guardProcess.Dispose() }
        $originalLocalAppData = $env:LOCALAPPDATA
        try {
            foreach ($name in @('podcast_host_check', 'podcast_vst3_host_check')) {
                $fixture = Join-Path $output ($name + '-private-cache')
                New-Item -ItemType Directory -Path $fixture | Out-Null
                $env:LOCALAPPDATA = $fixture
                $hostBinary = Join-Path $output ($name + '.exe')
                Copy-Item -LiteralPath (Join-Path $settings.TargetDir "debug/examples/$name.exe") -Destination $hostBinary
                $normalInfo = [Diagnostics.ProcessStartInfo]::new($hostBinary)
                $normalInfo.ArgumentList.Add($normalReference)
                $normalInfo.UseShellExecute = $false
                $normalInfo.CreateNoWindow = $true
                $normalInfo.RedirectStandardOutput = $true
                $normalInfo.RedirectStandardError = $true
                $normalInfo.Environment['LOCALAPPDATA'] = Join-Path $fixture 'normal-driver-rejection'
                New-Item -ItemType Directory -Path $normalInfo.Environment['LOCALAPPDATA'] | Out-Null
                $normalProcess = [Diagnostics.Process]::Start($normalInfo)
                try {
                    $normalOut = $normalProcess.StandardOutput.ReadToEndAsync()
                    $normalError = $normalProcess.StandardError.ReadToEndAsync()
                    $normalProcess.WaitForExit()
                    $normalText = $normalOut.GetAwaiter().GetResult() + $normalError.GetAwaiter().GetResult()
                    $normalText | Set-Content -LiteralPath (Join-Path $output ($name + '-normal-rejection.log'))
                    if ($normalProcess.ExitCode -eq 0 -or $normalText -notmatch 'Acceptance mode requires the debug-only driver') { throw 'Normal candidate did not reject acceptance mode as expected.' }
                } finally { $normalProcess.Dispose() }
                $lines = & $hostBinary $dll
                $exitCode = $LASTEXITCODE
                $lines | Set-Content -LiteralPath (Join-Path $output ($name + '.log'))
                $lines | Write-Output
                if ($exitCode -ne 0 -or -not ($lines -match '^PASS actual-format Session:')) { throw "$name failed actual-format Session acceptance." }
            }
        } finally { $env:LOCALAPPDATA = $originalLocalAppData }
        Get-FileHash -Algorithm SHA256 -LiteralPath $dll
    } catch {
        # A restoration failure must not erase the original acceptance failure.
        $_ | Out-String | Set-Content -LiteralPath (Join-Path $output 'acceptance-failure.log')
        throw
    } finally {
        # Feature builds replace shared outputs. Restore ordinary outputs even when a check fails.
        & (Join-Path $PSScriptRoot 'build-podcast.ps1')
    }
    Write-Output "Checked acceptance-only artifact: $output"
} finally {
    $env:PATH = $originalPath
    $env:RUSTC = $originalRustc
}
