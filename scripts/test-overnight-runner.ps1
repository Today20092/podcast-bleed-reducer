$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$runner = Join-Path $PSScriptRoot 'overnight-runner.ps1'
$temporary = Join-Path ([IO.Path]::GetTempPath()) ('cleanslate-runner-' + [Guid]::NewGuid())
$repository = Join-Path $temporary 'repository'
$tracker = Join-Path $repository '.scratch/offline-supervised-editor/issues'
New-Item -ItemType Directory -Path $tracker -Force | Out-Null
& git init -q -b development $repository
& git -C $repository config user.name 'Runner fixture'
& git -C $repository config user.email 'runner-fixture@example.invalid'
& git -C $repository config core.autocrlf false
foreach ($id in @(1,2,3,4,87,88,99)) {
    $status = if ($id -in @(87,88,99)) { 'done' } elseif ($id -eq 3) { 'claimed' } else { 'open' }
    $dependency = if ($id -eq 1) { '4' } else { 'none' }
    "# Fixture $id`nStatus: $status`nReadiness: ready-for-agent`nAssignee: Another agent`nBlocked by: $dependency`n" | Set-Content -LiteralPath (Join-Path $tracker "$id-fixture.md")
}
'fixture source' | Set-Content -LiteralPath (Join-Path $repository 'source.txt')
& git -C $repository add .
& git -C $repository commit -q -m 'Disposable runner fixture'
$commit = (& git -C $repository rev-parse HEAD).Trim()
$queuePath = Join-Path $temporary 'queue.json'
$planPath = Join-Path $temporary 'plan.json'
$queue = @{ schema_version=1; canonical_tracker='.scratch/offline-supervised-editor/issues'; max_active_tickets=1; max_heavy_process_trees=1; requires_before_unattended_execution=@(87,88,99); non_executable_umbrellas=@(85,86); tasks=@(@{id=1;ticket='1-fixture.md';lane='desktop';blocked_by=@(4)},@{id=2;ticket='2-fixture.md';lane='desktop';blocked_by=@()},@{id=3;ticket='3-fixture.md';lane='desktop';blocked_by=@()}) }
$queue | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $queuePath
$plan = @{schema_version=1;run_id='fixture';workspaces=@{desktop=@{path=$repository;branch='development';commit=$commit}};steps=@(@{ticket_id=2;commands=@(@{kind='fixture';operation='pass';evidence_directory=(Join-Path $temporary 'output')});risks=@('Disposable test only')})}
$plan | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $planPath
$result = & $runner -RepositoryRoot $repository -QueuePath $queuePath -PlanPath $planPath -StateRoot (Join-Path $temporary 'runs')
if (($result.tasks.ticket_id -join ',') -ne '1,2,3' -or $result.tasks[0].reason -ne 'dependencies' -or $result.tasks[1].status -ne 'ready' -or $result.tasks[2].reason -ne 'claimed-other') { throw 'Dry-run lost queue order, dependency skipping or claim ownership.' }
if ((Get-Content -LiteralPath (Join-Path $tracker '2-fixture.md') -Raw) -notmatch 'Status: open' -or (Test-Path -LiteralPath (Join-Path $temporary 'runs'))) { throw 'Dry-run changed a ticket or created execution state.' }
Write-Output "Runner dry-run checks passed; evidence: $temporary"
$result = & $runner -RepositoryRoot $repository -QueuePath $queuePath -PlanPath $planPath -StateRoot (Join-Path $temporary 'runs') -Start
if ($result.tasks[1].status -ne 'checks-passed') { $result.tasks | ConvertTo-Json | Write-Host; throw 'The selected guarded fixture did not pass.' }
if ((Get-Content -LiteralPath (Join-Path $tracker '2-fixture.md') -Raw) -notmatch 'Status: claimed') { throw 'Command success improperly closed the ticket.' }
$calls = Join-Path $temporary 'output/calls.txt'
if (@(Get-Content -LiteralPath $calls).Count -ne 1) { throw 'Selected command did not run exactly once.' }
$result = & $runner -RepositoryRoot $repository -QueuePath $queuePath -PlanPath $planPath -StateRoot (Join-Path $temporary 'runs') -Start
if ($result.tasks[1].status -ne 'checks-passed' -or @(Get-Content -LiteralPath $calls).Count -ne 1) { throw 'Verified restart repeated completed work.' }
Write-Output "Runner guarded execution/restart checks passed; evidence: $temporary"

