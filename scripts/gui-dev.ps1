param(
    [string]$Project,
    [string]$TargetDir,
    [switch]$BuildOnly
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$settings = & (Join-Path $PSScriptRoot 'local-build-settings.ps1') -TargetDir $TargetDir
$TargetDir = $settings.TargetDir
$cargoArguments = $settings.CargoArguments
$exe = Join-Path $TargetDir 'debug/rust-app.exe'
if (Get-Process -Name rust-app -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $exe }) {
    throw 'Save your edits and close the development window before rebuilding.'
}

# Keep the development project local; source audio and the saved owner session stay intact.
if (-not $BuildOnly) {
    if (-not $Project) { $Project = Join-Path $root 'test-audio/quick-test-10min/261005-quick-test.cleanslate' }
    $Project = (Resolve-Path -LiteralPath $Project).Path
    if (-not $env:LOCALAPPDATA -or -not [IO.Path]::IsPathRooted($env:LOCALAPPDATA)) {
        throw 'A valid LOCALAPPDATA directory is required for the local development project.'
    }
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        $identity = [BitConverter]::ToString($hasher.ComputeHash([Text.Encoding]::UTF8.GetBytes($Project))).Replace('-', '').ToLowerInvariant()
    } finally {
        $hasher.Dispose()
    }
    $developmentDirectory = Join-Path $env:LOCALAPPDATA "CleanSlate/Projects/GUI-development/$identity"
    New-Item -ItemType Directory -Path $developmentDirectory -Force | Out-Null
    $devProject = Join-Path $developmentDirectory (([IO.Path]::GetFileNameWithoutExtension($Project)) + '.gui-dev.cleanslate')
    if (-not (Test-Path -LiteralPath $devProject)) {
        Copy-Item -LiteralPath $Project -Destination $devProject
    }
}

$timer = [Diagnostics.Stopwatch]::StartNew()
$previousIncremental = $env:CARGO_INCREMENTAL
try {
    $env:CARGO_INCREMENTAL = '1'
    & cargo build --manifest-path (Join-Path $root 'Cargo.toml') --bin rust-app @cargoArguments --timings
    if ($LASTEXITCODE -ne 0) { throw 'GUI build failed.' }
} finally {
    $env:CARGO_INCREMENTAL = $previousIncremental
}
$timer.Stop()
Write-Output ('GUI build: {0:N2} seconds' -f $timer.Elapsed.TotalSeconds)
if (-not $BuildOnly) {
    $previousProject = $env:CLEANSLATE_DEV_PROJECT
    try {
        $env:CLEANSLATE_DEV_PROJECT = $devProject
        Start-Process -FilePath $exe -WorkingDirectory $root -WindowStyle Hidden
    } finally {
        $env:CLEANSLATE_DEV_PROJECT = $previousProject
    }
    Write-Output "Opened development copy: $devProject"
}
