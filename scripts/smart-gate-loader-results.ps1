function Assert-GateLoaderNegativeResult([int]$ExitCode, [string]$Stdout, [string]$Stderr) {
    $pathRejected = $ExitCode -eq 42 -and $Stderr -match '(?m)^LOADER_RUNTIME_PATH_REJECTED: '
    $dependencyMissing = $ExitCode -eq 43 -and $Stderr -match '(?m)^LOADER_DEPENDENCY_MISSING: Windows error 126\r?$'
    if ($Stdout -match '(?m)^PASS:' -or -not ($pathRejected -or $dependencyMissing)) {
        throw "Unqualified missing-sidecar result: exit $ExitCode."
    }
}
