#requires -Version 7.0
param(
    [Parameter(Mandatory)][string]$PluginDirectory,
    [Parameter(Mandatory)][string]$HostDirectory
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$PluginDirectory = (Resolve-Path -LiteralPath $PluginDirectory).Path
$HostDirectory = (Resolve-Path -LiteralPath $HostDirectory).Path
$reviewed = Get-Content -LiteralPath (Join-Path $PSScriptRoot '../docs/licenses/directml-1.15.4-runtime.json') -Raw | ConvertFrom-Json
$runtime = @($reviewed.files | Where-Object { $_.path -ceq 'DirectML.dll' })
if ($runtime.Count -ne 1 -or $runtime[0].sha256 -cnotmatch '^[0-9a-f]{64}$') { throw 'Invalid reviewed DirectML inventory.' }
$expectedHash = $runtime[0].sha256
$cases = @(
    @{ Host = 'smart_gate_clap_check.exe'; Plugin = 'CleanSlate Smart Gate.clap' },
    @{ Host = 'smart_gate_vst3_check.exe'; Plugin = 'CleanSlate Smart Gate.vst3/Contents/x86_64-win/CleanSlate Smart Gate.vst3' }
)
# Validate every format before launching any process or copying candidate files.
foreach ($case in $cases) {
    $sourceRuntime = Join-Path (Split-Path -Parent (Join-Path $PluginDirectory $case.Plugin)) 'DirectML.dll'
    if ((Get-Item -LiteralPath $sourceRuntime).Length -ne $runtime[0].bytes -or
        (Get-FileHash -LiteralPath $sourceRuntime -Algorithm SHA256).Hash.ToLowerInvariant() -cne $expectedHash) {
        throw "Candidate DirectML differs from reviewed inventory: $sourceRuntime"
    }
}
if (-not ('CleanSlate.Build.OwnedTask' -as [type])) { Add-Type -Path (Join-Path $PSScriptRoot 'resource-guard.cs') }
if (-not [CleanSlate.Build.OwnedTask]::IsContained($env:CLEANSLATE_RESOURCE_JOB)) {
    & (Join-Path $PSScriptRoot 'resource-guard.ps1') -FilePath (Get-Process -Id $PID).Path `
        -ArgumentList @('-NoProfile', '-File', $PSCommandPath, '-PluginDirectory', $PluginDirectory, '-HostDirectory', $HostDirectory) `
        -WorkingDirectory $PSScriptRoot -TimeoutSeconds 180
    return
}
$fixture = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) "CleanSlate/PluginLoaderChecks/$([Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $fixture | Out-Null
Write-Output "Retained loader fixture: $fixture"
foreach ($case in $cases) {
    foreach ($negative in @($false, $true)) {
        $run = Join-Path $fixture "$($case.Host)-missing-$negative"
        $candidate = Join-Path $run 'candidate'
        $hostRoot = Join-Path $run 'host'
        $empty = Join-Path $run 'empty'
        New-Item -ItemType Directory -Path $candidate, $hostRoot, $empty | Out-Null
        Get-ChildItem -LiteralPath $PluginDirectory | Copy-Item -Destination $candidate -Recurse
        $hostBinary = Join-Path $hostRoot $case.Host
        Copy-Item -LiteralPath (Join-Path $HostDirectory $case.Host) -Destination $hostBinary
        $plugin = Join-Path $candidate $case.Plugin
        if ($negative) {
            # Remove only disposable fixture sidecars; retain the original candidate.
            foreach ($runtime in Get-ChildItem -LiteralPath $candidate -Filter DirectML.dll -File -Recurse) {
                Remove-Item -LiteralPath $runtime.FullName
            }
        }
        $info = [Diagnostics.ProcessStartInfo]::new()
        $info.FileName = $hostBinary
        $info.ArgumentList.Add('--candidate-loader')
        $info.ArgumentList.Add($plugin)
        $info.WorkingDirectory = $empty
        $info.UseShellExecute = $false
        $info.CreateNoWindow = $true
        $info.RedirectStandardOutput = $true
        $info.RedirectStandardError = $true
        $info.Environment['PATH'] = Join-Path $env:SystemRoot 'System32'
        $info.Environment['LOCALAPPDATA'] = $empty
        $process = [Diagnostics.Process]::new()
        $process.StartInfo = $info
        try {
            if (-not $process.Start()) { throw 'Could not launch isolated host.' }
            $stdout = $process.StandardOutput.ReadToEndAsync()
            $stderr = $process.StandardError.ReadToEndAsync()
            if (-not $process.WaitForExit(30000)) { $process.Kill($true); throw 'Isolated host exceeded 30 seconds.' }
            $outText = $stdout.GetAwaiter().GetResult()
            $errorText = $stderr.GetAwaiter().GetResult()
            $outText | Set-Content -LiteralPath (Join-Path $run 'stdout.txt')
            $errorText | Set-Content -LiteralPath (Join-Path $run 'stderr.txt')
            if ($negative) {
                if ($process.ExitCode -eq 0 -or $outText -match '(?m)^PASS:') { throw 'Missing-sidecar negative control unexpectedly passed.' }
                Write-Output "PASS: $($case.Host) missing-DirectML control rejected (exit $($process.ExitCode))."
            } else {
                if ($process.ExitCode -ne 0 -or $outText -notmatch '(?m)^PASS:') { throw "Isolated loader failed: $errorText" }
                $match = [regex]::Match($outText, '(?m)^DIRECTML_PATH: (.+)\r?$')
                if (-not $match.Success) { throw 'Host did not report the loaded runtime path.' }
                # Rust canonicalize reports the Win32 extended-length prefix.
                $loaded = $match.Groups[1].Value.Trim() -replace '^\\\\\?\\', ''
                $expected = (Resolve-Path -LiteralPath (Join-Path (Split-Path -Parent $plugin) 'DirectML.dll')).Path
                if ([IO.Path]::GetFullPath($loaded) -ne [IO.Path]::GetFullPath($expected)) { throw 'Host loaded a runtime outside the staged sidecar.' }
                if ((Get-FileHash -LiteralPath $loaded -Algorithm SHA256).Hash.ToLowerInvariant() -cne $expectedHash) { throw 'Loaded runtime hash differs from reviewed inventory.' }
                Write-Output $outText.Trim()
                Write-Output "DirectML SHA256: $expectedHash"
            }
        } finally { $process.Dispose() }
    }
}
