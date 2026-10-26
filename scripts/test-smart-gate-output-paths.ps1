#requires -Version 7.0
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ([Environment]::OSVersion.Platform -ne 'Win32NT') { throw 'This filesystem fixture requires Windows junctions.' }
$source = Join-Path $PSScriptRoot 'build-smart-gate.ps1'
$temporary = Join-Path ([IO.Path]::GetTempPath()) ('cleanslate-gate-output-' + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $temporary | Out-Null
$results = @()
foreach ($case in @('dist-junction', 'candidate-junction', 'dist-file', 'candidate-file')) {
    $caseRoot = Join-Path $temporary $case
    $workspace = Join-Path $caseRoot 'marked-workspace'
    $scripts = Join-Path $workspace 'scripts'
    $outside = Join-Path $caseRoot 'outside'
    New-Item -ItemType Directory -Path $scripts, $outside -Force | Out-Null
    'disposable-output-path-fixture-v1' | Set-Content -LiteralPath (Join-Path $workspace '.cleanslate-gate-output-fixture')
    Copy-Item -LiteralPath $source -Destination (Join-Path $scripts 'build-smart-gate.ps1')
    $sentinel = Join-Path $outside 'sentinel.txt'
    'preserved outside output' | Set-Content -LiteralPath $sentinel
    $before = (Get-FileHash -LiteralPath $sentinel -Algorithm SHA256).Hash
    $dist = Join-Path $workspace 'dist'
    $destination = $dist
    if ($case -like 'candidate-*') {
        New-Item -ItemType Directory -Path $dist | Out-Null
        $destination = Join-Path $dist 'smart-gate-candidates'
    }
    if ($case -like '*-junction') {
        New-Item -ItemType Junction -Path $destination -Target $outside | Out-Null
        $expected = 'Smart Gate output must not follow a link:'
    } else {
        'regular file collision' | Set-Content -LiteralPath $destination
        $expected = 'Smart Gate output parent must be a directory:'
    }
    # The copied entrypoint has no helper/Cargo project: even a regression cannot compile real code.
    $message = $null
    try { & (Join-Path $scripts 'build-smart-gate.ps1') }
    catch { $message = $_.Exception.Message }
    if (-not $message -or -not $message.StartsWith($expected, [StringComparison]::Ordinal)) {
        throw "Expected $case refusal before the helper/build boundary; actual: $message"
    }
    $outsideItems = @(Get-ChildItem -LiteralPath $outside -Force)
    if ($outsideItems.Count -ne 1 -or $outsideItems[0].Name -ne 'sentinel.txt' -or (Get-FileHash -LiteralPath $sentinel -Algorithm SHA256).Hash -ne $before) {
        throw "The $case fixture changed its owned outside sentinel/contents."
    }
    $results += [pscustomobject]@{case=$case;rejection=$message;sentinel_sha256=$before;outside_unchanged=$true}
}
$results | ConvertTo-Json -Depth 3 | Set-Content -LiteralPath (Join-Path $temporary 'results.json') -Encoding utf8
Write-Output "PASS: Gate output junction/non-directory parents rejected before helpers/builds; outside sentinels unchanged. Evidence: $temporary"
# Retain the owned junctions and both fixture directories as evidence; no recursive link/target cleanup.
