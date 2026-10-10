param(
    [Parameter(Mandatory)][string]$Package,
    [string]$Results = (Join-Path $PSScriptRoot '../.scratch/release-checks/sandbox')
)
$ErrorActionPreference = 'Stop'
$Package = (Resolve-Path -LiteralPath $Package).Path
if (-not (Test-Path -LiteralPath (Join-Path $Package 'CleanSlate.exe'))) { throw 'Extracted package executable missing' }
$Results = [IO.Path]::GetFullPath($Results)
if ($Results.StartsWith($Package + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) -or $Results -eq $Package) { throw 'Results must be separate from the package' }
$Results = Join-Path $Results ([guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $Results -Force | Out-Null
$runner = @'
$ErrorActionPreference = 'Stop'
try {
    $process = Start-Process -FilePath 'C:\candidate\CleanSlate.exe' -ArgumentList '--package-self-check --report C:\results\smoke.txt' -Wait -PassThru -WindowStyle Hidden
    if ($process.ExitCode -ne 0) { throw "Packaged self-check failed: $($process.ExitCode)" }
    Start-Process -FilePath 'C:\candidate\CleanSlate.exe'
} catch {
    "FAIL: $_" | Set-Content -LiteralPath 'C:\results\failure.txt'
}
'@
$runner | Set-Content -LiteralPath (Join-Path $Results 'run.ps1') -Encoding utf8
$packageXml = [Security.SecurityElement]::Escape($Package)
$resultsXml = [Security.SecurityElement]::Escape($Results)
$config = @"
<Configuration>
  <Networking>Disable</Networking>
  <vGPU>Disable</vGPU>
  <ClipboardRedirection>Disable</ClipboardRedirection>
  <MappedFolders>
    <MappedFolder><HostFolder>$packageXml</HostFolder><SandboxFolder>C:\candidate</SandboxFolder><ReadOnly>true</ReadOnly></MappedFolder>
    <MappedFolder><HostFolder>$resultsXml</HostFolder><SandboxFolder>C:\results</SandboxFolder><ReadOnly>false</ReadOnly></MappedFolder>
  </MappedFolders>
  <LogonCommand><Command>powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\results\run.ps1</Command></LogonCommand>
</Configuration>
"@
$path = Join-Path $Results 'CleanSlate.wsb'
$config | Set-Content -LiteralPath $path -Encoding utf8
Start-Process -FilePath (Join-Path $env:WINDIR 'System32/WindowsSandbox.exe') -ArgumentList ('"' + $path + '"') -WindowStyle Hidden
Write-Output "Sandbox launched with networking disabled. Check $Results/smoke.txt, then inspect the open GUI."
