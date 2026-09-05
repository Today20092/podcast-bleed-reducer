param(
    [Parameter(Mandatory)][string]$Binary,
    [Parameter(Mandatory)][string]$Settings,
    [Parameter(Mandatory)][string[]]$Sources,
    [int]$Iterations = 20
)
$ErrorActionPreference = 'Stop'
if ($Iterations -lt 1) { throw 'Iterations must be positive' }
$arguments = @("$Iterations", '--ownership', $Settings) + $Sources
# Windows file names cannot contain quotes. Quote each argument for spaces.
if ($arguments.Where({ $_.Contains('"') }).Count) { throw 'Invalid quote in argument' }
$info = New-Object System.Diagnostics.ProcessStartInfo
$info.FileName = (Resolve-Path -LiteralPath $Binary).Path
$info.Arguments = ($arguments | ForEach-Object { '"' + $_ + '"' }) -join ' '
$info.UseShellExecute = $false
$info.CreateNoWindow = $true
$info.RedirectStandardOutput = $true
$info.RedirectStandardError = $true
$process = New-Object System.Diagnostics.Process
$process.StartInfo = $info
$watch = [System.Diagnostics.Stopwatch]::StartNew()
[void]$process.Start()
$stdout = $process.StandardOutput.ReadToEndAsync()
$stderr = $process.StandardError.ReadToEndAsync()
$peakWorkingSet = 0L
$peakPrivate = 0L
while (-not $process.WaitForExit(100)) {
    $process.Refresh()
    if (-not $process.HasExited) {
        $peakWorkingSet = [Math]::Max($peakWorkingSet, $process.PeakWorkingSet64)
        $peakPrivate = [Math]::Max($peakPrivate, $process.PrivateMemorySize64)
    }
}
$watch.Stop()
$out = $stdout.GetAwaiter().GetResult()
$err = $stderr.GetAwaiter().GetResult()
$exitCode = $process.ExitCode
$process.Dispose()
if ($exitCode -ne 0) { throw "Benchmark exited ${exitCode}: $err $out" }
[ordered]@{
    benchmark = ($out | ConvertFrom-Json)
    wall_ms = $watch.Elapsed.TotalMilliseconds
    observed_peak_working_set_bytes = $peakWorkingSet
    observed_peak_private_bytes = $peakPrivate
    memory_poll_ms = 100
    memory_scope = 'Benchmark process only; excludes GUI, waveform cache, and audition PCM.'
    stderr = $err
} | ConvertTo-Json -Depth 20
