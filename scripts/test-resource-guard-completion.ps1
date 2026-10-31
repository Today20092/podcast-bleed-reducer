$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$guard = Join-Path $PSScriptRoot 'resource-guard.ps1'
$temporary = Join-Path ([IO.Path]::GetTempPath()) ('cleanslate-guard-completion-' + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $temporary | Out-Null
$shell = (Get-Process -Id $PID).Path
$marker = Join-Path $temporary 'descendant-completed.txt'
$receipt = Join-Path $temporary 'completion.json'
$diagnostic = [IO.Path]::ChangeExtension($receipt, '.diagnostics.log')
$childCommand = "`$timer=[Diagnostics.Stopwatch]::StartNew(); while (-not (Test-Path -LiteralPath '$diagnostic')) { if (`$timer.Elapsed.TotalSeconds -ge 6) { exit 23 }; Start-Sleep -Milliseconds 50 }; Set-Content -LiteralPath '$marker' -Value completed"
$encodedChild = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($childCommand))
$rootCommand = "Start-Process -FilePath '$shell' -ArgumentList @('-NoProfile','-EncodedCommand','$encodedChild') -WindowStyle Hidden; exit 17"
try {
    & $guard -FilePath $shell -ArgumentList @('-NoProfile','-Command',$rootCommand) -ReceiptPath $receipt -TimeoutSeconds 10
    throw 'Nonzero root exit was accepted.'
} catch { if (-not (Test-Path -LiteralPath $receipt)) { throw } }
$result = Get-Content -LiteralPath $receipt -Raw | ConvertFrom-Json
if ($result.reason -ne 'command-failed' -or $result.exit_code -ne 17 -or -not $result.cleanup_verified) { throw 'Root exit or cleanup was lost.' }
if (-not (Test-Path -LiteralPath $marker)) { throw 'Guard stopped before the real descendant completed.' }
if ($result.root_exit_code -ne 17 -or $result.remaining_process_ids.Count -ne 0) { throw 'Final root/job diagnostics are missing or incorrect.' }
$log = Get-Content -LiteralPath $result.diagnostic_path -Raw
if ($log -notmatch 'Owned root exited: 17; active job processes: [1-9]') { throw 'The root-exit diagnostic did not capture the surviving job members.' }
Write-Output "Resource guard delayed-descendant completion passed; evidence: $temporary"
