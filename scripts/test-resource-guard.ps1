$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$guard = Join-Path $PSScriptRoot 'resource-guard.ps1'
$temporary = Join-Path ([IO.Path]::GetTempPath()) ('cleanslate-guard-' + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $temporary | Out-Null
$shell = (Get-Process -Id $PID).Path
$success = Join-Path $temporary 'success.json'
& $guard -FilePath $shell -ArgumentList @('-NoProfile','-Command','Write-Output ''guarded output''') -ReceiptPath $success
$result = Get-Content -LiteralPath $success -Raw | ConvertFrom-Json
if ($result.status -ne 'done' -or -not $result.cleanup_verified -or $result.peak_job_commit_bytes -le 0 -or $result.peak_working_set_bytes -le 0) { throw 'Successful work was not accounted and cleaned up.' }
if (($result.job_limit_flags -band 0x300) -ne 0 -or ($result.job_limit_flags -band 0x2000) -eq 0) { throw 'The owned job must have no memory limit and retain kill-on-close.' }
if ($result.policy.PSObject.Properties.Name -ne 'timeout_seconds') { throw 'Memory thresholds remain in the active policy.' }
if ((Get-Content -LiteralPath $result.log_path -Raw) -notmatch 'guarded output') { throw 'Child output was lost.' }
$timed = Join-Path $temporary 'timeout.json'
$childIdPath = Join-Path $temporary 'child-id.txt'
$preserved = Join-Path $temporary 'last-good-output.txt'
'preserve me' | Set-Content -LiteralPath $preserved
$treeCommand = "`$child=Start-Process -FilePath '$shell' -ArgumentList @('-NoProfile','-Command','Start-Sleep -Seconds 30') -WindowStyle Hidden -PassThru; `$child.Id | Set-Content -LiteralPath '$childIdPath'; Start-Sleep -Seconds 30"
try {
    & $guard -FilePath $shell -ArgumentList @('-NoProfile','-Command',$treeCommand) -WorkingDirectory $temporary -ReceiptPath $timed -TimeoutSeconds 4
    throw 'Timeout was accepted.'
} catch { if (-not (Test-Path -LiteralPath $timed)) { throw } }
$result = Get-Content -LiteralPath $timed -Raw | ConvertFrom-Json
if ($result.reason -ne 'timeout' -or -not $result.cleanup_verified -or $result.max_active_processes -lt 2) { throw 'Timeout failed to account for and stop the child tree.' }
$childId = [int](Get-Content -LiteralPath $childIdPath)
if (Get-Process -Id $childId -ErrorAction SilentlyContinue) { throw 'A descendant survived the timeout.' }
if ((Get-Content -LiteralPath $preserved).Trim() -ne 'preserve me') { throw 'Last-good output was changed.' }
$failed = Join-Path $temporary 'failed.json'
try {
    & $guard -FilePath $shell -ArgumentList @('-NoProfile','-Command','exit 17') -ReceiptPath $failed
    throw 'Nonzero exit was accepted.'
} catch { if (-not (Test-Path -LiteralPath $failed)) { throw } }
$result = Get-Content -LiteralPath $failed -Raw | ConvertFrom-Json
if ($result.reason -ne 'command-failed' -or $result.exit_code -ne 17 -or -not $result.cleanup_verified) { throw 'Command failure was lost.' }
Write-Output "Resource guard process/tree/timeout/failure checks passed; evidence: $temporary"
$nested = Join-Path $temporary 'nested.json'
$nestedCommand = "& '$guard' -FilePath '$shell' -ArgumentList @('-NoProfile','-Command','(Get-Location).Path') -WorkingDirectory '$temporary'"
& $guard -FilePath $shell -ArgumentList @('-NoProfile','-Command',$nestedCommand) -ReceiptPath $nested -TimeoutSeconds 10
$result = Get-Content -LiteralPath $nested -Raw | ConvertFrom-Json
if ((Get-Content -LiteralPath $result.log_path -Raw) -notlike "*$temporary*") { throw 'Nested guarded work did not preserve its working directory.' }

$controllerReceipt = Join-Path $temporary 'controller.json'
$controllerChildIdPath = Join-Path $temporary 'controller-child-id.txt'
$controllerRootIdPath = Join-Path $temporary 'controller-root-id.txt'
$controllerWork = "`$PID | Set-Content -LiteralPath '$controllerRootIdPath'; `$child=Start-Process -FilePath '$shell' -ArgumentList @('-NoProfile','-Command','Start-Sleep -Seconds 30') -WindowStyle Hidden -PassThru; `$child.Id | Set-Content -LiteralPath '$controllerChildIdPath'; Start-Sleep -Seconds 30"
$encodedWork = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($controllerWork))
$controllerCommand = "& '$guard' -FilePath '$shell' -ArgumentList @('-NoProfile','-EncodedCommand','$encodedWork') -ReceiptPath '$controllerReceipt' -TimeoutSeconds 25"
$encodedController = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($controllerCommand))
$controller = Start-Process -FilePath $shell -ArgumentList @('-NoProfile','-EncodedCommand',$encodedController) -WindowStyle Hidden -PassThru
try {
    $readyTimer = [Diagnostics.Stopwatch]::StartNew()
    while (-not (Test-Path -LiteralPath $controllerChildIdPath)) {
        if ($controller.HasExited -or $readyTimer.Elapsed.TotalSeconds -gt 10) { throw 'Disposable controller never became ready.' }
        Start-Sleep -Milliseconds 100
    }
    $busy = Join-Path $temporary 'busy.json'
    try {
        & $guard -FilePath $shell -ArgumentList @('-NoProfile','-Command','exit 0') -WorkingDirectory $temporary -ReceiptPath $busy
        throw 'Another workspace entered the heavy-task lock.'
    } catch { if (-not (Test-Path -LiteralPath $busy)) { throw } }
    $result = Get-Content -LiteralPath $busy -Raw | ConvertFrom-Json
    if ($result.reason -ne 'heavy-task-busy' -or $result.process_id) { throw 'The machine-global lock did not reject the competing workspace.' }
    # Only this fixture's captured controller is terminated to simulate guard process death.
    $controller.Kill(); $controller.WaitForExit()
    $ids = @([int](Get-Content -LiteralPath $controllerChildIdPath),[int](Get-Content -LiteralPath $controllerRootIdPath))
    $cleanupTimer = [Diagnostics.Stopwatch]::StartNew()
    while (@(Get-Process -Id $ids -ErrorAction SilentlyContinue).Count -and $cleanupTimer.Elapsed.TotalSeconds -lt 5) { Start-Sleep -Milliseconds 100 }
    if (@(Get-Process -Id $ids -ErrorAction SilentlyContinue).Count) { throw 'Owned descendants survived guard process death.' }
} finally {
    if (-not $controller.HasExited) { $controller.Kill(); $controller.WaitForExit() }
    $controller.Dispose()
}
$recovered = Join-Path $temporary 'recovered.json'
& $guard -FilePath $shell -ArgumentList @('-NoProfile','-Command','exit 0') -ReceiptPath $recovered
$result = Get-Content -LiteralPath $recovered -Raw | ConvertFrom-Json
if ($result.status -ne 'done') { throw 'The abandoned global lock was not recoverable.' }
Write-Output "Resource guard unlimited-memory/nesting/competing-workspace/crash checks passed; evidence: $temporary"
