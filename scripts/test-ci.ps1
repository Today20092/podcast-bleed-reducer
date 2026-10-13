$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path -Parent $PSScriptRoot
$pr = Get-Content -LiteralPath "$root/.github/workflows/pr-checks.yml" -Raw
$release = Get-Content -LiteralPath "$root/.github/workflows/portable-release.yml" -Raw
foreach ($workflow in @($pr, $release)) {
    if ($workflow -notmatch 'run: ./scripts/check.ps1' -or $workflow -match 'cargo (fmt|clippy|test)') { throw 'PR and release must use the shared check command' }
}
$prMatrix = [regex]::Matches($pr, '- platform: ([\w-]+)\s+runner: ([\w-]+)') | ForEach-Object { $_.Groups[1].Value + ':' + $_.Groups[2].Value }
$releaseMatrix = [regex]::Matches($release, '- platform: ([\w-]+)\s+runner: ([\w-]+)') | ForEach-Object { $_.Groups[1].Value + ':' + $_.Groups[2].Value }
if ($prMatrix.Count -ne 4 -or (Compare-Object $prMatrix $releaseMatrix)) { throw 'PR checks must cover all native release runners' }
if ($pr -notmatch '(?m)^  pull_request:$' -or $pr -match 'pull_request_target|contents: write|secrets\.|actions/cache(?:/save)?@|upload-artifact|package.ps1|gh release' -or $pr -notmatch 'persist-credentials: false') { throw 'PR workflow must remain read-only and check-only' }
$check = Get-Content -LiteralPath "$PSScriptRoot/check.ps1" -Raw
foreach ($command in @('cargo fmt --all -- --check', 'cargo clippy --all-targets --all-features --locked -- -D warnings', 'cargo test --all-targets --locked -- --nocapture')) {
    if (-not $check.Contains($command)) { throw "Missing required check: $command" }
}
if ([regex]::Matches($check, 'if \(\$LASTEXITCODE -ne 0\)').Count -ne 3) { throw 'Every Cargo check must propagate failure' }
Write-Output 'Native PR workflow regression checks: pass'
