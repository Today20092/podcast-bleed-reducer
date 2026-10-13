param(
    [long]$RunId,
    [string]$Repository,
    [switch]$Json
)
$ErrorActionPreference = 'Stop'

function Get-CheckState($Job, [string]$Name) {
    if (-not $Job) { return 'missing' }
    $step = @($Job.steps | Where-Object name -EQ $Name | Select-Object -Last 1)
    if ($step.Count -eq 0) { return 'missing' }
    if ($step[0].status -ne 'completed') { return $step[0].status }
    if (-not $step[0].conclusion) { return 'unknown' }
    return $step[0].conclusion
}

function Get-ReleaseReport($Run, [array]$Jobs, [array]$Artifacts, [hashtable]$Logs = @{}) {
    $Jobs = @($Jobs | Group-Object name | ForEach-Object { $_.Group | Sort-Object id -Descending | Select-Object -First 1 })
    $platforms = @('windows-x64', 'macos-arm64', 'macos-x64', 'linux-x64')
    $packages = @(foreach ($platform in $platforms) {
        # Latest job for each name wins; reruns can retain successful older jobs.
        $job = $Jobs | Where-Object name -EQ "Package $platform" | Sort-Object id -Descending | Select-Object -First 1
        $tests = Get-CheckState $job 'Check and test'
        $verification = Get-CheckState $job 'Build and verify application package'
        $upload = Get-CheckState $job 'Save checked package'
        $artifact = $Artifacts | Where-Object name -EQ $platform | Sort-Object id -Descending | Select-Object -First 1
        $availability = if (-not $artifact) { 'missing' } elseif ($artifact.expired -or ([DateTimeOffset]$artifact.expires_at -le [DateTimeOffset]::UtcNow)) { 'expired' } else { 'available' }
        $evidence = @()
        $logState = 'not needed'
        $failedSteps = @(if ($job) { $job.steps | Where-Object conclusion -EQ 'failure' | ForEach-Object name })
        if ($job -and ($job.conclusion -eq 'failure' -or $failedSteps.Count -gt 0)) {
            $logState = 'unavailable'
            if ($Logs.ContainsKey([string]$job.id)) {
                $logState = 'available'
                # Keep diagnostics bounded; never print full native logs or command environments.
                $cleanLog = $Logs[[string]$job.id] -replace '(?:\x1b\[|\^\[\[)[0-9;]*[a-zA-Z]', ''
                $evidence = @($cleanLog -split "`n" | Where-Object { $_ -cmatch 'test result:|STATUS_ACCESS_VIOLATION|0xc0000005|exit code:|error:|\.\.\. FAILED|panicked at|hdiutil:|##\[error\]|Exception' } | Select-Object -Last 12 | ForEach-Object { if ($_.Length -gt 500) { $_.Substring(0, 500) } else { $_.Trim() } })
            }
        }
        [pscustomobject]@{
            platform = $platform
            jobId = if ($job) { $job.id } else { $null }
            jobStatus = if ($job) { $job.status } else { 'missing' }
            jobConclusion = if ($job) { $job.conclusion } else { $null }
            tests = $tests; verification = $verification; upload = $upload
            artifact = $availability
            failedSteps = @($failedSteps)
            ready = ($job -and $job.status -eq 'completed' -and $job.conclusion -eq 'success' -and $tests -eq 'success' -and $verification -eq 'success' -and $upload -eq 'success' -and $availability -eq 'available')
            logStatus = $logState; evidence = $evidence
        }
    })
    $release = $Jobs | Where-Object name -EQ 'Prepare one draft with all desktop packages' | Sort-Object id -Descending | Select-Object -First 1
    $draft = Get-CheckState $release 'Prepare draft prerelease'
    [pscustomobject]@{
        runId = $Run.id; url = $Run.html_url; attempt = $Run.run_attempt
        workflowStatus = $Run.status; workflowConclusion = $Run.conclusion
        failedJobs = @($Jobs | Where-Object { $_.conclusion -eq 'failure' -or @($_.steps | Where-Object conclusion -EQ 'failure').Count -gt 0 } | ForEach-Object { [pscustomobject]@{ name = $_.name; failedSteps = @($_.steps | Where-Object conclusion -EQ 'failure' | ForEach-Object name) } })
        packagesReady = (@($packages | Where-Object { -not $_.ready }).Count -eq 0)
        draftPreparation = $draft
        releaseReady = ($Run.status -eq 'completed' -and $Run.conclusion -eq 'success' -and @($packages | Where-Object { -not $_.ready }).Count -eq 0 -and $draft -eq 'success')
        packages = $packages
    }
}

if ($MyInvocation.InvocationName -ne '.') {
    if ($RunId -le 0) { throw 'Specify -RunId with a positive GitHub Actions run ID.' }
    $ghCommand = Get-Command gh -ErrorAction SilentlyContinue
    $gh = if ($ghCommand) { $ghCommand.Source } elseif (Test-Path -LiteralPath "$env:ProgramFiles/GitHub CLI/gh.exe") { "$env:ProgramFiles/GitHub CLI/gh.exe" } else { throw 'GitHub CLI (gh) is required.' }
    function Invoke-Gh([string[]]$Arguments) {
        $output = & $gh @Arguments 2>&1
        if ($LASTEXITCODE -ne 0) { throw "GitHub CLI request failed (exit $LASTEXITCODE). Check gh authentication and repository/run access." }
        return $output -join "`n"
    }
    if (-not $Repository) { $Repository = Invoke-Gh @('repo', 'view', '--json', 'nameWithOwner', '--jq', '.nameWithOwner') }
    if ($Repository -notmatch '^[\w.-]+/[\w.-]+$') { throw 'Repository must be OWNER/REPO on github.com.' }
    $endpoint = "repos/$Repository/actions/runs/$RunId"
    $run = Invoke-Gh @('api', $endpoint) | ConvertFrom-Json
    $pages = Invoke-Gh @('api', "${endpoint}/jobs?filter=latest&per_page=100", '--paginate', '--slurp') | ConvertFrom-Json
    $jobs = @($pages | ForEach-Object { $_.jobs })
    $pages = Invoke-Gh @('api', "${endpoint}/artifacts?per_page=100", '--paginate', '--slurp') | ConvertFrom-Json
    $artifacts = @($pages | ForEach-Object { $_.artifacts })
    $logs = @{}
    foreach ($job in $jobs | Where-Object { $_.conclusion -eq 'failure' -or @($_.steps | Where-Object conclusion -EQ 'failure').Count -gt 0 }) {
        try { $logs[[string]$job.id] = Invoke-Gh @('run', 'view', [string]$RunId, '--repo', $Repository, '--job', [string]$job.id, '--log-failed') } catch { }
    }
    $report = Get-ReleaseReport $run $jobs $artifacts $logs
    if ($Json) { $report | ConvertTo-Json -Depth 8 } else {
        "Run $RunId attempt $($report.attempt): $($report.workflowStatus) / $($report.workflowConclusion)"
        "Packages ready: $($report.packagesReady); draft preparation: $($report.draftPreparation); release ready: $($report.releaseReady)"
        foreach ($failed in $report.failedJobs) { "Failed job: $($failed.name); steps: $($failed.failedSteps -join ', ')" }
        foreach ($package in $report.packages) {
            "$($package.platform): tests=$($package.tests), verify=$($package.verification), upload=$($package.upload), artifact=$($package.artifact), ready=$($package.ready)"
            if ($package.failedSteps.Count) { "  Failed: $($package.failedSteps -join ', '); logs=$($package.logStatus)" }
            $package.evidence | ForEach-Object { "  $_" }
        }
        $report.url
    }
}
