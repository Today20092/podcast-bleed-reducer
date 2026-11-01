#requires -Version 7.0
param(
    [Parameter(Mandatory)][string]$PlanPath,
    [string]$RepositoryRoot = (Split-Path -Parent $PSScriptRoot),
    [string]$QueuePath,
    [string]$StateRoot,
    [switch]$Start
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$RepositoryRoot = (Resolve-Path -LiteralPath $RepositoryRoot).Path
if (-not $QueuePath) { $QueuePath = Join-Path $RepositoryRoot '.scratch/plugin-products/overnight-queue.json' }
$queue = Get-Content -LiteralPath $QueuePath -Raw | ConvertFrom-Json -AsHashtable
$plan = Get-Content -LiteralPath $PlanPath -Raw | ConvertFrom-Json -AsHashtable
if ($queue.schema_version -ne 1 -or $plan.schema_version -ne 1 -or $queue.max_active_tickets -ne 1 -or $queue.max_heavy_process_trees -ne 1) { throw 'Only the sequential schema-1 queue is supported.' }
if ($plan.run_id -notmatch '^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$') { throw 'A safe, explicit run_id is required.' }
if (-not $StateRoot) { $StateRoot = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'CleanSlate/OvernightRuns' }
$runDirectory = Join-Path ([IO.Path]::GetFullPath($StateRoot)) $plan.run_id
foreach ($workspacePath in @($RepositoryRoot) + @($plan.workspaces.Values | ForEach-Object { [IO.Path]::GetFullPath($_.path) })) {
    if ($runDirectory -eq $workspacePath -or $runDirectory.StartsWith($workspacePath + [IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) { throw 'Execution state must live outside source workspaces.' }
}
$tracker = [IO.Path]::GetFullPath((Join-Path $RepositoryRoot $queue.canonical_tracker))
if (-not $tracker.StartsWith($RepositoryRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'The canonical tracker must stay inside the repository.' }
$guard = Join-Path $PSScriptRoot 'resource-guard.ps1'
$owner = 'overnight/' + $plan.run_id
$summary = [ordered]@{run_id=$plan.run_id;mode=$(if ($Start) {'execution'} else {'dry-run'});tasks=@();handoff_path=$null;stopped=$false}
function Write-Atomic([string]$Path, [string]$Text) {
    $temporary = $Path + '.' + [Guid]::NewGuid().ToString('N') + '.tmp'
    [IO.File]::WriteAllText($temporary, $Text, [Text.UTF8Encoding]::new($false))
    if ([IO.File]::Exists($Path)) { [IO.File]::Replace($temporary,$Path,[NullString]::Value) } else { [IO.File]::Move($temporary,$Path) }
}
function Save-State([string]$Path, $Value) { Write-Atomic $Path ($Value | ConvertTo-Json -Depth 20) }
function Hash-Text([string]$Text) {
    $hash = [Security.Cryptography.SHA256]::Create()
    try { return [BitConverter]::ToString($hash.ComputeHash([Text.Encoding]::UTF8.GetBytes($Text))).Replace('-','').ToLowerInvariant() }
    finally { $hash.Dispose() }
}
function File-Hash([string]$Path) { return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() }
function Git-Text([string]$Path, [string[]]$Arguments) {
    $output = & git -C $Path @Arguments 2>&1
    if ($LASTEXITCODE -ne 0) { throw "Git inspection failed in ${Path}: $output" }
    return ($output -join "`n")
}
function Source-State([string]$Path) {
    $diff = Git-Text $Path @('diff','--no-ext-diff','--binary','HEAD','--','.',':(exclude).scratch/offline-supervised-editor/issues/**')
    $untracked = Git-Text $Path @('-c','core.quotepath=false','ls-files','--others','--exclude-standard','--','.',':(exclude).scratch/offline-supervised-editor/issues/**')
    $extra = @()
    if ($untracked) { $extra = @($untracked -split "`n" | ForEach-Object { $_ + ':' + (File-Hash (Join-Path $Path $_)) }) }
    return @{commit=(Git-Text $Path @('rev-parse','HEAD'));branch=(Git-Text $Path @('branch','--show-current'));fingerprint=(Hash-Text ($diff + "`n" + ($extra -join "`n")));dirty=[bool]($diff -or $untracked);edits=(Git-Text $Path @('status','--short','--untracked-files=all','--','.',':(exclude).scratch/offline-supervised-editor/issues/**'))}
}
function Review-State([string]$Path) {
    return @{source=(Source-State $Path);plan_sha256=(File-Hash $PlanPath);queue_sha256=(File-Hash $QueuePath)}
}
function Same-Review($Expected, $Observed) {
    return $Expected.source.commit -eq $Observed.source.commit -and $Expected.source.branch -eq $Observed.source.branch -and $Expected.source.fingerprint -eq $Observed.source.fingerprint -and $Expected.plan_sha256 -eq $Observed.plan_sha256 -and $Expected.queue_sha256 -eq $Observed.queue_sha256
}
function Read-Ticket([int]$Id) {
    $files = @(Get-ChildItem -LiteralPath $tracker -Filter '*.md' | Where-Object { $_.Name -match ('^0?' + $Id + '-') })
    if ($files.Count -ne 1) { throw "Ticket $Id must resolve to exactly one canonical file." }
    $text = [IO.File]::ReadAllText($files[0].FullName)
    $dependencies = [regex]::Match($text,'(?m)^Blocked by:\s*([^\r\n]*)').Groups[1].Value.Trim()
    if ($dependencies -and $dependencies -ne 'none' -and $dependencies -notmatch '^\d+(?:\s*,\s*\d+)*$') { throw "Ticket $Id has an ambiguous dependency list." }
    return @{path=$files[0].FullName;text=$text;status=([regex]::Match($text,'(?m)^Status:\s*([^\r\n]*)').Groups[1].Value.Trim());assignee=([regex]::Match($text,'(?m)^Assignee:\s*([^\r\n]*)').Groups[1].Value.Trim());ready=[regex]::IsMatch($text,'(?m)^(?:Implementation readiness|Readiness):\s*ready-for-agent\s*\r?$');dependencies=@(if ($dependencies -and $dependencies -ne 'none') { $dependencies -split '\s*,\s*' | ForEach-Object { [int]$_ } })}
}
function Command-Spec($Command, [string]$Workspace) {
    if ($Command.ContainsKey('policy')) { throw 'Caller-selected execution policies are not accepted by the coordinator.' }
    # Disposable fixtures have a fixed shipped implementation, never caller-provided shell code.
    if ($Command.kind -eq 'fixture') {
        if ($Command.operation -notin @('pass','fail','wait','mutate-tracked','mutate-untracked','mutate-branch','mutate-head','mutate-external-input')) { throw 'Unknown disposable fixture operation.' }
        $fixture = Join-Path $PSScriptRoot 'fixtures/overnight-command.ps1'
        return @{file=(Get-Process -Id $PID).Path;arguments=@('-NoProfile','-File',$fixture,'-Operation',$Command.operation,'-EvidenceDirectory',$Command.evidence_directory);policy=@{TimeoutSeconds=5};inputs=@(@{path=$fixture;sha256=(File-Hash $fixture)})}
    }
    if ($Command.kind -notin @('build','check','lightweight-check')) { throw 'Unsupported action: native hosts, private signing and shared REA require a separate attended lane.' }
    $file = (Get-Command $Command.file_path -CommandType Application | Select-Object -First 1).Source
    if ($Command.sha256 -notmatch '^[0-9a-fA-F]{64}$' -or (File-Hash $file) -ne $Command.sha256.ToLowerInvariant()) { throw 'The command executable must match its reviewed SHA-256.' }
    $name = [IO.Path]::GetFileNameWithoutExtension($file)
    $arguments = @($Command.arguments)
    if (@($arguments | Where-Object { $_ -isnot [string] }).Count) { throw 'Command arguments must be an explicit array of strings.' }
    $inputs = @($Command['inputs'] | Where-Object { $null -ne $_ })
    $policy = @{TimeoutSeconds=1800}
    if ($name -in @('pwsh','powershell')) {
        if ($arguments.Count -lt 3 -or $arguments[0] -ne '-NoProfile' -or $arguments[1] -ne '-File') { throw 'Shell command strings are unsupported; select a pinned script with -NoProfile -File.' }
        $script = (Resolve-Path -LiteralPath $arguments[2]).Path
        $relative = [IO.Path]::GetRelativePath($Workspace,$script).Replace('\','/')
        if ($relative -notin @('scripts/check.ps1','scripts/gui-dev.ps1','scripts/test-plugin-hosts.ps1')) { throw 'This script has no unattended execution contract; leave it for agent review.' }
        if ($relative -eq 'scripts/check.ps1' -and '-Local' -notin $arguments) { throw 'Local checks must use -Local.' }
        if ($relative -eq 'scripts/gui-dev.ps1' -and '-BuildOnly' -notin $arguments) { throw 'The coordinator never launches the GUI.' }
        if (-not @($inputs | Where-Object { [IO.Path]::GetFullPath($_.path) -eq $script }).Count) { throw 'Pin the selected script in inputs.' }
        if ($Command.kind -eq 'lightweight-check' -and $relative -ne 'scripts/test-plugin-hosts.ps1') { throw 'Build/check scripts require a build/check action kind.' }
    } elseif ($name -eq 'cargo') {
        if ($Command.kind -eq 'lightweight-check' -or $arguments.Count -lt 1 -or $arguments[0] -notin @('build','test','clippy','fmt')) { throw 'Cargo only supports guarded heavy build/test/clippy/fmt commands.' }
        if ($arguments[0] -ne 'fmt') {
            $settings = & (Join-Path $PSScriptRoot 'local-build-settings.ps1')
            foreach ($flag in @('--locked','--offline')) { if ($flag -notin $arguments) { throw "Cargo requires $flag." } }
            foreach ($pair in @(@('-j','1'),@('--target-dir',$settings.TargetDir),@('--config',(Join-Path $Workspace 'scripts/gui-dev.toml')))) {
                $index = [Array]::IndexOf($arguments,$pair[0])
                if (@($arguments | Where-Object { $_ -eq $pair[0] }).Count -ne 1 -or $index+1 -ge $arguments.Count -or $arguments[$index+1] -ne $pair[1]) { throw "Cargo must preserve shared local setting $($pair[0])." }
            }
            if (@($arguments | Where-Object { $_ -match '^(?:-j.+|--jobs(?:=|$)|--config=|--target-dir=|--release$|-r$|--profile(?:=|$))' }).Count) { throw 'Cargo job/profile/config overrides are forbidden in the unattended queue.' }
        }
    } elseif ($name -eq 'rust-app') {
        if (($arguments -join '|') -ne '--package-self-check') { throw 'Only the existing executable package self-check is unattended.' }
    } else { throw 'Unsupported executable; use the pinned plugin-host runner for its isolated child cache/PASS contract.' }
    if ($Command.kind -eq 'lightweight-check') { $policy=@{TimeoutSeconds=90} }
    foreach ($inputFile in $inputs) {
        if (-not $inputFile -or (File-Hash $inputFile.path) -ne $inputFile.sha256) { throw 'A reviewed input artifact/script changed or is missing.' }
    }
    return @{file=$file;arguments=$arguments;policy=$policy;inputs=$inputs}
}
function Artifact-State($Paths, [string]$BackupDirectory) {
    $result = @()
    if (-not $Paths) { return ,$result }
    foreach ($path in @($Paths)) {
        $full = [IO.Path]::GetFullPath($path)
        if (-not (Test-Path -LiteralPath $full -PathType Leaf)) { $result += @{path=$full;sha256=$null;backup_path=$null}; continue }
        $hash = File-Hash $full
        $backup = $null
        if ($BackupDirectory) {
            New-Item -ItemType Directory -Path $BackupDirectory -Force | Out-Null
            $backup = Join-Path $BackupDirectory ($hash + '-' + [IO.Path]::GetFileName($full))
            if (-not (Test-Path -LiteralPath $backup)) { [IO.File]::Copy($full,$backup,$false) }
            if ((File-Hash $backup) -ne $hash) { throw 'Last-good artifact backup verification failed.' }
        }
        $result += @{path=$full;sha256=$hash;backup_path=$backup}
    }
    return ,$result
}
$mutex=$null; $locked=$false
try {
    $stepIds=@($plan.steps | ForEach-Object { $_.ticket_id })
    if (@($queue.tasks.id | Select-Object -Unique).Count -ne @($queue.tasks).Count -or @($stepIds | Select-Object -Unique).Count -ne @($plan.steps).Count) { throw 'Duplicate queue tickets or execution steps.' }
    if (@($plan.steps | Where-Object { $_.ticket_id -notin $queue.tasks.id }).Count) { throw 'Execution steps must reference queue tickets.' }
    if ($Start) {
        if ([Environment]::OSVersion.Platform -ne 'Win32NT') { throw 'Execution requires the Windows resource guard; dry-run remains read-only.' }
        $mutex=[Threading.Mutex]::new($false,'Global\CleanSlate.OvernightCoordinator.v1')
        try { $locked=$mutex.WaitOne(0) } catch [Threading.AbandonedMutexException] { $locked=$true }
        if (-not $locked) { throw 'Another coordinator owns the sequential queue; no claims or commands were started.' }
        foreach ($dependency in $queue.requires_before_unattended_execution) { if ((Read-Ticket $dependency).status -ne 'done') { throw "Coordinator prerequisite $dependency is not done." } }
        New-Item -ItemType Directory -Path $runDirectory -Force | Out-Null
        $identity=@{schema_version=1;repository_root=$RepositoryRoot;plan_sha256=(File-Hash $PlanPath);queue_sha256=(File-Hash $QueuePath)}
        $identityPath=Join-Path $runDirectory 'identity.json'
        if (Test-Path -LiteralPath $identityPath) {
            $existing=Get-Content -LiteralPath $identityPath -Raw | ConvertFrom-Json -AsHashtable
            foreach ($key in $identity.Keys) { if ($existing[$key] -ne $identity[$key]) { throw 'Run identity changed; select a new run_id after review.' } }
        } else { Save-State $identityPath $identity }
        # Persisted failed or unknown command outcomes stop the whole run, even if a later claim changed.
        foreach ($file in @(Get-ChildItem -LiteralPath $runDirectory -Filter 'ticket-*.json' | Where-Object Name -match '^ticket-\d+\.json$')) {
            $prior=Get-Content -LiteralPath $file.FullName -Raw | ConvertFrom-Json -AsHashtable
            if ($prior.status -in @('running','failed','review-required')) { $summary.stopped=$true }
        }
    }
    foreach ($task in $queue.tasks) {
        $entry=[ordered]@{ticket_id=$task.id;status='blocked';reason=$null;next_action=$null;checkpoint_path=$null}
        $summary.tasks+=,$entry
        if ($summary.stopped) { $entry.reason='previous-command-failed'; continue }
        $executionStarted=$false
        try {
            $ticket=Read-Ticket $task.id
            if ([IO.Path]::GetFileName($ticket.path) -ne $task.ticket) { throw 'Queue ticket path does not match the canonical ticket.' }
            if ($ticket.status -eq 'done') { $entry.status='done';$entry.reason='already-done';continue }
            if ($task.id -in $queue.non_executable_umbrellas) { $entry.reason='coordination-umbrella';continue }
            if (($ticket.dependencies -join ',') -ne (@($task.blocked_by) -join ',')) { throw 'Queue and canonical ticket dependencies differ.' }
            $blocked=@($ticket.dependencies | Where-Object { (Read-Ticket $_).status -ne 'done' })
            if ($blocked.Count) { $entry.reason='dependencies';$entry.next_action='Complete canonical dependencies: ' + ($blocked -join ', ');continue }
            $checkpointPath=Join-Path $runDirectory ("ticket-$($task.id).json")
            $checkpoint=if (Test-Path -LiteralPath $checkpointPath) { Get-Content -LiteralPath $checkpointPath -Raw | ConvertFrom-Json -AsHashtable } else { $null }
            if ($ticket.status -eq 'claimed' -and $ticket.assignee -ne $owner) { $entry.reason='claimed-other';$entry.next_action='Existing claims require their owner to hand off; never auto-steal stale claims.';continue }
            if ($ticket.status -eq 'claimed' -and -not $checkpoint) { $entry.reason='stale-claim';$entry.next_action='An owned claim without a checkpoint needs manual reconciliation.';continue }
            if ($ticket.status -notin @('open','claimed') -or -not $ticket.ready) { $entry.reason='not-agent-ready';continue }
            $step=@($plan.steps | Where-Object ticket_id -eq $task.id)
            if ($step.Count -ne 1 -or -not @($step[0]['commands'] | Where-Object { $null -ne $_ }).Count) { $entry.reason='agent-action-required';$entry.next_action='Implement/review this ticket in the active agent; supply explicit checked commands afterward.';continue }
            $step=$step[0]
            $workspace=$plan.workspaces[$task.lane]
            if (-not $workspace -or $workspace.commit -notmatch '^[0-9a-fA-F]{40}$') { throw 'Select an existing workspace, exact branch and full commit for this lane.' }
            $path=(Resolve-Path -LiteralPath $workspace.path).Path
            if ($task.lane -ne 'desktop') {
                $localQueue=Join-Path $path '.scratch/plugin-products/overnight-queue.json'
                if (-not (Test-Path -LiteralPath $localQueue) -or (File-Hash $localQueue) -ne (File-Hash $QueuePath)) { throw 'Product workspace queue revision differs; checkpoint/integrate the planning revision explicitly.' }
            }
            $source=Source-State $path
            if ($source.commit -ne $workspace.commit -or $source.branch -ne $workspace.branch) { $entry.reason='workspace-changed';$entry.next_action='Review the selected branch/commit; no checkout, reset or merge is automatic.';continue }
            if ($checkpoint) {
                if ($checkpoint.ticket_sha256 -ne (Hash-Text $ticket.text)) { throw 'The claimed ticket changed since its checkpoint; review its acceptance before resuming.' }
                foreach ($dependency in $checkpoint.dependency_hashes.Keys) { if ($checkpoint.dependency_hashes[$dependency] -ne (Hash-Text (Read-Ticket ([int]$dependency)).text)) { throw 'A prerequisite ticket changed since the checkpoint.' } }
                if ($checkpoint.source.fingerprint -ne $source.fingerprint) { $entry.reason='source-changed';$entry.next_action='Tracked or untracked source changed since the checkpoint; review before resuming.';continue }
            } elseif ($source.dirty) { $entry.reason='dirty-workspace';$entry.next_action='Preserve/review source edits before selecting this baseline.';continue }
            $specs=@($step.commands | ForEach-Object { Command-Spec $_ $path })
            if (-not $Start) { $entry.status='ready';$entry.reason='explicit-command-selected';continue }
            $entry.checkpoint_path=$checkpointPath
            if (-not $checkpoint) {
                $dependencyHashes=@{}
                foreach ($dependency in $ticket.dependencies) { $dependencyHashes["$dependency"]=Hash-Text (Read-Ticket $dependency).text }
                $checkpoint=@{schema_version=1;ticket_id=$task.id;owner=$owner;status='claiming';source=$source;plan_sha256=(File-Hash $PlanPath);queue_sha256=(File-Hash $QueuePath);ticket_sha256=$null;dependency_hashes=$dependencyHashes;risks=@($step['risks']);commands=@();next_action='Acquire canonical claim before commands.';updated_utc=[DateTime]::UtcNow.ToString('o')}
                Save-State $checkpointPath $checkpoint
                if ([IO.File]::ReadAllText($ticket.path) -ne $ticket.text) { throw 'Canonical ticket changed while preparing its claim.' }
                $claimed=[regex]::Replace($ticket.text,'(?m)^Status:\s*open\s*\r?$','Status: claimed')
                if ([regex]::IsMatch($claimed,'(?m)^Assignee:')) { $claimed=[regex]::Replace($claimed,'(?m)^Assignee:[^\r\n]*',"Assignee: $owner") } else { $claimed=[regex]::Replace($claimed,'(?m)^Status: claimed',"Status: claimed`nAssignee: $owner") }
                Write-Atomic $ticket.path $claimed
                $checkpoint.ticket_sha256=Hash-Text $claimed
                $checkpoint.status='claimed';$checkpoint.next_action='Execute selected guarded command.';Save-State $checkpointPath $checkpoint
            }
            $entry.status='checks-passed';$entry.reason='acceptance-review-required'
            for ($i=0;$i -lt $specs.Count;$i++) {
                $command=$step.commands[$i];$spec=$specs[$i]
                $record=if ($checkpoint.commands.Count -gt $i) { $checkpoint.commands[$i] } else { $null }
                if ($record) { $executionStarted=$true }
                if ($record -and $record.status -eq 'passed') {
                    foreach ($artifact in $record.artifacts_after) { if (-not $artifact.sha256 -or (File-Hash $artifact.path) -ne $artifact.sha256) { throw 'A completed command artifact changed; do not repeat it blindly.' } }
                    continue
                }
                if ($record -and $record.status -notin @('resource-blocked')) { $entry.status='blocked';$entry.reason='interrupted-or-failed';$entry.next_action='Review interrupted/failed work and its receipt; never automatically repeat an unknown outcome.';$summary.stopped=$true;break }
                if ($record -and $record.attempts -ge 2) { $entry.status='resource-blocked';$entry.reason='resource-retry-exhausted';$entry.next_action='Review the checkpoint; two timeout/competing-task attempts are the maximum.';break }
                $attempt=if ($record) { $record.attempts+1 } else { 1 }
                $receiptPath=Join-Path $runDirectory ("ticket-$($task.id)-command-$i-attempt-$attempt.json")
                $record=@{index=$i;status='running';attempts=$attempt;started_utc=[DateTime]::UtcNow.ToString('o');finished_utc=$null;execution=$spec;command_sha256=(Hash-Text ($command | ConvertTo-Json -Depth 10 -Compress));receipt_path=$receiptPath;log_path=[IO.Path]::ChangeExtension($receiptPath,'.log');before_source=$null;before_review=$null;after_review=$null;artifacts_before=(Artifact-State $command['artifacts'] (Join-Path $runDirectory "last-good/$($task.id)/$i"));artifacts_after=@();metrics=$null;error=$null}
                if ($checkpoint.commands.Count -gt $i) { $checkpoint.commands[$i]=$record } else { $checkpoint.commands+=,$record }
                $record.before_review=Review-State $path
                $record.before_source=$record.before_review.source
                if (-not (Same-Review $checkpoint $record.before_review)) {
                    $record.status='review-required';$checkpoint.status='review-required'
                    $entry.status='blocked';$entry.reason='reviewed-identity-changed';$entry.next_action='Review observed source/plan changes; the selected baseline is unchanged and no command was launched.'
                    $checkpoint.next_action=$entry.next_action;$checkpoint.updated_utc=[DateTime]::UtcNow.ToString('o');$summary.stopped=$true
                    Save-State $checkpointPath $checkpoint
                    break
                }
                try { $spec=Command-Spec $command $path; $record.execution=$spec }
                catch {
                    $record.error=$_.Exception.Message;$record.status='review-required';$checkpoint.status='review-required'
                    $entry.status='blocked';$entry.reason='command-identity-changed';$entry.next_action='Review the rejected executable/input identity; the command was not launched.'
                    $checkpoint.next_action=$entry.next_action;$checkpoint.updated_utc=[DateTime]::UtcNow.ToString('o');$summary.stopped=$true
                    Save-State $checkpointPath $checkpoint
                    break
                }
                $checkpoint.status='running';$checkpoint.next_action='An interrupted command requires receipt/source/artifact review before resumption.';Save-State $checkpointPath $checkpoint
                $failure=$null
                $policy=$spec.policy
                $executionStarted=$true
                try { & $guard -FilePath $spec.file -ArgumentList $spec.arguments -WorkingDirectory $path -ReceiptPath $receiptPath @policy }
                catch { $failure=$_.Exception.Message }
                $metrics=if (Test-Path -LiteralPath $receiptPath) { Get-Content -LiteralPath $receiptPath -Raw | ConvertFrom-Json -AsHashtable } else { $null }
                $record.metrics=$metrics;$record.error=$failure
                $record.finished_utc=[DateTime]::UtcNow.ToString('o')
                $record.artifacts_after=Artifact-State $command['artifacts'] ''
                $record.after_review=Review-State $path
                if (-not (Same-Review $checkpoint $record.after_review)) {
                    $record.status='review-required';$checkpoint.status='review-required'
                    $entry.status='blocked';$entry.reason='reviewed-identity-changed';$entry.next_action='Review the observed post-command source/plan changes; they are evidence, never a new accepted baseline.'
                    $checkpoint.next_action=$entry.next_action;$summary.stopped=$true
                } elseif ($metrics -and $metrics.status -eq 'done' -and $metrics.cleanup_verified -and -not $failure -and -not @($record.artifacts_after | Where-Object { -not $_.sha256 }).Count) {
                    $record.status='passed';$checkpoint.status='checks-passed';$checkpoint.next_action='Review ticket acceptance; command success never marks the ticket done.'
                } elseif ($metrics -and $metrics.status -eq 'resource-blocked' -and $metrics.cleanup_verified) {
                    $record.status='resource-blocked';$entry.status='resource-blocked';$entry.reason=$metrics.reason;$entry.next_action='Review the stopped or competing task before resuming the same plan; at most one retry.';$checkpoint.status='resource-blocked';$checkpoint.next_action=$entry.next_action
                } else {
                    $record.status='failed';$entry.status='failed';$entry.reason='command-or-check-failed';$entry.next_action='Fix/review the failed check before running subsequent tickets.';$checkpoint.status='failed';$checkpoint.next_action=$entry.next_action;$summary.stopped=$true
                }
                $checkpoint.updated_utc=[DateTime]::UtcNow.ToString('o');Save-State $checkpointPath $checkpoint
                if ($record.status -ne 'passed') { break }
            }
        } catch { $entry.status='blocked';$entry.reason='review-required';$entry.next_action=$_.Exception.Message;if ($executionStarted) { $summary.stopped=$true } }
    }
    if ($Start) {
        $handoff=Join-Path $runDirectory 'handoff.md'
        $lines=@("# Queue handoff: $($plan.run_id)",'',"Generated $([DateTime]::UtcNow.ToString('o')). Commands run sequentially; no ticket is closed automatically.",'','| Ticket | State | Reason / next action |','| --- | --- | --- |')
        foreach ($entry in $summary.tasks) { $lines+="| $($entry.ticket_id) | $($entry.status) | $(($entry.reason + '; ' + $entry.next_action).Replace('|','/').Replace("`n",' ')) |" }
        $lines+=@('','Per-ticket JSON records retain source edits, artifact hashes/backups, command receipts, logs, memory/time and risks.','The host chat cannot restart itself after cancellation or process/context loss. Resume this explicit command in a supported session; scheduling requires a separate owner request.','Native/listening/signing and unresolved acceptance remain separate from mechanical passes.')
        Write-Atomic $handoff ($lines -join "`n")
        $summary.handoff_path=$handoff
        Save-State (Join-Path $runDirectory 'run.json') $summary
    }
    return [pscustomobject]$summary
} finally {
    if ($locked) { $mutex.ReleaseMutex() }
    if ($mutex) { $mutex.Dispose() }
}
