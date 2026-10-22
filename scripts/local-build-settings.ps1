param([string]$TargetDir)

$ErrorActionPreference = 'Stop'
if (-not $TargetDir) {
    $localRoot = [Environment]::GetFolderPath('LocalApplicationData', 'DoNotVerify')
    if (-not $localRoot -or -not [IO.Path]::IsPathRooted($localRoot)) {
        throw 'A local application-data directory is required for the shared Cargo cache.'
    }
    $TargetDir = Join-Path $localRoot 'CleanSlate/BuildCache'
}
$TargetDir = [IO.Path]::GetFullPath($TargetDir)
[pscustomobject]@{
    TargetDir = $TargetDir
    CargoArguments = @('--locked', '--offline', '-j', '1', '--target-dir', $TargetDir,
        '--config', (Join-Path $PSScriptRoot 'gui-dev.toml'))
}
