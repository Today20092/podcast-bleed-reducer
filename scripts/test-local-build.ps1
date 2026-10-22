$ErrorActionPreference = 'Stop'
$settingsScript = Join-Path $PSScriptRoot 'local-build-settings.ps1'
$settings = & $settingsScript
$expected = [IO.Path]::GetFullPath((Join-Path ([Environment]::GetFolderPath('LocalApplicationData', 'DoNotVerify')) 'CleanSlate/BuildCache'))
if ($settings.TargetDir -ne $expected) { throw 'Local builds must share the application-data Cargo target.' }
$temporary = Join-Path ([IO.Path]::GetTempPath()) ('cleanslate-build-settings-' + [Guid]::NewGuid())
$overridden = & $settingsScript -TargetDir $temporary
if ($overridden.TargetDir -ne [IO.Path]::GetFullPath($temporary)) { throw 'Explicit target override was lost.' }
$oldLocation = Get-Location
try {
    Set-Location ([IO.Path]::GetTempPath())
    $fromAnotherDirectory = & $settingsScript
    if ($fromAnotherDirectory.TargetDir -ne $settings.TargetDir) { throw 'Cargo target depends on the working directory.' }
} finally {
    Set-Location $oldLocation
}
$arguments = $settings.CargoArguments
if (($arguments -join '|') -ne (@('--locked', '--offline', '-j', '1', '--target-dir', $expected, '--config', (Join-Path $PSScriptRoot 'gui-dev.toml')) -join '|')) {
    throw 'Local Cargo flags must preserve the shared target, locked offline dependencies and one build job.'
}
if (-not (Test-Path -LiteralPath $arguments[-1])) { throw 'Shared development profile is missing.' }
Write-Output 'Shared local build settings checks: pass'
