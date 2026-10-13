$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    & "$PSScriptRoot/test-ci.ps1"
    & "$PSScriptRoot/test-packaging.ps1"
    & "$PSScriptRoot/test-release-status.ps1"
    cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting failed' }
    cargo clippy --all-targets --all-features --locked -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Clippy failed' }
    cargo test --all-targets --locked -- --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
} finally {
    Pop-Location
}
