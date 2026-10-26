param(
    [ValidateSet('pass','fail','wait','hold-coordinator')][string]$Operation,
    [Parameter(Mandatory)][string]$EvidenceDirectory
)
$ErrorActionPreference = 'Stop'
$directory=[IO.Path]::GetFullPath($EvidenceDirectory)
$temporary=[IO.Path]::GetFullPath([IO.Path]::GetTempPath())
if (-not $directory.StartsWith($temporary,[StringComparison]::OrdinalIgnoreCase)) { throw 'Disposable runner fixture evidence must stay under the temporary directory.' }
New-Item -ItemType Directory -Path $directory -Force | Out-Null
if ($Operation -eq 'hold-coordinator') {
    $mutex=[Threading.Mutex]::new($false,'Global\CleanSlate.OvernightCoordinator.v1')
    $locked=$false
    try {
        try { $locked=$mutex.WaitOne(0) } catch [Threading.AbandonedMutexException] { $locked=$true }
        if (-not $locked) { throw 'Fixture coordinator lock was already owned.' }
        'ready' | Set-Content -LiteralPath (Join-Path $directory 'ready.txt')
        Start-Sleep -Seconds 30
    } finally { if ($locked) {$mutex.ReleaseMutex()};$mutex.Dispose() }
    return
}
Add-Content -LiteralPath (Join-Path $directory 'calls.txt') -Value $Operation
if ($Operation -eq 'fail') { throw 'Deliberate disposable check failure.' }
if ($Operation -eq 'wait') { Start-Sleep -Seconds 30 }
Write-Output 'Disposable runner check passed.'
