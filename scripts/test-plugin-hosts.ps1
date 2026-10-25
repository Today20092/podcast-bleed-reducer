#requires -Version 7.0
param([string]$PluginDirectory, [string]$TargetDir)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path -Parent $PSScriptRoot
if (-not $PluginDirectory) { $PluginDirectory = Join-Path $root 'dist/fl-probe-candidate' }
$PluginDirectory = (Resolve-Path -LiteralPath $PluginDirectory).Path
$settings = & (Join-Path $PSScriptRoot 'local-build-settings.ps1') -TargetDir $TargetDir
$cases = @(
    @{ Name = 'CLAP'; Host = 'host_check.exe'; Plugin = 'CleanSlate FL Probe.clap' },
    @{ Name = 'VST3'; Host = 'vst3_host_check.exe'; Plugin = 'CleanSlate FL Probe.vst3/Contents/x86_64-win/CleanSlate FL Probe.vst3' }
)
foreach ($case in $cases) {
    $binary = Join-Path $settings.TargetDir "debug/examples/$($case.Host)"
    $plugin = Join-Path $PluginDirectory $case.Plugin
    if (-not (Test-Path -LiteralPath $binary) -or -not (Test-Path -LiteralPath $plugin)) { throw 'Build the plugin and host examples first.' }
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $binary
    $info.ArgumentList.Add($plugin)
    $info.WorkingDirectory = $root
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $fixtureParent = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'CleanSlate/PluginHostChecks'
    $fixtureRoot = [IO.Path]::GetFullPath((Join-Path $fixtureParent ([Guid]::NewGuid().ToString('N'))))
    New-Item -ItemType Directory -Path $fixtureRoot -Force | Out-Null
    # Redirect this child only; synthetic audio must not populate the owner's Session cache.
    $info.Environment['LOCALAPPDATA'] = $fixtureRoot
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $observedPeak = [long]0
    try {
        if (-not $process.Start()) { throw "Could not start $($case.Name) host." }
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        while (-not $process.HasExited) {
            if ($timer.Elapsed.TotalSeconds -gt 30) { $process.Kill($true); throw "$($case.Name) host exceeded 30 seconds." }
            try { $process.Refresh(); $observedPeak = [Math]::Max($observedPeak, $process.PeakWorkingSet64) }
            catch { if (-not $process.HasExited) { throw } }
            Start-Sleep -Milliseconds 10
        }
        $process.WaitForExit()
        $outText = $stdout.GetAwaiter().GetResult()
        $errorText = $stderr.GetAwaiter().GetResult()
        if ($outText) { Write-Output $outText.Trim() }
        if ($errorText) { Write-Output $errorText.Trim() }
        if ($process.ExitCode -ne 0) { throw "$($case.Name) host failed: exit $($process.ExitCode)" }
        if ($outText -notmatch '^PASS:') { throw "$($case.Name) host returned no passing check receipt." }
        Write-Output ('{0}: {1:N2} seconds; largest sampled peak working set {2:N2} MiB (10 ms polling; test process only).' -f $case.Name, $timer.Elapsed.TotalSeconds, ($observedPeak / 1MB))
    } finally {
        $timer.Stop(); $process.Dispose()
        if ([IO.Path]::GetDirectoryName($fixtureRoot) -ne [IO.Path]::GetFullPath($fixtureParent)) { throw 'Unsafe host-fixture cleanup target.' }
        if (Test-Path -LiteralPath $fixtureRoot) { Remove-Item -LiteralPath $fixtureRoot -Recurse -Force }
    }
}
