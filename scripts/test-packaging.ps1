$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$temporary = Join-Path ([IO.Path]::GetTempPath()) ('cleanslate-packaging-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $temporary | Out-Null
function Assert-Fails([scriptblock]$Action, [string]$Expected) {
    try { & $Action | Out-Null } catch {
        if ($_.Exception.Message -notmatch $Expected) { throw "Expected $Expected, got $($_.Exception.Message)" }
        return
    }
    throw "Expected failure: $Expected"
}
try {
    foreach ($platform in @('windows-x64', 'macos-arm64', 'macos-x64', 'linux-x64')) {
        $package = Join-Path $temporary $platform
        $resources = ''
        if ($platform -eq 'windows-x64') {
            $files = @('CleanSlate.exe', 'concrt140.dll', 'msvcp140.dll', 'msvcp140_1.dll', 'msvcp140_2.dll', 'msvcp140_atomic_wait.dll', 'msvcp140_codecvt_ids.dll', 'vcruntime140.dll', 'vcruntime140_1.dll')
        } elseif ($platform -like 'macos-*') {
            $resources = 'CleanSlate.app/Contents/Resources/'
            $files = @('CleanSlate.app/Contents/MacOS/CleanSlate', 'CleanSlate.app/Contents/Info.plist', 'CleanSlate.app/Contents/_CodeSignature/CodeResources')
        } else {
            $resources = 'usr/share/cleanslate/'
            $files = @('AppRun', 'usr/bin/CleanSlate', 'cleanslate.desktop', 'cleanslate.svg', '.DirIcon', 'usr/lib/libexample.so.1')
        }
        $files += @("${resources}LICENSE", "${resources}README.txt", "${resources}THIRD-PARTY-NOTICES.txt", "${resources}licenses/example/LICENSE")
        $inventory = @($files | ForEach-Object {
            $path = Join-Path $package $_
            New-Item -ItemType Directory -Path (Split-Path -Parent $path) -Force | Out-Null
            [IO.File]::WriteAllText($path, 'package fixture')
            @{ path = $_; sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() }
        })
        $manifestPath = Join-Path $package 'package-manifest.json'
        $original = @{ platform = $platform; version = '0.1.0-alpha.1'; files = $inventory } | ConvertTo-Json -Depth 5
        [IO.File]::WriteAllText($manifestPath, $original)
        & "$PSScriptRoot/check-package.ps1" -Package $package -IntegrityOnly | Out-Null

        [IO.File]::WriteAllText((Join-Path $package $files[0]), 'corrupted')
        Assert-Fails { & "$PSScriptRoot/check-package.ps1" -Package $package -IntegrityOnly } 'Integrity failure'
        [IO.File]::WriteAllText((Join-Path $package $files[0]), 'package fixture')
        [IO.File]::WriteAllText((Join-Path $package 'recording.wav'), 'must not ship')
        Assert-Fails { & "$PSScriptRoot/check-package.ps1" -Package $package -IntegrityOnly } 'unexpected or missing files'
        Remove-Item -LiteralPath (Join-Path $package 'recording.wav')

        $manifest = $original | ConvertFrom-Json
        $manifest.files += $manifest.files[0]
        $manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $manifestPath
        Assert-Fails { & "$PSScriptRoot/check-package.ps1" -Package $package -IntegrityOnly } 'Duplicate manifest paths'
        $manifest = $original | ConvertFrom-Json
        $manifest.files[-1].path = "${resources}licenses/../outside"
        $manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $manifestPath
        Assert-Fails { & "$PSScriptRoot/check-package.ps1" -Package $package -IntegrityOnly } 'Invalid manifest path'
        $manifest = $original | ConvertFrom-Json
        $manifest.platform = 'unknown'
        $manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $manifestPath
        Assert-Fails { & "$PSScriptRoot/check-package.ps1" -Package $package -IntegrityOnly } 'Invalid package identity'
        [IO.File]::WriteAllText($manifestPath, $original)
        Remove-Item -LiteralPath (Join-Path $package $files[0])
        Assert-Fails { & "$PSScriptRoot/check-package.ps1" -Package $package -IntegrityOnly } 'Cannot find|does not exist'
    }

    $readme = Join-Path $temporary 'README.md'
    $releaseFile = Join-Path $temporary 'releases.json'
    [IO.File]::WriteAllText($readme, "before`n<!-- downloads:start -->`nold`n<!-- downloads:end -->`nafter`n")
    $draft = @{ draft = $true; tag_name = 'v9.0.0'; published_at = $null; assets = @(@{name='CleanSlate-9.0.0-windows-x64.zip'}) }
    $published = @{ draft = $false; tag_name = 'v0.1.0-alpha.1'; published_at = '2026-10-10T00:00:00Z'; assets = @(@{name='CleanSlate-0.1.0-alpha.1-windows-x64.zip'}, @{name='CleanSlate-0.1.0-alpha.1-windows-x64.zip.sha256'}) }
    @($draft, $published) | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $releaseFile
    & "$PSScriptRoot/update-downloads.ps1" -Readme $readme -ReleasesFile $releaseFile | Out-Null
    $result = Get-Content -LiteralPath $readme -Raw
    if ($result -notmatch '/releases/download/v0.1.0-alpha.1/CleanSlate-0.1.0-alpha.1-windows-x64.zip' -or $result -match '9.0.0' -or ([regex]::Matches($result, 'Not published yet')).Count -ne 3 -or $result -notmatch '^before' -or $result -notmatch 'after\s*$') { throw 'Published alpha/draft filtering or README preservation failed' }
    $published.assets = @()
    @($published) | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $releaseFile
    & "$PSScriptRoot/update-downloads.ps1" -Readme $readme -ReleasesFile $releaseFile | Out-Null
    if (([regex]::Matches((Get-Content -LiteralPath $readme -Raw), 'Not published yet')).Count -ne 4) { throw 'Removed assets still have download links' }
    # Invoke-RestMethod returns JSON arrays as one pipeline object, including an empty public release list.
    function Invoke-RestMethod { Write-Output -NoEnumerate @() }
    try { & "$PSScriptRoot/update-downloads.ps1" -Readme $readme | Out-Null } finally { Remove-Item Function:\Invoke-RestMethod }
    if (([regex]::Matches((Get-Content -LiteralPath $readme -Raw), 'Not published yet')).Count -ne 4) { throw 'Empty public release list failed' }
    [IO.File]::WriteAllText($readme, 'no download markers')
    Assert-Fails { & "$PSScriptRoot/update-downloads.ps1" -Readme $readme -ReleasesFile $releaseFile } 'one downloads block'
    Write-Output 'Packaging integrity and published-download regression checks: pass'
} finally {
    $temporary = [IO.Path]::GetFullPath($temporary)
    if (-not $temporary.StartsWith([IO.Path]::GetFullPath([IO.Path]::GetTempPath()), [StringComparison]::OrdinalIgnoreCase) -or (Split-Path -Leaf $temporary) -notlike 'cleanslate-packaging-*') { throw 'Test cleanup escaped temporary directory' }
    Remove-Item -LiteralPath $temporary -Recurse -Force
}
