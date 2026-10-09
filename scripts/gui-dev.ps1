param(
    [string]$Project,
    [string]$TargetDir,
    [switch]$BuildOnly
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if (-not $TargetDir) {
    $retained = Join-Path $env:USERPROFILE '.codex/worktrees/offline-timebase/podcast-automix-rust-gui/target/owner-ui'
    $TargetDir = if (Test-Path -LiteralPath $retained) { $retained } else { Join-Path $root 'target/owner-ui' }
}
$TargetDir = [IO.Path]::GetFullPath($TargetDir)
$exe = Join-Path $TargetDir 'debug/rust-app.exe'
if (Get-Process -Name rust-app -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $exe }) {
    throw 'Save your edits and close the development window before rebuilding.'
}

# Keep edits in a separate project; source audio and the saved owner session stay intact.
if (-not $BuildOnly) {
    if (-not $Project) { $Project = Join-Path $root 'test-audio/quick-test-10min/261005-quick-test.cleanslate' }
    $Project = (Resolve-Path -LiteralPath $Project).Path
    $devProject = Join-Path ([IO.Path]::GetDirectoryName($Project)) (([IO.Path]::GetFileNameWithoutExtension($Project)) + '.gui-dev.cleanslate')
    if (-not (Test-Path -LiteralPath $devProject)) {
        Copy-Item -LiteralPath $Project -Destination $devProject
        foreach ($extension in @('analysis-cache', 'waveforms.json')) {
            $cache = [IO.Path]::ChangeExtension($Project, $extension)
            if (Test-Path -LiteralPath $cache) {
                Copy-Item -LiteralPath $cache -Destination ([IO.Path]::ChangeExtension($devProject, $extension)) -Recurse
            }
        }
    }
}

$timer = [Diagnostics.Stopwatch]::StartNew()
$previousIncremental = $env:CARGO_INCREMENTAL
try {
    $env:CARGO_INCREMENTAL = '1'
    & cargo build --manifest-path (Join-Path $root 'Cargo.toml') --bin rust-app --locked --offline -j 1 --target-dir $TargetDir --config (Join-Path $PSScriptRoot 'gui-dev.toml') --timings
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