# A finished checkpoint must reject new tracked or untracked source content.
'changed source' | Set-Content -LiteralPath (Join-Path $repository 'source.txt')
$result = & $runner -RepositoryRoot $repository -QueuePath $queuePath -PlanPath $planPath -StateRoot (Join-Path $temporary 'runs') -Start
if ($result.tasks[1].reason -ne 'source-changed' -or @(Get-Content -LiteralPath $calls).Count -ne 1) { throw 'Restart ignored changed tracked source.' }
'fixture source' | Set-Content -LiteralPath (Join-Path $repository 'source.txt')
'new source' | Set-Content -LiteralPath (Join-Path $repository 'new-source.txt')
$result = & $runner -RepositoryRoot $repository -QueuePath $queuePath -PlanPath $planPath -StateRoot (Join-Path $temporary 'runs') -Start
if ($result.tasks[1].reason -ne 'source-changed') { throw 'Restart ignored untracked source.' }
# Preserve the new source; a fresh clean fixture workspace is selected below.
& git -C $repository add source.txt new-source.txt
& git -C $repository commit -q -m 'Preserve additional fixture source'
$plan.workspaces.desktop.commit=(& git -C $repository rev-parse HEAD).Trim()

function Select-Scenario([string]$RunId, [string]$Operation, [switch]$Artifact) {
    "# Fixture 2`nStatus: open`nReadiness: ready-for-agent`nBlocked by: none`n" | Set-Content -LiteralPath (Join-Path $tracker '2-fixture.md')
    "# Fixture 4`nStatus: open`nReadiness: ready-for-agent`nBlocked by: none`n" | Set-Content -LiteralPath (Join-Path $tracker '4-fixture.md')
    $script:queue.tasks=@(@{id=2;ticket='2-fixture.md';lane='desktop';blocked_by=@()},@{id=4;ticket='4-fixture.md';lane='desktop';blocked_by=@()})
    $script:queue | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $queuePath
    $script:plan.run_id=$RunId
    $first=@{kind='fixture';operation=$Operation;evidence_directory=(Join-Path $temporary $RunId)}
    if ($Artifact) { $first.artifacts=@($preserved) }
    $script:plan.steps=@(@{ticket_id=2;commands=@($first);risks=@('Fixture only')},@{ticket_id=4;commands=@(@{kind='fixture';operation='pass';evidence_directory=(Join-Path $temporary ($RunId+'-later'))});risks=@()})
    $script:plan | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $planPath
}
function Run-Scenario { return & $runner -RepositoryRoot $repository -QueuePath $queuePath -PlanPath $planPath -StateRoot (Join-Path $temporary 'runs') -Start }

Select-Scenario 'failed-test' 'fail'
$result=Run-Scenario
if (-not $result.stopped -or $result.tasks[0].status -ne 'failed' -or (Test-Path -LiteralPath (Join-Path $temporary 'failed-test-later'))) { throw 'A failed test did not stop subsequent tickets.' }
$result=Run-Scenario
if (-not $result.stopped -or $result.tasks[0].reason -notin @('interrupted-or-failed','previous-command-failed') -or (Test-Path -LiteralPath (Join-Path $temporary 'failed-test-later'))) { throw 'Restart skipped over a known failed test.' }

$preserved=Join-Path $temporary 'last-good.bin'
'last good artifact' | Set-Content -LiteralPath $preserved
$preservedHash=(Get-FileHash -LiteralPath $preserved).Hash
Select-Scenario 'memory-refusal' 'memory-refusal' -Artifact
$result=Run-Scenario
if ($result.tasks[0].reason -ne 'low-memory-before-start' -or $result.tasks[1].status -ne 'checks-passed' -or (Test-Path -LiteralPath (Join-Path $temporary 'memory-refusal/calls.txt'))) { throw 'Memory refusal did not skip to independent bounded work.' }
$checkpoint=Get-Content -LiteralPath (Join-Path $temporary 'runs/memory-refusal/ticket-2.json') -Raw | ConvertFrom-Json
if ($checkpoint.commands[0].attempts -ne 1 -or (Get-FileHash -LiteralPath $preserved).Hash -ne $preservedHash -or (Get-FileHash -LiteralPath $checkpoint.commands[0].artifacts_before[0].backup_path).Hash -ne $preservedHash) { throw 'Refused work lost its checkpoint or last-good artifact.' }
$result=Run-Scenario
$checkpoint=Get-Content -LiteralPath (Join-Path $temporary 'runs/memory-refusal/ticket-2.json') -Raw | ConvertFrom-Json
if ($result.tasks[0].reason -ne 'awaiting-resource-recovery' -or $checkpoint.commands[0].attempts -ne 1) { throw 'A resource retry was consumed before memory recovered.' }

Select-Scenario 'retry-limit' 'wait'
$result=Run-Scenario
$result=Run-Scenario
$result=Run-Scenario
$checkpoint=Get-Content -LiteralPath (Join-Path $temporary 'runs/retry-limit/ticket-2.json') -Raw | ConvertFrom-Json
if ($result.tasks[0].reason -ne 'resource-retry-exhausted' -or $checkpoint.commands[0].attempts -ne 2 -or @(Get-Content -LiteralPath (Join-Path $temporary 'retry-limit/calls.txt')).Count -ne 2) { throw 'Timeout retried more than once.' }
Write-Output "Runner source/failure/resource/retry/artifact checks passed; evidence: $temporary"

