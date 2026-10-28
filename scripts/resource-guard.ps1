param(
    [Parameter(Mandatory)][string]$FilePath,
    [string[]]$ArgumentList = @(),
    [string]$WorkingDirectory = (Get-Location).Path,
    [string]$ReceiptPath,
    [ValidateRange(0, 1048576)][double]$MinAvailableGiB = 14,
    [ValidateRange(0, 1048576)][double]$ReserveGiB = 8,
    [ValidateRange(0.0625, 1024)][double]$MaxCommitGiB = 6,
    [ValidateRange(1, 86400)][int]$TimeoutSeconds = 1800
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ([Environment]::OSVersion.Platform -ne 'Win32NT') { throw 'The local resource guard requires Windows Job Object containment.' }
if (-not ('CleanSlate.Build.OwnedTask' -as [type])) { Add-Type -Path (Join-Path $PSScriptRoot 'resource-guard.cs') }
$executable = (Get-Command $FilePath -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
$WorkingDirectory = (Resolve-Path -LiteralPath $WorkingDirectory).Path
# An inherited string alone never bypasses the lock: verify membership in that live job.
if ([CleanSlate.Build.OwnedTask]::IsContained($env:CLEANSLATE_RESOURCE_JOB)) {
    if ($ReceiptPath) { throw 'Nested guarded work uses the outer task receipt; omit ReceiptPath.' }
    Push-Location -LiteralPath $WorkingDirectory
    try {
        & $executable @ArgumentList
        if ($LASTEXITCODE -ne 0) { throw "Nested guarded command failed ($LASTEXITCODE)." }
    } finally { Pop-Location }
    return
}
if (-not $ReceiptPath) {
    $directory = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'CleanSlate/BuildReceipts'
    $ReceiptPath = Join-Path $directory ((Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [Guid]::NewGuid().ToString('N') + '.json')
}
$ReceiptPath = [IO.Path]::GetFullPath($ReceiptPath)
New-Item -ItemType Directory -Path (Split-Path -Parent $ReceiptPath) -Force | Out-Null
$logPath = [IO.Path]::ChangeExtension($ReceiptPath, '.log')
if ((Test-Path -LiteralPath $ReceiptPath) -or (Test-Path -LiteralPath $logPath)) { throw 'Choose a new receipt path; existing evidence is never overwritten.' }
$timer = [Diagnostics.Stopwatch]::StartNew()
$receipt = [ordered]@{
    schema_version=1
    source_commit=(& git -C $WorkingDirectory rev-parse HEAD 2>$null)
    executable=$executable
    executable_sha256=(Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash.ToLowerInvariant()
    arguments=$ArgumentList
    working_directory=$WorkingDirectory
    log_path=$logPath
    reason='guard-error'
    status='failed'
    exit_code=$null
    process_id=$null
    cleanup_verified=$true
    elapsed_seconds=0
    peak_private_bytes=0L
    peak_working_set_bytes=0L
    peak_job_commit_bytes=0L
    minimum_available_physical_bytes=$null
    minimum_available_commit_bytes=$null
    max_active_processes=0
    policy=@{ min_available_gib=$MinAvailableGiB; reserve_gib=$ReserveGiB; max_commit_gib=$MaxCommitGiB; timeout_seconds=$TimeoutSeconds }
    competing_processes=@()
    error=$null
}
$mutex = $null; $locked = $false; $task = $null
try {
    $mutex = [Threading.Mutex]::new($false, 'Global\CleanSlate.HeavyTask.v1')
    try { $locked = $mutex.WaitOne(0) } catch [Threading.AbandonedMutexException] { $locked = $true }
    if (-not $locked) { $receipt.reason='heavy-task-busy'; throw 'Another workspace owns the global heavy-task lock.' }
    $memory = [CleanSlate.Build.OwnedTask]::Memory()
    $receipt.minimum_available_physical_bytes=$memory.AvailablePhysical
    $receipt.minimum_available_commit_bytes=$memory.AvailableCommit
    if ($memory.AvailablePhysical -lt $MinAvailableGiB*1GB) { $receipt.reason='low-memory-before-start'; throw 'Available physical memory is below the start requirement.' }
    if ($memory.AvailableCommit -lt $MaxCommitGiB*1GB) { $receipt.reason='low-commit-before-start'; throw 'System commit headroom is below the owned-task budget.' }
    $receipt.competing_processes=@(Get-Process -Name cargo,rustc,rust-analyzer,link,lld-link,ida,ida64,ghidra -ErrorAction SilentlyContinue | Select-Object Id,ProcessName)
    if ($receipt.competing_processes.Count) { $receipt.reason='unowned-heavy-process'; throw 'An existing compiler/analyzer must finish before starting this task.' }
    $task = [CleanSlate.Build.OwnedTask]::new($executable, $ArgumentList, $WorkingDirectory, [uint64]($MaxCommitGiB*1GB), $logPath)
    $receipt.process_id=$task.ProcessId
    while ($true) {
        $sample=$task.Sample(); $memory=[CleanSlate.Build.OwnedTask]::Memory()
        $receipt.peak_private_bytes=[Math]::Max($receipt.peak_private_bytes,[long]$sample.PrivateBytes)
        $receipt.peak_working_set_bytes=[Math]::Max($receipt.peak_working_set_bytes,[long]$sample.WorkingSetBytes)
        $receipt.peak_job_commit_bytes=[Math]::Max($receipt.peak_job_commit_bytes,[long]$sample.PeakCommitBytes)
        $receipt.max_active_processes=[Math]::Max($receipt.max_active_processes,[int]$sample.ActiveProcesses)
        $receipt.minimum_available_physical_bytes=[Math]::Min([long]$receipt.minimum_available_physical_bytes,[long]$memory.AvailablePhysical)
        $receipt.minimum_available_commit_bytes=[Math]::Min([long]$receipt.minimum_available_commit_bytes,[long]$memory.AvailableCommit)
        if ($sample.MemoryLimitHit) { $receipt.reason='job-memory-limit'; break }
        if ($memory.AvailablePhysical -lt $ReserveGiB*1GB) { $receipt.reason='low-memory-running'; break }
        if ($memory.AvailableCommit -lt $ReserveGiB*1GB) { $receipt.reason='low-commit-running'; break }
        if ($timer.Elapsed.TotalSeconds -ge $TimeoutSeconds) { $receipt.reason='timeout'; break }
        if ($sample.ActiveProcesses -eq 0) {
            $receipt.exit_code=$task.ExitCode
            if ($receipt.exit_code -eq 0) { $receipt.reason='completed'; $receipt.status='done' } else { $receipt.reason='command-failed' }
            break
        }
        Start-Sleep -Seconds 1
    }
    if ($receipt.status -ne 'done') { throw "Guarded task stopped: $($receipt.reason)." }
} catch {
    $receipt.error=$_.Exception.Message
    if ($receipt.reason -in @('heavy-task-busy','low-memory-before-start','low-commit-before-start','unowned-heavy-process','job-memory-limit','low-memory-running','low-commit-running','timeout')) { $receipt.status='resource-blocked' }
} finally {
    try {
        if ($task) {
            try { $receipt.cleanup_verified=$task.Stop() }
            catch { $receipt.cleanup_verified=$false; $receipt.error=$_.Exception.Message }
            finally { $task.Dispose() }
        }
    } finally {
        if ($locked) { $mutex.ReleaseMutex() }
        if ($mutex) { $mutex.Dispose() }
        $receipt.elapsed_seconds=$timer.Elapsed.TotalSeconds
        $receipt | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $ReceiptPath -Encoding utf8
        Write-Host "Resource receipt: $ReceiptPath"
        if (Test-Path -LiteralPath $logPath) { Get-Content -LiteralPath $logPath -Tail 60 | Write-Host }
    }
}
if (-not $receipt.cleanup_verified) { throw "Owned process cleanup could not be verified. Receipt: $ReceiptPath" }
if ($receipt.status -ne 'done') { throw "$($receipt.error) Receipt: $ReceiptPath" }
