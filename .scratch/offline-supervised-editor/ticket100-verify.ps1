param([ValidateSet('Focused','Full','Build')][string]$Mode = 'Focused')
$ErrorActionPreference = 'Stop'
Add-Type -Path ./scripts/resource-guard.cs
if (-not [CleanSlate.Build.OwnedTask]::IsContained($env:CLEANSLATE_RESOURCE_JOB)) {
    throw 'Run this verification script through scripts/resource-guard.ps1; an inherited job is required.'
}
$toolBin = Split-Path (& rustup which cargo)
$env:PATH = "$toolBin;$env:PATH"
$settings = & ./scripts/local-build-settings.ps1
$result = 0
try {
    switch ($Mode) {
        Focused {
            & cargo test --test project @($settings.CargoArguments)
            if ($LASTEXITCODE) { throw 'Project regression tests failed.' }
            & cargo test --lib presets @($settings.CargoArguments)
            if ($LASTEXITCODE) { throw 'Preset regression tests failed.' }
            & cargo test --lib named_files_in_working_storage @($settings.CargoArguments)
            if ($LASTEXITCODE) { throw 'Recovery identity regression failed.' }
        }
        Full { & ./scripts/check.ps1 -Local }
        Build { & ./scripts/gui-dev.ps1 -BuildOnly }
    }
} catch { Write-Output $_; $result = 1 }
finally {
    # shortcut: this MSVC installation leaves owned VCTIP telemetry alive after Cargo exits; remove when the toolchain exits cleanly.
    Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class Ticket100Job {
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)] static extern IntPtr OpenJobObject(uint access, bool inherit, string name);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool QueryInformationJobObject(IntPtr job, int kind, IntPtr value, uint size, IntPtr returned);
    [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll", SetLastError=true)] static extern bool IsProcessInJob(IntPtr process, IntPtr job, out bool contained);
    public static bool Contains(string name, IntPtr process) {
        IntPtr job = OpenJobObject(4, false, name);
        if (job == IntPtr.Zero) throw new System.ComponentModel.Win32Exception();
        try {
            bool contained;
            if (!IsProcessInJob(process, job, out contained)) throw new System.ComponentModel.Win32Exception();
            return contained;
        } finally { CloseHandle(job); }
    }
    public static int[] Members(string name) {
        IntPtr job = OpenJobObject(4, false, name);
        if (job == IntPtr.Zero) throw new System.ComponentModel.Win32Exception();
        IntPtr data = Marshal.AllocHGlobal(8 + 4096 * IntPtr.Size);
        try {
            if (!QueryInformationJobObject(job, 3, data, (uint)(8 + 4096 * IntPtr.Size), IntPtr.Zero)) throw new System.ComponentModel.Win32Exception();
            var result = new List<int>();
            for (int i=0; i<Marshal.ReadInt32(data,4); i++) result.Add((int)Marshal.ReadIntPtr(data,8+i*IntPtr.Size));
            return result.ToArray();
        } finally { Marshal.FreeHGlobal(data); CloseHandle(job); }
    }
}
'@
    if ($env:CLEANSLATE_RESOURCE_JOB) {
        $remaining = @()
        foreach ($member in [Ticket100Job]::Members($env:CLEANSLATE_RESOURCE_JOB)) {
            if ($member -ne $PID) {
                $owned = Get-Process -Id $member -ErrorAction SilentlyContinue
                if ($owned) { $remaining += $owned }
            }
        }
        $compilerRoot = [IO.Path]::GetFullPath((Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/2022/BuildTools/VC/Tools/MSVC/'))
        $consoleImage = Join-Path $env:SystemRoot 'System32/conhost.exe'
        foreach ($owned in $remaining) {
            $path = $owned.Path
            if (-not (($owned.ProcessName -ieq 'vctip' -and $path.StartsWith($compilerRoot, [StringComparison]::OrdinalIgnoreCase)) -or ($path -ieq $consoleImage))) {
                throw "Unexpected owned child remains; preserve guard containment: $($owned.Id) $($owned.ProcessName)"
            }
        }
        foreach ($owned in ($remaining | Sort-Object @{Expression={ $_.ProcessName -ieq 'conhost' }})) {
            Write-Output "Cleaning completed task's owned helper: $($owned.Id) $($owned.ProcessName)"
            if (-not $owned.HasExited) {
                if (-not [Ticket100Job]::Contains($env:CLEANSLATE_RESOURCE_JOB, $owned.Handle)) { throw 'Helper ownership changed; refusing termination.' }
                $owned.Kill(); $owned.WaitForExit(5000) | Out-Null
            }
        }
    }
}
exit $result
