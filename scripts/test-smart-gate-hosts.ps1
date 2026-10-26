#requires -Version 7.0
param([Parameter(Mandatory)][string]$PluginDirectory, [string]$TargetDir, [string]$ProbeBinary)
$ErrorActionPreference = 'Stop'
& (Join-Path $PSScriptRoot 'test-plugin-hosts.ps1') -Product SmartGate -PluginDirectory $PluginDirectory -TargetDir $TargetDir -ProbeBinary $ProbeBinary
