param([Parameter(Mandatory)][string]$OldExecutable, [Parameter(Mandatory)][string]$NewExecutable,
    [Parameter(Mandatory)][string]$RuntimePackage, [Parameter(Mandatory)][string]$Minisign,
    [Parameter(Mandatory)][string]$TestSecretKey, [string]$OldVersion = '1.0.0', [string]$NewVersion = '1.0.1',
    [string[]]$Cases = @('success','self-check','gui-crash','gui-timeout','interrupted-swap','mixed','corrupt','locked','permissions'))
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
# Explicit test-key binaries only. Never pass owner projects or an owner signing key.
$root = Join-Path ([IO.Path]::GetTempPath()) ('cleanslate-updater-e2e-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root | Out-Null
$dlls = @('concrt140.dll','msvcp140.dll','msvcp140_1.dll','msvcp140_2.dll','msvcp140_atomic_wait.dll','msvcp140_codecvt_ids.dll','vcruntime140.dll','vcruntime140_1.dll')
function Make-Package([string]$Path, [string]$Executable, [string]$Version, [bool]$Old) {
    New-Item -ItemType Directory -Path (Join-Path $Path 'licenses') -Force | Out-Null
    Copy-Item -LiteralPath $Executable -Destination (Join-Path $Path 'CleanSlate.exe')
    foreach ($dll in $dlls) { Copy-Item -LiteralPath (Join-Path $RuntimePackage $dll) -Destination $Path }
    foreach ($name in @('LICENSE','README.txt','THIRD-PARTY-NOTICES.txt','licenses/fixture.txt')) {
        [IO.File]::WriteAllText((Join-Path $Path $name), "Synthetic package $Version; $name")
    }
    if ($Old) { [IO.File]::WriteAllText((Join-Path $Path 'obsolete-owned.txt'), 'must disappear from the new tree') }
    $files = @(Get-ChildItem -LiteralPath $Path -File -Recurse | ForEach-Object {
        @{ path = [IO.Path]::GetRelativePath($Path,$_.FullName).Replace('\','/'); sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant(); bytes = $_.Length }
    })
    @{schema=1;product='CleanSlate';package_kind='portable-zip';entrypoint='CleanSlate.exe';version=$Version;platform='windows-x64';commit='synthetic';dirty=$true;rustc='fixture';cargo_lock_sha256=('0'*64);files=$files} |
        ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $Path 'package-manifest.json') -Encoding utf8NoBOM
}
function Start-Hidden([string]$Exe, [string[]]$Arguments) {
    $start = [Diagnostics.ProcessStartInfo]::new($Exe)
    $start.UseShellExecute=$false; $start.CreateNoWindow=$true
    foreach ($argument in $Arguments) { $start.ArgumentList.Add($argument) }
    [Diagnostics.Process]::Start($start)
}
function Wait-State([string]$Parent, [string[]]$States, [int]$Seconds=200) {
    $deadline=[datetime]::UtcNow.AddSeconds($Seconds)
    while ([datetime]::UtcNow -lt $deadline) {
        $journals=@(Get-ChildItem -LiteralPath $Parent -Directory -Filter '.cleanslate-update-*' | ForEach-Object {Join-Path $_.FullName 'journal.json'} | Where-Object {Test-Path -LiteralPath $_})
        foreach ($path in $journals) {
            try { $journal=Get-Content -LiteralPath $path -Raw | ConvertFrom-Json } catch { continue }
            if ($journal.state -in $States) {return @{Path=$path;Journal=$journal}}
        }
        Start-Sleep -Milliseconds 200
    }
    throw "Timed out waiting for states $States in $Parent"
}
function Stop-FixtureProcesses([string]$Parent) {
    # Close only processes whose executable path is inside this unique disposable fixture.
    Get-Process CleanSlate -ErrorAction SilentlyContinue | ForEach-Object {
        if ($_.Path -and $_.Path.StartsWith($Parent+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) { Stop-Process -Id $_.Id -Force }
    }
}
$previousFailure=$env:CLEANSLATE_UPDATE_FIXTURE_FAILURE
try {
    $candidate=Join-Path $root "CleanSlate-$NewVersion-windows-x64"
    Make-Package $candidate $NewExecutable $NewVersion $false
    $archive="$candidate.zip"
    Compress-Archive -LiteralPath $candidate -DestinationPath $archive
    $manifest=Get-Content -LiteralPath (Join-Path $candidate 'package-manifest.json') -Raw | ConvertFrom-Json
    $descriptor=[ordered]@{schema=1;product='CleanSlate';repository='Today20092/podcast-bleed-reducer';version=$NewVersion;tag="v$NewVersion";platform='windows-x64';archive=[IO.Path]::GetFileName($archive);sha256=(Get-FileHash -LiteralPath $archive).Hash.ToLowerInvariant();bytes=(Get-Item -LiteralPath $archive).Length;expanded_bytes=[long](($manifest.files | Measure-Object -Property bytes -Sum).Sum+(Get-Item -LiteralPath (Join-Path $candidate 'package-manifest.json')).Length);entries=20000;manifest_sha256=(Get-FileHash -LiteralPath (Join-Path $candidate 'package-manifest.json')).Hash.ToLowerInvariant()}
    $metadata="$archive.update.json"
    [IO.File]::WriteAllText($metadata,($descriptor|ConvertTo-Json),[Text.UTF8Encoding]::new($false))
    & $Minisign -S -W -s $TestSecretKey -m $metadata
    if ($LASTEXITCODE -ne 0) {throw 'Fixture signing failed'}
    $releaseMetadata=Join-Path $root 'releases.json'
    @(@{tag_name="v$NewVersion";draft=$false;prerelease=$false;body='Synthetic signed update';assets=@(@{name=[IO.Path]::GetFileName($archive);size=$descriptor.bytes})}) | ConvertTo-Json -Depth 6 -AsArray | Set-Content -LiteralPath $releaseMetadata
    $receipts=[Collections.Generic.List[object]]::new()
    foreach ($case in $Cases) {
        $caseRoot=Join-Path $root $case
        $installed=Join-Path $caseRoot "CleanSlate-$OldVersion-windows-x64"
        Make-Package $installed $OldExecutable $OldVersion $true
        $oldHash=(Get-FileHash -LiteralPath (Join-Path $installed 'CleanSlate.exe')).Hash
        $userRoot=Join-Path $caseRoot 'user-data'; New-Item -ItemType Directory -Path $userRoot | Out-Null
        foreach ($file in @('fixture.cleanslate','source.wav','export.wav','preferences.json','cache.bin')) { [IO.File]::WriteAllText((Join-Path $userRoot $file),"preserve $file") }
        $before=@(Get-ChildItem -LiteralPath $userRoot -File | Get-FileHash | Select-Object Path,Hash | ConvertTo-Json -Compress)
        $env:CLEANSLATE_UPDATE_FIXTURE_FAILURE=$case
        $inputArchive=$archive
        if ($case -eq 'mixed') { [IO.File]::WriteAllText((Join-Path $installed 'owner-recording.wav'),'preserve mixed folder') }
        if ($case -eq 'corrupt') { $inputArchive=Join-Path $caseRoot 'corrupt.zip'; [IO.File]::WriteAllText($inputArchive,'truncated') }
        $lock=$null
        $account=[Security.Principal.WindowsIdentity]::GetCurrent().Name
        if ($case -eq 'permissions') {
            & icacls $installed /deny "$($account):(D)" /q | Out-Null
            if ($LASTEXITCODE -ne 0) {throw 'Cannot arrange disposable permission fixture'}
            & icacls $caseRoot /deny "$($account):(DC)" /q | Out-Null
            if ($LASTEXITCODE -ne 0) {throw 'Cannot arrange disposable parent permission fixture'}
        }
        if ($case -eq 'locked') { $lock=[IO.File]::Open((Join-Path $installed 'CleanSlate.exe'),[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::ReadWrite) }
        try {
            $old=Start-Hidden (Join-Path $installed 'CleanSlate.exe') @('--update-fixture',$inputArchive,$metadata,"$metadata.minisig",$releaseMetadata)
            if (-not $old.WaitForExit(30000)) {throw 'Fixture parent failed to exit'}
            if ($case -in @('mixed','corrupt','permissions')) {
                if ($old.ExitCode -eq 0) {throw "$case unexpectedly accepted"}
                if ($case -eq 'permissions') { & icacls $installed /remove:d $account /q | Out-Null; & icacls $caseRoot /remove:d $account /q | Out-Null }
            } elseif ($case -eq 'interrupted-swap') {
                $receipt=Wait-State $caseRoot @('moving_old') 30
                $tx=Split-Path -Parent $receipt.Path
                Start-Sleep -Milliseconds 1000
                if (Test-Path -LiteralPath $installed) {throw 'Interrupted-swap fixture did not reproduce the missing install path'}
                $recovery=Start-Hidden (Join-Path $tx 'helper/CleanSlate.exe') @('--update-recover',$tx)
                if (-not $recovery.WaitForExit(30000) -or $recovery.ExitCode -ne 0) {throw 'Retained helper did not recover interrupted swap'}
                $receipt=Wait-State $caseRoot @('rolled_back') 30
            } else {
                $receipt=Wait-State $caseRoot @('committed','rolled_back')
                $expected=if($case -eq 'success'){'committed'}else{'rolled_back'}
                if ($receipt.Journal.state -ne $expected) {throw "Expected $expected for $case, got $($receipt.Journal.state)"}
            }
            Stop-FixtureProcesses $caseRoot
            Start-Sleep -Milliseconds 300
            $actual=(Get-FileHash -LiteralPath (Join-Path $installed 'CleanSlate.exe')).Hash
            if ($case -eq 'success') {
                if ($actual -ne (Get-FileHash -LiteralPath $NewExecutable).Hash -or (Test-Path -LiteralPath (Join-Path $installed 'obsolete-owned.txt'))) {throw 'Whole-package replacement failed'}
                & (Join-Path $PSScriptRoot 'check-package.ps1') -Package $installed -IntegrityOnly
                $tx=Split-Path -Parent $receipt.Path
                if (-not (Test-Path -LiteralPath (Join-Path $tx 'backup/CleanSlate.exe'))) {throw 'Previous executable was not retained'}
                $cleanup=Start-Hidden (Join-Path $installed 'CleanSlate.exe') @('--update-fixture-cleanup')
                if (-not $cleanup.WaitForExit(30000) -or $cleanup.ExitCode -ne 0 -or (Test-Path -LiteralPath $tx)) {throw 'Verified retained-package cleanup failed'}
                & (Join-Path $PSScriptRoot 'check-package.ps1') -Package $installed -IntegrityOnly
            } elseif ($actual -ne $oldHash) {throw "Previous executable changed after $case"}
            $after=@(Get-ChildItem -LiteralPath $userRoot -File | Get-FileHash | Select-Object Path,Hash | ConvertTo-Json -Compress)
            if (Compare-Object $before $after) {throw "User fixture files changed after $case"}
            if ($case -eq 'mixed' -and (Get-Content -LiteralPath (Join-Path $installed 'owner-recording.wav') -Raw) -ne 'preserve mixed folder') {throw 'Mixed-folder recording changed'}
            $receipts.Add(@{case=$case;result='pass';executable=(Join-Path $installed 'CleanSlate.exe');sha256=$actual;archive_sha256=$descriptor.sha256})
            Write-Output "Packaged updater $case : pass"
        } finally {
            if ($lock) {$lock.Dispose()}; Stop-FixtureProcesses $caseRoot
            if ($case -eq 'permissions') { & icacls $installed /remove:d $account /q | Out-Null; & icacls $caseRoot /remove:d $account /q | Out-Null }
        }
    }
    $receipts | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $root 'receipt.json')
    Write-Output "Preserved disposable packages and receipt: $root"
} finally { $env:CLEANSLATE_UPDATE_FIXTURE_FAILURE=$previousFailure; Stop-FixtureProcesses $root }
