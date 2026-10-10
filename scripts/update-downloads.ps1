param([string]$Readme = (Join-Path $PSScriptRoot '../README.md'), [string]$ReleasesFile = '')
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repository = 'Today20092/podcast-bleed-reducer'
if ($ReleasesFile) {
    $releases = @(Get-Content -LiteralPath $ReleasesFile -Raw | ConvertFrom-Json)
} else {
    $headers = @{ Accept = 'application/vnd.github+json'; 'X-GitHub-Api-Version' = '2022-11-28' }
    if ($env:GH_TOKEN) { $headers.Authorization = "Bearer $env:GH_TOKEN" }
    $releases = @()
    $page = 1
    do {
        $batch = @(Invoke-RestMethod -Uri "https://api.github.com/repos/$repository/releases?per_page=100&page=$page" -Headers $headers | Write-Output)
        $releases += $batch
        $page++
    } while ($batch.Count -eq 100)
}
$published = @($releases | Where-Object { -not $_.draft -and $_.published_at -and $_.tag_name -match '^v\d+\.\d+\.\d+(?:-[0-9A-Za-z]+(?:\.[0-9A-Za-z]+)*)?$' } | Sort-Object { [datetime]$_.published_at } -Descending)
$rows = [Collections.Generic.List[string]]::new()
$rows.Add('| Platform | Download | How to run |')
$rows.Add('| --- | --- | --- |')
foreach ($platform in @(
    @{ name = 'Windows 11, 64-bit'; asset = 'windows-x64.zip'; steps = 'Extract the entire ZIP, then open `CleanSlate.exe`. Keep the support files together.' },
    @{ name = 'Mac, Apple Silicon, macOS 13+'; asset = 'macos-arm64.dmg'; steps = 'Open the DMG and drag `CleanSlate.app` to Applications.' },
    @{ name = 'Mac, Intel, macOS 13+'; asset = 'macos-x64.dmg'; steps = 'Open the DMG and drag `CleanSlate.app` to Applications.' },
    @{ name = 'Linux, 64-bit x86, glibc 2.35+'; asset = 'linux-x64.AppImage'; steps = 'Allow the AppImage to run as a program in its file properties, then open it.' }
)) {
    $download = 'Not published yet'
    foreach ($release in $published) {
        $assetName = "CleanSlate-$($release.tag_name.Substring(1))-$($platform.asset)"
        $assetNames = @($release.assets | ForEach-Object { $_.name })
        if ($assetName -in $assetNames -and "$assetName.sha256" -in $assetNames) {
            $baseUrl = "https://github.com/$repository/releases/download/$($release.tag_name)/$assetName"
            $download = "[Download $($release.tag_name)]($baseUrl) · [SHA-256]($baseUrl.sha256)"
            break
        }
    }
    $rows.Add("| $($platform.name) | $download | $($platform.steps) |")
}
$text = Get-Content -LiteralPath $Readme -Raw
$pattern = '(?s)(<!-- downloads:start -->\r?\n).*?(\r?\n<!-- downloads:end -->)'
if ([regex]::Matches($text, $pattern).Count -ne 1) { throw 'README must contain one downloads block' }
$replacement = $rows -join "`n"
$text = [regex]::Replace($text, $pattern, [Text.RegularExpressions.MatchEvaluator]{ param($match) $match.Groups[1].Value + $replacement + $match.Groups[2].Value })
[IO.File]::WriteAllText([IO.Path]::GetFullPath($Readme), $text, [Text.UTF8Encoding]::new($false))
Write-Output 'README download links now match published release assets.'