Select-Scenario 'validation' 'pass'
$plan.workspaces.desktop.branch='codex/wrong-branch'
$plan | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $planPath
$result=& $runner -RepositoryRoot $repository -QueuePath $queuePath -PlanPath $planPath -StateRoot (Join-Path $temporary 'runs')
if ($result.tasks[0].reason -ne 'workspace-changed') { throw 'Wrong branch was not rejected.' }
$plan.workspaces.desktop.branch='development'
$plan.workspaces.desktop.commit='0000000000000000000000000000000000000000'
$plan | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $planPath
$result=& $runner -RepositoryRoot $repository -QueuePath $queuePath -PlanPath $planPath -StateRoot (Join-Path $temporary 'runs')
if ($result.tasks[0].reason -ne 'workspace-changed') { throw 'Wrong source commit was not rejected.' }
$plan.workspaces.desktop.commit=(& git -C $repository rev-parse HEAD).Trim()
"# Fixture 2`nStatus: claimed`nReadiness: ready-for-agent`nAssignee: overnight/validation`nBlocked by: none`n" | Set-Content -LiteralPath (Join-Path $tracker '2-fixture.md')
$plan | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $planPath
$result=& $runner -RepositoryRoot $repository -QueuePath $queuePath -PlanPath $planPath -StateRoot (Join-Path $temporary 'runs')
if ($result.tasks[0].reason -ne 'stale-claim') { throw 'A stale claim without a checkpoint was taken over.' }
Select-Scenario 'validation' 'pass'
$cargo=(Get-Command cargo -CommandType Application | Select-Object -First 1).Source
$settings=& (Join-Path $PSScriptRoot 'local-build-settings.ps1')
$command=@{kind='lightweight-check';file_path=$cargo;sha256=(Get-FileHash -LiteralPath $cargo).Hash;arguments=@('build')+$settings.CargoArguments}
$plan.steps[0].commands=@($command)
$plan | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $planPath
$result=& $runner -RepositoryRoot $repository -QueuePath $queuePath -PlanPath $planPath -StateRoot (Join-Path $temporary 'runs')
if ($result.tasks[0].reason -ne 'review-required' -or $result.tasks[0].next_action -notmatch 'guarded heavy') { throw 'Cargo accepted a lightweight label.' }
$command.kind='build'
$command.arguments=@('build','--locked','--offline','-j','1','--target-dir',$settings.TargetDir,'--config',(Join-Path $repository 'scripts/gui-dev.toml'),'-j16')
$plan | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $planPath
$result=& $runner -RepositoryRoot $repository -QueuePath $queuePath -PlanPath $planPath -StateRoot (Join-Path $temporary 'runs')
if ($result.tasks[0].reason -ne 'review-required' -or $result.tasks[0].next_action -notmatch 'overrides') { throw 'A later Cargo job override bypassed the shared policy.' }
Select-Scenario 'exclusive' 'pass'
$shell=(Get-Process -Id $PID).Path
$lockEvidence=Join-Path $temporary 'exclusive-lock'
$helper=Join-Path $PSScriptRoot 'fixtures/overnight-command.ps1'
$controller=Start-Process -FilePath $shell -ArgumentList @('-NoProfile','-File',('"'+$helper+'"'),'-Operation','hold-coordinator','-EvidenceDirectory',('"'+$lockEvidence+'"')) -WindowStyle Hidden -PassThru
try {
    $timer=[Diagnostics.Stopwatch]::StartNew()
    while (-not (Test-Path -LiteralPath (Join-Path $lockEvidence 'ready.txt'))) {
        if ($controller.HasExited -or $timer.Elapsed.TotalSeconds -gt 10) { throw 'Exclusive-lock fixture did not start.' }
        Start-Sleep -Milliseconds 100
    }
    $rejected=$false
    try { $null=Run-Scenario } catch { if ($_.Exception.Message -match 'Another coordinator owns') { $rejected=$true } else { throw } }
    if (-not $rejected -or (Test-Path -LiteralPath (Join-Path $temporary 'runs/exclusive')) -or (Get-Content -LiteralPath (Join-Path $tracker '2-fixture.md') -Raw) -notmatch 'Status: open') { throw 'Competing coordinator changed state or claimed work.' }
} finally { if (-not $controller.HasExited) {$controller.Kill();$controller.WaitForExit()};$controller.Dispose() }
Write-Output "Runner branch/commit/stale-claim/policy/exclusive checks passed; evidence: $temporary"
