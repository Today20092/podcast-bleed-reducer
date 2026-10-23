$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path -Parent $PSScriptRoot
$pr = (Get-Content -LiteralPath "$root/.github/workflows/pr-checks.yml" -Raw).Replace("`r`n", "`n")
$release = (Get-Content -LiteralPath "$root/.github/workflows/portable-release.yml" -Raw).Replace("`r`n", "`n")
foreach ($workflow in @($pr, $release)) {
    if ($workflow -notmatch 'run: ./scripts/check.ps1' -or $workflow -match 'cargo (fmt|clippy|test)') { throw 'PR and release must use the shared check command' }
}
$prMatrix = [regex]::Matches($pr, '- platform: ([\w-]+)\s+runner: ([\w-]+)') | ForEach-Object { $_.Groups[1].Value + ':' + $_.Groups[2].Value }
$releaseMatrix = [regex]::Matches($release, '- platform: ([\w-]+)\s+runner: ([\w-]+)') | ForEach-Object { $_.Groups[1].Value + ':' + $_.Groups[2].Value }
if ($prMatrix.Count -ne 4 -or (Compare-Object $prMatrix $releaseMatrix)) { throw 'PR checks must cover all native release runners' }
if ($pr -notmatch '(?m)^  pull_request:$' -or $pr -match 'pull_request_target|contents: write|secrets\.|actions/cache(?:/save)?@|upload-artifact|package.ps1|gh release' -or $pr -notmatch 'persist-credentials: false') { throw 'PR workflow must remain read-only and check-only' }
$check = Get-Content -LiteralPath "$PSScriptRoot/check.ps1" -Raw
foreach ($command in @('cargo fmt --all -- --check', 'cargo clippy --all-targets --all-features @cargoArguments -- -D warnings', 'cargo test --all-targets @cargoArguments -- --nocapture')) {
    if (-not $check.Contains($command)) { throw "Missing required check: $command" }
}
if (-not $check.Contains('$cargoArguments = @(''--locked'')')) { throw 'Hosted checks must retain locked dependency resolution' }
if ([regex]::Matches($check, 'if \(\$LASTEXITCODE -ne 0\)').Count -ne 3) { throw 'Every Cargo check must propagate failure' }
Write-Output 'Native PR workflow regression checks: pass'

$downloads = Get-Content -LiteralPath "$root/.github/workflows/release-downloads.yml" -Raw
foreach ($required in @('pull-requests: write', 'actions: write', 'gh pr create', 'gh workflow run pr-checks.yml', 'HEAD:refs/heads/$branch')) {
    if (-not $downloads.Contains($required)) { throw "Missing protected-branch download automation: $required" }
}
if ($downloads -match '(?m)^\s+git push\s*\r?$|git push.*(?:--force|refs/heads/main)') { throw 'Download automation must push only a new PR branch' }
Write-Output 'Protected-branch download automation checks: pass'

# Execute the actual release step with a local CLI double, without network or assets.
$draftMatch = [regex]::Match($release, '(?s)- name: Prepare draft release\n\s+shell: pwsh\n\s+run: \|\n(?<code>.*)\z')
if (-not $draftMatch.Success) { throw 'Draft preparation step missing' }
$draftStep = [scriptblock]::Create($draftMatch.Groups['code'].Value)
function Test-DraftPreparation([string]$Tag, [string]$Existing, [string]$Failure = '', [switch]$MissingAsset) {
    $calls = [Collections.Generic.List[object]]::new()
    function Test-Path { param([string]$LiteralPath) return -not $MissingAsset }
    function gh {
        $calls.Add(@($args))
        $global:LASTEXITCODE = 0
        if ($args[1] -eq 'view') {
            if ($Existing -eq 'missing') { $global:LASTEXITCODE = 1 } else { @{ isDraft = $Existing -eq 'draft' } | ConvertTo-Json }
        } elseif ($args[1] -eq $Failure) { $global:LASTEXITCODE = 1 }
    }
    $previousTag = $env:RELEASE_TAG
    $caught = ''
    try {
        $env:RELEASE_TAG = $Tag
        & $draftStep | Out-Null
    } catch { $caught = $_.Exception.Message } finally { $env:RELEASE_TAG = $previousTag }
    if ($Existing -eq 'published') {
        if ($caught -ne 'Refusing to replace a published release' -or $calls.Count -ne 1) { throw 'Published releases must remain untouched' }
        return
    }
    if ($MissingAsset) {
        if ($caught -notlike 'Missing checked asset:*' -or $calls.Count -ne 0) { throw 'Missing checked assets must prevent release mutation' }
        return
    }
    if ($Failure) {
        if (-not $caught -or $calls[-1][1] -ne $Failure) { throw "Failed $Failure must stop draft preparation" }
        return
    }
    if ($caught) { throw $caught }
    $command = if ($Existing -eq 'draft') { 'edit' } else { 'create' }
    $metadata = @($calls | Where-Object { $_[1] -eq $command })[0]
    $isPrerelease = $Tag.Contains('-')
    $notes = if ($isPrerelease) { 'docs/alpha-release-notes.md' } else { 'docs/release-notes.md' }
    if ($metadata -notcontains '--draft' -or $metadata -notcontains "--prerelease=$($isPrerelease.ToString().ToLowerInvariant())" -or $metadata -notcontains $notes) { throw "Incorrect draft metadata for $Tag ($Existing)" }
    $upload = if ($Existing -eq 'draft') { $calls[-1] } else { $metadata }
    $assets = @($upload | Where-Object { $_ -like 'release-assets/*' })
    if ($assets.Count -ne 9 -or ($Existing -eq 'draft' -and $upload[1] -ne 'upload')) { throw 'All four packages, checksums and Windows update descriptor must be attached' }
    if ($assets -notcontains "release-assets/CleanSlate-$($Tag.Substring(1))-windows-x64.zip.update.json") { throw 'Missing Windows update descriptor argument' }
    foreach ($suffix in @('windows-x64.zip', 'macos-arm64.dmg', 'macos-x64.dmg', 'linux-x64.AppImage')) {
        $asset = "release-assets/CleanSlate-$($Tag.Substring(1))-$suffix"
        if ($assets -notcontains $asset -or $assets -notcontains "$asset.sha256") { throw "Missing asset arguments: $asset" }
    }
}
foreach ($tag in @('v1.0.0', 'v1.1.0-beta.1')) {
    Test-DraftPreparation $tag missing
    Test-DraftPreparation $tag draft
    Test-DraftPreparation $tag published
}
Test-DraftPreparation v1.0.0 missing -MissingAsset
Test-DraftPreparation v1.0.0 missing -Failure create
Test-DraftPreparation v1.0.0 draft -Failure edit
Test-DraftPreparation v1.0.0 draft -Failure upload
Write-Output 'Stable/prerelease draft preparation fixture checks: pass'
