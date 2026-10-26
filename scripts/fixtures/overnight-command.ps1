param(
    [ValidateSet('pass','fail','wait','hold-coordinator','mutate-tracked','mutate-untracked','mutate-branch','mutate-head','mutate-external-input')][string]$Operation,
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
if ($Operation -like 'mutate-*') {
    $root=[IO.Path]::GetFullPath((Get-Location).Path)
    $marker=Join-Path $root '.cleanslate-runner-source-fixture'
    if (-not $root.StartsWith($temporary,[StringComparison]::OrdinalIgnoreCase) -or -not (Test-Path -LiteralPath $marker) -or ([IO.File]::ReadAllText($marker)).Trim() -ne 'disposable-source-review-fixture-v1') { throw 'Source mutation requires an explicitly marked disposable temporary repository.' }
    switch ($Operation) {
        'mutate-tracked' { Add-Content -LiteralPath (Join-Path $root 'source.txt') -Value 'unexpected tracked source change' }
        'mutate-untracked' { [IO.File]::WriteAllText((Join-Path $root 'new-source.txt'),'unexpected untracked source') }
        'mutate-branch' { & git -C $root switch -q -c unexpected-fixture-branch; if ($LASTEXITCODE -ne 0) { throw 'Fixture branch change failed.' } }
        'mutate-head' { & git -C $root commit --allow-empty -q -m 'Unexpected fixture commit'; if ($LASTEXITCODE -ne 0) { throw 'Fixture commit change failed.' } }
        'mutate-external-input' { [IO.File]::WriteAllText((Join-Path (Split-Path -Parent $root) 'pinned-input.bin'),'unreviewed external input') }
    }
}
if ($Operation -eq 'fail') { throw 'Deliberate disposable check failure.' }
if ($Operation -eq 'wait') { Start-Sleep -Seconds 30 }
Write-Output 'Disposable runner check passed.'
