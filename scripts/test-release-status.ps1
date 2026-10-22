$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. "$PSScriptRoot/release-status.ps1"
function Assert($Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
function New-Fixture {
    $platforms = @('windows-x64', 'macos-arm64', 'macos-x64', 'linux-x64')
    $jobs = @(foreach ($platform in $platforms) {
        @{ id = 100 + [array]::IndexOf($platforms, $platform); name = "Package $platform"; status = 'completed'; conclusion = 'success'; steps = @(
            @{ name = 'Check and test'; status = 'completed'; conclusion = 'success' },
            @{ name = 'Build and verify application package'; status = 'completed'; conclusion = 'success' },
            @{ name = 'Save checked package'; status = 'completed'; conclusion = 'success' }
        ) }
    })
    $jobs += @{ id = 200; name = 'Prepare one draft with all desktop packages'; status = 'completed'; conclusion = 'success'; steps = @(@{ name = 'Prepare draft release'; status = 'completed'; conclusion = 'success' }) }
    return @{
        Run = @{ id = 1; html_url = 'https://github.com/example/repo/actions/runs/1'; run_attempt = 2; status = 'completed'; conclusion = 'success' }
        Jobs = $jobs
        Artifacts = @(foreach ($platform in $platforms) { @{ id = 1000 + [array]::IndexOf($platforms, $platform); name = $platform; expired = $false; expires_at = [DateTimeOffset]::UtcNow.AddDays(1).ToString('o') } })
    }
}
$fixture = New-Fixture
$report = Get-ReleaseReport @fixture
Assert ($report.releaseReady -and $report.packagesReady) 'All actual checks and artifacts should pass'
$fixture.Jobs[4].steps[0].name = 'Prepare draft prerelease'
$report = Get-ReleaseReport @fixture
Assert $report.releaseReady 'Historical prerelease workflow evidence must remain readable'
$fixture = New-Fixture
$fixture.Jobs[4].conclusion = 'skipped'
$fixture.Jobs[4].steps = @()
$report = Get-ReleaseReport @fixture
Assert ($report.packagesReady -and -not $report.releaseReady -and $report.draftPreparation -eq 'missing') 'Build-only success must not claim draft preparation'

$fixture = New-Fixture
$fixture.Run.status = 'in_progress'; $fixture.Run.conclusion = $null
$fixture.Jobs[0].status = 'in_progress'; $fixture.Jobs[0].conclusion = $null
$fixture.Jobs[0].steps[0].conclusion = 'failure'
$fixture.Jobs[0].steps += @{ name = 'Post Save cache'; status = 'in_progress'; conclusion = $null }
$logs = @{ '100' = 'error: test failed; exit code: 0xc0000005, STATUS_ACCESS_VIOLATION' }
$report = Get-ReleaseReport @fixture -Logs $logs
Assert (-not $report.packagesReady -and $report.packages[0].tests -eq 'failure' -and $report.packages[0].evidence.Count -eq 1) 'Active cache save must not hide a failed test or native crash'

$fixture = New-Fixture
$fixture.Jobs[0].conclusion = 'failure'
$fixture.Jobs[0].steps[1].conclusion = 'failure'; $fixture.Jobs[0].steps[2].conclusion = 'skipped'
$fixture.Artifacts = @($fixture.Artifacts | Where-Object name -NE 'windows-x64')
$report = Get-ReleaseReport @fixture
Assert (-not $report.packagesReady -and $report.packages[0].artifact -eq 'missing' -and $report.packages[0].logStatus -eq 'unavailable') 'Failed package without logs/artifact must remain failed'

$fixture = New-Fixture
$fixture.Artifacts[0].expired = $true
$fixture.Artifacts[1].expires_at = [DateTimeOffset]::UtcNow.AddDays(-1).ToString('o')
$fixture.Jobs[2].steps[1].status = 'in_progress'; $fixture.Jobs[2].steps[1].conclusion = $null
$fixture.Jobs[3].steps = @()
$report = Get-ReleaseReport @fixture
Assert (-not $report.packagesReady -and $report.packages[0].artifact -eq 'expired' -and $report.packages[1].artifact -eq 'expired' -and $report.packages[2].verification -eq 'in_progress' -and $report.packages[3].tests -eq 'missing') 'Expired, pending and missing evidence must not count as green'

$fixture = New-Fixture
$old = @{ id = 50; name = 'Package windows-x64'; status = 'completed'; conclusion = 'failure'; steps = @() }
$fixture.Jobs += $old
$report = Get-ReleaseReport @fixture
Assert ($report.packagesReady -and $report.packages[0].jobId -eq 100 -and $report.failedJobs.Count -eq 0) 'Older failed attempt must not override the latest job or failure summary'
$fixture.Jobs += @{ id = 300; name = 'Package windows-x64'; status = 'queued'; conclusion = $null; steps = @() }
$report = Get-ReleaseReport @fixture
Assert (-not $report.packagesReady -and $report.packages[0].jobId -eq 300) 'Newer pending retry must not inherit old success'
$fixture = New-Fixture
$fixture.Jobs[0].steps[0].conclusion = 'skipped'
$fixture.Jobs[1].steps[0].conclusion = 'cancelled'
$fixture.Jobs = @($fixture.Jobs | Where-Object name -NE 'Package linux-x64')
$report = Get-ReleaseReport @fixture
Assert (-not $report.packagesReady -and $report.packages[0].tests -eq 'skipped' -and $report.packages[1].tests -eq 'cancelled' -and $report.packages[3].jobStatus -eq 'missing') 'Skipped, cancelled and missing jobs must remain non-ready'
$fixture = New-Fixture
$fixture.Jobs[1].status = 'in_progress'; $fixture.Jobs[1].conclusion = $null
$fixture.Jobs[1].steps[1].conclusion = 'failure'
$logs = @{ '101' = "hdiutil: create failed - Resource busy`n##[error]Exception: Package build failed" }
$report = Get-ReleaseReport @fixture -Logs $logs
Assert ($report.packages[1].evidence.Count -eq 2 -and -not $report.packages[1].ready) 'Active failed package must include native packaging and exception evidence'
Write-Output 'Release status fixture checks: pass'
