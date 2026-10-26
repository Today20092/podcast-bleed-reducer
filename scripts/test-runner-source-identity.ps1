$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
$runner=Join-Path $PSScriptRoot 'overnight-runner.ps1'
$temporary=Join-Path ([IO.Path]::GetTempPath()) ('cleanslate-source-review-'+[Guid]::NewGuid())
foreach ($operation in @('mutate-tracked','mutate-untracked','mutate-branch','mutate-head')) {
    $scenario=Join-Path $temporary $operation
    $repository=Join-Path $scenario 'repository'
    $tracker=Join-Path $repository '.scratch/offline-supervised-editor/issues'
    New-Item -ItemType Directory -Path $tracker -Force | Out-Null
    & git init -q -b development $repository
    & git -C $repository config user.name 'Runner fixture'
    & git -C $repository config user.email 'runner-fixture@example.invalid'
    & git -C $repository config core.autocrlf false
    foreach ($id in @(2,4,87,88,99)) {
        $status=if ($id -in @(87,88,99)) {'done'} else {'open'}
        "# Fixture $id`nStatus: $status`nReadiness: ready-for-agent`nBlocked by: none`n" | Set-Content -LiteralPath (Join-Path $tracker "$id-fixture.md")
    }
    'fixture source' | Set-Content -LiteralPath (Join-Path $repository 'source.txt')
    'disposable-source-review-fixture-v1' | Set-Content -LiteralPath (Join-Path $repository '.cleanslate-runner-source-fixture')
    & git -C $repository add .
    & git -C $repository commit -q -m 'Reviewed disposable source'
    $commit=(& git -C $repository rev-parse HEAD).Trim()
    $queue=@{schema_version=1;canonical_tracker='.scratch/offline-supervised-editor/issues';max_active_tickets=1;max_heavy_process_trees=1;requires_before_unattended_execution=@(87,88,99);non_executable_umbrellas=@(85,86);tasks=@(@{id=2;ticket='2-fixture.md';lane='desktop';blocked_by=@()},@{id=4;ticket='4-fixture.md';lane='desktop';blocked_by=@()})}
    $plan=@{schema_version=1;run_id='source-review';workspaces=@{desktop=@{path=$repository;branch='development';commit=$commit}};steps=@(@{ticket_id=2;commands=@(@{kind='fixture';operation=$operation;evidence_directory=(Join-Path $scenario 'first')},@{kind='fixture';operation='pass';evidence_directory=(Join-Path $scenario 'second')})},@{ticket_id=4;commands=@(@{kind='fixture';operation='pass';evidence_directory=(Join-Path $scenario 'later')})})}
    $queuePath=Join-Path $scenario 'queue.json'
    $planPath=Join-Path $scenario 'plan.json'
    $stateRoot=Join-Path $scenario 'runs'
    $queue | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $queuePath
    $plan | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $planPath
    $result=& $runner -RepositoryRoot $repository -QueuePath $queuePath -PlanPath $planPath -StateRoot $stateRoot -Start
    if (-not $result.stopped -or (Test-Path -LiteralPath (Join-Path $scenario 'second/calls.txt')) -or (Test-Path -LiteralPath (Join-Path $scenario 'later/calls.txt'))) { throw "Source changed during $operation but subsequent commands executed." }
    $checkpointPath=Join-Path $stateRoot 'source-review/ticket-2.json'
    $checkpoint=Get-Content -LiteralPath $checkpointPath -Raw | ConvertFrom-Json
    if ($checkpoint.status -ne 'review-required' -or $checkpoint.commands.Count -ne 1 -or $checkpoint.commands[0].metrics.status -ne 'done') { throw 'Successful source-changing command lost its result or review requirement.' }
    if ($checkpoint.source.commit -ne $commit -or $checkpoint.source.branch -ne 'development' -or $checkpoint.source.fingerprint -ne $checkpoint.commands[0].before_source.fingerprint -or -not $checkpoint.commands[0].after_review.source) { throw 'Observed source replaced the reviewed baseline or was not preserved as evidence.' }
    $result=& $runner -RepositoryRoot $repository -QueuePath $queuePath -PlanPath $planPath -StateRoot $stateRoot -Start
    if (-not $result.stopped -or @(Get-Content -LiteralPath (Join-Path $scenario 'first/calls.txt')).Count -ne 1 -or (Test-Path -LiteralPath (Join-Path $scenario 'second/calls.txt'))) { throw 'Restart accepted a source-changing command as the reviewed baseline.' }
    Write-Output "Runner $operation identity stop/restart passed."
}
Write-Output "Runner source identity evidence: $temporary"
