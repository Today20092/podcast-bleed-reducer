# Gate detector feasibility only

Ticket 92 experiment, independently locked and not linked into Smart Gate.
See [decision and resource-blocked checkpoint](../../docs/research/smart-gate-detector-92.md).
Compilation green and inference measurements remain pending; do not enable 93.

From repository root, use shared Cargo settings and the standard resource guard:

```powershell
$s = & ./scripts/local-build-settings.ps1
& ./scripts/resource-guard.ps1 -FilePath (Get-Command cargo).Source -ArgumentList (@('test', '--all-targets', '--manifest-path', 'plugins/gate-detector/Cargo.toml') + $s.CargoArguments)
& ./scripts/resource-guard.ps1 -FilePath (Get-Command cargo).Source -ArgumentList (@('clippy', '--all-targets', '--manifest-path', 'plugins/gate-detector/Cargo.toml') + $s.CargoArguments + @('--', '-D', 'warnings'))
& ./scripts/resource-guard.ps1 -FilePath (Get-Command cargo).Source -ArgumentList (@('build', '--example', 'feasibility', '--manifest-path', 'plugins/gate-detector/Cargo.toml') + $s.CargoArguments)
```

Then guard `$s.TargetDir/debug/examples/feasibility.exe` with two arguments: the
canonical local authorized `test-audio` root and a new private report JSON path.
Never commit audio or infer missing labels. The harness verifies source hashes,
reads at most three seconds per human reference, retains all probabilities and
compares exact evidence across five blocks and three rates. It creates its report
without overwriting an existing file. The harness is synchronous and allocates;
no audio callback safety, gain processing or DAW qualification is implied.
